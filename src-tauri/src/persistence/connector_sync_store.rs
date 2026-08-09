use rusqlite::{OptionalExtension, params};

use crate::domain::{
    ConnectorOutboxItem, ConnectorOutboxItemId, ConnectorOutboxState, ConnectorReconciliationItem,
    ConnectorReconciliationItemId, ConnectorSharedField, ExternalLinkId, WorkItemId,
};

use super::{EventStoreError, SqliteEventStore, execution_store::query_records_for_work_items};

impl SqliteEventStore {
    pub fn record_connector_outbox_item(
        &mut self,
        item: ConnectorOutboxItem,
    ) -> Result<ConnectorOutboxItem, EventStoreError> {
        self.require_connector_link(
            &item.external_link_id,
            &item.work_item_id,
            &item.connector_id,
        )?;
        if let Some(recorded) = self.connector_outbox_item(&item.id)? {
            return same_or_outbox_item_conflict(recorded, item);
        }
        if let Some(recorded) =
            self.connector_outbox_item_for_idempotency(&item.connector_id, &item.idempotency_key)?
        {
            return same_or_outbox_idempotency_conflict(recorded, item);
        }

        self.connection.execute(
            "INSERT INTO connector_outbox_items (
                connector_outbox_item_id, work_item_id, connector_id, external_link_id,
                idempotency_key, state, item_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id.0,
                item.work_item_id.0,
                item.connector_id,
                item.external_link_id.0,
                item.idempotency_key,
                state_key(item.state),
                serde_json::to_string(&item)?,
            ],
        )?;
        Ok(item)
    }

    pub fn claim_connector_outbox_item(
        &mut self,
        item_id: &ConnectorOutboxItemId,
    ) -> Result<ConnectorOutboxItem, EventStoreError> {
        let mut item = self.connector_outbox_item(item_id)?.ok_or_else(|| {
            EventStoreError::ConnectorOutboxItemConflict {
                item_id: item_id.clone(),
            }
        })?;
        if item.state != ConnectorOutboxState::Pending {
            return Err(EventStoreError::ConnectorOutboxCannotTransition { item_id: item.id });
        }
        item.state = ConnectorOutboxState::Delivering;
        self.save_connector_outbox_item_from_state(&item, ConnectorOutboxState::Pending)?;
        Ok(item)
    }

    pub fn mark_connector_outbox_delivered(
        &mut self,
        item_id: &ConnectorOutboxItemId,
        delivered_at: String,
    ) -> Result<ConnectorOutboxItem, EventStoreError> {
        self.finish_connector_outbox_item(
            item_id,
            ConnectorOutboxState::Delivered,
            Some(delivered_at),
        )
    }

    pub fn mark_connector_outbox_delivery_uncertain(
        &mut self,
        item_id: &ConnectorOutboxItemId,
    ) -> Result<ConnectorOutboxItem, EventStoreError> {
        self.finish_connector_outbox_item(item_id, ConnectorOutboxState::DeliveryUncertain, None)
    }

    pub fn recover_connector_outbox_deliveries(&mut self) -> Result<(), EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT item_json FROM connector_outbox_items WHERE state = ?1 ORDER BY rowid",
        )?;
        let items = statement
            .query_map([state_key(ConnectorOutboxState::Delivering)], |row| {
                row.get::<_, String>(0)
            })?
            .map(|row| Ok(serde_json::from_str::<ConnectorOutboxItem>(&row?)?))
            .collect::<Result<Vec<_>, EventStoreError>>()?;
        drop(statement);
        for mut item in items {
            item.state = ConnectorOutboxState::DeliveryUncertain;
            item.delivered_at = None;
            self.save_connector_outbox_item_from_state(&item, ConnectorOutboxState::Delivering)?;
        }
        Ok(())
    }

    pub fn connector_outbox_items_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ConnectorOutboxItem>, EventStoreError> {
        query_records_for_work_items(
            &self.connection,
            "connector_outbox_items",
            "item_json",
            work_item_ids,
        )
    }

    pub fn record_connector_reconciliation_item(
        &mut self,
        item: ConnectorReconciliationItem,
    ) -> Result<ConnectorReconciliationItem, EventStoreError> {
        self.require_connector_link(
            &item.external_link_id,
            &item.work_item_id,
            &item.connector_id,
        )?;
        if let Some(recorded) = self.connector_reconciliation_item(&item.id)? {
            return same_or_reconciliation_item_conflict(recorded, item);
        }
        if let Some(recorded) = self.connector_reconciliation_for_revision(
            &item.external_link_id,
            item.field,
            &item.remote_revision,
        )? {
            return same_or_reconciliation_revision_conflict(recorded, item);
        }

        self.connection.execute(
            "INSERT INTO connector_reconciliation_items (
                connector_reconciliation_item_id, work_item_id, connector_id, external_link_id,
                field, remote_revision, item_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id.0,
                item.work_item_id.0,
                item.connector_id,
                item.external_link_id.0,
                field_key(item.field),
                item.remote_revision,
                serde_json::to_string(&item)?,
            ],
        )?;
        Ok(item)
    }

    pub fn connector_reconciliation_items_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ConnectorReconciliationItem>, EventStoreError> {
        query_records_for_work_items(
            &self.connection,
            "connector_reconciliation_items",
            "item_json",
            work_item_ids,
        )
    }

    fn connector_outbox_item(
        &self,
        item_id: &ConnectorOutboxItemId,
    ) -> Result<Option<ConnectorOutboxItem>, EventStoreError> {
        self.read_record(
            "SELECT item_json FROM connector_outbox_items WHERE connector_outbox_item_id = ?1",
            &item_id.0,
        )
    }

    fn connector_outbox_item_for_idempotency(
        &self,
        connector_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<ConnectorOutboxItem>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT item_json FROM connector_outbox_items
                 WHERE connector_id = ?1 AND idempotency_key = ?2",
                params![connector_id, idempotency_key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|json| Ok(serde_json::from_str(&json)?))
            .transpose()
    }

    fn connector_reconciliation_item(
        &self,
        item_id: &ConnectorReconciliationItemId,
    ) -> Result<Option<ConnectorReconciliationItem>, EventStoreError> {
        self.read_record(
            "SELECT item_json FROM connector_reconciliation_items
             WHERE connector_reconciliation_item_id = ?1",
            &item_id.0,
        )
    }

    fn connector_reconciliation_for_revision(
        &self,
        external_link_id: &ExternalLinkId,
        field: ConnectorSharedField,
        remote_revision: &str,
    ) -> Result<Option<ConnectorReconciliationItem>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT item_json FROM connector_reconciliation_items
                 WHERE external_link_id = ?1 AND field = ?2 AND remote_revision = ?3",
                params![external_link_id.0, field_key(field), remote_revision],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|json| Ok(serde_json::from_str(&json)?))
            .transpose()
    }

    fn finish_connector_outbox_item(
        &mut self,
        item_id: &ConnectorOutboxItemId,
        state: ConnectorOutboxState,
        delivered_at: Option<String>,
    ) -> Result<ConnectorOutboxItem, EventStoreError> {
        let mut item = self.connector_outbox_item(item_id)?.ok_or_else(|| {
            EventStoreError::ConnectorOutboxItemConflict {
                item_id: item_id.clone(),
            }
        })?;
        if item.state != ConnectorOutboxState::Delivering {
            return Err(EventStoreError::ConnectorOutboxCannotTransition { item_id: item.id });
        }
        item.state = state;
        item.delivered_at = delivered_at;
        self.save_connector_outbox_item_from_state(&item, ConnectorOutboxState::Delivering)?;
        Ok(item)
    }

    fn save_connector_outbox_item_from_state(
        &mut self,
        item: &ConnectorOutboxItem,
        expected_state: ConnectorOutboxState,
    ) -> Result<(), EventStoreError> {
        let changed = self.connection.execute(
            "UPDATE connector_outbox_items
             SET state = ?1, item_json = ?2
             WHERE connector_outbox_item_id = ?3 AND state = ?4",
            params![
                state_key(item.state),
                serde_json::to_string(item)?,
                item.id.0,
                state_key(expected_state),
            ],
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(EventStoreError::ConnectorOutboxCannotTransition {
                item_id: item.id.clone(),
            })
        }
    }

    fn require_connector_link(
        &self,
        external_link_id: &ExternalLinkId,
        work_item_id: &WorkItemId,
        connector_id: &str,
    ) -> Result<(), EventStoreError> {
        let link = self.external_link(external_link_id)?.ok_or_else(|| {
            EventStoreError::ExternalLinkNotFound {
                link_id: external_link_id.clone(),
            }
        })?;
        if link.work_item_id == *work_item_id && link.connector_id == connector_id {
            Ok(())
        } else {
            Err(EventStoreError::ExternalLinkNotFound {
                link_id: external_link_id.clone(),
            })
        }
    }

    fn read_record<Record>(&self, sql: &str, id: &str) -> Result<Option<Record>, EventStoreError>
    where
        Record: serde::de::DeserializeOwned,
    {
        self.connection
            .query_row(sql, [id], |row| row.get::<_, String>(0))
            .optional()?
            .map(|json| Ok(serde_json::from_str(&json)?))
            .transpose()
    }
}

