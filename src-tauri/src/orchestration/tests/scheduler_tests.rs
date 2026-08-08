use std::collections::BTreeMap;

use crate::{
    domain::{WorkItemId, WorkItemState},
    orchestration::SchedulerError,
    policy::PolicyUsage,
};

use super::{
    DaemonScheduler, InMemoryPolicyAudit, confirmed_scheduler, dependency,
    fully_budgeted_work_item, policy_gate, progress, proposal, tick,
};
use crate::domain::DependencyKind;

#[test]
fn scheduler_requires_confirmation_before_it_considers_a_launch() {
    let scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler.schedule(
        tick(
            BTreeMap::from([(
                WorkItemId::from("task"),
                progress(WorkItemState::Ready, false),
            )]),
            [],
        ),
        &policy_gate(1),
        &mut audit_store,
    );

    assert!(matches!(
        result,
        Err(SchedulerError::PlanNotConfirmed { .. })
    ));
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn scheduler_runs_only_dependency_safe_tasks_and_respects_parallelism() {
    let scheduler = confirmed_scheduler(
        vec![
            fully_budgeted_work_item("foundation"),
            fully_budgeted_work_item("dependent"),
            fully_budgeted_work_item("independent"),
        ],
        vec![dependency(
            "foundation-dependent",
            "foundation",
            "dependent",
            DependencyKind::Blocks,
        )],
    );
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler
        .schedule(
            tick(
                BTreeMap::from([
                    (
                        WorkItemId::from("foundation"),
                        progress(WorkItemState::Ready, false),
                    ),
                    (
                        WorkItemId::from("dependent"),
                        progress(WorkItemState::Ready, false),
                    ),
                    (
                        WorkItemId::from("independent"),
                        progress(WorkItemState::Ready, false),
                    ),
                ]),
                [
                    ("foundation", "decision-foundation"),
                    ("independent", "decision-independent"),
                ],
            ),
            &policy_gate(1),
            &mut audit_store,
        )
        .expect("the scheduler should make and audit policy decisions");

    assert_eq!(
        result
            .launches
            .iter()
            .map(|launch| launch.work_item_id.clone())
            .collect::<Vec<_>>(),
        vec![WorkItemId::from("foundation")]
    );
    assert_eq!(
        result
            .deferred_by_policy
            .iter()
            .map(|deferred| deferred.work_item_id.clone())
            .collect::<Vec<_>>(),
        vec![WorkItemId::from("independent")]
    );
    assert_eq!(audit_store.decisions.len(), 2);
    assert!(
        result
            .deferred_by_policy
            .first()
            .expect("one task should be deferred")
            .decision
            .reason
            .contains("Parallel execution limit 1")
    );
}

#[test]
fn scheduler_defers_work_when_the_repository_has_no_execution_capacity() {
    let scheduler = confirmed_scheduler(
        vec![
            fully_budgeted_work_item("first"),
            fully_budgeted_work_item("second"),
        ],
        vec![],
    );
    let mut audit_store = InMemoryPolicyAudit::default();
    let mut scheduler_tick = tick(
        BTreeMap::from([
            (
                WorkItemId::from("first"),
                progress(WorkItemState::Ready, false),
            ),
            (
                WorkItemId::from("second"),
                progress(WorkItemState::Ready, false),
            ),
        ]),
        [],
    );
    scheduler_tick.repository_execution_capacity = 0;

    let result = scheduler
        .schedule(scheduler_tick, &policy_gate(2), &mut audit_store)
        .expect("repository capacity should defer work before policy authorization");

    assert!(result.launches.is_empty());
    assert!(result.deferred_by_policy.is_empty());
    assert_eq!(
        result
            .deferred_by_repository_capacity
            .iter()
            .map(|deferred| deferred.work_item_id.clone())
            .collect::<Vec<_>>(),
        vec![WorkItemId::from("first"), WorkItemId::from("second")]
    );
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn scheduler_can_launch_a_dependent_task_after_accepted_completion_without_a_ui_connection() {
    let scheduler = confirmed_scheduler(
        vec![
            fully_budgeted_work_item("foundation"),
            fully_budgeted_work_item("dependent"),
        ],
        vec![dependency(
            "foundation-dependent",
            "foundation",
            "dependent",
            DependencyKind::Blocks,
        )],
    );
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler
        .schedule(
            tick(
                BTreeMap::from([
                    (
                        WorkItemId::from("foundation"),
                        progress(WorkItemState::Done, true),
                    ),
                    (
                        WorkItemId::from("dependent"),
                        progress(WorkItemState::Ready, false),
                    ),
                ]),
                [("dependent", "decision-dependent")],
            ),
            &policy_gate(1),
            &mut audit_store,
        )
        .expect("a daemon tick does not need a UI connection");

    assert_eq!(
        result.launches[0].work_item_id,
        WorkItemId::from("dependent")
    );
    assert!(result.deferred_by_policy.is_empty());
}

#[test]
fn missing_a_policy_decision_id_fails_before_an_untracked_launch_or_audit_write() {
    let scheduler = confirmed_scheduler(vec![fully_budgeted_work_item("task")], vec![]);
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler.schedule(
        tick(
            BTreeMap::from([(
                WorkItemId::from("task"),
                progress(WorkItemState::Ready, false),
            )]),
            [],
        ),
        &policy_gate(1),
        &mut audit_store,
    );

    assert_eq!(
        result.unwrap_err().to_string(),
        "daemon scheduling requires a policy-decision id for work item task"
    );
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn a_blank_policy_decision_id_fails_before_the_daemon_writes_an_audit_record() {
    let scheduler = confirmed_scheduler(
        vec![
            fully_budgeted_work_item("first"),
            fully_budgeted_work_item("second"),
        ],
        vec![],
    );
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler.schedule(
        tick(
            BTreeMap::from([
                (
                    WorkItemId::from("first"),
                    progress(WorkItemState::Ready, false),
                ),
                (
                    WorkItemId::from("second"),
                    progress(WorkItemState::Ready, false),
                ),
            ]),
            [("first", " "), ("second", "second-decision")],
        ),
        &policy_gate(2),
        &mut audit_store,
    );

    assert!(matches!(
        result,
        Err(SchedulerError::BlankPolicyDecisionId { .. })
    ));
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn a_reused_policy_decision_id_fails_before_the_daemon_writes_an_audit_record() {
    let scheduler = confirmed_scheduler(
        vec![
            fully_budgeted_work_item("first"),
            fully_budgeted_work_item("second"),
        ],
        vec![],
    );
    let mut audit_store = InMemoryPolicyAudit::default();

    let result = scheduler.schedule(
        tick(
            BTreeMap::from([
                (
                    WorkItemId::from("first"),
                    progress(WorkItemState::Ready, false),
                ),
                (
                    WorkItemId::from("second"),
                    progress(WorkItemState::Ready, false),
                ),
            ]),
            [("first", "shared-decision"), ("second", "shared-decision")],
        ),
        &policy_gate(2),
        &mut audit_store,
    );

    assert!(matches!(
        result,
        Err(SchedulerError::DuplicatePolicyDecisionId { .. })
    ));
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn scheduler_passes_existing_usage_to_policy_before_authorizing_a_start() {
    let scheduler = confirmed_scheduler(vec![fully_budgeted_work_item("task")], vec![]);
    let mut audit_store = InMemoryPolicyAudit::default();
    let mut scheduler_tick = tick(
        BTreeMap::from([(
            WorkItemId::from("task"),
            progress(WorkItemState::Ready, false),
        )]),
        [("task", "over-budget")],
    );
    scheduler_tick.usage_by_work_item.insert(
        WorkItemId::from("task"),
        PolicyUsage {
            agent_turns: 3,
            ..PolicyUsage::default()
        },
    );

    let result = scheduler
        .schedule(scheduler_tick, &policy_gate(1), &mut audit_store)
        .expect("policy denial should be recorded rather than fail scheduling");

    assert!(result.launches.is_empty());
    assert_eq!(result.deferred_by_policy.len(), 1);
    assert_eq!(
        result.deferred_by_policy[0].decision.reason,
        "Agent-turn limit 3 has been reached."
    );
}
