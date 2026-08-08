use crate::{
    agent::{NormalizedAgentEvent, NormalizedAgentEventKind},
    desktop_execution_runtime::ExecutionRuntime,
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock},
    domain::{Execution, ExecutionStatus},
};

impl ExecutionRuntime {
    pub(crate) fn request_stop(
        &self,
        execution_id: &str,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError> {
        let execution = self.execution(execution_id)?;
        ensure_stoppable(&execution)?;
        if !lock(&self.agents, "agent runtime")?.contains_key(execution_id) {
            return Err(ExecutionRuntimeError::MissingLiveExecution {
                execution_id: execution_id.to_owned(),
            });
        }
        lock(&self.stop_requests, "execution stop requests")?.insert(execution_id.to_owned());
        let service = lock(&self.service, "board service")?;
        let work_item = service
            .work_item(&execution.work_item_id)
            .map_err(ExecutionRuntimeError::Board)?;
        service
            .snapshot(&work_item.work_item.board_id)
            .map_err(ExecutionRuntimeError::Board)
    }

    pub(crate) fn take_stop_request(&self, execution_id: &str) -> bool {
        lock(&self.stop_requests, "execution stop requests")
            .map(|mut requests| requests.remove(execution_id))
            .unwrap_or(false)
    }

    pub(crate) fn clear_stop_request(&self, execution_id: &str) {
        if let Ok(mut requests) = lock(&self.stop_requests, "execution stop requests") {
            requests.remove(execution_id);
        }
    }

    pub(crate) fn interrupt_execution(&self, execution: &Execution, session_id: &str) {
        self.stop_agent(&execution.id.0, session_id);
        let event = NormalizedAgentEvent {
            sequence: execution.last_event_sequence.saturating_add(1),
            kind: NormalizedAgentEventKind::Interrupted {
                reason: "User requested that the direct worker process stop. Any child processes may need manual cleanup.".to_owned(),
            },
        };
        if self.record_event(&execution.id.0, event).is_err() {
            self.record_monitor_failure(
                &execution.id.0,
                "the direct worker stopped but its interruption outcome could not be recorded",
            );
        }
    }
}

fn ensure_stoppable(execution: &Execution) -> Result<(), ExecutionRuntimeError> {
    if matches!(
        execution.status,
        ExecutionStatus::Running | ExecutionStatus::AwaitingInput
    ) {
        Ok(())
    } else {
        Err(ExecutionRuntimeError::ExecutionNotStoppable {
            execution_id: execution.id.0.clone(),
            status: execution.status,
        })
    }
}
