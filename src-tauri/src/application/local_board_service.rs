use chrono::Utc;
use uuid::Uuid;

use crate::domain::{Board, BoardId, Project, ProjectId, SchemaMetadata};

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, CreateLocalBoardRequest,
};

const STANDARD_POLICY_SET_ID: &str = "standard";

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn create_local_board(
        &mut self,
        request: CreateLocalBoardRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.name, "board name")?;
        validate_required(&request.repository_path, "repository path")?;
        let base_ref = required_or_default(request.base_ref, "base ref", "HEAD")?;
        let policy_set_id = required_or_default(
            request.policy_set_id,
            "policy set id",
            STANDARD_POLICY_SET_ID,
        )?;
        let project_id = ProjectId::from(Uuid::new_v4().to_string().as_str());
        let board_id = BoardId::from(Uuid::new_v4().to_string().as_str());
        let project = Project {
            schema: SchemaMetadata::current(),
            id: project_id.clone(),
            name: request.name.clone(),
            repository_path: request.repository_path,
            base_ref,
            policy_set_id,
        };
        let board = Board {
            schema: SchemaMetadata::current(),
            id: board_id.clone(),
            project_id,
            name: request.name,
        };

        self.repository
            .create_local_board(project, board, Utc::now().to_rfc3339())
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }
}

fn required_or_default<RepositoryError>(
    value: Option<String>,
    field: &'static str,
    default: &str,
) -> Result<String, BoardServiceError<RepositoryError>> {
    let value = value.unwrap_or_else(|| default.to_owned());
    validate_required(&value, field)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::super::board_service_tests::service;
    use super::super::{BoardServiceError, CreateLocalBoardRequest};

    #[test]
    fn generates_identifiers_and_uses_local_defaults() {
        let mut service = service();

        let snapshot = service
            .create_local_board(CreateLocalBoardRequest {
                name: "Project board".to_owned(),
                repository_path: "/projects/project".to_owned(),
                base_ref: None,
                policy_set_id: None,
            })
            .expect("local board should be created");

        assert_ne!(snapshot.board.id.0, "Project board");
        assert_ne!(snapshot.board.project_id.0, "Project board");
        assert_eq!(snapshot.board.name, "Project board");
    }

    #[test]
    fn rejects_empty_advanced_values() {
        let mut service = service();

        assert!(matches!(
            service.create_local_board(CreateLocalBoardRequest {
                name: "Project board".to_owned(),
                repository_path: "/projects/project".to_owned(),
                base_ref: Some(" ".to_owned()),
                policy_set_id: None,
            }),
            Err(BoardServiceError::MissingRequiredField { field: "base ref" })
        ));
    }
}
