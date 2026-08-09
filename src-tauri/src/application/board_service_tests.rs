use super::{
    AddDependencyRequest, BoardService, BoardServiceError, CreateBoardRequest,
    CreateProjectRequest, CreateWorkItemRequest, RecordEvidenceRequest, RecordExecutionRequest,
    TransitionWorkItemRequest, UpdateExecutionRequest,
};
use crate::domain::{
    CompletionEvidence, DependencyKind, EvidenceKind, EvidenceResult, ExecutionStatus,
    ExecutionUsage, WorkItemBudget, WorkItemId, WorkItemState,
};
use crate::orchestration::PlannerProfile;
use crate::persistence::{BoardStoreError, EventStoreError, SqliteEventStore};

pub(super) fn service() -> BoardService<SqliteEventStore> {
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

pub(super) fn create_work_item_request(id: &str) -> CreateWorkItemRequest {
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

pub(super) fn transition_request(
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

fn execution_request(work_item_id: &str) -> RecordExecutionRequest {
    RecordExecutionRequest {
        execution_id: "execution-1".to_owned(),
        work_item_id: work_item_id.to_owned(),
        role: Default::default(),
        adapter_name: "codex-cli".to_owned(),
        workspace_path: "/workspaces/task-1".to_owned(),
    }
}

fn evidence_request(work_item_id: &str) -> RecordEvidenceRequest {
    RecordEvidenceRequest {
        evidence_id: "evidence-1".to_owned(),
        work_item_id: work_item_id.to_owned(),
        kind: EvidenceKind::Check,
        result: EvidenceResult::Passed,
        summary: "The required checks passed.".to_owned(),
        recorded_at: "2026-08-08T00:02:00Z".to_owned(),
    }
}

pub(super) fn create_board(service: &mut BoardService<SqliteEventStore>) {
    service
        .create_project(create_project_request())
        .expect("project should be created");
    service
        .create_board(create_board_request())
        .expect("board should be created");
}

mod clean_code_review;
mod plan;
mod review;

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
    assert_eq!(planned_snapshot.activity.len(), 2);
    assert!(planned_snapshot.executions.is_empty());
    assert!(planned_snapshot.evidence.is_empty());
    assert_eq!(
        planned_snapshot.activity[1].summary,
        "State changed from inbox to planned: The user requested the lifecycle update."
    );
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
fn resolves_a_saved_planner_profile_and_the_declared_project_repository() {
    let mut service = service();
    create_board(&mut service);
    service
        .save_planner_profile(PlannerProfile {
            name: "local planner".to_owned(),
            program: "planner-bridge".to_owned(),
            arguments: vec!["--strict-json".to_owned()],
        })
        .expect("planner profile should save");

    let context = service
        .planner_context("board-1", "local planner")
        .expect("planner context should resolve");

    assert_eq!(context.profile.name, "local planner");
    assert_eq!(context.repository_path, "/projects/desktop-application");
    assert!(
        service
            .snapshot(&"board-1".into())
            .expect("board should remain readable")
            .work_items
            .is_empty()
    );
}

#[test]
fn records_a_pending_worker_attempt_and_review_evidence_through_the_use_case() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
    ] {
        service
            .transition_work_item(transition_request(event_id, "task-1", next_state, None))
            .expect("task should become ready before execution is recorded");
    }

    let execution_snapshot = service
        .record_execution(execution_request("task-1"))
        .expect("execution should be recorded");
    let evidence_snapshot = service
        .record_evidence(evidence_request("task-1"))
        .expect("evidence should be recorded");

    assert_eq!(execution_snapshot.executions.len(), 1);
    assert_eq!(
        execution_snapshot.executions[0].status,
        ExecutionStatus::Pending
    );
    let updated_execution_snapshot = service
        .update_execution(UpdateExecutionRequest {
            execution_id: "execution-1".to_owned(),
            status: ExecutionStatus::Running,
            session_id: Some("session-1".to_owned()),
            usage: ExecutionUsage {
                input_tokens: 10,
                output_tokens: 5,
                cost_micros: Some(100),
            },
            last_event_sequence: 1,
        })
        .expect("execution should start");
    assert_eq!(
        updated_execution_snapshot.executions[0].status,
        ExecutionStatus::Running
    );
    assert_eq!(
        updated_execution_snapshot.executions[0]
            .session_id
            .as_deref(),
        Some("session-1")
    );
    assert_eq!(evidence_snapshot.evidence.len(), 1);
    assert_eq!(
        evidence_snapshot.evidence[0].summary,
        "The required checks passed."
    );
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
    let completion = CompletionEvidence {
        quality_gate_passed: true,
        completion_report_present: true,
        review_accepted: false,
    };
    assert!(matches!(
        service.transition_work_item(transition_request(
            "done-task-1-without-records",
            "task-1",
            WorkItemState::Done,
            Some(completion),
        )),
        Err(BoardServiceError::MissingRecordedEvidence {
            kind: EvidenceKind::CompletionReport,
            result: EvidenceResult::Recorded,
            ..
        })
    ));
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "report-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::CompletionReport,
            result: EvidenceResult::Recorded,
            summary: "The agent submitted a completion report.".to_owned(),
            recorded_at: "2026-08-08T00:02:00Z".to_owned(),
        })
        .expect("completion report should persist");
    assert!(matches!(
        service.transition_work_item(transition_request(
            "done-task-1-without-quality-gate",
            "task-1",
            WorkItemState::Done,
            Some(completion),
        )),
        Err(BoardServiceError::MissingRecordedEvidence {
            kind: EvidenceKind::QualityGate,
            result: EvidenceResult::Passed,
            ..
        })
    ));
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "quality-gate-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::QualityGate,
            result: EvidenceResult::Passed,
            summary: "The full quality gate passed.".to_owned(),
            recorded_at: "2026-08-08T00:02:01Z".to_owned(),
        })
        .expect("quality gate should persist");
    let completed_snapshot = service
        .transition_work_item(transition_request(
            "done-task-1-with-evidence",
            "task-1",
            WorkItemState::Done,
            Some(completion),
        ))
        .expect("complete evidence should allow done");
    assert_eq!(
        completed_snapshot.work_items[0].work_item.state,
        WorkItemState::Done
    );
    assert_eq!(
        completed_snapshot
            .activity
            .last()
            .expect("completion activity should be retained")
            .completion_evidence,
        Some(completion)
    );
}

#[test]
fn bounds_snapshot_activity_without_discarding_durable_event_history() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");

    service
        .transition_work_item(transition_request(
            "transition-planned",
            "task-1",
            WorkItemState::Planned,
            None,
        ))
        .expect("the task should enter planning");
    let transitions = [
        WorkItemState::Ready,
        WorkItemState::Running,
        WorkItemState::Failed,
    ];
    for (index, next_state) in transitions.iter().cycle().take(24).enumerate() {
        let event_id = format!("transition-{index}");
        service
            .transition_work_item(transition_request(&event_id, "task-1", *next_state, None))
            .expect("recovery transition should be valid");
    }

    let snapshot = service
        .snapshot(&crate::domain::BoardId::from("board-1"))
        .expect("board snapshot should load");
    assert_eq!(snapshot.activity.len(), 20);
    assert_eq!(snapshot.activity[0].sequence, 7);
    assert_eq!(snapshot.activity[19].sequence, 26);
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
