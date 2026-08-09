use crate::domain::{
    ConnectorOutboxItem, ConnectorOutboxItemId, ConnectorOutboxOperation, ConnectorOutboxState,
    ConnectorReconciliationItem, ConnectorReconciliationItemId, ConnectorReconciliationState,
    ConnectorSharedField, ExternalConnectionMode, ExternalLink, ExternalLinkId, SchemaMetadata,
    WorkItemId, WorkItemState,
};

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot,
    ObserveLinearSharedFieldRequest, QueueLinearCommentRequest,
};

const LINEAR_CONNECTOR_ID: &str = "linear";
const MAX_PUBLIC_SUMMARY_BYTES: usize = 512;
const MAX_REMOTE_VALUE_BYTES: usize = 4_096;
const MAX_REMOTE_REVISION_BYTES: usize = 256;
const UNSAFE_PUBLIC_COMMENT_MARKERS: [&str; 6] = [
    "-----begin",
    "authorization:",
    "diff --git",
    "```diff",
    "ghp_",
    "sk-",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearCommentDelivery {
    pub outbox_item_id: String,
    pub issue_id: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearIssueSyncTarget {
    pub external_link_id: String,
    pub issue_id: String,
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn queue_linear_comment(
        &mut self,
        request: QueueLinearCommentRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.outbox_item_id, "Linear outbox item id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.idempotency_key, "Linear idempotency key")?;
        validate_required(&request.recorded_at, "recorded at")?;
        validate_public_summary(&request.public_summary)?;

        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let materialized = self.require_materialized_work_item(&work_item_id)?;
        let link = self.linked_linear_link(&work_item_id)?;
        let body = public_comment_body(
            materialized.work_item.state,
            &request.public_summary,
            &request.idempotency_key,
        );
        self.repository
            .record_connector_outbox_item(ConnectorOutboxItem {
                schema: SchemaMetadata::current(),
                id: ConnectorOutboxItemId::from(request.outbox_item_id.as_str()),
                work_item_id: work_item_id.clone(),
                connector_id: LINEAR_CONNECTOR_ID.to_owned(),
                external_link_id: link.id,
                idempotency_key: request.idempotency_key,
                operation: ConnectorOutboxOperation::Comment { body },
                state: ConnectorOutboxState::Pending,
                created_at: request.recorded_at,
                delivered_at: None,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&materialized.work_item.board_id)
    }

    pub fn observe_linear_shared_field(
        &mut self,
        request: ObserveLinearSharedFieldRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        for (value, field) in [
            (
                &request.reconciliation_item_id,
                "Linear reconciliation item id",
            ),
            (&request.external_link_id, "Linear external link id"),
            (&request.remote_revision, "Linear remote revision"),
            (&request.observed_at, "observed at"),
        ] {
            validate_required(value, field)?;
        }
        validate_maximum_bytes(
            &request.remote_value,
            "Linear remote value",
            MAX_REMOTE_VALUE_BYTES,
        )?;
        validate_maximum_bytes(
            &request.remote_revision,
            "Linear remote revision",
            MAX_REMOTE_REVISION_BYTES,
        )?;

        let link_id = ExternalLinkId::from(request.external_link_id.as_str());
        let link = self.required_linear_link(&link_id)?;
        let materialized = self.require_materialized_work_item(&link.work_item_id)?;
        let local_value = local_shared_value(request.field, &materialized.work_item);
        let reconciliation_state = reconciliation_state(local_value, &request.remote_value);
        self.repository
            .record_connector_reconciliation_item(ConnectorReconciliationItem {
                schema: SchemaMetadata::current(),
                id: ConnectorReconciliationItemId::from(request.reconciliation_item_id.as_str()),
                work_item_id: link.work_item_id,
                connector_id: LINEAR_CONNECTOR_ID.to_owned(),
                external_link_id: link.id,
                field: request.field,
                local_value: local_value.to_owned(),
                remote_value: request.remote_value,
                remote_revision: request.remote_revision,
                state: reconciliation_state,
                observed_at: request.observed_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&materialized.work_item.board_id)
    }

    pub fn claim_linear_comment_delivery(
        &mut self,
        outbox_item_id: &str,
    ) -> Result<LinearCommentDelivery, BoardServiceError<Repository::Error>> {
        validate_required(outbox_item_id, "Linear outbox item id")?;
        let item = self
            .repository
            .claim_connector_outbox_item(&ConnectorOutboxItemId::from(outbox_item_id))
            .map_err(BoardServiceError::Repository)?;
        let link = self.required_linear_link(&item.external_link_id)?;
        let ConnectorOutboxOperation::Comment { body } = item.operation;
        Ok(LinearCommentDelivery {
            outbox_item_id: item.id.0,
            issue_id: link.external_id,
            body,
        })
    }

    pub fn linear_issue_sync_target(
        &self,
        external_link_id: &str,
    ) -> Result<LinearIssueSyncTarget, BoardServiceError<Repository::Error>> {
        validate_required(external_link_id, "Linear external link id")?;
        let link = self.required_linear_link(&ExternalLinkId::from(external_link_id))?;
        Ok(LinearIssueSyncTarget {
            external_link_id: link.id.0,
            issue_id: link.external_id,
        })
    }

    pub fn mark_linear_comment_delivered(
        &mut self,
        outbox_item_id: &str,
        delivered_at: String,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(outbox_item_id, "Linear outbox item id")?;
        validate_required(&delivered_at, "delivered at")?;
        let item = self
            .repository
            .mark_connector_outbox_delivered(
                &ConnectorOutboxItemId::from(outbox_item_id),
                delivered_at,
            )
            .map_err(BoardServiceError::Repository)?;
        let materialized = self.require_materialized_work_item(&item.work_item_id)?;
        self.snapshot(&materialized.work_item.board_id)
    }

    pub fn mark_linear_comment_delivery_uncertain(
        &mut self,
        outbox_item_id: &str,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(outbox_item_id, "Linear outbox item id")?;
        let item = self
            .repository
            .mark_connector_outbox_delivery_uncertain(&ConnectorOutboxItemId::from(outbox_item_id))
            .map_err(BoardServiceError::Repository)?;
        let materialized = self.require_materialized_work_item(&item.work_item_id)?;
        self.snapshot(&materialized.work_item.board_id)
    }

    fn require_materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<crate::domain::MaterializedWorkItem, BoardServiceError<Repository::Error>> {
        self.repository
            .materialized_work_item(work_item_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            })
    }

    fn linked_linear_link(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<ExternalLink, BoardServiceError<Repository::Error>> {
        self.repository
            .external_links_for_work_items(std::slice::from_ref(work_item_id))
            .map_err(BoardServiceError::Repository)?
            .into_iter()
            .find(|link| {
                link.connector_id == LINEAR_CONNECTOR_ID
                    && link.connection_mode == ExternalConnectionMode::LinkedExecution
            })
            .ok_or_else(|| BoardServiceError::ExternalSyncRequiresLinkedExecution {
                work_item_id: work_item_id.clone(),
            })
    }

    fn required_linear_link(
        &self,
        link_id: &ExternalLinkId,
    ) -> Result<ExternalLink, BoardServiceError<Repository::Error>> {
        let link = self
            .repository
            .external_link(link_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::ExternalLinkNotFound {
                link_id: link_id.clone(),
            })?;
        if link.connector_id == LINEAR_CONNECTOR_ID {
            Ok(link)
        } else {
            Err(BoardServiceError::ExternalLinkNotFound {
                link_id: link_id.clone(),
            })
        }
    }
}

fn validate_public_summary<RepositoryError>(
    summary: &str,
) -> Result<(), BoardServiceError<RepositoryError>> {
    validate_required(summary, "public Linear summary")?;
    validate_maximum_bytes(summary, "public Linear summary", MAX_PUBLIC_SUMMARY_BYTES)?;
    if summary.contains(['\n', '\r']) {
        return Err(BoardServiceError::InvalidPublicExternalComment {
            reason: "it must be a single concise line",
        });
    }
    let lowercase = summary.to_ascii_lowercase();
    if UNSAFE_PUBLIC_COMMENT_MARKERS
        .iter()
        .any(|marker| lowercase.contains(marker))
    {
        return Err(BoardServiceError::InvalidPublicExternalComment {
            reason: "it resembles a credential, patch, or raw diagnostic",
        });
    }
    Ok(())
}

fn validate_maximum_bytes<RepositoryError>(
    value: &str,
    field: &'static str,
    maximum_bytes: usize,
) -> Result<(), BoardServiceError<RepositoryError>> {
    if value.len() > maximum_bytes {
        Err(BoardServiceError::ExternalSyncValueTooLong {
            field,
            maximum_bytes,
        })
    } else {
        Ok(())
    }
}

fn public_comment_body(state: WorkItemState, summary: &str, idempotency_key: &str) -> String {
    format!(
        "**Kanban AI Orchestrator update**\n\n- Local task state: {state}\n- Update: {summary}\n\n<!-- kanban-outbox:{idempotency_key} -->"
    )
}

fn local_shared_value(field: ConnectorSharedField, work_item: &crate::domain::WorkItem) -> &str {
    match field {
        ConnectorSharedField::Title => &work_item.title,
        ConnectorSharedField::Description => &work_item.description,
        ConnectorSharedField::WorkflowState => match work_item.state {
            WorkItemState::Inbox => "inbox",
            WorkItemState::Planned => "planned",
            WorkItemState::Ready => "ready",
            WorkItemState::Running => "running",
            WorkItemState::AwaitingInput => "awaiting_input",
            WorkItemState::Review => "review",
            WorkItemState::Done => "done",
            WorkItemState::Blocked => "blocked",
            WorkItemState::Failed => "failed",
            WorkItemState::Cancelled => "cancelled",
            WorkItemState::Interrupted => "interrupted",
        },
    }
}

fn reconciliation_state(local_value: &str, remote_value: &str) -> ConnectorReconciliationState {
    if local_value == remote_value {
        ConnectorReconciliationState::Matched
    } else {
        ConnectorReconciliationState::NeedsResolution
    }
}
