use serde::Serialize;

use crate::domain::{
    Board, CompletionEvidence, ConnectorOutboxItem, ConnectorReconciliationItem, Dependency,
    Evidence, Execution, ExternalLink, MaterializedWorkItem, RecordedWorkItemEvent,
    WorkItemEventKind, WorkItemId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSnapshot {
    pub board: Board,
    pub work_items: Vec<MaterializedWorkItem>,
    pub dependencies: Vec<Dependency>,
    pub activity: Vec<BoardActivity>,
    pub executions: Vec<Execution>,
    pub evidence: Vec<Evidence>,
    pub external_links: Vec<ExternalLink>,
    pub connector_outbox_items: Vec<ConnectorOutboxItem>,
    pub connector_reconciliation_items: Vec<ConnectorReconciliationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardActivity {
    pub work_item_id: WorkItemId,
    pub sequence: u64,
    pub recorded_at: String,
    pub summary: String,
    pub completion_evidence: Option<CompletionEvidence>,
}

pub fn board_activity(recorded_event: RecordedWorkItemEvent) -> BoardActivity {
    let (summary, completion_evidence) = match recorded_event.event.kind {
        WorkItemEventKind::Created { .. } => ("Task created.".to_owned(), None),
        WorkItemEventKind::StateTransitioned {
            from,
            to,
            evidence,
            reason,
            ..
        } => (
            format!("State changed from {from} to {to}: {reason}"),
            evidence,
        ),
        WorkItemEventKind::DetailsRefined { reason, .. } => {
            (format!("Task details refined: {reason}"), None)
        }
    };

    BoardActivity {
        work_item_id: recorded_event.event.work_item_id,
        sequence: recorded_event.sequence,
        recorded_at: recorded_event.event.recorded_at,
        summary,
        completion_evidence,
    }
}
