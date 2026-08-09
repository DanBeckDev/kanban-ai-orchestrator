use chrono::Utc;

use crate::domain::BoardId;

use super::{
    BoardLibraryEntry, BoardRepository, BoardService, BoardServiceError, repository_available,
    sort_board_library,
};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn board_library(
        &self,
    ) -> Result<Vec<BoardLibraryEntry>, BoardServiceError<Repository::Error>> {
        let mut entries = self
            .repository
            .board_library_records()
            .map_err(BoardServiceError::Repository)?
            .into_iter()
            .map(BoardLibraryEntry::from_record)
            .collect::<Vec<_>>();
        sort_board_library(&mut entries);
        Ok(entries)
    }

    pub fn open_board(
        &mut self,
        board_id: &str,
    ) -> Result<super::BoardSnapshot, BoardServiceError<Repository::Error>> {
        super::board_service::validate_required(board_id, "board id")?;
        let board_id = BoardId::from(board_id);
        let board = self
            .repository
            .board(&board_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        let project = self
            .repository
            .project(&board.project_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::ProjectNotFound {
                project_id: board.project_id.clone(),
            })?;
        if !repository_available(&project.repository_path) {
            return Err(BoardServiceError::RepositoryUnavailable {
                project_id: project.id,
                repository_path: project.repository_path,
            });
        }
        self.repository
            .record_board_opened(&board_id, Utc::now().to_rfc3339())
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }
}
