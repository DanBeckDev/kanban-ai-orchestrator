use super::board_service_tests::{create_board, create_work_item_request, service};
use super::{
    BoardServiceError, ImportLinearIssueRequest, ObserveLinearSharedFieldRequest,
    QueueLinearCommentRequest,
};
use crate::domain::{
    ConnectorOutboxOperation, ConnectorOutboxState, ConnectorReconciliationState,
    ConnectorSharedField, ExternalConnectionMode,
};

const ISSUE_ID: &str = "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad";

fn linked_service() -> super::BoardService<crate::persistence::SqliteEventStore> {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("task should persist");
    service
        .import_linear_issue(ImportLinearIssueRequest {
            external_link_id: "linear-link-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            issue_id: ISSUE_ID.to_owned(),
            display_identifier: "LIN-42".to_owned(),
            url: "https://linear.app/example/issue/LIN-42/sync".to_owned(),
            connection_mode: ExternalConnectionMode::LinkedExecution,
        })
        .expect("linked Linear issue should persist");
    service
}

fn comment_request() -> QueueLinearCommentRequest {
    QueueLinearCommentRequest {
        outbox_item_id: "outbox-1".to_owned(),
        work_item_id: "task-1".to_owned(),
        idempotency_key: "task-1:review:1".to_owned(),
        public_summary: "Quality checks passed and the task is ready for review.".to_owned(),
        recorded_at: "2026-08-09T09:00:00Z".to_owned(),
    }
}

#[test]
fn queues_an_immutable_safe_comment_for_a_linked_linear_issue() {
    let mut service = linked_service();

    let snapshot = service
        .queue_linear_comment(comment_request())
        .expect("public comment should queue");

    assert_eq!(snapshot.connector_outbox_items.len(), 1);
    let item = &snapshot.connector_outbox_items[0];
    assert_eq!(item.state, ConnectorOutboxState::Pending);
    assert_eq!(item.connector_id, "linear");
    assert_eq!(item.external_link_id.0, "linear-link-1");
    assert!(matches!(
        item.operation,
        ConnectorOutboxOperation::Comment { ref body }
            if body.contains("Local task state: inbox")
                && body.contains("kanban-outbox:task-1:review:1")
    ));
}

#[test]
fn repeats_the_same_comment_intent_without_creating_a_second_outbox_item() {
    let mut service = linked_service();

    service
        .queue_linear_comment(comment_request())
        .expect("first intent should queue");
    let repeated = service
        .queue_linear_comment(comment_request())
        .expect("same idempotency key should be safe");

    assert_eq!(repeated.connector_outbox_items.len(), 1);
}

#[test]
fn claims_then_records_one_explicit_linear_comment_delivery() {
    let mut service = linked_service();
    service
        .queue_linear_comment(comment_request())
        .expect("comment intent should queue");

    let delivery = service
        .claim_linear_comment_delivery("outbox-1")
        .expect("outbox item should become the single delivery claim");
    let snapshot = service
        .mark_linear_comment_delivered(&delivery.outbox_item_id, "2026-08-09T09:00:01Z".to_owned())
        .expect("successful delivery should become durable");

    assert_eq!(delivery.issue_id, ISSUE_ID);
    assert!(delivery.body.contains("Kanban AI Orchestrator update"));
    assert_eq!(
        snapshot.connector_outbox_items[0].state,
        ConnectorOutboxState::Delivered
    );
}

#[test]
fn rejects_raw_transcript_patch_and_credential_like_comment_material() {
    let mut service = linked_service();
    for unsafe_summary in [
        "-----BEGIN PRIVATE KEY-----",
        "diff --git a/private b/private",
        "Authorization: Bearer secret",
        "First line\nsecond line",
    ] {
        let mut request = comment_request();
        request.public_summary = unsafe_summary.to_owned();
        assert!(matches!(
            service.queue_linear_comment(request),
            Err(BoardServiceError::InvalidPublicExternalComment { .. })
        ));
    }
    assert!(
        service
            .snapshot(&"board-1".into())
            .expect("board should remain readable")
            .connector_outbox_items
            .is_empty()
    );
}

