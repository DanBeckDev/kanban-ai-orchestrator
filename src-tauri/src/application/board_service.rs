use std::{error::Error, fmt};

use crate::agent::AgentProfile;
use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, DependencyId, DependencySource, Evidence,
    EvidenceKind, EvidenceResult, Execution, ExecutionId, ExternalLink, MaterializedWorkItem,
    Project, ProjectId, RecordedWorkItemEvent, SchemaMetadata, TransitionConfig,
    TransitionWorkItemCommand, WorkItem, WorkItemEventId, WorkItemId, WorkItemState,
};

use super::{
    AddDependencyRequest, BoardSnapshot, CreateBoardRequest, CreateProjectRequest,
    CreateWorkItemRequest, TransitionWorkItemRequest,
};

pub trait BoardRepository {
    type Error: Error;

    fn create_project(&mut self, project: Project) -> Result<Project, Self::Error>;
    fn project(&self, project_id: &ProjectId) -> Result<Option<Project>, Self::Error>;
    fn create_board(&mut self, board: Board) -> Result<Board, Self::Error>;
    fn board(&self, board_id: &BoardId) -> Result<Option<Board>, Self::Error>;
    fn create_board_work_item(
        &mut self,
        command: CreateWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error>;
    fn add_board_dependency(&mut self, dependency: Dependency) -> Result<Dependency, Self::Error>;
    fn materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<MaterializedWorkItem>, Self::Error>;
    fn transition_work_item(
        &mut self,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error>;
    fn record_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error>;
    fn execution(&self, execution_id: &ExecutionId) -> Result<Option<Execution>, Self::Error>;
    fn update_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error>;
    fn active_execution_count_for_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<u32, Self::Error>;
    fn activate_execution_and_start_work_item(
        &mut self,
        execution: Execution,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error>;
    fn record_evidence(&mut self, evidence: Evidence) -> Result<Evidence, Self::Error>;
    fn record_external_link(&mut self, link: ExternalLink) -> Result<ExternalLink, Self::Error>;
    fn external_link_for_connector_resource(
        &self,
        connector_id: &str,
        external_id: &str,
    ) -> Result<Option<ExternalLink>, Self::Error>;
    fn external_links_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ExternalLink>, Self::Error>;
    fn evidence_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Evidence>, Self::Error>;
    fn save_agent_profile(&mut self, profile: AgentProfile) -> Result<AgentProfile, Self::Error>;
    fn agent_profile(&self, name: &str) -> Result<Option<AgentProfile>, Self::Error>;
    fn agent_profiles(&self) -> Result<Vec<AgentProfile>, Self::Error>;
    fn board_snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, Self::Error>;
}

pub struct BoardService<Repository> {
    pub(super) repository: Repository,
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub fn create_project(
        &mut self,
        request: CreateProjectRequest,
    ) -> Result<Project, BoardServiceError<Repository::Error>> {
        validate_required(&request.project_id, "project id")?;
        validate_required(&request.name, "project name")?;
        validate_required(&request.repository_path, "repository path")?;
        validate_required(&request.base_ref, "base ref")?;
        validate_required(&request.policy_set_id, "policy set id")?;

        self.repository
            .create_project(Project {
                schema: SchemaMetadata::current(),
                id: ProjectId::from(request.project_id.as_str()),
                name: request.name,
                repository_path: request.repository_path,
                base_ref: request.base_ref,
                policy_set_id: request.policy_set_id,
            })
            .map_err(BoardServiceError::Repository)
    }

    pub fn create_board(
        &mut self,
        request: CreateBoardRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.board_id, "board id")?;
        validate_required(&request.project_id, "project id")?;
        validate_required(&request.name, "board name")?;

        let board = self
            .repository
            .create_board(Board {
                schema: SchemaMetadata::current(),
                id: BoardId::from(request.board_id.as_str()),
                project_id: ProjectId::from(request.project_id.as_str()),
                name: request.name,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board.id)
    }

    pub fn create_work_item(
        &mut self,
        request: CreateWorkItemRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.event_id, "event id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.board_id, "board id")?;
        validate_required(&request.title, "work item title")?;
        validate_required(&request.description, "work item description")?;
        validate_required(&request.recorded_at, "recorded at")?;
        validate_criteria(&request.acceptance_criteria)?;

        let board_id = BoardId::from(request.board_id.as_str());
        self.repository
            .create_board_work_item(CreateWorkItemCommand {
                event_id: WorkItemEventId::from(request.event_id.as_str()),
                work_item: WorkItem {
                    schema: SchemaMetadata::current(),
                    id: WorkItemId::from(request.work_item_id.as_str()),
                    board_id: board_id.clone(),
                    title: request.title,
                    description: request.description,
                    acceptance_criteria: request.acceptance_criteria,
                    budget: request.budget,
                    state: WorkItemState::Inbox,
                    requires_human_review: request.requires_human_review,
                },
                recorded_at: request.recorded_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn add_dependency(
        &mut self,
        request: AddDependencyRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.dependency_id, "dependency id")?;
        validate_required(&request.upstream_work_item_id, "upstream work item id")?;
        validate_required(&request.downstream_work_item_id, "downstream work item id")?;
        validate_required(&request.reason, "dependency reason")?;
        validate_required(&request.owner, "dependency owner")?;
        validate_required(&request.next_action, "dependency next action")?;
        validate_required(&request.created_by, "dependency creator")?;
        validate_required(&request.created_at, "dependency created at")?;

        let upstream_work_item_id = WorkItemId::from(request.upstream_work_item_id.as_str());
        let board_id = self.board_id_for(&upstream_work_item_id)?;
        self.repository
            .add_board_dependency(Dependency {
                schema: SchemaMetadata::current(),
                id: DependencyId::from(request.dependency_id.as_str()),
                upstream_work_item_id,
                downstream_work_item_id: WorkItemId::from(request.downstream_work_item_id.as_str()),
                kind: request.kind,
                source: DependencySource::User,
                reason: request.reason,
                owner: request.owner,
                next_action: request.next_action,
                created_by: request.created_by,
                created_at: request.created_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn transition_work_item(
        &mut self,
        request: TransitionWorkItemRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.event_id, "event id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.reason, "transition reason")?;
        validate_required(&request.recorded_at, "recorded at")?;

        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let materialized_work_item = self
            .repository
            .materialized_work_item(&work_item_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            })?;
        let board_id = materialized_work_item.work_item.board_id.clone();
        self.require_recorded_completion_evidence(&work_item_id, &request)?;
        self.repository
            .transition_work_item(TransitionWorkItemCommand {
                event_id: WorkItemEventId::from(request.event_id.as_str()),
                work_item_id,
                next_state: request.next_state,
                config: TransitionConfig {
                    human_review_required: materialized_work_item.work_item.requires_human_review,
                },
                evidence: request.evidence,
                reason: request.reason,
                recorded_at: request.recorded_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn snapshot(
        &self,
        board_id: &BoardId,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        self.repository
            .board_snapshot(board_id)
            .map_err(BoardServiceError::Repository)
    }

    pub(super) fn board_id_for(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<BoardId, BoardServiceError<Repository::Error>> {
        self.repository
            .materialized_work_item(work_item_id)
            .map_err(BoardServiceError::Repository)?
            .map(|materialized_work_item| materialized_work_item.work_item.board_id)
            .ok_or_else(|| BoardServiceError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            })
    }
}

pub(super) fn validate_required<RepositoryError>(
    value: &str,
    field: &'static str,
) -> Result<(), BoardServiceError<RepositoryError>> {
    if value.trim().is_empty() {
        Err(BoardServiceError::MissingRequiredField { field })
    } else {
        Ok(())
    }
}

fn validate_criteria<RepositoryError>(
    acceptance_criteria: &[String],
) -> Result<(), BoardServiceError<RepositoryError>> {
    if acceptance_criteria.is_empty()
        || acceptance_criteria
            .iter()
            .any(|criterion| criterion.trim().is_empty())
    {
        Err(BoardServiceError::InvalidAcceptanceCriteria)
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum BoardServiceError<RepositoryError> {
    Repository(RepositoryError),
    MissingRequiredField {
        field: &'static str,
    },
    InvalidAcceptanceCriteria,
    InvalidExternalIdentifier {
        field: &'static str,
    },
    InvalidExternalUrl,
    ProjectNotFound {
        project_id: ProjectId,
    },
    BoardNotFound {
        board_id: BoardId,
    },
    WorkItemNotFound {
        work_item_id: WorkItemId,
    },
    ExecutionNotFound {
        execution_id: ExecutionId,
    },
    ExecutionNotPending {
        execution_id: ExecutionId,
        status: crate::domain::ExecutionStatus,
    },
    WorkItemNotReady {
        work_item_id: WorkItemId,
        state: WorkItemState,
    },
    WorkItemNotInReview {
        work_item_id: WorkItemId,
        state: WorkItemState,
    },
    ExternalResourceNotLinked {
        connector_id: &'static str,
        external_id: String,
    },
    MissingRecordedEvidence {
        work_item_id: WorkItemId,
        kind: EvidenceKind,
        result: EvidenceResult,
    },
}

impl<RepositoryError> fmt::Display for BoardServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "board repository error: {error}"),
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::InvalidAcceptanceCriteria => {
                formatter.write_str("at least one non-empty acceptance criterion is required")
            }
            Self::InvalidExternalIdentifier { field } => {
                write!(formatter, "{field} must be a valid UUID")
            }
            Self::InvalidExternalUrl => {
                formatter.write_str("Linear issue URL must be an HTTPS linear.app URL")
            }
            Self::ProjectNotFound { project_id } => {
                write!(formatter, "project {} was not found", project_id.0)
            }
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::WorkItemNotFound { work_item_id } => {
                write!(formatter, "work item {} was not found", work_item_id.0)
            }
            Self::ExecutionNotFound { execution_id } => {
                write!(formatter, "execution {} was not found", execution_id.0)
            }
            Self::ExecutionNotPending {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {} cannot start because it is {status:?}",
                execution_id.0
            ),
            Self::WorkItemNotReady {
                work_item_id,
                state,
            } => write!(
                formatter,
                "work item {} cannot start because it is {state:?}",
                work_item_id.0
            ),
            Self::WorkItemNotInReview {
                work_item_id,
                state,
            } => write!(
                formatter,
                "work item {} cannot record review evidence because it is {state:?}",
                work_item_id.0
            ),
            Self::ExternalResourceNotLinked {
                connector_id,
                external_id,
            } => write!(
                formatter,
                "external resource {connector_id}:{external_id} is not linked to a local task"
            ),
            Self::MissingRecordedEvidence {
                work_item_id,
                kind,
                result,
            } => write!(
                formatter,
                "work item {} requires recorded {result:?} {kind:?} evidence before Done",
                work_item_id.0
            ),
        }
    }
}

impl<RepositoryError> Error for BoardServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::MissingRequiredField { .. }
            | Self::InvalidAcceptanceCriteria
            | Self::InvalidExternalIdentifier { .. }
            | Self::InvalidExternalUrl
            | Self::ProjectNotFound { .. }
            | Self::BoardNotFound { .. }
            | Self::WorkItemNotFound { .. }
            | Self::ExecutionNotFound { .. }
            | Self::ExecutionNotPending { .. }
            | Self::WorkItemNotReady { .. }
            | Self::WorkItemNotInReview { .. }
            | Self::ExternalResourceNotLinked { .. }
            | Self::MissingRecordedEvidence { .. } => None,
        }
    }
}
