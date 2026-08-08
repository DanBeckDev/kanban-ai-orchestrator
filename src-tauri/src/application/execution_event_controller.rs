use std::{error::Error, fmt};

use crate::{
    agent::{
        AgentAdapterError, AgentEventIngestor, NormalizedAgentEvent, NormalizedAgentEventKind,
    },
    domain::{
        EvidenceKind, EvidenceResult, Execution, ExecutionId, ExecutionStatus, ExecutionUsage,
        MaterializedWorkItem, TransitionConfig, WorkItemState,
    },
};

use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, RecordEvidenceRequest,
    TransitionWorkItemRequest, UpdateExecutionRequest,
};

pub struct ExecutionEventController;

impl ExecutionEventController {
    pub fn activate<Repository>(
        service: &mut BoardService<Repository>,
        execution_id: &str,
        session_id: &str,
    ) -> Result<BoardSnapshot, ExecutionEventControllerError<Repository::Error>>
    where
        Repository: BoardRepository,
    {
        validate_session_id(session_id)?;
        let execution = execution(service, execution_id)?;
        if execution.status != ExecutionStatus::Pending {
            return Err(ExecutionEventControllerError::ExecutionNotPending {
                execution_id: execution.id,
                status: execution.status,
            });
        }

        service
            .update_execution(UpdateExecutionRequest {
                execution_id: execution_id.to_owned(),
                status: ExecutionStatus::Running,
                session_id: Some(session_id.to_owned()),
                usage: execution.usage,
                last_event_sequence: execution.last_event_sequence,
            })
            .map_err(ExecutionEventControllerError::BoardService)
    }

    pub fn record_event<Repository>(
        service: &mut BoardService<Repository>,
        execution_id: &str,
        event: NormalizedAgentEvent,
        recorded_at: &str,
    ) -> Result<BoardSnapshot, ExecutionEventControllerError<Repository::Error>>
    where
        Repository: BoardRepository,
    {
        validate_recorded_at(recorded_at)?;
        let execution = execution(service, execution_id)?;
        ensure_execution_accepts_events(&execution)?;
        let work_item = work_item(service, &execution)?;
        let mut ingestor = AgentEventIngestor::new(execution.last_event_sequence);
        let next_work_item_state = ingestor
            .apply_to_work_item(
                work_item.work_item.state,
                &event,
                TransitionConfig {
                    human_review_required: work_item.work_item.requires_human_review,
                },
            )
            .map_err(ExecutionEventControllerError::AgentAdapter)?;

        if next_work_item_state != work_item.work_item.state {
            transition_work_item(
                service,
                &execution,
                next_work_item_state,
                &event,
                recorded_at,
            )?;
        }
        if let Some(evidence) = evidence_for_event(&execution, &event, recorded_at) {
            service
                .record_evidence(evidence)
                .map_err(ExecutionEventControllerError::BoardService)?;
        }

        service
            .update_execution(UpdateExecutionRequest {
                execution_id: execution.id.0.clone(),
                status: execution_status_for(&execution, &event.kind),
                session_id: execution.session_id,
                usage: usage_for(&execution.usage, &event.kind),
                last_event_sequence: ingestor.last_sequence(),
            })
            .map_err(ExecutionEventControllerError::BoardService)
    }
}

fn execution<Repository>(
    service: &BoardService<Repository>,
    execution_id: &str,
) -> Result<Execution, ExecutionEventControllerError<Repository::Error>>
where
    Repository: BoardRepository,
{
    service
        .execution(&ExecutionId::from(execution_id))
        .map_err(ExecutionEventControllerError::BoardService)
}

fn work_item<Repository>(
    service: &BoardService<Repository>,
    execution: &Execution,
) -> Result<MaterializedWorkItem, ExecutionEventControllerError<Repository::Error>>
where
    Repository: BoardRepository,
{
    service
        .work_item(&execution.work_item_id)
        .map_err(ExecutionEventControllerError::BoardService)
}

fn transition_work_item<Repository>(
    service: &mut BoardService<Repository>,
    execution: &Execution,
    next_state: WorkItemState,
    event: &NormalizedAgentEvent,
    recorded_at: &str,
) -> Result<(), ExecutionEventControllerError<Repository::Error>>
where
    Repository: BoardRepository,
{
    service
        .transition_work_item(TransitionWorkItemRequest {
            event_id: format!("{}-agent-event-{}", execution.id.0, event.sequence),
            work_item_id: execution.work_item_id.0.clone(),
            next_state,
            evidence: None,
            reason: event_summary(&event.kind),
            recorded_at: recorded_at.to_owned(),
        })
        .map(|_| ())
        .map_err(ExecutionEventControllerError::BoardService)
}

