use rusqlite::{OptionalExtension, params};

use crate::domain::{MaterializedWorkItem, RecordedWorkItemEvent, WorkItemId};

use super::{
    EventStoreError, SqliteEventStore,
    event_store_support::{deserialize_materialized_work_item, deserialize_recorded_event},
};

impl SqliteEventStore {
    pub fn materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<MaterializedWorkItem>, EventStoreError> {
        let stored_work_item = self
            .connection
            .query_row(
                "SELECT work_item_json, last_event_sequence
                 FROM materialized_work_items
                 WHERE work_item_id = ?1",
                [work_item_id.0.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;

        stored_work_item
            .map(|(work_item_json, last_event_sequence)| {
                deserialize_materialized_work_item(&work_item_json, last_event_sequence)
            })
            .transpose()
    }

    pub fn work_item_events(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<RecordedWorkItemEvent>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT sequence, event_json
             FROM work_item_events
             WHERE work_item_id = ?1
             ORDER BY sequence",
        )?;
        let rows = statement.query_map([work_item_id.0.as_str()], event_row)?;

        rows.map(deserialize_row).collect()
    }

    pub fn recent_work_item_events(
        &self,
        work_item_id: &WorkItemId,
        limit: u32,
    ) -> Result<Vec<RecordedWorkItemEvent>, EventStoreError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut statement = self.connection.prepare(
            "SELECT sequence, event_json
             FROM work_item_events
             WHERE work_item_id = ?1
             ORDER BY sequence DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(
            params![work_item_id.0.as_str(), i64::from(limit)],
            event_row,
        )?;
        let events = rows.map(deserialize_row).collect::<Result<Vec<_>, _>>()?;

        Ok(events.into_iter().rev().collect())
    }

    pub(crate) fn all_materialized_work_items(
        &self,
    ) -> Result<Vec<MaterializedWorkItem>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT work_item_json, last_event_sequence
             FROM materialized_work_items
             ORDER BY work_item_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        rows.map(|row| {
            let (work_item_json, last_event_sequence) = row?;
            deserialize_materialized_work_item(&work_item_json, last_event_sequence)
        })
        .collect()
    }
}

fn event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(i64, String)> {
    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
}

fn deserialize_row(
    row: rusqlite::Result<(i64, String)>,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    let (sequence, event_json) = row?;
    deserialize_recorded_event(&event_json, sequence)
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            BoardId, CreateWorkItemCommand, SchemaMetadata, TransitionConfig,
            TransitionWorkItemCommand, WorkItem, WorkItemBudget, WorkItemEventId, WorkItemId,
            WorkItemState,
        },
        persistence::SqliteEventStore,
    };

    fn create_work_item(store: &mut SqliteEventStore) {
        store
            .create_work_item(CreateWorkItemCommand {
                event_id: WorkItemEventId::from("create-task-1"),
                work_item: WorkItem {
                    schema: SchemaMetadata::current(),
                    id: WorkItemId::from("task-1"),
                    board_id: BoardId::from("board-1"),
                    title: "Persisted task".to_owned(),
                    description: "A task used to test event-history reads.".to_owned(),
                    acceptance_criteria: vec!["History remains available.".to_owned()],
                    budget: WorkItemBudget::default(),
                    state: WorkItemState::Inbox,
                    requires_human_review: false,
                    assigned_agent_profile_name: None,
                    assigned_agent_model: Default::default(),
                    assigned_agent_effort: Default::default(),
                },
                recorded_at: "2026-08-08T00:00:00Z".to_owned(),
            })
            .expect("work item creation should persist");
    }

    fn transition(store: &mut SqliteEventStore, event_id: &str, next_state: WorkItemState) {
        store
            .transition_work_item(TransitionWorkItemCommand {
                event_id: WorkItemEventId::from(event_id),
                work_item_id: WorkItemId::from("task-1"),
                next_state,
                config: TransitionConfig::default(),
                evidence: None,
                reason: "The event should remain durable.".to_owned(),
                recorded_at: "2026-08-08T00:00:01Z".to_owned(),
            })
            .expect("transition should persist");
    }

    #[test]
    fn returns_recent_events_in_chronological_order_without_truncating_history() {
        let mut store = SqliteEventStore::in_memory().expect("event store should open");
        create_work_item(&mut store);
        for (event_id, next_state) in [
            ("plan-task-1", WorkItemState::Planned),
            ("ready-task-1", WorkItemState::Ready),
            ("run-task-1", WorkItemState::Running),
        ] {
            transition(&mut store, event_id, next_state);
        }

        let task_id = WorkItemId::from("task-1");
        let recent_sequences = store
            .recent_work_item_events(&task_id, 2)
            .expect("recent events should load")
            .into_iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>();
        let all_sequences = store
            .work_item_events(&task_id)
            .expect("all events should remain available")
            .into_iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>();

        assert_eq!(recent_sequences, vec![3, 4]);
        assert_eq!(all_sequences, vec![1, 2, 3, 4]);
        assert!(
            store
                .recent_work_item_events(&task_id, 0)
                .expect("zero-limit query should succeed")
                .is_empty()
        );
    }
}
