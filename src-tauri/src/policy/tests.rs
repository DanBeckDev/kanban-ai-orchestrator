use std::collections::BTreeSet;

use crate::domain::{
    PolicyDecision, PolicyDecisionId, PolicyDecisionKind, ProjectId, WorkItemBudget, WorkItemId,
};

use super::{
    PolicyAction, PolicyAuditStore, PolicyGate, PolicyLimits, PolicyRequest, PolicySet,
    PolicyUsage, ProtectedGitAction, ProtectedGitApproval, ToolScope,
};

fn policy() -> PolicySet {
    PolicySet {
        allowed_tool_scopes: BTreeSet::from([
            ToolScope::ReadAssignedWorkspace,
            ToolScope::WriteAssignedWorkspace,
            ToolScope::RunProjectChecks,
        ]),
        limits: PolicyLimits {
            max_parallel_executions: 2,
            max_agent_turns: Some(12),
            max_duration_seconds: Some(1_800),
            max_cost_micros: Some(5_000_000),
        },
        protected_git_actions: BTreeSet::from([
            ProtectedGitAction::Commit,
            ProtectedGitAction::Push,
        ]),
    }
}

fn request(id: &str, action: PolicyAction) -> PolicyRequest {
    PolicyRequest {
        decision_id: PolicyDecisionId::from(id),
        project_id: ProjectId::from("project-1"),
        work_item_id: Some(WorkItemId::from("work-item-1")),
        actor: "worker-agent-1".to_owned(),
        action,
        usage: PolicyUsage::default(),
        work_item_budget: None,
        protected_git_approval: None,
        decided_at: "2026-08-08T14:00:00Z".to_owned(),
    }
}

fn approval(action: ProtectedGitAction) -> ProtectedGitApproval {
    ProtectedGitApproval {
        decision_id: PolicyDecisionId::from("user-approved-push"),
        project_id: ProjectId::from("project-1"),
        work_item_id: Some(WorkItemId::from("work-item-1")),
        action,
        approved_by: "Daniel".to_owned(),
        approved_at: "2026-08-08T14:01:00Z".to_owned(),
    }
}

#[test]
fn allows_an_explicitly_scoped_workspace_tool_and_returns_a_capability() {
    let authorization = PolicyGate::new(policy()).authorize(request(
        "allow-workspace-write",
        PolicyAction::Tool {
            scope: ToolScope::WriteAssignedWorkspace,
        },
    ));

    assert_eq!(authorization.decision().decision, PolicyDecisionKind::Allow);
    assert_eq!(
        authorization
            .authorized_action()
            .expect("allowed action should have a capability")
            .action(),
        &PolicyAction::Tool {
            scope: ToolScope::WriteAssignedWorkspace,
        }
    );
    assert_eq!(
        authorization.decision().outcome_summary,
        "Execution may proceed."
    );
}

#[test]
fn denies_an_unscoped_tool_without_consulting_agent_instruction_text() {
    let authorization = PolicyGate::new(policy()).authorize(request(
        "deny-network",
        PolicyAction::Tool {
            scope: ToolScope::NetworkAccess,
        },
    ));

    assert_eq!(authorization.decision().decision, PolicyDecisionKind::Deny);
    assert!(authorization.authorized_action().is_none());
    assert_eq!(
        authorization.decision().reason,
        "Tool scope network_access is not allowed by the active policy."
    );
    assert!(!authorization.decision().input_summary.contains("prompt"));
}

#[test]
fn denies_execution_start_when_parallel_capacity_is_exhausted() {
    let mut policy_request = request("deny-parallel-limit", PolicyAction::StartExecution);
    policy_request.usage.active_execution_count = 2;

    let authorization = PolicyGate::new(policy()).authorize(policy_request);

    assert_eq!(authorization.decision().decision, PolicyDecisionKind::Deny);
    assert!(authorization.authorized_action().is_none());
    assert_eq!(
        authorization.decision().reason,
        "Parallel execution limit 2 has been reached."
    );
}

#[test]
fn applies_the_strictest_project_or_work_item_budget_limit() {
    let mut policy_request = request(
        "deny-work-item-cost-limit",
        PolicyAction::Tool {
            scope: ToolScope::ReadAssignedWorkspace,
        },
    );
    policy_request.usage.cost_micros = 500;
    policy_request.work_item_budget = Some(WorkItemBudget {
        max_agent_turns: Some(8),
        max_duration_seconds: Some(900),
        max_cost_micros: Some(500),
    });

    let authorization = PolicyGate::new(policy()).authorize(policy_request);

    assert_eq!(authorization.decision().decision, PolicyDecisionKind::Deny);
    assert_eq!(
        authorization.decision().reason,
        "Cost limit 500 micros has been reached."
    );
}

#[test]
fn denies_agent_turn_and_duration_budget_exhaustion() {
    let mut agent_turn_request = request("deny-agent-turn-limit", PolicyAction::StartExecution);
    agent_turn_request.usage.agent_turns = 12;
    let mut duration_request = request(
        "deny-duration-limit",
        PolicyAction::Tool {
            scope: ToolScope::ReadAssignedWorkspace,
        },
    );
    duration_request.usage.duration_seconds = 1_800;
    let gate = PolicyGate::new(policy());

    let agent_turn_authorization = gate.authorize(agent_turn_request);
    let duration_authorization = gate.authorize(duration_request);

    assert_eq!(
        agent_turn_authorization.decision().reason,
        "Agent-turn limit 12 has been reached."
    );
    assert_eq!(
        duration_authorization.decision().reason,
        "Duration limit 1800 seconds has been reached."
    );
}

