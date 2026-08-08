#![cfg(unix)]

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    agent::{AgentProfile, AgentProfileKind},
    application::{
        BoardService, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
        ExecutionEventController, RecordExecutionRequest, StartExecutionRequest,
        TransitionWorkItemRequest,
    },
    desktop::LocalBoardService,
    desktop_execution_runtime::ExecutionRuntime,
    desktop_execution_runtime_support::ExecutionRuntimeError,
    domain::{ExecutionId, WorkItemBudget, WorkItemState},
    persistence::SqliteEventStore,
};

fn prepared_runtime(
    policy_set_id: &str,
) -> (
    tempfile::TempDir,
    Arc<Mutex<LocalBoardService>>,
    ExecutionRuntime,
) {
    let (temporary_directory, repository_path) = crate::workspace::tests::repository();
    let mut service = BoardService::new(SqliteEventStore::in_memory().expect("store should open"));
    service
        .create_project(CreateProjectRequest {
            project_id: "project-1".to_owned(),
            name: "Runtime project".to_owned(),
            repository_path: repository_path.display().to_string(),
            base_ref: "main".to_owned(),
            policy_set_id: policy_set_id.to_owned(),
        })
        .expect("project should persist");
    service
        .create_board(CreateBoardRequest {
            board_id: "board-1".to_owned(),
            project_id: "project-1".to_owned(),
            name: "MVP".to_owned(),
        })
        .expect("board should persist");
    create_ready_work_item(&mut service, "task-1", "board-1");
    service
        .save_agent_profile(AgentProfile {
            name: "structured-script".to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "sh".to_owned(),
            arguments: vec![
                "-c".to_owned(),
                "IFS= read -r brief; [ \"$brief\" = \"Complete the task.\" ] || exit 7; printf '%s\\n' '{\"sequence\":1,\"type\":\"activity\",\"summary\":\"Working\"}' '{\"sequence\":2,\"type\":\"completed\",\"summary\":\"Ready for review\"}'".to_owned(),
            ],
        })
        .expect("agent profile should persist");
    let service = Arc::new(Mutex::new(service));
    let runtime = ExecutionRuntime::new(
        service.clone(),
        temporary_directory.path().join("workspaces"),
    );
    (temporary_directory, service, runtime)
}

fn create_ready_work_item(service: &mut LocalBoardService, work_item_id: &str, board_id: &str) {
    service
        .create_work_item(CreateWorkItemRequest {
            event_id: format!("create-{work_item_id}"),
            work_item_id: work_item_id.to_owned(),
            board_id: board_id.to_owned(),
            title: format!("Run {work_item_id}"),
            description: "A task whose worker emits normalized events.".to_owned(),
            acceptance_criteria: vec!["The worker reports completion.".to_owned()],
            budget: WorkItemBudget::default(),
            requires_human_review: true,
            recorded_at: "2026-08-08T00:00:00Z".to_owned(),
        })
        .expect("work item should persist");
    for (event_id, next_state) in [
        (format!("plan-{work_item_id}"), WorkItemState::Planned),
        (format!("ready-{work_item_id}"), WorkItemState::Ready),
    ] {
        service
            .transition_work_item(TransitionWorkItemRequest {
                event_id,
                work_item_id: work_item_id.to_owned(),
                next_state,
                evidence: None,
                reason: "The worker is eligible to start.".to_owned(),
                recorded_at: "2026-08-08T00:01:00Z".to_owned(),
            })
            .expect("work item should become ready");
    }
}

