use std::path::Path;

use tempfile::TempDir;

use super::{
    BoardService, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
    ExecutionLaunchError, RecordExecutionRequest, TransitionWorkItemRequest,
    prepare_execution_launch,
};
use crate::{
    domain::{WorkItemBudget, WorkItemState},
    persistence::SqliteEventStore,
    workspace::{WorkspaceAssignment, WorkspaceManager},
};

fn prepared_launch() -> (
    TempDir,
    BoardService<SqliteEventStore>,
    WorkspaceManager,
    WorkspaceAssignment,
) {
    let (temporary_directory, repository_path) = crate::workspace::tests::repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = crate::workspace::tests::manager(&repository_path, &workspace_root);
    let assignment = manager
        .provision(crate::workspace::tests::request("task-1"))
        .expect("workspace should provision");
    let mut service = BoardService::new(SqliteEventStore::in_memory().expect("store should open"));
    create_board(&mut service, &repository_path);
    create_ready_task(&mut service);
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            adapter_name: "fake".to_owned(),
            workspace_path: assignment.path().display().to_string(),
        })
        .expect("execution should persist");
    (temporary_directory, service, manager, assignment)
}

fn create_board(service: &mut BoardService<SqliteEventStore>, repository_path: &Path) {
    service
        .create_project(CreateProjectRequest {
            project_id: "project-1".to_owned(),
            name: "Workspace project".to_owned(),
            repository_path: repository_path.display().to_string(),
            base_ref: "main".to_owned(),
            policy_set_id: "standard".to_owned(),
        })
        .expect("project should persist");
    service
        .create_board(CreateBoardRequest {
            board_id: "board-1".to_owned(),
            project_id: "project-1".to_owned(),
            name: "MVP".to_owned(),
        })
        .expect("board should persist");
}

fn create_ready_task(service: &mut BoardService<SqliteEventStore>) {
    service
        .create_work_item(CreateWorkItemRequest {
            event_id: "create-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            board_id: "board-1".to_owned(),
            title: "Implement task one".to_owned(),
            description: "A bounded implementation task.".to_owned(),
            acceptance_criteria: vec!["Tests pass.".to_owned()],
            budget: WorkItemBudget::default(),
            requires_human_review: false,
            recorded_at: "2026-08-08T00:00:00Z".to_owned(),
        })
        .expect("task should persist");
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
    ] {
        service
            .transition_work_item(TransitionWorkItemRequest {
                event_id: event_id.to_owned(),
                work_item_id: "task-1".to_owned(),
                next_state,
                evidence: None,
                reason: "The worker can progress.".to_owned(),
                recorded_at: "2026-08-08T00:01:00Z".to_owned(),
            })
            .expect("task should progress");
    }
}

#[test]
fn prepares_a_request_only_for_the_recorded_adapter_and_verified_worktree() {
    let (_temporary_directory, service, manager, assignment) = prepared_launch();
    let execution = service
        .execution(&crate::domain::ExecutionId::from("execution-1"))
        .expect("execution should load");
    let work_item = service
        .work_item(&execution.work_item_id)
        .expect("work item should load");

    let preparation = prepare_execution_launch(
        &execution,
        &work_item,
        &manager,
        &assignment,
        "fake",
        "Implement the acceptance criteria and report evidence.",
    )
    .expect("matching execution should prepare");

    assert_eq!(preparation.execution_id(), "execution-1");
    assert_eq!(preparation.request().work_item_id, "task-1");
    assert_eq!(
        preparation.request().workspace_path,
        assignment.path().display().to_string()
    );
}

#[test]
fn rejects_a_configured_adapter_that_does_not_match_the_execution_record() {
    let (_temporary_directory, service, manager, assignment) = prepared_launch();
    let execution = service
        .execution(&crate::domain::ExecutionId::from("execution-1"))
        .expect("execution should load");
    let work_item = service
        .work_item(&execution.work_item_id)
        .expect("work item should load");

    assert!(matches!(
        prepare_execution_launch(
            &execution,
            &work_item,
            &manager,
            &assignment,
            "other-adapter",
            "Implement the task.",
        ),
        Err(ExecutionLaunchError::AdapterNameMismatch { .. })
    ));
}

#[test]
fn rejects_a_blank_brief_before_it_can_reach_an_agent_process() {
    let (_temporary_directory, service, manager, assignment) = prepared_launch();
    let execution = service
        .execution(&crate::domain::ExecutionId::from("execution-1"))
        .expect("execution should load");
    let work_item = service
        .work_item(&execution.work_item_id)
        .expect("work item should load");

    assert!(matches!(
        prepare_execution_launch(&execution, &work_item, &manager, &assignment, "fake", " "),
        Err(ExecutionLaunchError::MissingRequiredField {
            field: "task brief"
        })
    ));
}
