use tempfile::TempDir;

use super::board_store_test_fixtures::{
    board, create_work_item_command, dependency, evidence, execution, opened_store, project,
};
use super::{BoardStoreError, SqliteEventStore};
use crate::domain::{
    Board, BoardId, DependencyGraphError, ProjectId, SchemaMetadata, WorkItemState,
};

use super::sqlite_event_store_tests::transition_command;

fn prepared_board(store: &mut SqliteEventStore) {
    store
        .create_project(project("project-1"))
        .expect("project should persist");
    store
        .create_board(board("board-1", "project-1"))
        .expect("board should persist");
}

#[test]
fn persists_a_board_snapshot_across_reopen() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("board.sqlite");
    let mut store = opened_store(&database_path);
    prepared_board(&mut store);
    store
        .create_board_work_item(create_work_item_command("task-1", "board-1"))
        .expect("work item should persist in its board");
    store
        .create_board_work_item(create_work_item_command("task-2", "board-1"))
        .expect("second work item should persist in its board");
    store
        .add_board_dependency(dependency("task-1-blocks-task-2", "task-1", "task-2"))
        .expect("dependency should persist in its board");
    store
        .record_execution(execution("task-1"))
        .expect("execution should persist");
    store
        .record_evidence(evidence("task-1"))
        .expect("evidence should persist");
    drop(store);

    let reopened_store = opened_store(&database_path);
    assert_eq!(
        reopened_store
            .project(&ProjectId::from("project-1"))
            .expect("project should load"),
        Some(project("project-1"))
    );
    assert_eq!(
        reopened_store
            .boards_for_project(&ProjectId::from("project-1"))
            .expect("project boards should load"),
        vec![board("board-1", "project-1")]
    );
    let snapshot = reopened_store
        .board_snapshot(&BoardId::from("board-1"))
        .expect("board snapshot should load");
    assert_eq!(snapshot.work_items.len(), 2);
    assert_eq!(snapshot.activity.len(), 2);
    assert_eq!(snapshot.activity[0].summary, "Task created.");
    assert_eq!(snapshot.executions, vec![execution("task-1")]);
    assert_eq!(snapshot.evidence, vec![evidence("task-1")]);
    assert_eq!(
        snapshot.dependencies,
        vec![dependency("task-1-blocks-task-2", "task-1", "task-2")]
    );
}

#[test]
fn requires_a_persisted_project_before_creating_a_board() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");

    assert!(matches!(
        store.create_board(board("board-1", "missing-project")),
        Err(BoardStoreError::ProjectNotFound { project_id }) if project_id == ProjectId::from("missing-project")
    ));
}

#[test]
fn rejects_duplicate_project_and_board_ids() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    prepared_board(&mut store);

    assert!(matches!(
        store.create_project(project("project-1")),
        Err(BoardStoreError::ProjectAlreadyExists { project_id }) if project_id == ProjectId::from("project-1")
    ));
    assert!(matches!(
        store.create_board(board("board-1", "project-1")),
        Err(BoardStoreError::BoardAlreadyExists { board_id }) if board_id == BoardId::from("board-1")
    ));
}

#[test]
fn creates_a_local_board_and_its_recent_entry_as_one_transaction() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");

    store
        .create_local_board(
            project("project-1"),
            board("board-1", "project-1"),
            "2026-08-09T08:00:00Z".to_owned(),
        )
        .expect("local board should persist atomically");

    assert!(
        store
            .project(&ProjectId::from("project-1"))
            .expect("project should load")
            .is_some()
    );
    assert!(
        store
            .board(&BoardId::from("board-1"))
            .expect("board should load")
            .is_some()
    );
    assert_eq!(
        store
            .board_library_records()
            .expect("board library should load")[0]
            .last_opened_at,
        Some("2026-08-09T08:00:00Z".to_owned())
    );
}

#[test]
fn rolls_back_local_board_creation_when_its_recent_entry_cannot_persist() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .connection
        .execute_batch("DROP TABLE board_access")
        .expect("test should remove the recency table");

    assert!(
        store
            .create_local_board(
                project("project-1"),
                board("board-1", "project-1"),
                "2026-08-09T08:00:00Z".to_owned(),
            )
            .is_err()
    );
    assert!(
        store
            .project(&ProjectId::from("project-1"))
            .expect("project lookup should work")
            .is_none()
    );
    assert!(
        store
            .board(&BoardId::from("board-1"))
            .expect("board lookup should work")
            .is_none()
    );
}

#[test]
fn requires_a_persisted_board_before_creating_a_work_item() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");

    assert!(matches!(
        store.create_board_work_item(create_work_item_command("task-1", "missing-board")),
        Err(BoardStoreError::BoardNotFound { board_id }) if board_id == BoardId::from("missing-board")
    ));
}

