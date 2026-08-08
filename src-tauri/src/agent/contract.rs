use std::{error::Error, fmt};

use crate::domain::TransitionError;

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
            Self::UnknownSession { session_id } => {
                write!(
                    formatter,
                    "agent session {session_id} is not known to this adapter"
                )
            }
            Self::InvalidWorkItemTransition(error) => {
                write!(
                    formatter,
                    "agent event cannot change the work item: {error}"
                )
            }
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
