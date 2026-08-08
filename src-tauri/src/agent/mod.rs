use std::{collections::BTreeMap, error::Error, fmt};

use crate::domain::{TransitionConfig, TransitionError, WorkItemState, transition_work_item};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AgentCapabilities {
    pub supports_feedback: bool,
    pub supports_interrupt: bool,
    pub supports_resume: bool,
    pub streams_structured_events: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentSession {
    pub id: String,
    pub resumable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartAgentRequest {
    pub work_item_id: String,
    pub workspace_path: String,
    pub task_brief: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedAgentEventKind {
    Activity {
        summary: String,
    },
    ApprovalRequested {
        question: String,
    },
    AwaitingInput {
        question: String,
    },
    AwaitingReview {
        summary: String,
    },
    Completed {
        summary: String,
    },
    Failed {
        reason: String,
    },
    Interrupted {
        reason: String,
    },
    UsageUpdated {
        input_tokens: u64,
        output_tokens: u64,
        cost_micros: Option<u64>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedAgentEvent {
    pub sequence: u64,
    pub kind: NormalizedAgentEventKind,
}

pub trait AgentAdapter {
    fn name(&self) -> &str;
    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError>;
    fn start(&mut self, request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError>;
    fn resume(&mut self, session_id: &str) -> Result<AgentSession, AgentAdapterError>;
    fn send_feedback(&mut self, session_id: &str, feedback: &str) -> Result<(), AgentAdapterError>;
    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError>;
    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError>;
    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError>;
    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentAdapterError {
    CapabilityUnsupported { capability: &'static str },
    DuplicateEvent { sequence: u64 },
    EventOutOfOrder { expected: u64, received: u64 },
    UnknownSession { session_id: String },
    InvalidWorkItemTransition(TransitionError),
}

impl fmt::Display for AgentAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapabilityUnsupported { capability } => {
                write!(formatter, "agent adapter does not support {capability}")
            }
            Self::DuplicateEvent { sequence } => {
                write!(
                    formatter,
                    "agent event sequence {sequence} was already processed"
                )
            }
            Self::EventOutOfOrder { expected, received } => write!(
                formatter,
                "agent event sequence {received} is out of order; expected {expected}"
            ),
            Self::UnknownSession { session_id } => write!(
                formatter,
                "agent session {session_id} is not known to this adapter"
            ),
            Self::InvalidWorkItemTransition(error) => write!(
                formatter,
                "agent event cannot change the work item: {error}"
            ),
        }
    }
}

impl Error for AgentAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidWorkItemTransition(error) => Some(error),
            Self::CapabilityUnsupported { .. }
            | Self::DuplicateEvent { .. }
            | Self::EventOutOfOrder { .. }
            | Self::UnknownSession { .. } => None,
        }
    }
}

pub struct AgentEventIngestor {
    last_sequence: u64,
}

impl AgentEventIngestor {
    pub fn new(last_sequence: u64) -> Self {
        Self { last_sequence }
    }

    pub fn apply_to_work_item(
        &mut self,
        current_state: WorkItemState,
        event: &NormalizedAgentEvent,
        config: TransitionConfig,
    ) -> Result<WorkItemState, AgentAdapterError> {
        self.validate_sequence(event.sequence)?;
        let next_state = match proposed_work_item_state(&event.kind) {
            Some(next_state) => transition_work_item(current_state, next_state, config, None)
                .map_err(AgentAdapterError::InvalidWorkItemTransition)?,
            None => current_state,
        };
        self.last_sequence = event.sequence;
        Ok(next_state)
    }

    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }

    fn validate_sequence(&self, sequence: u64) -> Result<(), AgentAdapterError> {
        if sequence <= self.last_sequence {
            return Err(AgentAdapterError::DuplicateEvent { sequence });
        }
        let expected = self.last_sequence.saturating_add(1);
        if sequence != expected {
            return Err(AgentAdapterError::EventOutOfOrder {
                expected,
                received: sequence,
            });
        }

        Ok(())
    }
}

fn proposed_work_item_state(event: &NormalizedAgentEventKind) -> Option<WorkItemState> {
    match event {
        NormalizedAgentEventKind::ApprovalRequested { .. }
        | NormalizedAgentEventKind::AwaitingInput { .. } => Some(WorkItemState::AwaitingInput),
        NormalizedAgentEventKind::AwaitingReview { .. }
        | NormalizedAgentEventKind::Completed { .. } => Some(WorkItemState::Review),
        NormalizedAgentEventKind::Failed { .. } => Some(WorkItemState::Failed),
        NormalizedAgentEventKind::Interrupted { .. } => Some(WorkItemState::Interrupted),
        NormalizedAgentEventKind::Activity { .. }
        | NormalizedAgentEventKind::UsageUpdated { .. } => None,
    }
}

pub struct FakeAgentAdapter {
    name: String,
    capabilities: AgentCapabilities,
    next_session_number: u64,
    sessions: BTreeMap<String, FakeSession>,
}

#[derive(Default)]
struct FakeSession {
    events: Vec<NormalizedAgentEvent>,
    feedback: Vec<String>,
    interrupted: bool,
    terminated: bool,
}

impl FakeAgentAdapter {
    pub fn new(name: impl Into<String>, capabilities: AgentCapabilities) -> Self {
        Self {
            name: name.into(),
            capabilities,
            next_session_number: 1,
            sessions: BTreeMap::new(),
        }
    }

    pub fn queue_event(
        &mut self,
        session_id: &str,
        event: NormalizedAgentEvent,
    ) -> Result<(), AgentAdapterError> {
        self.session_mut(session_id)?.events.push(event);
        Ok(())
    }

    pub fn feedback(&self, session_id: &str) -> Result<&[String], AgentAdapterError> {
        Ok(&self.session(session_id)?.feedback)
    }

    fn session(&self, session_id: &str) -> Result<&FakeSession, AgentAdapterError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| AgentAdapterError::UnknownSession {
                session_id: session_id.to_owned(),
            })
    }

    fn session_mut(&mut self, session_id: &str) -> Result<&mut FakeSession, AgentAdapterError> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentAdapterError::UnknownSession {
                session_id: session_id.to_owned(),
            })
    }
}