#[test]
fn starts_a_verified_worker_and_records_its_review_outcome_in_the_background() {
    let (_temporary_directory, service, runtime) = prepared_runtime("standard");

    let snapshot = runtime
        .start(StartExecutionRequest {
            execution_id: "execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            agent_profile_name: "structured-script".to_owned(),
            task_brief: "Complete the task.".to_owned(),
            execution_role: Default::default(),
        })
        .expect("runtime should start the configured worker");
    assert_eq!(
        snapshot.work_items[0].work_item.state,
        WorkItemState::Running
    );

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let service = service.lock().expect("service should remain available");
        let execution = service
            .execution(&ExecutionId::from("execution-1"))
            .expect("execution should persist");
        let work_item = service
            .work_item(&crate::domain::WorkItemId::from("task-1"))
            .expect("work item should persist");
        if execution.status == crate::domain::ExecutionStatus::AwaitingReview {
            assert_eq!(work_item.work_item.state, WorkItemState::Review);
            assert!(
                service
                    .snapshot(&crate::domain::BoardId::from("board-1"))
                    .expect("review evidence should persist")
                    .evidence
                    .iter()
                    .any(|evidence| {
                        evidence.kind == crate::domain::EvidenceKind::CleanCodeReview
                            && evidence.result == crate::domain::EvidenceResult::Recorded
                    })
            );
            return;
        }
        drop(service);
        if Instant::now() >= deadline {
            panic!("the detached runtime should record the agent completion event");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn refuses_an_unsupported_policy_before_provisioning_a_workspace_or_process() {
    let (temporary_directory, service, runtime) = prepared_runtime("restricted");

    let error = runtime
        .start(StartExecutionRequest {
            execution_id: "execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            agent_profile_name: "structured-script".to_owned(),
            task_brief: "Complete the task.".to_owned(),
            execution_role: Default::default(),
        })
        .expect_err("the unsupported policy must block process launch");

    assert!(matches!(
        error,
        ExecutionRuntimeError::UnsupportedPolicySet { policy_set_id } if policy_set_id == "restricted"
    ));
    assert!(
        !temporary_directory.path().join("workspaces").exists(),
        "policy rejection must precede worktree provisioning"
    );
    assert_eq!(
        service
            .lock()
            .expect("service should remain available")
            .work_item(&crate::domain::WorkItemId::from("task-1"))
            .expect("task should persist")
            .work_item
            .state,
        WorkItemState::Ready
    );
}

#[test]
fn refuses_a_second_project_execution_before_worker_start() {
    let (temporary_directory, service, runtime) = prepared_runtime("standard");
    {
        let mut service = service.lock().expect("service should remain available");
        service
            .create_board(CreateBoardRequest {
                board_id: "board-2".to_owned(),
                project_id: "project-1".to_owned(),
                name: "Second board".to_owned(),
            })
            .expect("board should persist");
        create_ready_work_item(&mut service, "task-2", "board-2");
        service
            .record_execution(RecordExecutionRequest {
                execution_id: "execution-active".to_owned(),
                work_item_id: "task-2".to_owned(),
                role: Default::default(),
                adapter_name: "other-worker".to_owned(),
                workspace_path: "/tmp/other-worker".to_owned(),
            })
            .expect("active execution should persist");
        ExecutionEventController::activate(
            &mut service,
            "execution-active",
            "session-active",
            "2026-08-08T00:02:00Z",
        )
        .expect("active execution should attach its session");
    }

    let error = runtime
        .start(StartExecutionRequest {
            execution_id: "execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            agent_profile_name: "structured-script".to_owned(),
            task_brief: "Complete the task.".to_owned(),
            execution_role: Default::default(),
        })
        .expect_err("the standard policy permits only one project execution");

    assert!(matches!(error, ExecutionRuntimeError::PolicyDenied { .. }));
    assert!(
        !temporary_directory.path().join("workspaces").exists(),
        "a denied audit decision must precede worktree provisioning"
    );
}

#[test]
fn fails_a_feedback_request_from_a_profile_that_cannot_resume() {
    let (_temporary_directory, service, runtime) = prepared_runtime("standard");
    service
        .lock()
        .expect("service should remain available")
        .save_agent_profile(AgentProfile {
            name: "input-requesting-script".to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "sh".to_owned(),
            arguments: vec![
                "-c".to_owned(),
                "IFS= read -r brief; [ \"$brief\" = \"Need input.\" ] || exit 7; printf '%s\\n' '{\"sequence\":1,\"type\":\"awaiting_input\",\"question\":\"Choose an option.\"}'; sleep 5".to_owned(),
            ],
        })
        .expect("input-requesting profile should persist");

    runtime
        .start(StartExecutionRequest {
            execution_id: "execution-input".to_owned(),
            work_item_id: "task-1".to_owned(),
            agent_profile_name: "input-requesting-script".to_owned(),
            task_brief: "Need input.".to_owned(),
            execution_role: Default::default(),
        })
        .expect("runtime should begin the worker before it requests feedback");

    for _ in 0..50 {
        let service = service.lock().expect("service should remain available");
        let execution = service
            .execution(&ExecutionId::from("execution-input"))
            .expect("execution should persist");
        let work_item = service
            .work_item(&crate::domain::WorkItemId::from("task-1"))
            .expect("work item should persist");
        if execution.status == crate::domain::ExecutionStatus::Failed {
            assert_eq!(work_item.work_item.state, WorkItemState::Failed);
            return;
        }
        drop(service);
        thread::sleep(Duration::from_millis(20));
    }
    panic!("a generic profile must not remain falsely awaiting unavailable feedback");
}

#[test]
fn stops_a_live_direct_process_and_records_an_interrupted_attempt() {
    let (_temporary_directory, service, runtime) = prepared_runtime("standard");
    service
        .lock()
        .expect("service should remain available")
        .save_agent_profile(AgentProfile {
            name: "long-running-script".to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "sh".to_owned(),
            arguments: vec![
                "-c".to_owned(),
                "IFS= read -r brief; [ \"$brief\" = \"Stop the task.\" ] || exit 7; sleep 5"
                    .to_owned(),
            ],
        })
        .expect("long-running profile should persist");

    runtime
        .start(StartExecutionRequest {
            execution_id: "execution-stop".to_owned(),
            work_item_id: "task-1".to_owned(),
            agent_profile_name: "long-running-script".to_owned(),
            task_brief: "Stop the task.".to_owned(),
            execution_role: Default::default(),
        })
        .expect("runtime should start the worker");
    runtime
        .request_stop("execution-stop")
        .expect("a live direct process should accept a stop request");

    for _ in 0..50 {
        let service = service.lock().expect("service should remain available");
        let execution = service
            .execution(&ExecutionId::from("execution-stop"))
            .expect("execution should persist");
        let work_item = service
            .work_item(&crate::domain::WorkItemId::from("task-1"))
            .expect("work item should persist");
        if execution.status == crate::domain::ExecutionStatus::Interrupted {
            assert_eq!(work_item.work_item.state, WorkItemState::Interrupted);
            return;
        }
        drop(service);
        thread::sleep(Duration::from_millis(20));
    }
    panic!("the stop request should record an interrupted outcome");
}
