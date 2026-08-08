use crate::application::{BoardRepository, BoardSnapshot};
use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, MaterializedWorkItem, Project,
    RecordedWorkItemEvent, TransitionWorkItemCommand, WorkItemId,
};

use super::{BoardStoreError, SqliteEventStore};

impl BoardRepository for SqliteEventStore {
    type Error = BoardStoreError;

    fn create_project(&mut self, project: Project) -> Result<Project, Self::Error> {
        SqliteEventStore::create_project(self, project)
    }

    fn create_board(&mut self, board: Board) -> Result<Board, Self::Error> {
        SqliteEventStore::create_board(self, board)
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

    fn board_snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, Self::Error> {
        SqliteEventStore::board_snapshot(self, board_id)
    }
}
