use crate::domain::{
    RefineWorkItemDetailsCommand, SchemaMetadata, WorkItemEvent, WorkItemEventKind,
};

use super::{EventStoreError, SqliteEventStore, event_store_support::idempotent_refinement};

impl SqliteEventStore {
    pub fn refine_work_item_details(
        &mut self,
        command: RefineWorkItemDetailsCommand,
    ) -> Result<crate::domain::RecordedWorkItemEvent, EventStoreError> {
        if let Some(recorded_event) = self.event_by_id(&command.event_id)? {
            return idempotent_refinement(recorded_event, &command);
        }
        let materialized = self.required_materialized_work_item(&command.work_item_id)?;
        if materialized.last_event_sequence != command.expected_work_item_sequence {
            return Err(EventStoreError::StaleWorkItem {
                work_item_id: command.work_item_id,
                expected_sequence: command.expected_work_item_sequence,
                actual_sequence: materialized.last_event_sequence,
            });
        }
        let mut updated_work_item = materialized.work_item;
        updated_work_item.title = command.title.clone();
        updated_work_item.description = command.description.clone();
        updated_work_item.acceptance_criteria = command.acceptance_criteria.clone();
        self.persist_event(
            WorkItemEvent {
                schema: SchemaMetadata::current(),
                id: command.event_id,
                work_item_id: command.work_item_id,
                kind: WorkItemEventKind::DetailsRefined {
                    title: command.title,
                    description: command.description,
                    acceptance_criteria: command.acceptance_criteria,
                    reason: command.reason,
                },
                recorded_at: command.recorded_at,
            },
            updated_work_item,
        )
    }
}
