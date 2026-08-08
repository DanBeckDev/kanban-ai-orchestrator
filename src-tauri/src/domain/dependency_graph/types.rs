use crate::domain::{Dependency, WorkItemState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkItemProgress {
    pub state: WorkItemState,
    pub completion_evidence_accepted: bool,
    pub review_accepted: bool,
}

impl WorkItemProgress {
    pub(super) fn has_accepted_completion(self) -> bool {
        self.state == WorkItemState::Done && self.completion_evidence_accepted
    }

    pub(super) fn has_accepted_review(self) -> bool {
        self.review_accepted && matches!(self.state, WorkItemState::Review | WorkItemState::Done)
    }

    pub(super) fn is_ready_to_start(self) -> bool {
        self.state == WorkItemState::Ready
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyBlockerReason {
    UpstreamProgressUnavailable,
    CompletionEvidenceNotAccepted,
    ReviewNotAccepted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyContextField {
    Reason,
    Owner,
    NextAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyBlocker {
    pub dependency: Dependency,
    pub reason: DependencyBlockerReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEligibility {
    pub hard_blockers: Vec<DependencyBlocker>,
    pub advisories: Vec<Dependency>,
}

impl DependencyEligibility {
    pub const fn is_eligible(&self) -> bool {
        self.hard_blockers.is_empty()
    }
}
