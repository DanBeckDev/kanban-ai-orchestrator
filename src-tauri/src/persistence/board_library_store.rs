use rusqlite::params;

use crate::application::{BoardAttentionSummary, BoardLibraryRecord};
use crate::domain::{Board, BoardId, Project, WorkItemState};

use super::{BoardStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn board_library_records(&self) -> Result<Vec<BoardLibraryRecord>, BoardStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT boards.board_json, projects.project_json, board_access.last_opened_at
             FROM boards
             INNER JOIN projects ON projects.project_id = boards.project_id
             LEFT JOIN board_access ON board_access.board_id = boards.board_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let serialized_records = rows.collect::<Result<Vec<_>, _>>()?;
        drop(statement);

        serialized_records
            .into_iter()
            .map(|(board_json, project_json, last_opened_at)| {
                let board = serde_json::from_str::<Board>(&board_json)?;
                let project = serde_json::from_str::<Project>(&project_json)?;
                Ok(BoardLibraryRecord {
                    attention: self.board_attention_summary(&board.id)?,
                    board,
                    project,
                    last_opened_at,
                })
            })
            .collect()
    }

    pub fn record_board_opened(
        &mut self,
        board_id: &BoardId,
        opened_at: String,
    ) -> Result<(), BoardStoreError> {
        if self.board(board_id)?.is_none() {
            return Err(BoardStoreError::BoardNotFound {
                board_id: board_id.clone(),
            });
        }
        self.connection.execute(
            "INSERT INTO board_access (board_id, last_opened_at)
             VALUES (?1, ?2)
             ON CONFLICT(board_id) DO UPDATE SET last_opened_at = excluded.last_opened_at",
            params![board_id.0, opened_at],
        )?;
        Ok(())
    }

    fn board_attention_summary(
        &self,
        board_id: &BoardId,
    ) -> Result<BoardAttentionSummary, BoardStoreError> {
        let mut summary = BoardAttentionSummary::default();
        for work_item in self.work_items_for_board(board_id)? {
            match work_item.work_item.state {
                WorkItemState::Running => summary.active_work_item_count += 1,
                WorkItemState::AwaitingInput
                | WorkItemState::Review
                | WorkItemState::Blocked
                | WorkItemState::Failed
                | WorkItemState::Interrupted => summary.needs_attention_count += 1,
                _ => {}
            }
        }
        Ok(summary)
    }
}

pub(crate) fn create_board_library_schema(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS board_access (
            board_id TEXT PRIMARY KEY,
            last_opened_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS board_access_by_recency
            ON board_access (last_opened_at DESC, board_id);",
    )
}