fn evidence_for_event(
    execution: &Execution,
    event: &NormalizedAgentEvent,
    recorded_at: &str,
) -> Option<RecordEvidenceRequest> {
    let (kind, result) = match event.kind {
        NormalizedAgentEventKind::ApprovalRequested { .. }
        | NormalizedAgentEventKind::AwaitingInput { .. }
        | NormalizedAgentEventKind::Interrupted { .. } => {
            (EvidenceKind::AgentReport, EvidenceResult::Recorded)
        }
        NormalizedAgentEventKind::AwaitingReview { .. }
        | NormalizedAgentEventKind::Completed { .. } => {
            (EvidenceKind::CompletionReport, EvidenceResult::Recorded)
        }
        NormalizedAgentEventKind::Failed { .. } => {
            (EvidenceKind::AgentReport, EvidenceResult::Failed)
        }
        NormalizedAgentEventKind::Activity { .. }
        | NormalizedAgentEventKind::UsageUpdated { .. } => {
            return None;
        }
    };

    Some(RecordEvidenceRequest {
        evidence_id: format!("{}-agent-event-{}", execution.id.0, event.sequence),
        work_item_id: execution.work_item_id.0.clone(),
        kind,
        result,
        summary: event_summary(&event.kind),
        recorded_at: recorded_at.to_owned(),
    })
}

fn execution_status_for(
    execution: &Execution,
    event: &NormalizedAgentEventKind,
) -> ExecutionStatus {
    match event {
        NormalizedAgentEventKind::ApprovalRequested { .. }
        | NormalizedAgentEventKind::AwaitingInput { .. } => ExecutionStatus::AwaitingInput,
        NormalizedAgentEventKind::AwaitingReview { .. }
        | NormalizedAgentEventKind::Completed { .. } => ExecutionStatus::AwaitingReview,
        NormalizedAgentEventKind::Failed { .. } => ExecutionStatus::Failed,
        NormalizedAgentEventKind::Interrupted { .. } => ExecutionStatus::Interrupted,
        NormalizedAgentEventKind::Activity { .. }
        | NormalizedAgentEventKind::UsageUpdated { .. } => execution.status,
    }
}

fn usage_for(existing: &ExecutionUsage, event: &NormalizedAgentEventKind) -> ExecutionUsage {
    match event {
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens,
            output_tokens,
            cost_micros,
        } => ExecutionUsage {
            input_tokens: *input_tokens,
            output_tokens: *output_tokens,
            cost_micros: *cost_micros,
        },
        _ => existing.clone(),
    }
}

fn event_summary(event: &NormalizedAgentEventKind) -> String {
    match event {
        NormalizedAgentEventKind::Activity { summary }
        | NormalizedAgentEventKind::AwaitingReview { summary }
        | NormalizedAgentEventKind::Completed { summary } => summary.clone(),
        NormalizedAgentEventKind::ApprovalRequested { question }
        | NormalizedAgentEventKind::AwaitingInput { question } => question.clone(),
        NormalizedAgentEventKind::Failed { reason }
        | NormalizedAgentEventKind::Interrupted { reason } => reason.clone(),
        NormalizedAgentEventKind::UsageUpdated { .. } => {
            "Agent usage checkpoint recorded.".to_owned()
        }
    }
}

fn validate_session_id<RepositoryError>(
    session_id: &str,
) -> Result<(), ExecutionEventControllerError<RepositoryError>> {
    if session_id.trim().is_empty() {
        Err(ExecutionEventControllerError::MissingSessionId)
    } else {
        Ok(())
    }
}

fn validate_recorded_at<RepositoryError>(
    recorded_at: &str,
) -> Result<(), ExecutionEventControllerError<RepositoryError>> {
    if recorded_at.trim().is_empty() {
        Err(ExecutionEventControllerError::MissingRecordedAt)
    } else {
        Ok(())
    }
}

fn ensure_execution_accepts_events<RepositoryError>(
    execution: &Execution,
) -> Result<(), ExecutionEventControllerError<RepositoryError>> {
    if matches!(
        execution.status,
        ExecutionStatus::Running | ExecutionStatus::AwaitingInput | ExecutionStatus::AwaitingReview
    ) {
        Ok(())
    } else {
        Err(ExecutionEventControllerError::ExecutionNotActive {
            execution_id: execution.id.clone(),
            status: execution.status,
        })
    }
}

#[derive(Debug)]
pub enum ExecutionEventControllerError<RepositoryError> {
    BoardService(BoardServiceError<RepositoryError>),
    AgentAdapter(AgentAdapterError),
    MissingSessionId,
    MissingRecordedAt,
    ExecutionNotPending {
        execution_id: ExecutionId,
        status: ExecutionStatus,
    },
    ExecutionNotActive {
        execution_id: ExecutionId,
        status: ExecutionStatus,
    },
}

impl<RepositoryError> fmt::Display for ExecutionEventControllerError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoardService(error) => write!(formatter, "board service error: {error}"),
            Self::AgentAdapter(error) => write!(formatter, "agent event rejected: {error}"),
            Self::MissingSessionId => formatter.write_str("agent session id is required"),
            Self::MissingRecordedAt => {
                formatter.write_str("agent event recorded-at time is required")
            }
            Self::ExecutionNotPending {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {} cannot start because it is {status:?}",
                execution_id.0
            ),
            Self::ExecutionNotActive {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {} cannot accept agent events because it is {status:?}",
                execution_id.0
            ),
        }
    }
}

impl<RepositoryError> Error for ExecutionEventControllerError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BoardService(error) => Some(error),
            Self::AgentAdapter(error) => Some(error),
            Self::MissingSessionId
            | Self::MissingRecordedAt
            | Self::ExecutionNotPending { .. }
            | Self::ExecutionNotActive { .. } => None,
        }
    }
}
