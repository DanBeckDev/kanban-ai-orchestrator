use crate::domain::{
    Evidence, EvidenceId, Execution, ExecutionId, ExecutionStatus, ExecutionUsage,
    MaterializedWorkItem, Project, SchemaMetadata, TransitionConfig, TransitionWorkItemCommand,
    WorkItemEventId, WorkItemId, WorkItemState,
};

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, RecordEvidenceRequest,
    RecordExecutionRequest, UpdateExecutionRequest,
};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn record_execution(
        &mut self,
        request: RecordExecutionRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.execution_id, "execution id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.adapter_name, "adapter name")?;
        validate_required(&request.workspace_path, "workspace path")?;

        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let board_id = self.board_id_for(&work_item_id)?;
        self.repository
            .record_execution(Execution {
                schema: SchemaMetadata::current(),
                id: ExecutionId::from(request.execution_id.as_str()),
                work_item_id,
                adapter_name: request.adapter_name,
                status: ExecutionStatus::Pending,
                session_id: None,
                workspace_path: request.workspace_path,
                usage: ExecutionUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    cost_micros: None,
                },
                last_event_sequence: 0,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn record_evidence(
        &mut self,
        request: RecordEvidenceRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.evidence_id, "evidence id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.summary, "evidence summary")?;
        validate_required(&request.recorded_at, "evidence recorded at")?;

        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let board_id = self.board_id_for(&work_item_id)?;
        self.repository
            .record_evidence(Evidence {
                schema: SchemaMetadata::current(),
                id: EvidenceId::from(request.evidence_id.as_str()),
                work_item_id,
                kind: request.kind,
                result: request.result,
                summary: request.summary,
                recorded_at: request.recorded_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn update_execution(
        &mut self,
        request: UpdateExecutionRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.execution_id, "execution id")?;

        let execution_id = ExecutionId::from(request.execution_id.as_str());
        let mut execution = self.execution(&execution_id)?;
        let board_id = self.board_id_for(&execution.work_item_id)?;
        execution.status = request.status;
        execution.session_id = request.session_id;
        execution.usage = request.usage;
        execution.last_event_sequence = request.last_event_sequence;
        self.repository
            .update_execution(execution)
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn activate_execution(
        &mut self,
        execution_id: &str,
        session_id: &str,
        recorded_at: &str,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(execution_id, "execution id")?;
        validate_required(session_id, "agent session id")?;
        validate_required(recorded_at, "execution activation time")?;

        let execution = self.execution(&ExecutionId::from(execution_id))?;
        let work_item = self.work_item(&execution.work_item_id)?;
        if execution.status == ExecutionStatus::Running
            && execution.session_id.as_deref() == Some(session_id)
            && work_item.work_item.state == WorkItemState::Running
        {
            return self.snapshot(&work_item.work_item.board_id);
        }
        if execution.status != ExecutionStatus::Pending {
            return Err(BoardServiceError::ExecutionNotPending {
                execution_id: execution.id,
                status: execution.status,
            });
        }
        if work_item.work_item.state != WorkItemState::Ready {
            return Err(BoardServiceError::WorkItemNotReady {
                work_item_id: work_item.work_item.id,
                state: work_item.work_item.state,
            });
        }

        let mut active_execution = execution;
        active_execution.status = ExecutionStatus::Running;
        active_execution.session_id = Some(session_id.to_owned());
        self.repository
            .activate_execution_and_start_work_item(
                active_execution,
                TransitionWorkItemCommand {
                    event_id: WorkItemEventId::from(format!("start-{execution_id}").as_str()),
                    work_item_id: work_item.work_item.id.clone(),
                    next_state: WorkItemState::Running,
                    config: TransitionConfig {
                        human_review_required: work_item.work_item.requires_human_review,
                    },
                    evidence: None,
                    reason: format!("Agent session {session_id} started."),
                    recorded_at: recorded_at.to_owned(),
                },
            )
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&work_item.work_item.board_id)
    }

    pub fn execution(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<Execution, BoardServiceError<Repository::Error>> {
        self.repository
            .execution(execution_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::ExecutionNotFound {
                execution_id: execution_id.clone(),
            })
    }

    pub fn active_execution_count_for_project(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<u32, BoardServiceError<Repository::Error>> {
        self.repository
            .active_execution_count_for_project(project_id)
            .map_err(BoardServiceError::Repository)
    }

    pub fn work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<MaterializedWorkItem, BoardServiceError<Repository::Error>> {
        self.repository
            .materialized_work_item(work_item_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            })
    }

    pub fn project_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Project, BoardServiceError<Repository::Error>> {
        let board_id = self.board_id_for(work_item_id)?;
        let board = self
            .repository
            .board(&board_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        self.repository
            .project(&board.project_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::ProjectNotFound {
                project_id: board.project_id,
            })
    }
}
