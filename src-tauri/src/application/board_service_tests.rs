use super::{
    AddDependencyRequest, BoardService, BoardServiceError, CreateBoardRequest,
    CreateProjectRequest, CreateWorkItemRequest, TransitionWorkItemRequest,
};
use crate::domain::{
    CompletionEvidence, DependencyKind, WorkItemBudget, WorkItemId, WorkItemState,
};
use crate::persistence::{BoardStoreError, EventStoreError, SqliteEventStore};

fn service() -> BoardService<SqliteEventStore> {
    BoardService::new(SqliteEventStore::in_memory().expect("event store should open"))
}

fn create_project_request() -> CreateProjectRequest {
    CreateProjectRequest {
        project_id: "project-1".to_owned(),
        name: "Desktop application".to_owned(),
        repository_path: "/projects/desktop-application".to_owned(),
        base_ref: "main".to_owned(),
        policy_set_id: "standard".to_owned(),
    }
}

fn create_board_request() -> CreateBoardRequest {
    CreateBoardRequest {
        board_id: "board-1".to_owned(),
        project_id: "project-1".to_owned(),
        name: "MVP".to_owned(),
    }
}

fn create_work_item_request(id: &str) -> CreateWorkItemRequest {
    CreateWorkItemRequest {
        event_id: format!("create-{id}"),
        work_item_id: id.to_owned(),
        board_id: "board-1".to_owned(),
        title: format!("Implement {id}"),
        description: "A bounded implementation task.".to_owned(),
        acceptance_criteria: vec!["Tests pass.".to_owned()],
        budget: WorkItemBudget::default(),
        requires_human_review: false,
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn transition_request(
    event_id: &str,
    work_item_id: &str,
    next_state: WorkItemState,
    evidence: Option<CompletionEvidence>,
) -> TransitionWorkItemRequest {
    TransitionWorkItemRequest {
        event_id: event_id.to_owned(),
        work_item_id: work_item_id.to_owned(),
        next_state,
        evidence,
        reason: "The user requested the lifecycle update.".to_owned(),
        recorded_at: "2026-08-08T00:01:00Z".to_owned(),
    }
}

fn create_board(service: &mut BoardService<SqliteEventStore>) {
    service
        .create_project(create_project_request())
        .expect("project should be created");
    service
        .create_board(create_board_request())
        .expect("board should be created");
}

#[test]
fn creates_a_durable_board_task_and_guarded_transition_through_the_use_case() {
    let mut service = service();
    create_board(&mut service);

    let created_snapshot = service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    let planned_snapshot = service
        .transition_work_item(transition_request(
            "plan-task-1",
            "task-1",
            WorkItemState::Planned,
            None,
        ))
        .expect("work item should be planned");

    assert_eq!(
        created_snapshot.work_items[0].work_item.state,
        WorkItemState::Inbox
    );
    assert_eq!(
        planned_snapshot.work_items[0].work_item.state,
        WorkItemState::Planned
    );
    assert_eq!(planned_snapshot.board.id.0, "board-1");
}

#[test]
fn rejects_empty_product_input_before_it_reaches_the_repository() {
    let mut service = service();
    let mut request = create_project_request();
    request.repository_path = "  ".to_owned();

    assert!(matches!(
        service.create_project(request),
        Err(BoardServiceError::MissingRequiredField {
            field: "repository path"
        })
    ));

    create_board(&mut service);
    let mut work_item_request = create_work_item_request("task-1");
    work_item_request.acceptance_criteria = vec!["".to_owned()];
    assert!(matches!(
        service.create_work_item(work_item_request),
        Err(BoardServiceError::InvalidAcceptanceCriteria)
    ));
}

#[test]
fn preserves_done_evidence_requirements_at_the_command_use_case() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
        ("run-task-1", WorkItemState::Running),
        ("review-task-1", WorkItemState::Review),
    ] {
        service
            .transition_work_item(transition_request(event_id, "task-1", next_state, None))
            .expect("valid intermediate transition should succeed");
    }

    assert!(matches!(
        service.transition_work_item(transition_request(
            "done-task-1",
            "task-1",
            WorkItemState::Done,
            None,
        )),
        Err(BoardServiceError::Repository(BoardStoreError::EventStore(
            EventStoreError::StateTransition(_)
        )))
    ));
    let completed_snapshot = service
        .transition_work_item(transition_request(
            "done-task-1-with-evidence",
            "task-1",
            WorkItemState::Done,
            Some(CompletionEvidence {
                checks_passed: true,
                completion_report_present: true,
                review_accepted: true,
            }),
        ))
        .expect("complete evidence should allow done");
    assert_eq!(
        completed_snapshot.work_items[0].work_item.state,
        WorkItemState::Done
    );
}

#[test]
fn creates_user_owned_typed_dependencies_and_returns_the_updated_snapshot() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("first work item should be created");
    service
        .create_work_item(create_work_item_request("task-2"))
        .expect("second work item should be created");

    let snapshot = service
        .add_dependency(AddDependencyRequest {
            dependency_id: "task-1-blocks-task-2".to_owned(),
            upstream_work_item_id: "task-1".to_owned(),
            downstream_work_item_id: "task-2".to_owned(),
            kind: DependencyKind::Blocks,
            reason: "The second task needs the first task's API.".to_owned(),
            owner: "user".to_owned(),
            next_action: "Complete the first task.".to_owned(),
            created_by: "user".to_owned(),
            created_at: "2026-08-08T00:02:00Z".to_owned(),
        })
        .expect("dependency should be created");

    assert_eq!(snapshot.dependencies.len(), 1);
    assert_eq!(
        snapshot.dependencies[0].source,
        crate::domain::DependencySource::User
    );
}

#[test]
fn reports_a_missing_work_item_without_inventing_a_board_snapshot() {
    let mut service = service();
    create_board(&mut service);

    assert!(matches!(
        service.transition_work_item(transition_request(
            "plan-missing-task",
            "missing-task",
            WorkItemState::Planned,
            None,
        )),
        Err(BoardServiceError::WorkItemNotFound { work_item_id }) if work_item_id == WorkItemId::from("missing-task")
    ));
}
