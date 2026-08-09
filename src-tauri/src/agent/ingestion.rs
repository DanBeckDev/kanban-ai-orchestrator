use crate::domain::{TransitionConfig, WorkItemState, transition_work_item};

use super::{AgentAdapterError, NormalizedAgentEvent, NormalizedAgentEventKind};

pub struct AgentEventIngestor {
    last_sequence: u64,
}

impl AgentEventIngestor {
    pub fn new(last_sequence: u64) -> Self {
        Self { last_sequence }
    }

    pub fn apply_to_work_item(
        &mut self,
        current_state: WorkItemState,
        event: &NormalizedAgentEvent,
        config: TransitionConfig,
    ) -> Result<WorkItemState, AgentAdapterError> {
        self.validate_sequence(event.sequence)?;
        let next_state = match proposed_work_item_state(&event.kind) {
            Some(next_state) if next_state != current_state => {
                transition_work_item(current_state, next_state, config, None)
                    .map_err(AgentAdapterError::InvalidWorkItemTransition)?
            }
            None => current_state,
            Some(_) => current_state,
        };
        self.last_sequence = event.sequence;
        Ok(next_state)
    }

    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }

    pub fn record_without_work_item_transition(
        &mut self,
        event: &NormalizedAgentEvent,
    ) -> Result<(), AgentAdapterError> {
        self.validate_sequence(event.sequence)?;
        self.last_sequence = event.sequence;
        Ok(())
    }

    fn validate_sequence(&self, sequence: u64) -> Result<(), AgentAdapterError> {
        if sequence <= self.last_sequence {
            return Err(AgentAdapterError::DuplicateEvent { sequence });
        }
        let expected = self.last_sequence.saturating_add(1);
        if sequence != expected {
            return Err(AgentAdapterError::EventOutOfOrder {
                expected,
                received: sequence,
            });
        }

        Ok(())
    }
}

fn proposed_work_item_state(event: &NormalizedAgentEventKind) -> Option<WorkItemState> {
    match event {
        NormalizedAgentEventKind::ApprovalRequested { .. }
        | NormalizedAgentEventKind::AwaitingInput { .. } => Some(WorkItemState::AwaitingInput),
        NormalizedAgentEventKind::AwaitingReview { .. }
        | NormalizedAgentEventKind::Completed { .. } => Some(WorkItemState::Review),
        NormalizedAgentEventKind::Failed { .. } => Some(WorkItemState::Failed),
        NormalizedAgentEventKind::Interrupted { .. } => Some(WorkItemState::Interrupted),
        NormalizedAgentEventKind::Activity { .. }
        | NormalizedAgentEventKind::UsageUpdated { .. } => None,
    }
}
