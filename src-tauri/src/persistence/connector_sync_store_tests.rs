use crate::{
    domain::{
        ConnectorOutboxItem, ConnectorOutboxItemId, ConnectorOutboxOperation, ConnectorOutboxState,
        ConnectorReconciliationItem, ConnectorReconciliationItemId, ConnectorReconciliationState,
        ConnectorSharedField, ExternalConnectionMode, ExternalLink, ExternalLinkId,
        ExternalLinkProvenance, SchemaMetadata,
    },
    persistence::{EventStoreError, SqliteEventStore},
};

use super::sqlite_event_store_tests::create_command;

fn store_with_linear_link() -> SqliteEventStore {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    store
        .record_external_link(ExternalLink {
            schema: SchemaMetadata::current(),
            id: ExternalLinkId::from("linear-link-1"),
            work_item_id: "task-1".into(),
            connector_id: "linear".to_owned(),
            provenance: ExternalLinkProvenance::Imported,
            external_id: "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad".to_owned(),
            display_identifier: "LIN-42".to_owned(),
            url: "https://linear.app/example/issue/LIN-42/sync".to_owned(),
            connection_mode: ExternalConnectionMode::LinkedExecution,
        })
        .expect("external link should persist");
    store
}

fn outbox_item() -> ConnectorOutboxItem {
    ConnectorOutboxItem {
        schema: SchemaMetadata::current(),
        id: ConnectorOutboxItemId::from("outbox-1"),
        work_item_id: "task-1".into(),
        connector_id: "linear".to_owned(),
        external_link_id: ExternalLinkId::from("linear-link-1"),
        idempotency_key: "task-1:review:1".to_owned(),
        operation: ConnectorOutboxOperation::Comment {
            body: "A safe public update.".to_owned(),
        },
        state: ConnectorOutboxState::Pending,
        created_at: "2026-08-09T10:00:00Z".to_owned(),
        delivered_at: None,
    }
}

#[test]
fn claims_one_pending_item_and_rejects_a_second_delivery_claim() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("outbox intent should persist");

    let claimed = store
        .claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1"))
        .expect("one delivery worker should claim the item");

    assert_eq!(claimed.state, ConnectorOutboxState::Delivering);
    assert!(matches!(
        store.claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1")),
        Err(EventStoreError::ConnectorOutboxCannotTransition { .. })
    ));
}

#[test]
fn records_delivered_items_and_preserves_an_idempotent_repeat() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("outbox intent should persist");
    store
        .claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1"))
        .expect("delivery should claim the item");
    let delivered = store
        .mark_connector_outbox_delivered(
            &ConnectorOutboxItemId::from("outbox-1"),
            "2026-08-09T10:01:00Z".to_owned(),
        )
        .expect("delivery result should persist");
    let repeated = store
        .record_connector_outbox_item(outbox_item())
        .expect("same local intent should not duplicate a sent comment");

    assert_eq!(delivered.state, ConnectorOutboxState::Delivered);
    assert_eq!(repeated, delivered);
}

#[test]
fn rejects_conflicting_outbox_identifiers_and_idempotency_keys() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("first outbox intent should persist");

    let mut conflicting_id = outbox_item();
    conflicting_id.operation = ConnectorOutboxOperation::Comment {
        body: "A different public update.".to_owned(),
    };
    assert!(matches!(
        store.record_connector_outbox_item(conflicting_id),
        Err(EventStoreError::ConnectorOutboxItemConflict { .. })
    ));

    let mut conflicting_key = outbox_item();
    conflicting_key.id = ConnectorOutboxItemId::from("outbox-2");
    conflicting_key.operation = ConnectorOutboxOperation::Comment {
        body: "A different public update.".to_owned(),
    };
    assert!(matches!(
        store.record_connector_outbox_item(conflicting_key),
        Err(EventStoreError::ConnectorOutboxIdempotencyConflict { .. })
    ));
}

#[test]
fn rejects_outbox_items_that_do_not_match_the_linear_link() {
    let mut store = store_with_linear_link();

    let mut unknown_link = outbox_item();
    unknown_link.external_link_id = ExternalLinkId::from("missing-link");
    assert!(matches!(
        store.record_connector_outbox_item(unknown_link),
        Err(EventStoreError::ExternalLinkNotFound { .. })
    ));

    let mut wrong_work_item = outbox_item();
    wrong_work_item.work_item_id = "task-2".into();
    assert!(matches!(
        store.record_connector_outbox_item(wrong_work_item),
        Err(EventStoreError::ExternalLinkNotFound { .. })
    ));

    let mut wrong_connector = outbox_item();
    wrong_connector.connector_id = "not-linear".to_owned();
    assert!(matches!(
        store.record_connector_outbox_item(wrong_connector),
        Err(EventStoreError::ExternalLinkNotFound { .. })
    ));
}

