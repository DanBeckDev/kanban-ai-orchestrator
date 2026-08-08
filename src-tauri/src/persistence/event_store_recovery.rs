use crate::domain::{
    RecordedWorkItemEvent, RestartReconciliationCommand, TransitionWorkItemCommand, WorkItemState,
};

use super::{EventStoreError, SqliteEventStore};

const RESTART_UNCERTAINTY_REASON: &str =
    "The daemon restarted before a live execution could be confirmed.";

impl SqliteEventStore {
    pub fn reconcile_after_restart(
        &mut self,
        command: RestartReconciliationCommand,
    ) -> Result<Vec<RecordedWorkItemEvent>, EventStoreError> {
        let uncertain_work_items = self
            .all_materialized_work_items()?
            .into_iter()
            .filter(|materialized_work_item| {
                is_uncertain_after_restart(materialized_work_item.work_item.state)
                    && !command
                        .confirmed_active_work_item_ids
                        .contains(&materialized_work_item.work_item.id)
            })
            .collect::<Vec<_>>();
        for materialized_work_item in &uncertain_work_items {
            if !command
                .recovery_event_ids
                .contains_key(&materialized_work_item.work_item.id)
            {
                return Err(EventStoreError::MissingRecoveryEventId {
                    work_item_id: materialized_work_item.work_item.id.clone(),
                });
            }
        }
        uncertain_work_items
            .into_iter()
            .map(|materialized_work_item| {
                let event_id = command
                    .recovery_event_ids
                    .get(&materialized_work_item.work_item.id)
                    .cloned()
                    .ok_or_else(|| EventStoreError::MissingRecoveryEventId {
                        work_item_id: materialized_work_item.work_item.id.clone(),
                    })?;
                self.transition_work_item(TransitionWorkItemCommand {
                    event_id,
                    work_item_id: materialized_work_item.work_item.id,
                    next_state: WorkItemState::Interrupted,
                    config: Default::default(),
                    evidence: None,
                    reason: RESTART_UNCERTAINTY_REASON.to_owned(),
                    recorded_at: command.recorded_at.clone(),
                })
            })
            .collect()
    }
}

fn is_uncertain_after_restart(state: WorkItemState) -> bool {
    matches!(state, WorkItemState::Running | WorkItemState::AwaitingInput)
}
