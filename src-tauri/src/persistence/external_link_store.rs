use rusqlite::{OptionalExtension, params};

use crate::domain::{ExternalLink, ExternalLinkId, WorkItemId};

use super::{EventStoreError, SqliteEventStore, execution_store::query_records_for_work_items};

impl SqliteEventStore {
    pub fn record_external_link(
        &mut self,
        link: ExternalLink,
    ) -> Result<ExternalLink, EventStoreError> {
        if self.materialized_work_item(&link.work_item_id)?.is_none() {
            return Err(EventStoreError::WorkItemNotFound {
                work_item_id: link.work_item_id,
            });
        }
        if let Some(recorded_link) = self.external_link(&link.id)? {
            return if recorded_link == link {
                Ok(recorded_link)
            } else {
                Err(EventStoreError::ExternalLinkIdConflict { link_id: link.id })
            };
        }
        if self
            .external_link_for_connector_resource(&link.connector_id, &link.external_id)?
            .is_some()
        {
            return Err(EventStoreError::ExternalResourceAlreadyLinked {
                connector_id: link.connector_id,
                external_id: link.external_id,
            });
        }

        let link_json = serde_json::to_string(&link)?;
        self.connection.execute(
            "INSERT INTO external_links (
                external_link_id, work_item_id, connector_id, external_id, link_json
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                link.id.0,
                link.work_item_id.0,
                link.connector_id,
                link.external_id,
                link_json,
            ],
        )?;
        Ok(link)
    }

    pub fn external_link(
        &self,
        link_id: &ExternalLinkId,
    ) -> Result<Option<ExternalLink>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT link_json FROM external_links WHERE external_link_id = ?1",
                [link_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|link_json| Ok(serde_json::from_str(&link_json)?))
            .transpose()
    }

    pub fn external_link_for_connector_resource(
        &self,
        connector_id: &str,
        external_id: &str,
    ) -> Result<Option<ExternalLink>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT link_json FROM external_links
                 WHERE connector_id = ?1 AND external_id = ?2",
                params![connector_id, external_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|link_json| Ok(serde_json::from_str(&link_json)?))
            .transpose()
    }

    pub fn external_links_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ExternalLink>, EventStoreError> {
        query_records_for_work_items(
            &self.connection,
            "external_links",
            "link_json",
            work_item_ids,
        )
    }
}
