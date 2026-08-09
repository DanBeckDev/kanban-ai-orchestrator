use uuid::Uuid;

use crate::{
    application::{BoardSnapshot, StartExecutionRequest, TransitionWorkItemRequest},
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock, timestamp},
    domain::{BoardId, ExecutionRole, WorkItemState},
};

use super::ExecutionRuntime;

struct CoordinationPreparation {
    snapshot: BoardSnapshot,
    start_request: Option<StartExecutionRequest>,
}

impl ExecutionRuntime {
    pub(crate) fn coordinate_board(
        &self,
        board_id: &str,
        agent_profile_name: &str,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        require_value(board_id, "board id")?;
        require_value(agent_profile_name, "agent profile name")?;
        self.ensure_worker_profile_exists(agent_profile_name)?;
        let preparation = self.prepare_coordination(board_id, agent_profile_name)?;
        match preparation.start_request {
            Some(request) => self.start(request),
            None => Ok(preparation.snapshot),
        }
    }

    fn ensure_worker_profile_exists(
        &self,
        agent_profile_name: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .agent_profile(agent_profile_name)
            .map(|_| ())
            .map_err(ExecutionRuntimeError::Profile)
    }

    fn prepare_coordination(
        &self,
        board_id: &str,
        agent_profile_name: &str,
    ) -> Result<CoordinationPreparation, ExecutionRuntimeError> {
        let board_id = BoardId::from(board_id);
        let mut service = lock(&self.service, "board service")?;
        let initial_snapshot = service
            .snapshot(&board_id)
            .map_err(ExecutionRuntimeError::Board)?;

        for work_item_id in initial_snapshot
            .work_items
            .iter()
            .filter(|item| item.work_item.state == WorkItemState::Inbox)
            .map(|item| item.work_item.id.0.clone())
        {
            transition_for_coordination(
                &mut service,
                &work_item_id,
                WorkItemState::Planned,
                "Kanban prepared this task for dependency scheduling.",
            )?;
        }

        let planned_snapshot = service
            .snapshot(&board_id)
            .map_err(ExecutionRuntimeError::Board)?;
        for work_item_id in eligible_planned_work_items(&planned_snapshot) {
            transition_for_coordination(
                &mut service,
                &work_item_id,
                WorkItemState::Ready,
                "Kanban found that all required upstream work is complete.",
            )?;
        }

        let snapshot = service
            .snapshot(&board_id)
            .map_err(ExecutionRuntimeError::Board)?;
        let start_request = snapshot
            .work_items
            .iter()
            .find(|item| {
                item.work_item.state == WorkItemState::Ready
                    && dependencies_are_complete(&snapshot, &item.work_item.id.0)
            })
            .map(|item| StartExecutionRequest {
                execution_id: format!("orchestrator-{}-{}", item.work_item.id.0, Uuid::new_v4()),
                work_item_id: item.work_item.id.0.clone(),
                agent_profile_name: agent_profile_name.to_owned(),
                task_brief: implementation_brief(&item.work_item),
                execution_role: ExecutionRole::Implementation,
            });

        Ok(CoordinationPreparation {
            snapshot,
            start_request,
        })
    }
}

fn transition_for_coordination(
    service: &mut crate::desktop::LocalBoardService,
    work_item_id: &str,
    next_state: WorkItemState,
    reason: &str,
) -> Result<(), ExecutionRuntimeError> {
    service
        .transition_work_item(TransitionWorkItemRequest {
            event_id: format!("orchestrator-{}-{}", next_state, Uuid::new_v4()),
            work_item_id: work_item_id.to_owned(),
            next_state,
            evidence: None,
            reason: reason.to_owned(),
            recorded_at: timestamp(),
        })
        .map(|_| ())
        .map_err(ExecutionRuntimeError::Board)
}

fn eligible_planned_work_items(snapshot: &BoardSnapshot) -> Vec<String> {
    snapshot
        .work_items
        .iter()
        .filter(|item| {
            item.work_item.state == WorkItemState::Planned
                && dependencies_are_complete(snapshot, &item.work_item.id.0)
        })
        .map(|item| item.work_item.id.0.clone())
        .collect()
}

fn dependencies_are_complete(snapshot: &BoardSnapshot, work_item_id: &str) -> bool {
    snapshot
        .dependencies
        .iter()
        .filter(|dependency| {
            dependency.downstream_work_item_id.0 == work_item_id && dependency.kind.is_hard()
        })
        .all(|dependency| {
            snapshot.work_items.iter().any(|item| {
                item.work_item.id == dependency.upstream_work_item_id
                    && item.work_item.state == WorkItemState::Done
            })
        })
}

