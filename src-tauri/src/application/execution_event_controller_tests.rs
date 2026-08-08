use super::{
    BoardService, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
    ExecutionEventController, ExecutionEventControllerError, RecordExecutionRequest,
    TransitionWorkItemRequest,
};
use crate::{
    agent::{NormalizedAgentEvent, NormalizedAgentEventKind},
    domain::{
        EvidenceKind, EvidenceResult, ExecutionRole, ExecutionStatus, WorkItemBudget, WorkItemState,
    },
    persistence::SqliteEventStore,
};

fn service() -> BoardService<SqliteEventStore> {
    BoardService::new(SqliteEventStore::in_memory().expect("event store should open"))
}

fn prepared_service() -> BoardService<SqliteEventStore> {
    let mut service = service();
    service
        .create_project(CreateProjectRequest {
            project_id: "project-1".to_owned(),
            name: "Desktop application".to_owned(),
            repository_path: "/projects/desktop-application".to_owned(),
            base_ref: "main".to_owned(),
            policy_set_id: "standard".to_owned(),
        })
        .expect("project should be created");
    service
        .create_board(CreateBoardRequest {
            board_id: "board-1".to_owned(),
            project_id: "project-1".to_owned(),
            name: "MVP".to_owned(),
        })
        .expect("board should be created");
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
        .expect("task should be created");
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
                reason: "The worker is ready to progress.".to_owned(),
                recorded_at: "2026-08-08T00:01:00Z".to_owned(),
            })
            .expect("task should transition");
    }
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            role: Default::default(),
            adapter_name: "fake".to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        })
        .expect("execution should be recorded");
    service
}

fn event(sequence: u64, kind: NormalizedAgentEventKind) -> NormalizedAgentEvent {
    NormalizedAgentEvent { sequence, kind }
}

#[test]
fn activation_attaches_one_nonblank_session_to_a_pending_execution() {
    let mut service = prepared_service();

    assert!(matches!(
        ExecutionEventController::activate(
            &mut service,
            "execution-1",
            " ",
            "2026-08-08T00:01:00Z",
        ),
        Err(ExecutionEventControllerError::MissingSessionId)
    ));
    let snapshot = ExecutionEventController::activate(
        &mut service,
        "execution-1",
        "session-1",
        "2026-08-08T00:01:00Z",
    )
    .expect("pending execution should become active");
    assert_eq!(snapshot.executions[0].status, ExecutionStatus::Running);
    assert_eq!(
        snapshot.executions[0].session_id.as_deref(),
        Some("session-1")
    );
    assert_eq!(
        snapshot.work_items[0].work_item.state,
        WorkItemState::Running
    );
    let retry = ExecutionEventController::activate(
        &mut service,
        "execution-1",
        "session-1",
        "2026-08-08T00:01:00Z",
    )
    .expect("replaying the same activation should be safe");
    assert_eq!(retry.executions[0].status, ExecutionStatus::Running);
}

#[test]
fn records_monotonic_usage_and_an_agent_input_request_durably() {
    let mut service = prepared_service();
    ExecutionEventController::activate(
        &mut service,
        "execution-1",
        "session-1",
        "2026-08-08T00:01:00Z",
    )
    .expect("execution should start");

    ExecutionEventController::record_event(
        &mut service,
        "execution-1",
        event(
            1,
            NormalizedAgentEventKind::UsageUpdated {
                input_tokens: 11,
                output_tokens: 13,
                cost_micros: Some(17),
            },
        ),
        "2026-08-08T00:02:00Z",
    )
    .expect("usage checkpoint should persist");
    let snapshot = ExecutionEventController::record_event(
        &mut service,
        "execution-1",
        event(
            2,
            NormalizedAgentEventKind::AwaitingInput {
                question: "Which API version should I use?".to_owned(),
            },
        ),
        "2026-08-08T00:03:00Z",
    )
    .expect("input request should persist");

    assert_eq!(
        snapshot.work_items[0].work_item.state,
        WorkItemState::AwaitingInput
    );
    assert_eq!(
        snapshot.executions[0].status,
        ExecutionStatus::AwaitingInput
    );
    assert_eq!(snapshot.executions[0].last_event_sequence, 2);
    assert_eq!(snapshot.executions[0].usage.input_tokens, 11);
    assert_eq!(snapshot.evidence.len(), 1);
    assert_eq!(snapshot.evidence[0].kind, EvidenceKind::AgentReport);
    assert_eq!(snapshot.evidence[0].result, EvidenceResult::Recorded);
    assert!(snapshot.evidence[0].summary.contains("API version"));
}