impl AgentAdapter for FakeAgentAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(self.capabilities)
    }

    fn start(&mut self, _request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError> {
        let id = format!("fake-session-{}", self.next_session_number);
        self.next_session_number += 1;
        self.sessions.insert(id.clone(), FakeSession::default());
        Ok(AgentSession {
            id,
            resumable: self.capabilities.supports_resume,
        })
    }

    fn resume(&mut self, session_id: &str) -> Result<AgentSession, AgentAdapterError> {
        if !self.capabilities.supports_resume {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "resume",
            });
        }
        self.session(session_id)?;
        Ok(AgentSession {
            id: session_id.to_owned(),
            resumable: true,
        })
    }

    fn send_feedback(&mut self, session_id: &str, feedback: &str) -> Result<(), AgentAdapterError> {
        if !self.capabilities.supports_feedback {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "feedback",
            });
        }
        self.session_mut(session_id)?
            .feedback
            .push(feedback.to_owned());
        Ok(())
    }

    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        if !self.capabilities.supports_interrupt {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "interrupt",
            });
        }
        self.session_mut(session_id)?.interrupted = true;
        Ok(())
    }

    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.session_mut(session_id)?.terminated = true;
        Ok(())
    }

    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError> {
        Ok(self
            .session(session_id)?
            .events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect())
    }

    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError> {
        let session = self.session(session_id)?;
        if session.terminated {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "health check for a terminated session",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{
        AgentAdapter, AgentAdapterError, AgentCapabilities, AgentEventIngestor, FakeAgentAdapter,
        NormalizedAgentEvent, NormalizedAgentEventKind, StartAgentRequest,
    };
    use crate::domain::{TransitionConfig, WorkItemState};

    const ALL_CAPABILITIES: AgentCapabilities = AgentCapabilities {
        supports_feedback: true,
        supports_interrupt: true,
        supports_resume: true,
        streams_structured_events: true,
    };

    fn request() -> StartAgentRequest {
        StartAgentRequest {
            work_item_id: "work-item-1".to_owned(),
            workspace_path: "/workspaces/work-item-1".to_owned(),
            task_brief: "Add the adapter contract.".to_owned(),
        }
    }

    fn event(sequence: u64, kind: NormalizedAgentEventKind) -> NormalizedAgentEvent {
        NormalizedAgentEvent { sequence, kind }
    }

    #[test]
    fn fake_adapter_discovers_capabilities_and_runs_a_resumable_session() {
        let mut adapter = FakeAgentAdapter::new("fake", ALL_CAPABILITIES);

        assert_eq!(adapter.name(), "fake");
        assert_eq!(adapter.discover(), Ok(ALL_CAPABILITIES));
        let session = adapter.start(request()).expect("session should start");
        assert_eq!(session.id, "fake-session-1");
        assert!(session.resumable);
        assert_eq!(
            adapter.resume(&session.id),
            Ok(session.clone()),
            "resume retains the provider session identity"
        );
        adapter
            .send_feedback(&session.id, "Please add a regression test.")
            .expect("feedback should be supported");
        let expected_feedback = ["Please add a regression test.".to_owned()];
        assert_eq!(
            adapter.feedback(&session.id),
            Ok(expected_feedback.as_slice())
        );
        adapter
            .interrupt(&session.id)
            .expect("interruption should be supported");
        adapter
            .health_check(&session.id)
            .expect("an interrupted session can still report health");
        adapter
            .terminate(&session.id)
            .expect("termination should be available");
        assert!(matches!(
            adapter.health_check(&session.id),
            Err(AgentAdapterError::CapabilityUnsupported { .. })
        ));
    }

    #[test]
    fn fake_adapter_reports_unsupported_capabilities_and_unknown_sessions() {
        let mut adapter = FakeAgentAdapter::new("limited", AgentCapabilities::default());
        let session = adapter.start(request()).expect("session should start");

        assert!(matches!(
            adapter.resume(&session.id),
            Err(AgentAdapterError::CapabilityUnsupported {
                capability: "resume"
            })
        ));
        assert!(matches!(
            adapter.send_feedback(&session.id, "feedback"),
            Err(AgentAdapterError::CapabilityUnsupported {
                capability: "feedback"
            })
        ));
        assert!(matches!(
            adapter.interrupt(&session.id),
            Err(AgentAdapterError::CapabilityUnsupported {
                capability: "interrupt"
            })
        ));
        assert!(matches!(
            adapter.stream_events("missing", 0),
            Err(AgentAdapterError::UnknownSession { .. })
        ));
        assert!(matches!(
            adapter.terminate("missing"),
            Err(AgentAdapterError::UnknownSession { .. })
        ));
    }

    #[test]
    fn fake_adapter_streams_normalized_completion_failure_approval_and_interruption_events() {
        let mut adapter = FakeAgentAdapter::new("fake", ALL_CAPABILITIES);
        let session = adapter.start(request()).expect("session should start");
        let events = [
            event(
                1,
                NormalizedAgentEventKind::ApprovalRequested {
                    question: "May I modify the lockfile?".to_owned(),
                },
            ),
            event(
                2,
                NormalizedAgentEventKind::Completed {
                    summary: "Implementation is ready for review.".to_owned(),
                },
            ),
            event(
                3,
                NormalizedAgentEventKind::Failed {
                    reason: "Check failed.".to_owned(),
                },
            ),
            event(
                4,
                NormalizedAgentEventKind::Interrupted {
                    reason: "Process exited unexpectedly.".to_owned(),
                },
            ),
        ];

        for event in events.clone() {
            adapter
                .queue_event(&session.id, event)
                .expect("event should be queued");
        }

        assert_eq!(adapter.stream_events(&session.id, 0), Ok(events.to_vec()));
        assert_eq!(
            adapter.stream_events(&session.id, 2),
            Ok(events[2..].to_vec())
        );
    }

    #[test]
    fn event_ingestor_maps_provider_events_to_guarded_nonterminal_work_item_states() {
        let mut ingestor = AgentEventIngestor::new(0);
        let config = TransitionConfig::default();

        let awaiting_input = ingestor
            .apply_to_work_item(
                WorkItemState::Running,
                &event(
                    1,
                    NormalizedAgentEventKind::AwaitingInput {
                        question: "Which API version should I use?".to_owned(),
                    },
                ),
                config,
            )
            .expect("input request should be guarded");
        assert_eq!(awaiting_input, WorkItemState::AwaitingInput);
        let review = ingestor
            .apply_to_work_item(
                WorkItemState::Running,
                &event(
                    2,
                    NormalizedAgentEventKind::Completed {
                        summary: "Ready.".to_owned(),
                    },
                ),
                config,
            )
            .expect("completion should require review");
        assert_eq!(review, WorkItemState::Review);
        let failed = ingestor
            .apply_to_work_item(
                WorkItemState::Running,
                &event(
                    3,
                    NormalizedAgentEventKind::Failed {
                        reason: "Compilation failed.".to_owned(),
                    },
                ),
                config,
            )
            .expect("failure should stay distinct from completion");
        assert_eq!(failed, WorkItemState::Failed);
        let interrupted = ingestor
            .apply_to_work_item(
                WorkItemState::Running,
                &event(
                    4,
                    NormalizedAgentEventKind::Interrupted {
                        reason: "Stopped.".to_owned(),
                    },
                ),
                config,
            )
            .expect("interruption should stay distinct from completion");
        assert_eq!(interrupted, WorkItemState::Interrupted);
        assert_eq!(ingestor.last_sequence(), 4);
    }

    #[test]
    fn event_ingestor_rejects_duplicates_and_out_of_order_events() {
        let mut ingestor = AgentEventIngestor::new(0);
        let activity = event(
            1,
            NormalizedAgentEventKind::Activity {
                summary: "Reading files.".to_owned(),
            },
        );

        assert_eq!(
            ingestor.apply_to_work_item(
                WorkItemState::Running,
                &activity,
                TransitionConfig::default()
            ),
            Ok(WorkItemState::Running)
        );
        assert_eq!(
            ingestor.apply_to_work_item(
                WorkItemState::Running,
                &activity,
                TransitionConfig::default()
            ),
            Err(AgentAdapterError::DuplicateEvent { sequence: 1 })
        );
        assert_eq!(
            ingestor.apply_to_work_item(
                WorkItemState::Running,
                &event(
                    3,
                    NormalizedAgentEventKind::UsageUpdated {
                        input_tokens: 10,
                        output_tokens: 20,
                        cost_micros: None,
                    },
                ),
                TransitionConfig::default(),
            ),
            Err(AgentAdapterError::EventOutOfOrder {
                expected: 2,
                received: 3
            })
        );
    }

    #[test]
    fn event_ingestor_rejects_an_illegal_transition_without_consuming_the_event() {
        let mut ingestor = AgentEventIngestor::new(0);
        let completed = event(
            1,
            NormalizedAgentEventKind::Completed {
                summary: "Ready.".to_owned(),
            },
        );

        assert!(matches!(
            ingestor.apply_to_work_item(
                WorkItemState::Ready,
                &completed,
                TransitionConfig::default()
            ),
            Err(AgentAdapterError::InvalidWorkItemTransition(_))
        ));
        assert_eq!(ingestor.last_sequence(), 0);
        assert_eq!(
            ingestor.apply_to_work_item(
                WorkItemState::Running,
                &completed,
                TransitionConfig::default()
            ),
            Ok(WorkItemState::Review)
        );
    }

    #[test]
    fn adapter_errors_explain_actionable_failures_and_preserve_wrapped_sources() {
        let transition_error = AgentAdapterError::InvalidWorkItemTransition(
            crate::domain::TransitionError::IncompleteEvidence,
        );

        assert!(
            transition_error
                .to_string()
                .contains("cannot change the work item")
        );
        assert!(transition_error.source().is_some());
        assert_eq!(
            AgentAdapterError::DuplicateEvent { sequence: 4 }.to_string(),
            "agent event sequence 4 was already processed"
        );
        assert!(
            AgentAdapterError::UnknownSession {
                session_id: "missing".to_owned()
            }
            .source()
            .is_none()
        );
    }
}
