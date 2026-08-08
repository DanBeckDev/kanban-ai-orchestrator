use rusqlite::{Transaction, params};

use crate::domain::{
    Execution, ExecutionStatus, RecordedWorkItemEvent, SchemaMetadata, TransitionWorkItemCommand,
    WorkItemEvent, WorkItemEventKind, WorkItemState, transition_work_item,
};

use super::{
    EventStoreError, SqliteEventStore, execution_store::validate_execution_update,
    sqlite_event_store::persist_work_item_event,
};

impl SqliteEventStore {
    pub fn activate_execution_and_start_work_item(
        &mut self,
        execution: Execution,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, EventStoreError> {
        validate_activation_request(&execution, &command)?;
        let recorded_execution =
            self.execution(&execution.id)?
                .ok_or_else(|| EventStoreError::ExecutionNotFound {
                    execution_id: execution.id.clone(),
                })?;
        validate_execution_update(&recorded_execution, &execution)?;
        let materialized_work_item = self.required_materialized_work_item(&command.work_item_id)?;
        let next_state = transition_work_item(
            materialized_work_item.work_item.state,
            command.next_state,
            command.config,
            command.evidence,
        )?;
        let mut updated_work_item = materialized_work_item.work_item;
        let previous_state = updated_work_item.state;
        updated_work_item.state = next_state;
        let event = WorkItemEvent {
            schema: SchemaMetadata::current(),
            id: command.event_id,
            work_item_id: command.work_item_id,
            kind: WorkItemEventKind::StateTransitioned {
                from: previous_state,
                to: next_state,
                config: command.config,
                evidence: command.evidence,
                reason: command.reason,
            },
            recorded_at: command.recorded_at,
        };
        let transaction = self.connection.transaction()?;
        persist_execution(&transaction, &execution)?;
        let recorded_event = persist_work_item_event(&transaction, event, updated_work_item)?;
        transaction.commit()?;
        Ok(recorded_event)
    }
}

fn validate_activation_request(
    execution: &Execution,
    command: &TransitionWorkItemCommand,
) -> Result<(), EventStoreError> {
    if execution.status != ExecutionStatus::Running
        || execution.session_id.is_none()
        || command.work_item_id != execution.work_item_id
        || command.next_state != WorkItemState::Running
    {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id: execution.id.clone(),
            reason: "activation must attach a session and start its matching work item",
        });
    }
    if command.reason.trim().is_empty() {
        return Err(EventStoreError::MissingTransitionReason {
            event_id: command.event_id.clone(),
        });
    }
    Ok(())
}

fn persist_execution(
    transaction: &Transaction<'_>,
    execution: &Execution,
) -> Result<(), EventStoreError> {
    transaction.execute(
        "UPDATE executions SET execution_json = ?1 WHERE execution_id = ?2",
        params![serde_json::to_string(execution)?, execution.id.0],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{ExecutionId, ExecutionUsage, TransitionConfig, WorkItemEventId, WorkItemId},
        persistence::{EventStoreError, SqliteEventStore},
    };

    use super::{Execution, ExecutionStatus, TransitionWorkItemCommand, WorkItemState};
    use crate::persistence::sqlite_event_store_tests::{create_command, transition_command};

    fn prepared_store() -> SqliteEventStore {
        let mut store = SqliteEventStore::in_memory().expect("event store should open");
        store
            .create_work_item(create_command("task-1", WorkItemState::Inbox))
            .expect("task should be created");
        store
            .transition_work_item(transition_command(
                "plan-task-1",
                "task-1",
                WorkItemState::Planned,
            ))
            .expect("task should be planned");
        store
            .transition_work_item(transition_command(
                "ready-task-1",
                "task-1",
                WorkItemState::Ready,
            ))
            .expect("task should be ready");
        store
            .record_execution(pending_execution())
            .expect("execution should be recorded");
        store
    }

    fn pending_execution() -> Execution {
        Execution {
            schema: crate::domain::SchemaMetadata::current(),
            id: ExecutionId::from("execution-1"),
            work_item_id: WorkItemId::from("task-1"),
            role: Default::default(),
            adapter_name: "fake".to_owned(),
            status: ExecutionStatus::Pending,
            session_id: None,
            workspace_path: "/workspaces/task-1".to_owned(),
            usage: ExecutionUsage {
                input_tokens: 0,
                output_tokens: 0,
                cost_micros: None,
            },
            last_event_sequence: 0,
        }
    }

    fn start_command() -> TransitionWorkItemCommand {
        TransitionWorkItemCommand {
            event_id: WorkItemEventId::from("start-execution-1"),
            work_item_id: WorkItemId::from("task-1"),
            next_state: WorkItemState::Running,
            config: TransitionConfig::default(),
            evidence: None,
            reason: "Agent session session-1 started.".to_owned(),
            recorded_at: "2026-08-08T00:02:00Z".to_owned(),
        }
    }

    fn active_execution() -> Execution {
        let mut execution = pending_execution();
        execution.status = ExecutionStatus::Running;
        execution.session_id = Some("session-1".to_owned());
        execution
    }

    #[test]
    fn atomically_attaches_a_session_and_starts_its_ready_work_item() {
        let mut store = prepared_store();

        let event = store
            .activate_execution_and_start_work_item(active_execution(), start_command())
            .expect("activation should commit both records");

        assert_eq!(event.event.work_item_id, WorkItemId::from("task-1"));
        assert_eq!(
            store
                .execution(&ExecutionId::from("execution-1"))
                .expect("execution should load")
                .expect("execution should exist")
                .status,
            ExecutionStatus::Running
        );
        assert_eq!(
            store
                .materialized_work_item(&WorkItemId::from("task-1"))
                .expect("task should load")
                .expect("task should exist")
                .work_item
                .state,
            WorkItemState::Running
        );
    }

    #[test]
    fn rolls_back_the_execution_update_when_the_start_event_cannot_persist() {
        let mut store = prepared_store();
        store
            .create_work_item(crate::domain::CreateWorkItemCommand {
                event_id: WorkItemEventId::from("start-execution-1"),
                work_item: crate::persistence::sqlite_event_store_tests::work_item(
                    "task-2",
                    WorkItemState::Inbox,
                ),
                recorded_at: "2026-08-08T00:03:00Z".to_owned(),
            })
            .expect("conflicting event id should exist on another task");

        assert!(matches!(
            store.activate_execution_and_start_work_item(active_execution(), start_command()),
            Err(EventStoreError::Database(_))
        ));
        assert_eq!(
            store
                .execution(&ExecutionId::from("execution-1"))
                .expect("execution should load")
                .expect("execution should exist")
                .status,
            ExecutionStatus::Pending
        );
        assert_eq!(
            store
                .materialized_work_item(&WorkItemId::from("task-1"))
                .expect("task should load")
                .expect("task should exist")
                .work_item
                .state,
            WorkItemState::Ready
        );
    }
}