#[test]
fn rejects_cross_board_dependencies_without_changing_either_snapshot() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_project(project("project-1"))
        .expect("project should persist");
    store
        .create_board(board("board-1", "project-1"))
        .expect("first board should persist");
    store
        .create_board(board("board-2", "project-1"))
        .expect("second board should persist");
    store
        .create_board_work_item(create_work_item_command("task-1", "board-1"))
        .expect("first work item should persist");
    store
        .create_board_work_item(create_work_item_command("task-2", "board-2"))
        .expect("second work item should persist");

    assert!(matches!(
        store.add_board_dependency(dependency("cross-board", "task-1", "task-2")),
        Err(BoardStoreError::CrossBoardDependency { .. })
    ));
    assert!(
        store
            .board_snapshot(&BoardId::from("board-1"))
            .expect("first snapshot should load")
            .dependencies
            .is_empty()
    );
    assert!(
        store
            .board_snapshot(&BoardId::from("board-2"))
            .expect("second snapshot should load")
            .dependencies
            .is_empty()
    );
}

#[test]
fn validates_dependency_cycles_and_reuses_matching_dependency_commands() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    prepared_board(&mut store);
    for task_id in ["task-1", "task-2", "task-3"] {
        store
            .create_board_work_item(create_work_item_command(task_id, "board-1"))
            .expect("work item should persist");
    }
    let first_dependency = dependency("task-1-blocks-task-2", "task-1", "task-2");
    assert_eq!(
        store
            .add_board_dependency(first_dependency.clone())
            .expect("dependency should persist"),
        first_dependency
    );
    assert_eq!(
        store
            .add_board_dependency(first_dependency.clone())
            .expect("matching dependency should be idempotent"),
        first_dependency
    );
    store
        .add_board_dependency(dependency("task-2-blocks-task-3", "task-2", "task-3"))
        .expect("second dependency should persist");

    assert!(matches!(
        store.add_board_dependency(dependency("task-3-blocks-task-1", "task-3", "task-1")),
        Err(BoardStoreError::DependencyGraph(
            DependencyGraphError::HardDependencyCycle { .. }
        ))
    ));
}

#[test]
fn persists_recent_board_entries_with_derived_attention_counts() {
    let temporary_directory = TempDir::new().expect("temporary directory should exist");
    let repository_path = temporary_directory.path().join("project");
    let database_path = temporary_directory.path().join("board-library.sqlite");
    std::fs::create_dir(&repository_path).expect("repository directory should exist");
    let mut store = opened_store(&database_path);
    let mut project = project("project-1");
    project.repository_path = repository_path.display().to_string();
    store
        .create_project(project)
        .expect("project should persist");
    store
        .create_board(board("board-1", "project-1"))
        .expect("first board should persist");
    store
        .create_board(Board {
            schema: SchemaMetadata::current(),
            id: BoardId::from("board-2"),
            project_id: ProjectId::from("project-1"),
            name: "Planning".to_owned(),
        })
        .expect("second board should persist");
    store
        .create_board_work_item(create_work_item_command("task-1", "board-1"))
        .expect("work item should persist");
    store
        .create_board_work_item(create_work_item_command("task-2", "board-1"))
        .expect("second work item should persist");
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
        ("run-task-1", WorkItemState::Running),
    ] {
        store
            .transition_work_item(transition_command(event_id, "task-1", next_state))
            .expect("transition should persist");
    }
    for (event_id, next_state) in [
        ("plan-task-2", WorkItemState::Planned),
        ("ready-task-2", WorkItemState::Ready),
        ("run-task-2", WorkItemState::Running),
        ("review-task-2", WorkItemState::Review),
    ] {
        store
            .transition_work_item(transition_command(event_id, "task-2", next_state))
            .expect("second transition should persist");
    }
    store
        .record_board_opened(&BoardId::from("board-1"), "2026-08-09T08:00:00Z".to_owned())
        .expect("first board access should persist");
    store
        .record_board_opened(&BoardId::from("board-2"), "2026-08-09T09:00:00Z".to_owned())
        .expect("second board access should persist");
    drop(store);

    let reopened_store = opened_store(&database_path);
    let records = reopened_store
        .board_library_records()
        .expect("library should persist across reopen");

    assert_eq!(records.len(), 2);
    let running_board = records
        .iter()
        .find(|record| record.board.id == BoardId::from("board-1"))
        .expect("running board should remain in the library");
    assert_eq!(
        running_board.last_opened_at.as_deref(),
        Some("2026-08-09T08:00:00Z")
    );
    assert_eq!(running_board.attention.active_work_item_count, 1);
    assert_eq!(running_board.attention.needs_attention_count, 1);
    let recent_board = records
        .iter()
        .find(|record| record.board.id == BoardId::from("board-2"))
        .expect("second board should remain in the library");
    assert_eq!(
        recent_board.last_opened_at.as_deref(),
        Some("2026-08-09T09:00:00Z")
    );
}

#[test]
fn rejects_conflicting_dependency_identifier_and_formats_board_errors() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    prepared_board(&mut store);
    for task_id in ["task-1", "task-2", "task-3"] {
        store
            .create_board_work_item(create_work_item_command(task_id, "board-1"))
            .expect("work item should persist");
    }
    store
        .add_board_dependency(dependency("same-id", "task-1", "task-2"))
        .expect("first dependency should persist");

    assert_eq!(
        store
            .add_board_dependency(dependency("same-id", "task-1", "task-3"))
            .expect_err("changed duplicate dependency should fail")
            .to_string(),
        "dependency id same-id conflicts with a recorded dependency"
    );
}
