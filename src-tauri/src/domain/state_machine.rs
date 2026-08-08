use std::{error::Error, fmt};

use super::WorkItemState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TransitionConfig {
    pub human_review_required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionEvidence {
    pub checks_passed: bool,
    pub completion_report_present: bool,
    pub review_accepted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    Invalid {
        from: WorkItemState,
        to: WorkItemState,
    },
    IncompleteEvidence,
    HumanReviewRequired,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { from, to } => {
                write!(
                    formatter,
                    "transition from {from:?} to {to:?} is not allowed"
                )
            }
            Self::IncompleteEvidence => write!(
                formatter,
                "done requires passing checks and a completion report"
            ),
            Self::HumanReviewRequired => {
                write!(formatter, "done requires an accepted human review")
            }
        }
    }
}

impl Error for TransitionError {}

pub fn transition_work_item(
    current: WorkItemState,
    next: WorkItemState,
    config: TransitionConfig,
    evidence: Option<CompletionEvidence>,
) -> Result<WorkItemState, TransitionError> {
    if !is_allowed_transition(current, next) {
        return Err(TransitionError::Invalid {
            from: current,
            to: next,
        });
    }

    if next == WorkItemState::Done {
        validate_completion(config, evidence)?;
    }

    Ok(next)
}

fn is_allowed_transition(current: WorkItemState, next: WorkItemState) -> bool {
    use WorkItemState::{
        AwaitingInput, Blocked, Cancelled, Done, Failed, Inbox, Interrupted, Planned, Ready,
        Review, Running,
    };

    matches!(
        (current, next),
        (Inbox, Planned | Cancelled)
            | (Planned, Ready | Blocked | Cancelled)
            | (Ready, Running | Blocked | Cancelled)
            | (
                Running,
                AwaitingInput | Review | Blocked | Failed | Interrupted | Cancelled
            )
            | (
                AwaitingInput,
                Running | Blocked | Failed | Interrupted | Cancelled
            )
            | (
                Review,
                Done | Running | Blocked | Failed | Interrupted | Cancelled
            )
            | (Blocked, Planned | Ready | Cancelled)
            | (Failed, Ready | Cancelled)
            | (Interrupted, Ready | Cancelled)
    )
}

