use crate::{
    domain::{PolicyAction, PolicyDecision, PolicyDecisionKind},
    policy::ProtectedGitApproval,
};

use super::{EventStoreError, SqliteEventStore};

pub(super) fn idempotent_policy_decision(
    recorded: PolicyDecision,
    requested: &PolicyDecision,
) -> Result<PolicyDecision, EventStoreError> {
    if recorded == *requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::PolicyDecisionIdConflict {
            decision_id: requested.id.clone(),
        })
    }
}

pub(super) fn idempotent_protected_git_approval(
    recorded: ProtectedGitApproval,
    requested: &ProtectedGitApproval,
) -> Result<ProtectedGitApproval, EventStoreError> {
    if recorded == *requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::ProtectedGitApprovalIdConflict {
            decision_id: requested.decision_id.clone(),
        })
    }
}

pub(super) fn policy_decision_matches_protected_git_approval(
    decision: &PolicyDecision,
    approval: &ProtectedGitApproval,
) -> bool {
    decision.id == approval.decision_id
        && decision.project_id == approval.project_id
        && decision.work_item_id == approval.work_item_id
        && decision.action
            == Some(PolicyAction::ProtectedGit {
                action: approval.action,
            })
        && decision.decision == PolicyDecisionKind::Allow
        && decision.actor == approval.approved_by
}

impl crate::policy::PolicyAuditStore for SqliteEventStore {
    type Error = EventStoreError;
    fn record_policy_decision(&mut self, decision: PolicyDecision) -> Result<(), Self::Error> {
        SqliteEventStore::record_policy_decision(self, decision).map(|_| ())
    }
    fn has_recorded_protected_git_approval(
        &self,
        approval: &ProtectedGitApproval,
    ) -> Result<bool, Self::Error> {
        SqliteEventStore::has_recorded_protected_git_approval(self, approval)
    }
}