#[test]
fn requires_a_matching_approval_for_protected_git_actions() {
    let gate = PolicyGate::new(policy());
    let requested_action = PolicyAction::ProtectedGit {
        action: ProtectedGitAction::Push,
    };
    let required = gate.authorize(request("push-needs-approval", requested_action.clone()));
    let mut mismatched_request = request("push-wrong-approval", requested_action.clone());
    mismatched_request.protected_git_approval = Some(approval(ProtectedGitAction::Commit));
    let mismatched = gate.authorize(mismatched_request);
    let mut approved_request = request("push-approved", requested_action.clone());
    approved_request.protected_git_approval = Some(approval(ProtectedGitAction::Push));
    let approved = gate.authorize(approved_request);

    assert_eq!(
        required.decision().decision,
        PolicyDecisionKind::ApprovalRequired
    );
    assert!(required.authorized_action().is_none());
    assert_eq!(
        mismatched.decision().decision,
        PolicyDecisionKind::ApprovalRequired
    );
    assert_eq!(approved.decision().decision, PolicyDecisionKind::Allow);
    assert_eq!(
        approved.authorized_action().unwrap().action(),
        &requested_action
    );
}

#[test]
fn denies_protected_git_actions_that_are_not_enabled() {
    let authorization = PolicyGate::new(policy()).authorize(request(
        "deny-force-push",
        PolicyAction::ProtectedGit {
            action: ProtectedGitAction::ForcePush,
        },
    ));

    assert_eq!(authorization.decision().decision, PolicyDecisionKind::Deny);
    assert_eq!(
        authorization.decision().reason,
        "Protected Git action force_push is not enabled by the active policy."
    );
}

#[derive(Default)]
struct InMemoryRecorder {
    decisions: Vec<PolicyDecision>,
    approvals: Vec<ProtectedGitApproval>,
}

impl PolicyAuditStore for InMemoryRecorder {
    type Error = ();

    fn record_policy_decision(&mut self, decision: PolicyDecision) -> Result<(), Self::Error> {
        self.decisions.push(decision);
        Ok(())
    }

    fn has_recorded_protected_git_approval(
        &self,
        approval: &ProtectedGitApproval,
    ) -> Result<bool, Self::Error> {
        Ok(self.approvals.contains(approval))
    }
}

#[test]
fn records_allow_deny_and_approval_required_decisions_before_returning() {
    let gate = PolicyGate::new(policy());
    let mut recorder = InMemoryRecorder::default();
    let mut approved_write = request(
        "record-allow",
        PolicyAction::Tool {
            scope: ToolScope::WriteAssignedWorkspace,
        },
    );
    approved_write.actor = "orchestrator-1".to_owned();

    let allow = gate
        .authorize_and_record(approved_write, &mut recorder)
        .expect("allow decision should be recorded");
    let deny = gate
        .authorize_and_record(
            request(
                "record-deny",
                PolicyAction::Tool {
                    scope: ToolScope::NetworkAccess,
                },
            ),
            &mut recorder,
        )
        .expect("deny decision should be recorded");
    let approval_required = gate
        .authorize_and_record(
            request(
                "record-approval-required",
                PolicyAction::ProtectedGit {
                    action: ProtectedGitAction::Commit,
                },
            ),
            &mut recorder,
        )
        .expect("approval-required decision should be recorded");

    assert_eq!(allow.decision().actor, "orchestrator-1");
    assert_eq!(
        recorder
            .decisions
            .iter()
            .map(|decision| &decision.decision)
            .collect::<Vec<_>>(),
        vec![
            &PolicyDecisionKind::Allow,
            &PolicyDecisionKind::Deny,
            &PolicyDecisionKind::ApprovalRequired,
        ]
    );
    assert!(deny.authorized_action().is_none());
    assert!(approval_required.authorized_action().is_none());
}

#[test]
fn requires_an_approval_to_exist_in_the_audit_store_before_allowing_protected_git() {
    let gate = PolicyGate::new(policy());
    let mut policy_request = request(
        "audit-checked-push",
        PolicyAction::ProtectedGit {
            action: ProtectedGitAction::Push,
        },
    );
    let push_approval = approval(ProtectedGitAction::Push);
    policy_request.protected_git_approval = Some(push_approval.clone());
    let mut audit_store = InMemoryRecorder::default();

    let missing_approval = gate
        .authorize_and_record(policy_request.clone(), &mut audit_store)
        .expect("approval-required decision should be recorded");
    audit_store.approvals.push(push_approval);
    let recorded_approval = gate
        .authorize_and_record(policy_request, &mut audit_store)
        .expect("approved decision should be recorded");

    assert_eq!(
        missing_approval.decision().decision,
        PolicyDecisionKind::ApprovalRequired
    );
    assert_eq!(
        recorded_approval.decision().decision,
        PolicyDecisionKind::Allow
    );
}