#[test]
fn completion_requires_review_and_repeated_review_events_remain_auditable() {
    let mut service = prepared_service();
    ExecutionEventController::activate(
        &mut service,
        "execution-1",
        "session-1",
        "2026-08-08T00:01:00Z",
    )
    .expect("execution should start");

    let first_snapshot = ExecutionEventController::record_event(
        &mut service,
        "execution-1",
        event(
            1,
            NormalizedAgentEventKind::Completed {
                summary: "Implementation is ready for review.".to_owned(),
            },
        ),
        "2026-08-08T00:04:00Z",
    )
    .expect("completion should hand off to review");
    let second_snapshot = ExecutionEventController::record_event(
        &mut service,
        "execution-1",
        event(
            2,
            NormalizedAgentEventKind::AwaitingReview {
                summary: "Please inspect the implementation and checks.".to_owned(),
            },
        ),
        "2026-08-08T00:05:00Z",
    )
    .expect("a follow-up review report should not create an illegal self-transition");

    assert_eq!(
        first_snapshot.work_items[0].work_item.state,
        WorkItemState::Review
    );
    assert_eq!(
        first_snapshot.executions[0].status,
        ExecutionStatus::AwaitingReview
    );
    assert_eq!(
        first_snapshot.evidence[0].kind,
        EvidenceKind::CompletionReport
    );
    assert_eq!(
        second_snapshot.work_items[0].work_item.state,
        WorkItemState::Review
    );
    assert_eq!(second_snapshot.executions[0].last_event_sequence, 2);
    assert_eq!(second_snapshot.evidence.len(), 2);
}

#[test]
fn independent_reviewer_events_remain_in_review_and_never_replace_implementation_evidence() {
    let mut service = prepared_service();
    for (event_id, next_state) in [
        ("run-task-1", WorkItemState::Running),
        ("review-task-1", WorkItemState::Review),
    ] {
        service
            .transition_work_item(TransitionWorkItemRequest {
                event_id: event_id.to_owned(),
                work_item_id: "task-1".to_owned(),
                next_state,
                evidence: None,
                reason: "Prepare an independent review.".to_owned(),
                recorded_at: "2026-08-08T00:02:00Z".to_owned(),
            })
            .expect("task should enter review");
    }
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "review-execution".to_owned(),
            work_item_id: "task-1".to_owned(),
            role: ExecutionRole::IndependentReview,
            adapter_name: "independent-reviewer".to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        })
        .expect("review execution should persist");
    ExecutionEventController::activate(
        &mut service,
        "review-execution",
        "review-session",
        "2026-08-08T00:03:00Z",
    )
    .expect("review execution should start");

    let snapshot = ExecutionEventController::record_event(
        &mut service,
        "review-execution",
        event(
            1,
            NormalizedAgentEventKind::Completed {
                summary: "No findings were found.".to_owned(),
            },
        ),
        "2026-08-08T00:04:00Z",
    )
    .expect("review result should persist");

    assert_eq!(
        snapshot.work_items[0].work_item.state,
        WorkItemState::Review
    );
    assert!(matches!(
        snapshot.executions.iter().find(|execution| execution.id.0 == "review-execution"),
        Some(execution) if execution.status == ExecutionStatus::Completed
    ));
    assert!(matches!(
        snapshot.evidence.last(),
        Some(evidence) if evidence.kind == EvidenceKind::AgentReport
            && evidence.result == EvidenceResult::Recorded
    ));
}

#[test]
fn failure_and_interruption_remain_distinct_recovery_outcomes_with_reports() {
    for (event_kind, task_state, execution_status, evidence_result) in [
        (
            NormalizedAgentEventKind::Failed {
                reason: "The checks failed.".to_owned(),
            },
            WorkItemState::Failed,
            ExecutionStatus::Failed,
            EvidenceResult::Failed,
        ),
        (
            NormalizedAgentEventKind::Interrupted {
                reason: "The worker process stopped.".to_owned(),
            },
            WorkItemState::Interrupted,
            ExecutionStatus::Interrupted,
            EvidenceResult::Recorded,
        ),
    ] {
        let mut service = prepared_service();
        ExecutionEventController::activate(
            &mut service,
            "execution-1",
            "session-1",
            "2026-08-08T00:01:00Z",
        )
        .expect("execution should start");

        let snapshot = ExecutionEventController::record_event(
            &mut service,
            "execution-1",
            event(1, event_kind),
            "2026-08-08T00:06:00Z",
        )
        .expect("terminal agent report should persist");

        assert_eq!(snapshot.work_items[0].work_item.state, task_state);
        assert_eq!(snapshot.executions[0].status, execution_status);
        assert_eq!(snapshot.evidence[0].kind, EvidenceKind::AgentReport);
        assert_eq!(snapshot.evidence[0].result, evidence_result);
    }
}

#[test]
fn rejects_agent_events_until_activation_and_when_the_sequence_is_not_next() {
    let mut service = prepared_service();
    let activity = event(
        1,
        NormalizedAgentEventKind::Activity {
            summary: "Reading files.".to_owned(),
        },
    );

    assert!(matches!(
        ExecutionEventController::record_event(
            &mut service,
            "execution-1",
            activity.clone(),
            "2026-08-08T00:02:00Z",
        ),
        Err(ExecutionEventControllerError::ExecutionNotActive { .. })
    ));
    ExecutionEventController::activate(
        &mut service,
        "execution-1",
        "session-1",
        "2026-08-08T00:01:00Z",
    )
    .expect("execution should start");
    assert!(matches!(
        ExecutionEventController::record_event(
            &mut service,
            "execution-1",
            event(
                2,
                NormalizedAgentEventKind::Activity {
                    summary: "Skipped event.".to_owned(),
                },
            ),
            "2026-08-08T00:03:00Z",
        ),
        Err(ExecutionEventControllerError::AgentAdapter(_))
    ));
    ExecutionEventController::record_event(
        &mut service,
        "execution-1",
        activity,
        "2026-08-08T00:04:00Z",
    )
    .expect("the expected event should remain acceptable after a rejected gap");
}
