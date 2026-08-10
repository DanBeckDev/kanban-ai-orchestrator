use crate::domain::{
    CreateWorkItemCommand, EventSequence, MaterializedWorkItem, RecordedWorkItemEvent,
    RefineWorkItemDetailsCommand, TransitionWorkItemCommand, WorkItemEventKind,
};

use super::EventStoreError;

pub(super) fn idempotent_creation(
    recorded_event: RecordedWorkItemEvent,
    command: &CreateWorkItemCommand,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    match &recorded_event.event.kind {
        WorkItemEventKind::Created { work_item }
            if recorded_event.event.work_item_id == command.work_item.id
                && work_item == &command.work_item =>
        {
            Ok(recorded_event)
        }
        _ => Err(EventStoreError::EventIdConflict {
            event_id: command.event_id.clone(),
        }),
    }
}

pub(super) fn idempotent_transition(
    recorded_event: RecordedWorkItemEvent,
    command: &TransitionWorkItemCommand,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    match &recorded_event.event.kind {
        WorkItemEventKind::StateTransitioned {
            to,
            config,
            evidence,
            reason,
            ..
        } if recorded_event.event.work_item_id == command.work_item_id
            && *to == command.next_state
            && *config == command.config
            && *evidence == command.evidence
            && reason == &command.reason =>
        {
            Ok(recorded_event)
        }
        _ => Err(EventStoreError::EventIdConflict {
            event_id: command.event_id.clone(),
        }),
    }
}

pub(super) fn idempotent_refinement(
    recorded_event: RecordedWorkItemEvent,
    command: &RefineWorkItemDetailsCommand,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    match &recorded_event.event.kind {
        WorkItemEventKind::DetailsRefined {
            title,
            description,
            acceptance_criteria,
            reason,
        } if recorded_event.event.work_item_id == command.work_item_id
            && title == &command.title
            && description == &command.description
            && acceptance_criteria == &command.acceptance_criteria
            && reason == &command.reason =>
        {
            Ok(recorded_event)
        }
        _ => Err(EventStoreError::EventIdConflict {
            event_id: command.event_id.clone(),
        }),
    }
}

pub(super) fn deserialize_materialized_work_item(
    work_item_json: &str,
    last_event_sequence: i64,
) -> Result<MaterializedWorkItem, EventStoreError> {
    Ok(MaterializedWorkItem {
        work_item: serde_json::from_str(work_item_json)?,
        last_event_sequence: event_sequence(last_event_sequence)?,
    })
}

pub(super) fn deserialize_recorded_event(
    event_json: &str,
    sequence: i64,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    Ok(RecordedWorkItemEvent {
        sequence: event_sequence(sequence)?,
        event: serde_json::from_str(event_json)?,
    })
}

pub(super) fn event_sequence(value: i64) -> Result<EventSequence, EventStoreError> {
    u64::try_from(value).map_err(|_| EventStoreError::InvalidEventSequence { value })
}
