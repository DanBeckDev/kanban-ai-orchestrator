use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, Evidence, Execution, MaterializedWorkItem,
    Project, RecordedWorkItemEvent, TransitionWorkItemCommand, WorkItemId,
};
use crate::{
    agent::AgentProfile,
    application::{BoardRepository, BoardSnapshot},
};

use super::{BoardStoreError, SqliteEventStore};

impl BoardRepository for SqliteEventStore {
    type Error = BoardStoreError;

    fn create_project(&mut self, project: Project) -> Result<Project, Self::Error> {
        SqliteEventStore::create_project(self, project)
    }

    fn project(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<Option<Project>, Self::Error> {
        SqliteEventStore::project(self, project_id)
    }

    fn create_board(&mut self, board: Board) -> Result<Board, Self::Error> {
        SqliteEventStore::create_board(self, board)
    }

    fn board(&self, board_id: &BoardId) -> Result<Option<Board>, Self::Error> {
        SqliteEventStore::board(self, board_id)
    }

    fn create_board_work_item(
        &mut self,
        command: CreateWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::create_board_work_item(self, command)
    }

    fn add_board_dependency(&mut self, dependency: Dependency) -> Result<Dependency, Self::Error> {
        SqliteEventStore::add_board_dependency(self, dependency)
    }

    fn materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<MaterializedWorkItem>, Self::Error> {
        SqliteEventStore::materialized_work_item(self, work_item_id).map_err(BoardStoreError::from)
    }

    fn transition_work_item(
        &mut self,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::transition_work_item(self, command).map_err(BoardStoreError::from)
    }

    fn record_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error> {
        SqliteEventStore::record_execution(self, execution).map_err(BoardStoreError::from)
    }

    fn execution(
        &self,
        execution_id: &crate::domain::ExecutionId,
    ) -> Result<Option<Execution>, Self::Error> {
        SqliteEventStore::execution(self, execution_id).map_err(BoardStoreError::from)
    }

    fn update_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error> {
        SqliteEventStore::update_execution(self, execution).map_err(BoardStoreError::from)
    }

    fn active_execution_count_for_project(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<u32, Self::Error> {
        SqliteEventStore::active_execution_count_for_project(self, project_id)
    }

    fn activate_execution_and_start_work_item(
        &mut self,
        execution: Execution,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::activate_execution_and_start_work_item(self, execution, command)
            .map_err(BoardStoreError::from)
    }

    fn record_evidence(&mut self, evidence: Evidence) -> Result<Evidence, Self::Error> {
        SqliteEventStore::record_evidence(self, evidence).map_err(BoardStoreError::from)
    }

    fn evidence_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Evidence>, Self::Error> {
        SqliteEventStore::evidence_for_work_items(self, std::slice::from_ref(work_item_id))
            .map_err(BoardStoreError::from)
    }

    fn save_agent_profile(&mut self, profile: AgentProfile) -> Result<AgentProfile, Self::Error> {
        SqliteEventStore::save_agent_profile(self, profile).map_err(BoardStoreError::from)
    }

    fn agent_profile(&self, name: &str) -> Result<Option<AgentProfile>, Self::Error> {
        SqliteEventStore::agent_profile(self, name).map_err(BoardStoreError::from)
    }

    fn agent_profiles(&self) -> Result<Vec<AgentProfile>, Self::Error> {
        SqliteEventStore::agent_profiles(self).map_err(BoardStoreError::from)
    }

    fn board_snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, Self::Error> {
        SqliteEventStore::board_snapshot(self, board_id)
    }
}
