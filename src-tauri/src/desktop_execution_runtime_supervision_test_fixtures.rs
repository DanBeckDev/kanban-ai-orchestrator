use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tempfile::TempDir;

use crate::{
    agent::{AgentProfile, AgentProfileKind},
    application::{
        BoardService, ConfigureBoardSupervisionRequest, CreateBoardRequest, CreateProjectRequest,
        CreateWorkItemRequest, SaveProjectAgentSettingsRequest, TransitionWorkItemRequest,
    },
    domain::{
        AgentEffort, AgentModelPreference, Board, BoardId, BoardSupervision,
        BoardSupervisionLimits, BoardSupervisionMode, Dependency, DependencyId, DependencyKind,
        DependencySource, Execution, ExecutionId, ExecutionStatus, ExecutionUsage,
        MaterializedWorkItem, OrganiserDefaults, ProjectId, SchemaMetadata, SupervisionAction,
        SupervisionDecision, SupervisionDecisionId, SupervisionDecisionOutcome,
        SupervisionPolicyResult, TicketWorkerDefaults, WorkItem, WorkItemBudget, WorkItemId,
        WorkItemState,
    },
    orchestration::PlannerProfile,
    persistence::SqliteEventStore,
};

use super::ExecutionRuntime;

pub(super) fn configured_runtime(
    mode: BoardSupervisionMode,
) -> (
    Arc<Mutex<crate::desktop::LocalBoardService>>,
    ExecutionRuntime,
    TempDir,
) {
    configured_runtime_with_script(
        mode,
        "cat >/dev/null; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"foundation\",\"recommendation\":\"Prepare foundation.\",\"rationale\":\"It is confirmed work.\"}'",
    )
}

pub(super) fn configured_runtime_with_script(
    mode: BoardSupervisionMode,
    organiser_script: &str,
) -> (
    Arc<Mutex<crate::desktop::LocalBoardService>>,
    ExecutionRuntime,
    TempDir,
) {
    let repository = TempDir::new().expect("temporary repository should exist");
    let mut service = BoardService::new(SqliteEventStore::in_memory().expect("store should open"));
    service
        .create_project(CreateProjectRequest {
            project_id: "project-1".to_owned(),
            name: "Supervision project".to_owned(),
            repository_path: repository.path().to_string_lossy().into_owned(),
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
    service
        .create_work_item(CreateWorkItemRequest {
            event_id: "create-foundation".to_owned(),
            work_item_id: "foundation".to_owned(),
            board_id: "board-1".to_owned(),
            title: "Build foundation".to_owned(),
            description: "A bounded task.".to_owned(),
            acceptance_criteria: vec!["Tests pass.".to_owned()],
            budget: WorkItemBudget::default(),
            requires_human_review: false,
            recorded_at: "2026-08-10T00:00:00Z".to_owned(),
        })
        .expect("work item should persist");
    service
        .save_agent_profile(AgentProfile {
            name: "worker".to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "worker".to_owned(),
            arguments: Vec::new(),
        })
        .expect("worker profile should persist");
    service
        .save_planner_profile(PlannerProfile {
            name: "organiser".to_owned(),
            kind: Default::default(),
            program: "sh".to_owned(),
            arguments: vec!["-c".to_owned(), organiser_script.to_owned()],
        })
        .expect("organiser profile should persist");
    service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: Some(OrganiserDefaults {
                planner_profile_name: "organiser".to_owned(),
                model: AgentModelPreference::ProviderDefault,
                effort: AgentEffort::ProviderDefault,
            }),
            ticket_worker: Some(TicketWorkerDefaults {
                agent_profile_name: "worker".to_owned(),
                model: AgentModelPreference::ProviderDefault,
                effort: AgentEffort::ProviderDefault,
            }),
        })
        .expect("role settings should persist");
    service
        .configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: "board-1".to_owned(),
            mode,
            configured_by: "local-user".to_owned(),
            configured_at: "2026-08-10T00:00:00Z".to_owned(),
        })
        .expect("supervision should persist");
    let service = Arc::new(Mutex::new(service));
    let runtime = ExecutionRuntime::new(service.clone(), PathBuf::from("/workspaces"));
    (service, runtime, repository)
}

pub(super) fn supervision(mode: BoardSupervisionMode) -> BoardSupervision {
    BoardSupervision {
        schema: SchemaMetadata::current(),
        board_id: BoardId::from("board-1"),
        mode,
        organiser: OrganiserDefaults {
            planner_profile_name: "organiser".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        },
        ticket_worker: TicketWorkerDefaults {
            agent_profile_name: "worker".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        },
        limits: BoardSupervisionLimits::default(),
        permitted_actions: std::collections::BTreeSet::from([
            SupervisionAction::PrepareWork,
            SupervisionAction::MakeWorkReady,
            SupervisionAction::StartWork,
            SupervisionAction::RetryWork,
            SupervisionAction::ReturnForCorrection,
        ]),
        configured_by: "local-user".to_owned(),
        configured_at: "2026-08-10T00:00:00Z".to_owned(),
        paused_by: None,
        paused_at: None,
        revision: 1,
    }
}