fn implementation_brief(work_item: &crate::domain::WorkItem) -> String {
    let criteria = work_item
        .acceptance_criteria
        .iter()
        .map(|criterion| format!("- {criterion}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Implement {}.\n\n{}\n\nAcceptance criteria:\n{}",
        work_item.title, work_item.description, criteria
    )
}

fn require_value(value: &str, field: &'static str) -> Result<(), ExecutionRuntimeError> {
    if value.trim().is_empty() {
        Err(ExecutionRuntimeError::MissingRequiredField { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use crate::{
        application::{
            AddDependencyRequest, AgentProfileServiceError, BoardService, BoardSnapshot,
            CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
        },
        desktop::LocalBoardService,
        desktop_execution_runtime::ExecutionRuntime,
        domain::{
            Board, Dependency, DependencyId, DependencyKind, DependencySource,
            MaterializedWorkItem, ProjectId, SchemaMetadata, WorkItem, WorkItemBudget, WorkItemId,
        },
        persistence::SqliteEventStore,
    };

    use super::{
        ExecutionRuntimeError, dependencies_are_complete, eligible_planned_work_items,
        implementation_brief,
    };
    use crate::domain::{BoardId, WorkItemState};

    #[test]
    fn makes_only_dependency_ready_planned_work_available() {
        let initial = board_snapshot(WorkItemState::Planned, WorkItemState::Planned);

        assert_eq!(eligible_planned_work_items(&initial), vec!["foundation"]);
        assert!(!dependencies_are_complete(&initial, "interface"));

        let completed_root = board_snapshot(WorkItemState::Done, WorkItemState::Planned);

        assert_eq!(
            eligible_planned_work_items(&completed_root),
            vec!["interface"]
        );
        assert!(dependencies_are_complete(&completed_root, "interface"));
    }

    #[test]
    fn gives_the_worker_a_scoped_brief_with_its_acceptance_criteria() {
        let work_item = work_item("foundation", WorkItemState::Ready);

        assert_eq!(
            implementation_brief(&work_item.work_item),
            "Implement Build foundation.\n\nA bounded task.\n\nAcceptance criteria:\n- Tests pass."
        );
    }

    #[test]
    fn prepares_inbox_work_and_selects_only_the_first_ready_worker() {
        let (_service, runtime) = runtime_with_dependent_inbox_work();

        let preparation = runtime
            .prepare_coordination("board-1", "worker")
            .expect("coordination should prepare eligible work");

        assert_eq!(
            work_item_state(&preparation.snapshot, "foundation"),
            WorkItemState::Ready
        );
        assert_eq!(
            work_item_state(&preparation.snapshot, "interface"),
            WorkItemState::Planned
        );
        let request = preparation
            .start_request
            .expect("the ready root task should have a start request");
        assert_eq!(request.work_item_id, "foundation");
        assert_eq!(request.agent_profile_name, "worker");
    }

    #[test]
    fn rejects_empty_coordination_inputs_before_loading_a_board() {
        let (_service, runtime) = runtime_with_dependent_inbox_work();

        assert!(matches!(
            runtime.coordinate_board(" ", "worker"),
            Err(ExecutionRuntimeError::MissingRequiredField { field: "board id" })
        ));
        assert!(matches!(
            runtime.coordinate_board("board-1", " "),
            Err(ExecutionRuntimeError::MissingRequiredField {
                field: "agent profile name"
            })
        ));
    }

    #[test]
    fn rejects_an_unknown_worker_before_changing_the_board() {
        let (service, runtime) = runtime_with_dependent_inbox_work();

        assert!(matches!(
            runtime.coordinate_board("board-1", "missing-worker"),
            Err(ExecutionRuntimeError::Profile(AgentProfileServiceError::NotFound { name }))
                if name == "missing-worker"
        ));
        let snapshot = service
            .lock()
            .expect("service should remain available")
            .snapshot(&BoardId::from("board-1"))
            .expect("board should remain available");
        assert_eq!(
            work_item_state(&snapshot, "foundation"),
            WorkItemState::Inbox
        );
        assert_eq!(
            work_item_state(&snapshot, "interface"),
            WorkItemState::Inbox
        );
    }

    fn board_snapshot(
        foundation_state: WorkItemState,
        interface_state: WorkItemState,
    ) -> BoardSnapshot {
        BoardSnapshot {
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
                created_by: "orchestrator".to_owned(),
                created_at: "2026-08-09T00:00:00Z".to_owned(),
            }],
            activity: vec![],
            executions: vec![],
            evidence: vec![],
            external_links: vec![],
            connector_outbox_items: vec![],
            connector_reconciliation_items: vec![],
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
            },
            last_event_sequence: 1,
        }
    }

    fn runtime_with_dependent_inbox_work() -> (Arc<Mutex<LocalBoardService>>, ExecutionRuntime) {
        let mut service =
            BoardService::new(SqliteEventStore::in_memory().expect("event store should open"));
        service
            .create_project(CreateProjectRequest {
                project_id: "project-1".to_owned(),
                name: "Coordination project".to_owned(),
                repository_path: "/projects/coordination".to_owned(),
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
        for work_item_id in ["foundation", "interface"] {
            service
                .create_work_item(CreateWorkItemRequest {
                    event_id: format!("create-{work_item_id}"),
                    work_item_id: work_item_id.to_owned(),
                    board_id: "board-1".to_owned(),
                    title: format!("Build {work_item_id}"),
                    description: "A bounded task.".to_owned(),
                    acceptance_criteria: vec!["Tests pass.".to_owned()],
                    budget: WorkItemBudget::default(),
                    requires_human_review: false,
                    recorded_at: "2026-08-09T00:00:00Z".to_owned(),
                })
                .expect("work item should persist");
        }
        service
            .add_dependency(AddDependencyRequest {
                dependency_id: "foundation-blocks-interface".to_owned(),
                upstream_work_item_id: "foundation".to_owned(),
                downstream_work_item_id: "interface".to_owned(),
                kind: DependencyKind::Blocks,
                reason: "The interface needs the foundation.".to_owned(),
                owner: "Kanban".to_owned(),
                next_action: "Finish the foundation.".to_owned(),
                created_by: "orchestrator".to_owned(),
                created_at: "2026-08-09T00:00:00Z".to_owned(),
            })
            .expect("dependency should persist");
        let service = Arc::new(Mutex::new(service));
        let runtime = ExecutionRuntime::new(service.clone(), PathBuf::from("/workspaces"));
        (service, runtime)
    }

    fn work_item_state(snapshot: &BoardSnapshot, work_item_id: &str) -> WorkItemState {
        snapshot
            .work_items
            .iter()
            .find(|item| item.work_item.id.0 == work_item_id)
            .expect("work item should exist")
            .work_item
            .state
    }
}
