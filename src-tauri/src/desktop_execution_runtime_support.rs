use std::{
    error::Error,
    fmt,
    sync::{Mutex, MutexGuard},
};

use chrono::{SecondsFormat, Utc};

use crate::{
    agent::{AgentAdapterError, NormalizedAgentEvent, NormalizedAgentEventKind},
    application::{
        AgentProfileServiceError, BoardServiceError, ExecutionEventControllerError,
        ExecutionLaunchError, StartExecutionRequest,
    },
    domain::{ExecutionRole, ExecutionStatus, WorkItemId, WorkItemState},
    persistence::{BoardStoreError, EventStoreError},
    workspace::WorkspaceError,
};

type RuntimeServiceError = BoardServiceError<BoardStoreError>;
type RuntimeProfileError = AgentProfileServiceError<BoardStoreError>;
type RuntimeControllerError = ExecutionEventControllerError<BoardStoreError>;

pub(super) fn validate_start_request(
    request: &StartExecutionRequest,
) -> Result<(), ExecutionRuntimeError> {
    for (value, field) in [
        (&request.execution_id, "execution id"),
        (&request.work_item_id, "work item id"),
        (&request.agent_profile_name, "agent profile name"),
        (&request.task_brief, "task brief"),
    ] {
        if value.trim().is_empty() {
            return Err(ExecutionRuntimeError::MissingRequiredField { field });
        }
    }
    Ok(())
}

pub(super) fn ensure_startable(
    work_item_id: &WorkItemId,
    state: WorkItemState,
    role: ExecutionRole,
) -> Result<(), ExecutionRuntimeError> {
    let expected_state = match role {
        ExecutionRole::Implementation => WorkItemState::Ready,
        ExecutionRole::IndependentReview => WorkItemState::Review,
    };
    if state == expected_state {
        Ok(())
    } else {
        Err(ExecutionRuntimeError::WorkItemNotReady {
            work_item_id: work_item_id.clone(),
            state,
        })
    }
}

pub(super) fn is_terminal_event(event: &NormalizedAgentEvent) -> bool {
    matches!(
        event.kind,
        NormalizedAgentEventKind::Completed { .. }
            | NormalizedAgentEventKind::Failed { .. }
            | NormalizedAgentEventKind::Interrupted { .. }
    )
}

pub(super) fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub(super) fn lock<'a, Value>(
    mutex: &'a Mutex<Value>,
    resource: &'static str,
) -> Result<MutexGuard<'a, Value>, ExecutionRuntimeError> {
    mutex
        .lock()
        .map_err(|_| ExecutionRuntimeError::Synchronization { resource })
}

#[derive(Debug)]
pub(crate) enum ExecutionRuntimeError {
    Board(RuntimeServiceError),
    Profile(RuntimeProfileError),
    Workspace(WorkspaceError),
    Preflight(ExecutionLaunchError),
    Activation(RuntimeControllerError),
    Agent(AgentAdapterError),
    PolicyAudit(EventStoreError),
    PolicyDenied {
        reason: String,
    },
    UnsupportedPolicySet {
        policy_set_id: String,
    },
    MissingRequiredField {
        field: &'static str,
    },
    WorkItemNotReady {
        work_item_id: WorkItemId,
        state: WorkItemState,
    },
    MissingLiveExecution {
        execution_id: String,
    },
    ExecutionNotStoppable {
        execution_id: String,
        status: ExecutionStatus,
    },
    DuplicateLiveExecution {
        execution_id: String,
        session_id: String,
    },
    MonitorSpawn(String),
    Synchronization {
        resource: &'static str,
    },
}

impl fmt::Display for ExecutionRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board(error) => write!(formatter, "board error: {error}"),
            Self::Profile(error) => write!(formatter, "agent profile error: {error}"),
            Self::Workspace(error) => write!(formatter, "workspace error: {error}"),
            Self::Preflight(error) => write!(formatter, "execution launch rejected: {error}"),
            Self::Activation(error) => write!(formatter, "execution lifecycle error: {error}"),
            Self::Agent(error) => write!(formatter, "agent process error: {error}"),
            Self::PolicyAudit(error) => write!(formatter, "policy audit error: {error}"),
            Self::PolicyDenied { reason } => {
                write!(formatter, "execution denied by policy: {reason}")
            }
            Self::UnsupportedPolicySet { policy_set_id } => {
                write!(
                    formatter,
                    "policy set {policy_set_id} is not available in this MVP"
                )
            }
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::WorkItemNotReady {
                work_item_id,
                state,
            } => {
                write!(
                    formatter,
                    "work item {} cannot start because it is {state:?}",
                    work_item_id.0
                )
            }
            Self::MissingLiveExecution { execution_id } => {
                write!(
                    formatter,
                    "no live agent is registered for execution {execution_id}"
                )
            }
            Self::ExecutionNotStoppable {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {execution_id} cannot stop because it is {status:?}"
            ),
            Self::DuplicateLiveExecution {
                execution_id,
                session_id,
            } => write!(
                formatter,
                "execution {execution_id} already owns a live agent session {session_id}"
            ),
            Self::MonitorSpawn(error) => {
                write!(formatter, "agent monitor could not start: {error}")
            }
            Self::Synchronization { resource } => write!(formatter, "{resource} is unavailable"),
        }
    }
}

impl Error for ExecutionRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Board(error) => Some(error),
            Self::Profile(error) => Some(error),
            Self::Workspace(error) => Some(error),
            Self::Preflight(error) => Some(error),
            Self::Activation(error) => Some(error),
            Self::Agent(error) => Some(error),
            Self::PolicyAudit(error) => Some(error),
            Self::MissingRequiredField { .. }
            | Self::PolicyDenied { .. }
            | Self::UnsupportedPolicySet { .. }
            | Self::WorkItemNotReady { .. }
            | Self::MissingLiveExecution { .. }
            | Self::ExecutionNotStoppable { .. }
            | Self::DuplicateLiveExecution { .. }
            | Self::MonitorSpawn(_)
            | Self::Synchronization { .. } => None,
        }
    }
}