#[test]
fn refuses_to_finish_an_outbox_item_that_was_not_claimed() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("outbox intent should persist");

    assert!(matches!(
        store.mark_connector_outbox_delivered(
            &ConnectorOutboxItemId::from("outbox-1"),
            "2026-08-09T10:01:00Z".to_owned(),
        ),
        Err(EventStoreError::ConnectorOutboxCannotTransition { .. })
    ));
    assert!(matches!(
        store.claim_connector_outbox_item(&ConnectorOutboxItemId::from("missing-outbox")),
        Err(EventStoreError::ConnectorOutboxItemConflict { .. })
    ));
}

#[test]
fn marks_a_claimed_comment_delivery_uncertain_without_retrying_it() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("outbox intent should persist");
    store
        .claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1"))
        .expect("delivery should claim the item");

    let uncertain = store
        .mark_connector_outbox_delivery_uncertain(&ConnectorOutboxItemId::from("outbox-1"))
        .expect("an unknown remote outcome should remain visible");

    assert_eq!(uncertain.state, ConnectorOutboxState::DeliveryUncertain);
    assert!(uncertain.delivered_at.is_none());
    assert!(matches!(
        store.claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1")),
        Err(EventStoreError::ConnectorOutboxCannotTransition { .. })
    ));
}

#[test]
fn converts_in_flight_delivery_to_uncertain_during_restart_recovery() {
    let mut store = store_with_linear_link();
    store
        .record_connector_outbox_item(outbox_item())
        .expect("outbox intent should persist");
    store
        .claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1"))
        .expect("delivery should claim the item");

    store
        .recover_connector_outbox_deliveries()
        .expect("recovery should classify the unknown external result");

    let stored = store
        .connector_outbox_items_for_work_items(&["task-1".into()])
        .expect("outbox should remain readable");
    assert_eq!(stored[0].state, ConnectorOutboxState::DeliveryUncertain);
    assert!(matches!(
        store.claim_connector_outbox_item(&ConnectorOutboxItemId::from("outbox-1")),
        Err(EventStoreError::ConnectorOutboxCannotTransition { .. })
    ));
}

#[test]
fn records_one_immutable_reconciliation_per_remote_field_revision() {
    let mut store = store_with_linear_link();
    let item = ConnectorReconciliationItem {
        schema: SchemaMetadata::current(),
        id: ConnectorReconciliationItemId::from("reconciliation-1"),
        work_item_id: "task-1".into(),
        connector_id: "linear".to_owned(),
        external_link_id: ExternalLinkId::from("linear-link-1"),
        field: ConnectorSharedField::Title,
        local_value: "Local title".to_owned(),
        remote_value: "Linear title".to_owned(),
        remote_revision: "2026-08-09T10:02:00.000Z".to_owned(),
        state: ConnectorReconciliationState::NeedsResolution,
        observed_at: "2026-08-09T10:02:01Z".to_owned(),
    };

    let first = store
        .record_connector_reconciliation_item(item.clone())
        .expect("remote comparison should persist");
    let repeated = store
        .record_connector_reconciliation_item(item)
        .expect("replayed remote observation should be idempotent");

    assert_eq!(first, repeated);
    assert_eq!(
        store
            .connector_reconciliation_items_for_work_items(&["task-1".into()])
            .expect("reconciliation queue should load")
            .len(),
        1
    );
}

#[test]
fn rejects_conflicting_reconciliation_identifiers_and_revisions() {
    let mut store = store_with_linear_link();
    let item = ConnectorReconciliationItem {
        schema: SchemaMetadata::current(),
        id: ConnectorReconciliationItemId::from("reconciliation-1"),
        work_item_id: "task-1".into(),
        connector_id: "linear".to_owned(),
        external_link_id: ExternalLinkId::from("linear-link-1"),
        field: ConnectorSharedField::Title,
        local_value: "Local title".to_owned(),
        remote_value: "Linear title".to_owned(),
        remote_revision: "2026-08-09T10:02:00.000Z".to_owned(),
        state: ConnectorReconciliationState::NeedsResolution,
        observed_at: "2026-08-09T10:02:01Z".to_owned(),
    };
    store
        .record_connector_reconciliation_item(item.clone())
        .expect("first comparison should persist");

    let mut conflicting_id = item.clone();
    conflicting_id.remote_value = "A different Linear title".to_owned();
    assert!(matches!(
        store.record_connector_reconciliation_item(conflicting_id),
        Err(EventStoreError::ConnectorReconciliationItemConflict { .. })
    ));

    let mut conflicting_revision = item;
    conflicting_revision.id = ConnectorReconciliationItemId::from("reconciliation-2");
    conflicting_revision.remote_value = "A different Linear title".to_owned();
    assert!(matches!(
        store.record_connector_reconciliation_item(conflicting_revision),
        Err(EventStoreError::ConnectorReconciliationRevisionConflict { .. })
    ));
}
