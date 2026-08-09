use rusqlite::params;

use crate::domain::{Board, Project};

use super::{BoardStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn create_local_board(
        &mut self,
        project: Project,
        board: Board,
        opened_at: String,
    ) -> Result<(), BoardStoreError> {
        if self.project(&project.id)?.is_some() {
            return Err(BoardStoreError::ProjectAlreadyExists {
                project_id: project.id,
            });
        }
        if self.board(&board.id)?.is_some() {
            return Err(BoardStoreError::BoardAlreadyExists { board_id: board.id });
        }

        let project_json = serde_json::to_string(&project)?;
        let board_json = serde_json::to_string(&board)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO projects (project_id, project_json) VALUES (?1, ?2)",
            params![project.id.0, project_json],
        )?;
        transaction.execute(
            "INSERT INTO boards (board_id, project_id, board_json) VALUES (?1, ?2, ?3)",
            params![board.id.0, board.project_id.0, board_json],
        )?;
        transaction.execute(
            "INSERT INTO board_access (board_id, last_opened_at) VALUES (?1, ?2)",
            params![board.id.0, opened_at],
        )?;
        transaction.commit()?;
        Ok(())
    }
}