fn same_or_outbox_item_conflict(
    recorded: ConnectorOutboxItem,
    requested: ConnectorOutboxItem,
) -> Result<ConnectorOutboxItem, EventStoreError> {
    if recorded.work_item_id == requested.work_item_id
        && recorded.connector_id == requested.connector_id
        && recorded.external_link_id == requested.external_link_id
        && recorded.idempotency_key == requested.idempotency_key
        && recorded.operation == requested.operation
    {
        Ok(recorded)
    } else {
        Err(EventStoreError::ConnectorOutboxItemConflict {
            item_id: requested.id,
        })
    }
}

fn same_or_outbox_idempotency_conflict(
    recorded: ConnectorOutboxItem,
    requested: ConnectorOutboxItem,
) -> Result<ConnectorOutboxItem, EventStoreError> {
    if recorded.operation == requested.operation
        && recorded.work_item_id == requested.work_item_id
        && recorded.external_link_id == requested.external_link_id
    {
        Ok(recorded)
    } else {
        Err(EventStoreError::ConnectorOutboxIdempotencyConflict {
            connector_id: requested.connector_id,
            idempotency_key: requested.idempotency_key,
        })
    }
}

fn same_or_reconciliation_item_conflict(
    recorded: ConnectorReconciliationItem,
    requested: ConnectorReconciliationItem,
) -> Result<ConnectorReconciliationItem, EventStoreError> {
    if recorded == requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::ConnectorReconciliationItemConflict {
            item_id: requested.id,
        })
    }
}

fn same_or_reconciliation_revision_conflict(
    recorded: ConnectorReconciliationItem,
    requested: ConnectorReconciliationItem,
) -> Result<ConnectorReconciliationItem, EventStoreError> {
    if recorded == requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::ConnectorReconciliationRevisionConflict {
            external_link_id: requested.external_link_id,
            field: field_key(requested.field).to_owned(),
            remote_revision: requested.remote_revision,
        })
    }
}

fn state_key(state: ConnectorOutboxState) -> &'static str {
    match state {
        ConnectorOutboxState::Pending => "pending",
        ConnectorOutboxState::Delivering => "delivering",
        ConnectorOutboxState::Delivered => "delivered",
        ConnectorOutboxState::DeliveryUncertain => "delivery_uncertain",
    }
}

fn field_key(field: ConnectorSharedField) -> &'static str {
    match field {
        ConnectorSharedField::Title => "title",
        ConnectorSharedField::Description => "description",
        ConnectorSharedField::WorkflowState => "workflow_state",
    }
}
