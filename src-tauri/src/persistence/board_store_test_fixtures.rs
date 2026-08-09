use std::path::Path;

use super::SqliteEventStore;
use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, DependencyId, DependencyKind,
    DependencySource, Evidence, EvidenceId, EvidenceKind, EvidenceResult, Execution, ExecutionId,
    ExecutionStatus, ExecutionUsage, Project, ProjectId, SchemaMetadata, WorkItem, WorkItemBudget,
    WorkItemEventId, WorkItemId, WorkItemState,
};

pub(super) fn project(id: &str) -> Project {
    Project {
        schema: SchemaMetadata::current(),
        id: ProjectId::from(id),
        name: "Desktop application".to_owned(),
        repository_path: "/projects/desktop-application".to_owned(),
        base_ref: "main".to_owned(),
        policy_set_id: "standard".to_owned(),
    }
}

pub(super) fn board(id: &str, project_id: &str) -> Board {
    Board {
        schema: SchemaMetadata::current(),
        id: BoardId::from(id),
        project_id: ProjectId::from(project_id),
        name: "MVP".to_owned(),
    }
}

pub(super) fn work_item(id: &str, board_id: &str) -> WorkItem {
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
        assigned_agent_profile_name: None,
        assigned_agent_model: Default::default(),
        assigned_agent_effort: Default::default(),
    }
}

pub(super) fn create_work_item_command(id: &str, board_id: &str) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        event_id: WorkItemEventId::from(format!("create-{id}").as_str()),
        work_item: work_item(id, board_id),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

pub(super) fn execution(work_item_id: &str) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: ExecutionId::from("execution-1"),
        work_item_id: WorkItemId::from(work_item_id),
        role: Default::default(),
        adapter_name: "codex-cli".to_owned(),
        status: ExecutionStatus::AwaitingReview,
        session_id: Some("session-1".to_owned()),
        workspace_path: "/workspaces/task-1".to_owned(),
        usage: ExecutionUsage {
            input_tokens: 10,
            output_tokens: 5,
            cost_micros: Some(100),
        },
        last_event_sequence: 4,
    }
}

pub(super) fn evidence(work_item_id: &str) -> Evidence {
    Evidence {
        schema: SchemaMetadata::current(),
        id: EvidenceId::from("evidence-1"),
        work_item_id: WorkItemId::from(work_item_id),
        execution_id: None,
        kind: EvidenceKind::CompletionReport,
        result: EvidenceResult::Recorded,
        summary: "Agent requested review.".to_owned(),
        recorded_at: "2026-08-08T00:02:00Z".to_owned(),
    }
}

pub(super) fn dependency(id: &str, upstream: &str, downstream: &str) -> Dependency {
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

pub(super) fn opened_store(path: &Path) -> SqliteEventStore {
    SqliteEventStore::open(path).expect("event store should open")
}
