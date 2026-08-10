use crate::{
    agent::{NormalizedAgentEvent, NormalizedAgentEventKind},
    application::{RecordEvidenceRequest, UpdateExecutionRequest},
    desktop_execution_runtime::ExecutionRuntime,
    desktop_execution_runtime_support::{lock, timestamp},
    domain::{EvidenceKind, EvidenceResult, ExecutionId, ExecutionStatus},
};

impl ExecutionRuntime {
    pub(crate) fn record_monitor_failure(&self, execution_id: &str, reason: &str) {
        let Ok(execution) = self.execution(execution_id) else {
            return;
        };
        if !matches!(
            execution.status,
            ExecutionStatus::Running | ExecutionStatus::AwaitingInput
        ) {
            return;
        }
        let event = NormalizedAgentEvent {
            sequence: execution.last_event_sequence.saturating_add(1),
            kind: NormalizedAgentEventKind::Failed {
                reason: reason.to_owned(),
            },
        };
        if self.record_event(execution_id, event).is_ok() {
            self.coordinate_after_execution(&execution.work_item_id);
        }
    }

    pub(crate) fn fail_pending_execution(&self, execution_id: &str, reason: &str) {
        let Ok(mut service) = lock(&self.service, "board service") else {
            return;
        };
        let Ok(execution) = service.execution(&ExecutionId::from(execution_id)) else {
            return;
        };
        if execution.status != ExecutionStatus::Pending {
            return;
        }
        let _ = service.update_execution(UpdateExecutionRequest {
            execution_id: execution.id.0.clone(),
            status: ExecutionStatus::Failed,
            session_id: None,
            usage: execution.usage,
            last_event_sequence: execution.last_event_sequence,
        });
        let _ = service.record_evidence(RecordEvidenceRequest {
            evidence_id: format!("launch-failure-{execution_id}"),
            work_item_id: execution.work_item_id.0,
            kind: EvidenceKind::AgentReport,
            result: EvidenceResult::Failed,
            summary: reason.to_owned(),
            recorded_at: timestamp(),
        });
    }
}