pub(super) fn board_snapshot(
    foundation_state: WorkItemState,
    interface_state: WorkItemState,
) -> crate::application::BoardSnapshot {
    crate::application::BoardSnapshot {
        board: Board {
            schema: SchemaMetadata::current(),
            id: BoardId::from("board-1"),
            project_id: ProjectId::from("project-1"),
            name: "MVP".to_owned(),
        },
        work_items: vec![
            work_item("foundation", foundation_state),
            work_item("interface", interface_state),
        ],
        dependencies: vec![Dependency {
            schema: SchemaMetadata::current(),
            id: DependencyId::from("foundation-blocks-interface"),
            upstream_work_item_id: WorkItemId::from("foundation"),
            downstream_work_item_id: WorkItemId::from("interface"),
            kind: DependencyKind::Blocks,
            source: DependencySource::Orchestrator,
            reason: "The interface needs the foundation.".to_owned(),
            owner: "Kanban".to_owned(),
            next_action: "Finish the foundation.".to_owned(),
            created_by: "organiser".to_owned(),
            created_at: "2026-08-10T00:00:00Z".to_owned(),
        }],
        activity: Vec::new(),
        executions: Vec::new(),
        evidence: Vec::new(),
        external_links: Vec::new(),
        connector_outbox_items: Vec::new(),
        connector_reconciliation_items: Vec::new(),
    }
}

pub(super) fn failed_execution(work_item_id: &str) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: ExecutionId::from("attempt-1"),
        work_item_id: WorkItemId::from(work_item_id),
        role: crate::domain::ExecutionRole::Implementation,
        adapter_name: "worker".to_owned(),
        status: ExecutionStatus::Failed,
        session_id: None,
        workspace_path: "/workspaces/attempt-1".to_owned(),
        usage: ExecutionUsage {
            input_tokens: 0,
            output_tokens: 0,
            cost_micros: None,
        },
        last_event_sequence: 1,
    }
}

pub(super) fn transition_to_ready(
    service: &mut crate::desktop::LocalBoardService,
    work_item_id: &str,
) {
    for (event_id, next_state) in [
        ("plan-foundation", WorkItemState::Planned),
        ("ready-foundation", WorkItemState::Ready),
    ] {
        service
            .transition_work_item(transition_request(event_id, work_item_id, next_state))
            .expect("state transition should persist");
    }
}

pub(super) fn transition_to_blocked(
    service: &mut crate::desktop::LocalBoardService,
    work_item_id: &str,
) {
    for (event_id, next_state) in [
        ("plan-before-stale-decision", WorkItemState::Planned),
        ("block-before-stale-decision", WorkItemState::Blocked),
    ] {
        service
            .transition_work_item(transition_request(event_id, work_item_id, next_state))
            .expect("state transition should persist");
    }
}

pub(super) fn pending_decision() -> SupervisionDecision {
    SupervisionDecision {
        schema: SchemaMetadata::current(),
        id: SupervisionDecisionId::from("pending-decision"),
        board_id: BoardId::from("board-1"),
        work_item_id: Some(WorkItemId::from("foundation")),
        organiser_profile_name: "organiser".to_owned(),
        action: SupervisionAction::PrepareWork,
        recommendation: "Prepare foundation.".to_owned(),
        rationale: "It is confirmed work.".to_owned(),
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: SupervisionDecisionOutcome::Pending,
        idempotency_key: "pending-recovery".to_owned(),
        expected_work_item_sequence: Some(1),
        recorded_at: "2026-08-10T10:00:00Z".to_owned(),
        resolved_at: None,
    }
}

fn work_item(id: &str, state: WorkItemState) -> MaterializedWorkItem {
    MaterializedWorkItem {
        work_item: WorkItem {
            schema: SchemaMetadata::current(),
            id: WorkItemId::from(id),
            board_id: BoardId::from("board-1"),
            title: format!("Build {id}"),
            description: "A bounded task.".to_owned(),
            acceptance_criteria: vec!["Tests pass.".to_owned()],
            budget: WorkItemBudget::default(),
            state,
            requires_human_review: false,
            assigned_agent_profile_name: None,
            assigned_agent_model: AgentModelPreference::ProviderDefault,
            assigned_agent_effort: AgentEffort::ProviderDefault,
        },
        last_event_sequence: 1,
    }
}

fn transition_request(
    event_id: &str,
    work_item_id: &str,
    next_state: WorkItemState,
) -> TransitionWorkItemRequest {
    TransitionWorkItemRequest {
        event_id: event_id.to_owned(),
        work_item_id: work_item_id.to_owned(),
        next_state,
        evidence: None,
        reason: "Prepare the task for a safe capacity check.".to_owned(),
        recorded_at: "2026-08-10T10:00:00Z".to_owned(),
    }
}
