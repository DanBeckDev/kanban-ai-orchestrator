use tempfile::TempDir;

use super::{BoardService, BoardServiceError, CreateBoardRequest, CreateProjectRequest};
use crate::persistence::SqliteEventStore;

fn service() -> BoardService<SqliteEventStore> {
    BoardService::new(SqliteEventStore::in_memory().expect("event store should open"))
}

fn create_board(service: &mut BoardService<SqliteEventStore>, repository_path: String) {
    service
        .create_project(CreateProjectRequest {
            project_id: "project-1".to_owned(),
            name: "Project".to_owned(),
            repository_path,
            base_ref: "main".to_owned(),
            policy_set_id: "standard".to_owned(),
        })
        .expect("project should be created");
    service
        .create_board(CreateBoardRequest {
            board_id: "board-1".to_owned(),
            project_id: "project-1".to_owned(),
            name: "Website reliability".to_owned(),
        })
        .expect("board should be created");
}

#[test]
fn lists_a_local_board_with_repository_context_and_open_recency() {
    let temporary_directory = TempDir::new().expect("temporary directory should exist");
    let repository_path = temporary_directory.path().join("website-reliability");
    std::fs::create_dir(&repository_path).expect("repository directory should exist");
    let mut service = service();
    create_board(&mut service, repository_path.display().to_string());

    let entries = service.board_library().expect("library should load");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].board_id, "board-1");
    assert_eq!(entries[0].name, "Website reliability");
    assert_eq!(entries[0].repository_name, "website-reliability");
    assert!(entries[0].repository_available);
    assert!(entries[0].last_opened_at.is_some());
    assert_eq!(entries[0].attention.active_work_item_count, 0);
    assert_eq!(entries[0].attention.needs_attention_count, 0);
}

#[test]
fn opens_a_recognised_board_when_its_repository_is_available() {
    let temporary_directory = TempDir::new().expect("temporary directory should exist");
    let repository_path = temporary_directory.path().join("website-reliability");
    std::fs::create_dir(&repository_path).expect("repository directory should exist");
    let mut service = service();
    create_board(&mut service, repository_path.display().to_string());

    let snapshot = service
        .open_board("board-1")
        .expect("available board should open");

    assert_eq!(snapshot.board.id.0, "board-1");
    assert_eq!(snapshot.board.name, "Website reliability");
    assert!(
        service
            .board_library()
            .expect("library should refresh")
            .first()
            .and_then(|entry| entry.last_opened_at.as_ref())
            .is_some()
    );
}

#[test]
fn refuses_to_open_a_board_when_its_repository_is_unavailable() {
    let missing_repository = "/missing/website-reliability".to_owned();
    let mut service = service();
    create_board(&mut service, missing_repository.clone());

    assert!(matches!(
        service.open_board("board-1"),
        Err(BoardServiceError::RepositoryUnavailable {
            project_id,
            repository_path
        }) if project_id.0 == "project-1" && repository_path == missing_repository
    ));
}
