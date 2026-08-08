use crate::domain::{
    Dependency, DependencyId, DependencyKind, DependencySource, ExternalLink, ExternalLinkId,
    ExternalLinkProvenance, SchemaMetadata, WorkItemId,
};
use url::Url;
use uuid::Uuid;

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, ImportLinearBlockerRequest,
    ImportLinearIssueRequest,
};

const LINEAR_CONNECTOR_ID: &str = "linear";

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn import_linear_issue(
        &mut self,
        request: ImportLinearIssueRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_issue_request(&request)?;
        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let board_id = self.board_id_for(&work_item_id)?;
        self.repository
            .record_external_link(ExternalLink {
                schema: SchemaMetadata::current(),
                id: ExternalLinkId::from(request.external_link_id.as_str()),
                work_item_id,
                connector_id: LINEAR_CONNECTOR_ID.to_owned(),
                provenance: ExternalLinkProvenance::Imported,
                external_id: request.issue_id,
                display_identifier: request.display_identifier,
                url: request.url,
                connection_mode: request.connection_mode,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    pub fn import_linear_blocker(
        &mut self,
        request: ImportLinearBlockerRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_blocker_request(&request)?;
        let upstream_link = self.linear_link(&request.upstream_issue_id)?;
        let downstream_link = self.linear_link(&request.downstream_issue_id)?;
        let board_id = self.board_id_for(&upstream_link.work_item_id)?;
        self.repository
            .add_board_dependency(Dependency {
                schema: SchemaMetadata::current(),
                id: DependencyId::from(request.dependency_id.as_str()),
                upstream_work_item_id: upstream_link.work_item_id,
                downstream_work_item_id: downstream_link.work_item_id,
                kind: DependencyKind::Blocks,
                source: DependencySource::Connector {
                    connector_id: LINEAR_CONNECTOR_ID.to_owned(),
                },
                reason: request.reason,
                owner: request.owner,
                next_action: request.next_action,
                created_by: "linear-import".to_owned(),
                created_at: request.created_at,
            })
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }

    fn linear_link(
        &self,
        issue_id: &str,
    ) -> Result<ExternalLink, BoardServiceError<Repository::Error>> {
        self.repository
            .external_link_for_connector_resource(LINEAR_CONNECTOR_ID, issue_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::ExternalResourceNotLinked {
                connector_id: LINEAR_CONNECTOR_ID,
                external_id: issue_id.to_owned(),
            })
    }
}

fn validate_issue_request<RepositoryError>(
    request: &ImportLinearIssueRequest,
) -> Result<(), BoardServiceError<RepositoryError>> {
    for (value, field) in [
        (&request.external_link_id, "Linear link id"),
        (&request.work_item_id, "work item id"),
        (&request.issue_id, "Linear issue UUID"),
        (&request.display_identifier, "Linear issue identifier"),
        (&request.url, "Linear issue URL"),
    ] {
        validate_required(value, field)?;
    }
    validate_linear_issue_id(&request.issue_id)?;
    validate_linear_issue_url(&request.url)?;
    Ok(())
}

fn validate_blocker_request<RepositoryError>(
    request: &ImportLinearBlockerRequest,
) -> Result<(), BoardServiceError<RepositoryError>> {
    for (value, field) in [
        (&request.dependency_id, "dependency id"),
        (&request.upstream_issue_id, "upstream Linear issue UUID"),
        (&request.downstream_issue_id, "downstream Linear issue UUID"),
        (&request.reason, "dependency reason"),
        (&request.owner, "dependency owner"),
        (&request.next_action, "dependency next action"),
        (&request.created_at, "dependency created at"),
    ] {
        validate_required(value, field)?;
    }
    Ok(())
}

fn validate_linear_issue_id<RepositoryError>(
    issue_id: &str,
) -> Result<(), BoardServiceError<RepositoryError>> {
    Uuid::parse_str(issue_id).map_err(|_| BoardServiceError::InvalidExternalIdentifier {
        field: "Linear issue UUID",
    })?;
    Ok(())
}

fn validate_linear_issue_url<RepositoryError>(
    url: &str,
) -> Result<(), BoardServiceError<RepositoryError>> {
    let parsed_url = Url::parse(url).map_err(|_| BoardServiceError::InvalidExternalUrl)?;
    if parsed_url.scheme() == "https" && parsed_url.host_str() == Some("linear.app") {
        Ok(())
    } else {
        Err(BoardServiceError::InvalidExternalUrl)
    }
}
