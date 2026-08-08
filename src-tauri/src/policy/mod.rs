use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::domain::{
    PolicyDecision, PolicyDecisionId, PolicyDecisionKind, ProjectId, SchemaMetadata,
    WorkItemBudget, WorkItemId,
};

pub use crate::domain::{PolicyAction, ProtectedGitAction, ToolScope};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyLimits {
    pub max_parallel_executions: u32,
    pub max_agent_turns: Option<u32>,
    pub max_duration_seconds: Option<u64>,
    pub max_cost_micros: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicySet {
    pub allowed_tool_scopes: BTreeSet<ToolScope>,
    pub limits: PolicyLimits,
    pub protected_git_actions: BTreeSet<ProtectedGitAction>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyUsage {
    pub active_execution_count: u32,
    pub agent_turns: u32,
    pub duration_seconds: u64,
    pub cost_micros: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedGitApproval {
    pub decision_id: PolicyDecisionId,
    pub project_id: ProjectId,
    pub work_item_id: Option<WorkItemId>,
    pub action: ProtectedGitAction,
    pub approved_by: String,
    pub approved_at: String,
}

impl ProtectedGitApproval {
    fn permits(&self, request: &PolicyRequest, action: ProtectedGitAction) -> bool {
        !self.decision_id.0.trim().is_empty()
            && self.project_id == request.project_id
            && self.work_item_id == request.work_item_id
            && self.action == action
            && !self.approved_by.trim().is_empty()
            && !self.approved_at.trim().is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRequest {
    pub decision_id: PolicyDecisionId,
    pub project_id: ProjectId,
    pub work_item_id: Option<WorkItemId>,
    pub actor: String,
    pub action: PolicyAction,
    pub usage: PolicyUsage,
    pub work_item_budget: Option<WorkItemBudget>,
    pub protected_git_approval: Option<ProtectedGitApproval>,
    pub decided_at: String,
}

impl PolicyRequest {
    fn input_summary(&self) -> String {
        format!(
            "action={}; active_executions={}; agent_turns={}; duration_seconds={}; cost_micros={}",
            self.action,
            self.usage.active_execution_count,
            self.usage.agent_turns,
            self.usage.duration_seconds,
            self.usage.cost_micros,
        )
    }
}

pub trait PolicyAuditStore {
    type Error;

    fn record_policy_decision(&mut self, decision: PolicyDecision) -> Result<(), Self::Error>;

    fn has_recorded_protected_git_approval(
        &self,
        approval: &ProtectedGitApproval,
    ) -> Result<bool, Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedAction {
    action: PolicyAction,
}

impl AuthorizedAction {
    pub fn action(&self) -> &PolicyAction {
        &self.action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyAuthorization {
    decision: PolicyDecision,
    authorized_action: Option<AuthorizedAction>,
}

impl PolicyAuthorization {
    pub fn decision(&self) -> &PolicyDecision {
        &self.decision
    }

    pub fn authorized_action(&self) -> Option<&AuthorizedAction> {
        self.authorized_action.as_ref()
    }
}

pub struct PolicyGate {
    policy: PolicySet,
}

impl PolicyGate {
    pub fn new(policy: PolicySet) -> Self {
        Self { policy }
    }

    fn authorize(&self, request: PolicyRequest) -> PolicyAuthorization {
        let (decision, reason) = self.evaluate(&request);
        let is_allowed = decision == PolicyDecisionKind::Allow;
        let policy_decision = self.policy_decision(&request, decision, reason);
        let authorized_action = is_allowed.then_some(AuthorizedAction {
            action: request.action,
        });

        PolicyAuthorization {
            decision: policy_decision,
            authorized_action,
        }
    }

    pub fn authorize_and_record<AuditStore: PolicyAuditStore>(
        &self,
        request: PolicyRequest,
        audit_store: &mut AuditStore,
    ) -> Result<PolicyAuthorization, AuditStore::Error> {
        let request = self.request_with_verified_approval(request, audit_store)?;
        let authorization = self.authorize(request);
        audit_store.record_policy_decision(authorization.decision.clone())?;

        Ok(authorization)
    }

    fn request_with_verified_approval<AuditStore: PolicyAuditStore>(
        &self,
        mut request: PolicyRequest,
        audit_store: &AuditStore,
    ) -> Result<PolicyRequest, AuditStore::Error> {
        if let Some(approval) = request.protected_git_approval.as_ref()
            && !audit_store.has_recorded_protected_git_approval(approval)?
        {
            request.protected_git_approval = None;
        }

        Ok(request)
    }

    fn evaluate(&self, request: &PolicyRequest) -> (PolicyDecisionKind, String) {
        if let Some(reason) = self.limit_violation(request) {
            return (PolicyDecisionKind::Deny, reason);
        }

        match &request.action {
            PolicyAction::StartExecution => (
                PolicyDecisionKind::Allow,
                "Execution start is within the configured concurrency and budget limits."
                    .to_owned(),
            ),
            PolicyAction::Tool { scope } if self.policy.allowed_tool_scopes.contains(scope) => (
                PolicyDecisionKind::Allow,
                format!("Tool scope {scope} is allowed by the active policy."),
            ),
            PolicyAction::Tool { scope } => (
                PolicyDecisionKind::Deny,
                format!("Tool scope {scope} is not allowed by the active policy."),
            ),
            PolicyAction::ProtectedGit { action }
                if !self.policy.protected_git_actions.contains(action) =>
            {
                (
                    PolicyDecisionKind::Deny,
                    format!("Protected Git action {action} is not enabled by the active policy."),
                )
            }
            PolicyAction::ProtectedGit { action }
                if request
                    .protected_git_approval
                    .as_ref()
                    .is_some_and(|approval| approval.permits(request, *action)) =>
            {
                (
                    PolicyDecisionKind::Allow,
                    format!("Protected Git action {action} has a matching recorded approval."),
                )
            }
            PolicyAction::ProtectedGit { action } => (
                PolicyDecisionKind::ApprovalRequired,
                format!("Protected Git action {action} requires a matching user approval."),
            ),
        }
    }

    fn limit_violation(&self, request: &PolicyRequest) -> Option<String> {
        if matches!(request.action, PolicyAction::StartExecution)
            && request.usage.active_execution_count >= self.policy.limits.max_parallel_executions
        {
            return Some(format!(
                "Parallel execution limit {} has been reached.",
                self.policy.limits.max_parallel_executions
            ));
        }

        if let Some(limit) = effective_u32_limit(
            self.policy.limits.max_agent_turns,
            request
                .work_item_budget
                .as_ref()
                .and_then(|budget| budget.max_agent_turns),
        )
        .filter(|limit| request.usage.agent_turns >= *limit)
        {
            return Some(format!("Agent-turn limit {limit} has been reached."));
        }

        if let Some(limit) = effective_u64_limit(
            self.policy.limits.max_duration_seconds,
            request
                .work_item_budget
                .as_ref()
                .and_then(|budget| budget.max_duration_seconds),
        )
        .filter(|limit| request.usage.duration_seconds >= *limit)
        {
            return Some(format!("Duration limit {limit} seconds has been reached."));
        }

        effective_u64_limit(
            self.policy.limits.max_cost_micros,
            request
                .work_item_budget
                .as_ref()
                .and_then(|budget| budget.max_cost_micros),
        )
        .filter(|limit| request.usage.cost_micros >= *limit)
        .map(|limit| format!("Cost limit {limit} micros has been reached."))
    }

    fn policy_decision(
        &self,
        request: &PolicyRequest,
        decision: PolicyDecisionKind,
        reason: String,
    ) -> PolicyDecision {
        let outcome_summary = match decision {
            PolicyDecisionKind::Allow => "Execution may proceed.",
            PolicyDecisionKind::Deny => "Execution is blocked by policy.",
            PolicyDecisionKind::ApprovalRequired => "Execution is paused pending user approval.",
        };

        PolicyDecision {
            schema: SchemaMetadata::current(),
            id: request.decision_id.clone(),
            project_id: request.project_id.clone(),
            work_item_id: request.work_item_id.clone(),
            action: Some(request.action.clone()),
            decision,
            actor: request.actor.clone(),
            input_summary: request.input_summary(),
            outcome_summary: outcome_summary.to_owned(),
            reason,
            decided_at: request.decided_at.clone(),
        }
    }
}

fn effective_u32_limit(policy_limit: Option<u32>, work_item_limit: Option<u32>) -> Option<u32> {
    match (policy_limit, work_item_limit) {
        (Some(policy_limit), Some(work_item_limit)) => Some(policy_limit.min(work_item_limit)),
        (Some(policy_limit), None) => Some(policy_limit),
        (None, Some(work_item_limit)) => Some(work_item_limit),
        (None, None) => None,
    }
}

fn effective_u64_limit(policy_limit: Option<u64>, work_item_limit: Option<u64>) -> Option<u64> {
    match (policy_limit, work_item_limit) {
        (Some(policy_limit), Some(work_item_limit)) => Some(policy_limit.min(work_item_limit)),
        (Some(policy_limit), None) => Some(policy_limit),
        (None, Some(work_item_limit)) => Some(work_item_limit),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests;
