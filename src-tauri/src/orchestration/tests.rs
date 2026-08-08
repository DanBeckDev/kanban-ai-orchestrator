use std::{collections::BTreeMap, convert::Infallible};

use crate::{
    domain::{
        BoardId, Dependency, DependencyId, DependencyKind, DependencySource, PlanId,
        PolicyDecision, PolicyDecisionId, ProjectId, SchemaMetadata, WorkItem, WorkItemBudget,
        WorkItemId, WorkItemProgress, WorkItemState,
    },
    policy::{PolicyAuditStore, PolicyGate, PolicyLimits, PolicySet, PolicyUsage},
};

use super::{
    DaemonScheduler, PlanConfirmation, PlanConfirmationError, PlanProposal, PlanProposalError,
    SchedulerError, SchedulerTick,
};

#[derive(Default)]
struct InMemoryPolicyAudit {
    decisions: Vec<PolicyDecision>,
}

impl PolicyAuditStore for InMemoryPolicyAudit {
    type Error = Infallible;

    fn record_policy_decision(&mut self, decision: PolicyDecision) -> Result<(), Self::Error> {
        self.decisions.push(decision);
        Ok(())
    }

    fn has_recorded_protected_git_approval(
        &self,
        _approval: &crate::policy::ProtectedGitApproval,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

fn work_item(id: &str, budget: WorkItemBudget) -> WorkItem {
    WorkItem {
        schema: SchemaMetadata::current(),
        id: WorkItemId::from(id),
        board_id: BoardId::from("board-1"),
        title: format!("Implement {id}"),
        description: format!("Deliver the {id} work item."),
        acceptance_criteria: vec![format!("{id} behavior is verified")],
        budget,
        state: WorkItemState::Ready,
        requires_human_review: false,
    }
}

fn dependency(id: &str, upstream: &str, downstream: &str, kind: DependencyKind) -> Dependency {
    Dependency {
        schema: SchemaMetadata::current(),
        id: DependencyId::from(id),
        upstream_work_item_id: WorkItemId::from(upstream),
        downstream_work_item_id: WorkItemId::from(downstream),
        kind,
        source: DependencySource::Orchestrator,
        reason: "The downstream implementation needs the upstream contract.".to_owned(),
        owner: "orchestrator".to_owned(),
        next_action: "Complete the upstream work item and attach evidence.".to_owned(),
        created_by: "orchestrator".to_owned(),
        created_at: "2026-08-08T15:00:00Z".to_owned(),
    }
}

fn proposal(work_items: Vec<WorkItem>, dependencies: Vec<Dependency>) -> PlanProposal {
    PlanProposal {
        id: PlanId::from("plan-1"),
        project_id: ProjectId::from("project-1"),
        work_items,
        dependencies,
        unresolved_assumptions: vec![
            "The repository default branch is available locally.".to_owned(),
        ],
    }
}

fn confirmed_scheduler(
    work_items: Vec<WorkItem>,
    dependencies: Vec<Dependency>,
) -> DaemonScheduler {
    let mut scheduler = DaemonScheduler::propose(proposal(work_items, dependencies))
        .expect("proposal should be valid");
    scheduler
        .confirm(PlanConfirmation {
            plan_id: PlanId::from("plan-1"),
            confirmed_by: "Daniel".to_owned(),
            confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
        })
        .expect("confirmation should match the proposal");
    scheduler
}

fn progress(state: WorkItemState, completion_evidence_accepted: bool) -> WorkItemProgress {
    WorkItemProgress {
        state,
        completion_evidence_accepted,
        review_accepted: false,
    }
}

fn policy_gate(max_parallel_executions: u32) -> PolicyGate {
    PolicyGate::new(PolicySet {
        limits: PolicyLimits {
            max_parallel_executions,
            ..PolicyLimits::default()
        },
        ..PolicySet::default()
    })
}

fn tick(
    progress_by_work_item: BTreeMap<WorkItemId, WorkItemProgress>,
    decision_ids: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> SchedulerTick {
    SchedulerTick {
        actor: "daemon-scheduler".to_owned(),
        decided_at: "2026-08-08T15:02:00Z".to_owned(),
        active_repository_execution_count: 0,
        repository_execution_capacity: 2,
        progress_by_work_item,
        usage_by_work_item: BTreeMap::new(),
        decision_ids: decision_ids
            .into_iter()
            .map(|(work_item_id, decision_id)| {
                (
                    WorkItemId::from(work_item_id),
                    PolicyDecisionId::from(decision_id),
                )
            })
            .collect(),
    }
}

fn fully_budgeted_work_item(id: &str) -> WorkItem {
    work_item(
        id,
        WorkItemBudget {
            max_agent_turns: Some(3),
            max_duration_seconds: Some(60),
            max_cost_micros: Some(100),
        },
    )
}

#[test]
fn proposal_exposes_the_full_confirmable_execution_plan() {
    let scheduler = DaemonScheduler::propose(proposal(
        vec![
            fully_budgeted_work_item("foundation"),
            fully_budgeted_work_item("api"),
            fully_budgeted_work_item("ui"),
            work_item(
                "release",
                WorkItemBudget {
                    max_agent_turns: None,
                    max_duration_seconds: Some(90),
                    max_cost_micros: None,
                },
            ),
        ],
        vec![
            dependency(
                "foundation-api",
                "foundation",
                "api",
                DependencyKind::Blocks,
            ),
            dependency("foundation-ui", "foundation", "ui", DependencyKind::Blocks),
            dependency(
                "api-release",
                "api",
                "release",
                DependencyKind::ReviewRequired,
            ),
            dependency("ui-release-note", "ui", "release", DependencyKind::Soft),
        ],
    ))
    .expect("proposal should be valid");

    assert_eq!(
        scheduler
            .preview()
            .work_items
            .iter()
            .map(|work_item| work_item.id.clone())
            .collect::<Vec<_>>(),
        vec![
            WorkItemId::from("api"),
            WorkItemId::from("foundation"),
            WorkItemId::from("release"),
            WorkItemId::from("ui")
        ]
    );
    assert_eq!(
        scheduler.preview().critical_path,
        vec![
            WorkItemId::from("foundation"),
            WorkItemId::from("api"),
            WorkItemId::from("release")
        ]
    );
    assert_eq!(
        scheduler
            .preview()
            .dependencies
            .iter()
            .map(|dependency| (dependency.id.clone(), dependency.kind))
            .collect::<Vec<_>>(),
        vec![
            (
                DependencyId::from("api-release"),
                DependencyKind::ReviewRequired
            ),
            (DependencyId::from("foundation-api"), DependencyKind::Blocks),
            (DependencyId::from("foundation-ui"), DependencyKind::Blocks),
            (DependencyId::from("ui-release-note"), DependencyKind::Soft),
        ]
    );
    assert_eq!(
        scheduler.preview().work_items[0].acceptance_criteria,
        vec!["api behavior is verified"]
    );
    assert_eq!(
        scheduler.preview().parallel_stages,
        vec![
            vec![WorkItemId::from("foundation")],
            vec![WorkItemId::from("api"), WorkItemId::from("ui")],
            vec![WorkItemId::from("release")]
        ]
    );
    assert_eq!(scheduler.preview().budget.max_agent_turns, None);
    assert_eq!(scheduler.preview().budget.max_duration_seconds, Some(270));
    assert_eq!(scheduler.preview().budget.max_cost_micros, None);
    assert_eq!(
        scheduler
            .preview()
            .budget
            .work_items_missing_agent_turn_budget,
        vec![WorkItemId::from("release")]
    );
    assert_eq!(
        scheduler.preview().budget.work_items_missing_cost_budget,
        vec![WorkItemId::from("release")]
    );
    assert_eq!(
        scheduler.preview().unresolved_assumptions,
        vec!["The repository default branch is available locally."]
    );
}

#[test]
fn confirmation_rejects_a_different_plan() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let wrong_plan = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("other-plan"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    });

    assert!(matches!(
        wrong_plan,
        Err(PlanConfirmationError::PlanIdMismatch { .. })
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_requires_a_user_identity() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let blank_user = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: " ".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    });

    assert!(matches!(
        blank_user,
        Err(PlanConfirmationError::MissingConfirmedBy)
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_requires_a_timestamp() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let blank_time = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: String::new(),
    });

    assert!(matches!(
        blank_time,
        Err(PlanConfirmationError::MissingConfirmedAt)
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_uses_the_application_boundary_field_names() {
    let confirmation = PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    };

    let serialized = serde_json::to_value(confirmation).expect("confirmation should serialize");

    assert_eq!(serialized["planId"], "plan-1");
    assert_eq!(serialized["confirmedBy"], "Daniel");
    assert_eq!(serialized["confirmedAt"], "2026-08-08T15:01:00Z");
}

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
    let blank_id_result = scheduler.schedule(
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
        blank_id_result,
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
    let duplicate_id_result = scheduler.schedule(
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
        duplicate_id_result,
        Err(SchedulerError::DuplicatePolicyDecisionId { .. })
    ));
    assert!(audit_store.decisions.is_empty());
}

#[test]
fn proposal_requires_at_least_one_work_item() {
    assert!(matches!(
        DaemonScheduler::propose(proposal(vec![], vec![])),
        Err(PlanProposalError::EmptyPlan)
    ));
}

#[test]
fn proposal_rejects_duplicate_work_item_ids() {
    assert!(matches!(
        DaemonScheduler::propose(proposal(
            vec![
                fully_budgeted_work_item("same"),
                fully_budgeted_work_item("same")
            ],
            vec![],
        )),
        Err(PlanProposalError::DuplicateWorkItemId { .. })
    ));
}

#[test]
fn proposal_rejects_blank_unresolved_assumptions() {
    let mut invalid_assumption = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    invalid_assumption.unresolved_assumptions = vec!["\t".to_owned()];
    assert!(matches!(
        DaemonScheduler::propose(invalid_assumption),
        Err(PlanProposalError::BlankUnresolvedAssumption)
    ));
}

#[test]
fn proposal_requires_a_nonblank_plan_id() {
    let mut missing_plan_id = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    missing_plan_id.id = PlanId::from(" ");
    assert!(matches!(
        DaemonScheduler::propose(missing_plan_id),
        Err(PlanProposalError::MissingPlanId)
    ));
}

#[test]
fn proposal_requires_a_nonblank_project_id() {
    let mut missing_project_id = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    missing_project_id.project_id = ProjectId::from("");
    assert!(matches!(
        DaemonScheduler::propose(missing_project_id),
        Err(PlanProposalError::MissingProjectId)
    ));
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
