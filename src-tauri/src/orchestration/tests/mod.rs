use std::{collections::BTreeMap, convert::Infallible};

use crate::{
    domain::{
        BoardId, Dependency, DependencyId, DependencyKind, DependencySource, PlanId,
        PolicyDecision, PolicyDecisionId, ProjectId, SchemaMetadata, WorkItem, WorkItemBudget,
        WorkItemId, WorkItemProgress, WorkItemState,
    },
    orchestration::{DaemonScheduler, PlanConfirmation, PlanProposal, SchedulerTick},
    policy::{PolicyAuditStore, PolicyGate, PolicyLimits, PolicySet},
};

pub(super) mod plan_tests;
pub(super) mod planner_tests;
pub(super) mod scheduler_tests;

#[derive(Default)]
pub(super) struct InMemoryPolicyAudit {
    pub(super) decisions: Vec<PolicyDecision>,
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

pub(super) fn work_item(id: &str, budget: WorkItemBudget) -> WorkItem {
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
        assigned_agent_profile_name: None,
        assigned_agent_model: Default::default(),
        assigned_agent_effort: Default::default(),
    }
}

pub(super) fn dependency(
    id: &str,
    upstream: &str,
    downstream: &str,
    kind: DependencyKind,
) -> Dependency {
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

pub(super) fn proposal(work_items: Vec<WorkItem>, dependencies: Vec<Dependency>) -> PlanProposal {
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

pub(super) fn confirmed_scheduler(
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

pub(super) fn progress(
    state: WorkItemState,
    completion_evidence_accepted: bool,
) -> WorkItemProgress {
    WorkItemProgress {
        state,
        completion_evidence_accepted,
        review_accepted: false,
    }
}

pub(super) fn policy_gate(max_parallel_executions: u32) -> PolicyGate {
    PolicyGate::new(PolicySet {
        limits: PolicyLimits {
            max_parallel_executions,
            ..PolicyLimits::default()
        },
        ..PolicySet::default()
    })
}

pub(super) fn tick(
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

pub(super) fn fully_budgeted_work_item(id: &str) -> WorkItem {
    work_item(
        id,
        WorkItemBudget {
            max_agent_turns: Some(3),
            max_duration_seconds: Some(60),
            max_cost_micros: Some(100),
        },
    )
}