#[test]
fn preserves_both_values_when_linear_and_local_shared_fields_differ() {
    let mut service = linked_service();

    let snapshot = service
        .observe_linear_shared_field(ObserveLinearSharedFieldRequest {
            reconciliation_item_id: "reconciliation-1".to_owned(),
            external_link_id: "linear-link-1".to_owned(),
            field: ConnectorSharedField::Title,
            remote_value: "Changed in Linear".to_owned(),
            remote_revision: "2026-08-09T09:01:00.000Z".to_owned(),
            observed_at: "2026-08-09T09:01:01Z".to_owned(),
        })
        .expect("remote observation should persist");

    assert_eq!(snapshot.connector_reconciliation_items.len(), 1);
    let item = &snapshot.connector_reconciliation_items[0];
    assert_eq!(item.state, ConnectorReconciliationState::NeedsResolution);
    assert_eq!(item.local_value, "Implement task-1");
    assert_eq!(item.remote_value, "Changed in Linear");
    assert_eq!(snapshot.work_items[0].work_item.title, "Implement task-1");
}

#[test]
fn records_a_matching_remote_field_without_overwriting_the_work_item() {
    let mut service = linked_service();

    let snapshot = service
        .observe_linear_shared_field(ObserveLinearSharedFieldRequest {
            reconciliation_item_id: "reconciliation-1".to_owned(),
            external_link_id: "linear-link-1".to_owned(),
            field: ConnectorSharedField::Description,
            remote_value: "A bounded implementation task.".to_owned(),
            remote_revision: "2026-08-09T09:02:00.000Z".to_owned(),
            observed_at: "2026-08-09T09:02:01Z".to_owned(),
        })
        .expect("matching observation should persist");

    assert_eq!(
        snapshot.connector_reconciliation_items[0].state,
        ConnectorReconciliationState::Matched
    );
}

#[test]
fn records_an_intentionally_empty_linear_description_as_a_conflict() {
    let mut service = linked_service();

    let snapshot = service
        .observe_linear_shared_field(ObserveLinearSharedFieldRequest {
            reconciliation_item_id: "reconciliation-empty-description".to_owned(),
            external_link_id: "linear-link-1".to_owned(),
            field: ConnectorSharedField::Description,
            remote_value: String::new(),
            remote_revision: "2026-08-09T09:03:00.000Z".to_owned(),
            observed_at: "2026-08-09T09:03:01Z".to_owned(),
        })
        .expect("an empty remote description is still a valid Linear value");

    assert_eq!(
        snapshot.connector_reconciliation_items[0].state,
        ConnectorReconciliationState::NeedsResolution
    );
    assert!(
        snapshot.connector_reconciliation_items[0]
            .remote_value
            .is_empty()
    );
}

#[test]
fn records_a_matching_linear_workflow_state_without_transitioning_the_task() {
    let mut service = linked_service();

    let snapshot = service
        .observe_linear_shared_field(ObserveLinearSharedFieldRequest {
            reconciliation_item_id: "reconciliation-workflow-state".to_owned(),
            external_link_id: "linear-link-1".to_owned(),
            field: ConnectorSharedField::WorkflowState,
            remote_value: "inbox".to_owned(),
            remote_revision: "2026-08-09T09:04:00.000Z".to_owned(),
            observed_at: "2026-08-09T09:04:01Z".to_owned(),
        })
        .expect("workflow comparison should persist");

    assert_eq!(
        snapshot.connector_reconciliation_items[0].state,
        ConnectorReconciliationState::Matched
    );
    assert_eq!(
        snapshot.work_items[0].work_item.state,
        crate::domain::WorkItemState::Inbox
    );
}
