use std::path::Path;

use tempfile::TempDir;

use super::{BoardStoreError, SqliteEventStore};
use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, DependencyGraphError, DependencyId,
    DependencyKind, DependencySource, Project, ProjectId, SchemaMetadata, WorkItem, WorkItemBudget,
    WorkItemEventId, WorkItemId, WorkItemState,
};

fn project(id: &str) -> Project {
    Project {
        schema: SchemaMetadata::current(),
        id: ProjectId::from(id),
        name: "Desktop application".to_owned(),
        repository_path: "/projects/desktop-application".to_owned(),
        base_ref: "main".to_owned(),
        policy_set_id: "standard".to_owned(),
    }
}

fn board(id: &str, project_id: &str) -> Board {
    Board {
        schema: SchemaMetadata::current(),
        id: BoardId::from(id),
        project_id: ProjectId::from(project_id),
        name: "MVP".to_owned(),
    }
}

fn work_item(id: &str, board_id: &str) -> WorkItem {
    WorkItem {
        schema: SchemaMetadata::current(),
        id: WorkItemId::from(id),
        board_id: BoardId::from(board_id),
        title: format!("Implement {id}"),
        description: "A bounded implementation task.".to_owned(),
        acceptance_criteria: vec!["Tests pass.".to_owned()],
        budget: WorkItemBudget::default(),
        state: WorkItemState::Inbox,
        requires_human_review: false,
    }
}

fn create_work_item_command(id: &str, board_id: &str) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        event_id: WorkItemEventId::from(format!("create-{id}").as_str()),
        work_item: work_item(id, board_id),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn dependency(id: &str, upstream: &str, downstream: &str) -> Dependency {
    Dependency {
        schema: SchemaMetadata::current(),
        id: DependencyId::from(id),
        upstream_work_item_id: WorkItemId::from(upstream),
        downstream_work_item_id: WorkItemId::from(downstream),
        kind: DependencyKind::Blocks,
        source: DependencySource::Orchestrator,
        reason: "The downstream task requires the upstream result.".to_owned(),
        owner: "orchestrator".to_owned(),
        next_action: "Complete the upstream task.".to_owned(),
        created_by: "planner".to_owned(),
        created_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn opened_store(path: &Path) -> SqliteEventStore {
    SqliteEventStore::open(path).expect("event store should open")
}

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