fn validate_completion(
    config: TransitionConfig,
    evidence: Option<CompletionEvidence>,
) -> Result<(), TransitionError> {
    let Some(evidence) = evidence else {
        return Err(TransitionError::IncompleteEvidence);
    };

    if !evidence.checks_passed || !evidence.completion_report_present {
        return Err(TransitionError::IncompleteEvidence);
    }

    if config.human_review_required && !evidence.review_accepted {
        return Err(TransitionError::HumanReviewRequired);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CompletionEvidence, TransitionConfig, TransitionError, transition_work_item};
    use crate::domain::WorkItemState;

    const AUTOMATED_REVIEW: TransitionConfig = TransitionConfig {
        human_review_required: false,
    };
    const HUMAN_REVIEW: TransitionConfig = TransitionConfig {
        human_review_required: true,
    };
    const COMPLETE_EVIDENCE: CompletionEvidence = CompletionEvidence {
        checks_passed: true,
        completion_report_present: true,
        review_accepted: true,
    };

    #[test]
    fn allows_each_non_completion_transition_in_the_lifecycle() {
        let transitions = [
            (WorkItemState::Inbox, WorkItemState::Planned),
            (WorkItemState::Planned, WorkItemState::Ready),
            (WorkItemState::Planned, WorkItemState::Blocked),
            (WorkItemState::Ready, WorkItemState::Running),
            (WorkItemState::Ready, WorkItemState::Blocked),
            (WorkItemState::Running, WorkItemState::AwaitingInput),
            (WorkItemState::Running, WorkItemState::Review),
            (WorkItemState::Running, WorkItemState::Blocked),
            (WorkItemState::Running, WorkItemState::Failed),
            (WorkItemState::Running, WorkItemState::Interrupted),
            (WorkItemState::AwaitingInput, WorkItemState::Running),
            (WorkItemState::AwaitingInput, WorkItemState::Blocked),
            (WorkItemState::AwaitingInput, WorkItemState::Failed),
            (WorkItemState::AwaitingInput, WorkItemState::Interrupted),
            (WorkItemState::Review, WorkItemState::Running),
            (WorkItemState::Review, WorkItemState::Blocked),
            (WorkItemState::Review, WorkItemState::Failed),
            (WorkItemState::Review, WorkItemState::Interrupted),
            (WorkItemState::Blocked, WorkItemState::Planned),
            (WorkItemState::Blocked, WorkItemState::Ready),
            (WorkItemState::Failed, WorkItemState::Ready),
            (WorkItemState::Interrupted, WorkItemState::Ready),
        ];

        for (current, next) in transitions {
            assert_eq!(
                transition_work_item(current, next, AUTOMATED_REVIEW, None),
                Ok(next)
            );
        }
    }

    #[test]
    fn permits_cancellation_from_every_nonterminal_lifecycle_state() {
        let cancellable_states = [
            WorkItemState::Inbox,
            WorkItemState::Planned,
            WorkItemState::Ready,
            WorkItemState::Running,
            WorkItemState::AwaitingInput,
            WorkItemState::Review,
            WorkItemState::Blocked,
            WorkItemState::Failed,
            WorkItemState::Interrupted,
        ];

        for current in cancellable_states {
            assert_eq!(
                transition_work_item(current, WorkItemState::Cancelled, AUTOMATED_REVIEW, None,),
                Ok(WorkItemState::Cancelled)
            );
        }
    }

    #[test]
    fn permits_done_only_from_review_with_declared_evidence() {
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                AUTOMATED_REVIEW,
                Some(COMPLETE_EVIDENCE),
            ),
            Ok(WorkItemState::Done)
        );
    }

    #[test]
    fn rejects_illegal_or_terminal_state_transitions() {
        assert_eq!(
            transition_work_item(
                WorkItemState::Inbox,
                WorkItemState::Done,
                AUTOMATED_REVIEW,
                Some(COMPLETE_EVIDENCE),
            ),
            Err(TransitionError::Invalid {
                from: WorkItemState::Inbox,
                to: WorkItemState::Done,
            })
        );
        assert_eq!(
            transition_work_item(
                WorkItemState::Done,
                WorkItemState::Ready,
                AUTOMATED_REVIEW,
                None,
            ),
            Err(TransitionError::Invalid {
                from: WorkItemState::Done,
                to: WorkItemState::Ready,
            })
        );
        assert_eq!(
            transition_work_item(
                WorkItemState::Cancelled,
                WorkItemState::Ready,
                AUTOMATED_REVIEW,
                None,
            ),
            Err(TransitionError::Invalid {
                from: WorkItemState::Cancelled,
                to: WorkItemState::Ready,
            })
        );
    }

    #[test]
    fn requires_passing_checks_and_a_completion_report_before_done() {
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                AUTOMATED_REVIEW,
                None,
            ),
            Err(TransitionError::IncompleteEvidence)
        );
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                AUTOMATED_REVIEW,
                Some(CompletionEvidence {
                    checks_passed: false,
                    completion_report_present: true,
                    review_accepted: false,
                }),
            ),
            Err(TransitionError::IncompleteEvidence)
        );
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                AUTOMATED_REVIEW,
                Some(CompletionEvidence {
                    checks_passed: true,
                    completion_report_present: false,
                    review_accepted: false,
                }),
            ),
            Err(TransitionError::IncompleteEvidence)
        );
    }

    #[test]
    fn requires_an_accepted_human_review_when_policy_requires_it() {
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                HUMAN_REVIEW,
                Some(CompletionEvidence {
                    review_accepted: false,
                    ..COMPLETE_EVIDENCE
                }),
            ),
            Err(TransitionError::HumanReviewRequired)
        );
        assert_eq!(
            transition_work_item(
                WorkItemState::Review,
                WorkItemState::Done,
                HUMAN_REVIEW,
                Some(COMPLETE_EVIDENCE),
            ),
            Ok(WorkItemState::Done)
        );
    }

    #[test]
    fn reports_actionable_transition_errors() {
        assert_eq!(
            TransitionError::IncompleteEvidence.to_string(),
            "done requires passing checks and a completion report"
        );
        assert_eq!(
            TransitionError::HumanReviewRequired.to_string(),
            "done requires an accepted human review"
        );
        assert_eq!(
            TransitionError::Invalid {
                from: WorkItemState::Inbox,
                to: WorkItemState::Ready,
            }
            .to_string(),
            "transition from Inbox to Ready is not allowed"
        );
    }
}
