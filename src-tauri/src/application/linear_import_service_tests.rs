use super::board_service_tests::{create_board, create_work_item_request, service};
use super::{BoardServiceError, ImportLinearBlockerRequest, ImportLinearIssueRequest};
use crate::domain::{BoardId, DependencyKind, DependencySource, ExternalConnectionMode};

fn issue_request(
    link_id: &str,
    work_item_id: &str,
    issue_id: &str,
    mode: ExternalConnectionMode,
) -> ImportLinearIssueRequest {
    ImportLinearIssueRequest {
        external_link_id: link_id.to_owned(),
        work_item_id: work_item_id.to_owned(),
        issue_id: issue_id.to_owned(),
        display_identifier: format!("LIN-{issue_id}"),
        url: format!("https://linear.app/example/issue/{issue_id}"),
        connection_mode: mode,
    }
}

fn blocker_request(
    dependency_id: &str,
    upstream_issue_id: &str,
    downstream_issue_id: &str,
) -> ImportLinearBlockerRequest {
    ImportLinearBlockerRequest {
        dependency_id: dependency_id.to_owned(),
        upstream_issue_id: upstream_issue_id.to_owned(),
        downstream_issue_id: downstream_issue_id.to_owned(),
        reason: "Linear says the downstream issue is blocked.".to_owned(),
        owner: "linear-team".to_owned(),
        next_action: "Complete the upstream Linear issue.".to_owned(),
        created_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

#[test]
fn imports_immutable_linear_links_and_provenance_aware_blockers() {
    let mut service = service();
    create_board(&mut service);
    for work_item_id in ["task-1", "task-2"] {
        service
            .create_work_item(create_work_item_request(work_item_id))
            .expect("task should persist");
    }

    service
        .import_linear_issue(issue_request(
            "link-1",
            "task-1",
            "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
            ExternalConnectionMode::ReadOnly,
        ))
        .expect("read-only link should persist");
    let linked_snapshot = service
        .import_linear_issue(issue_request(
            "link-2",
            "task-2",
            "9a860a70-70a3-456f-9944-d3a474446e74",
            ExternalConnectionMode::LinkedExecution,
        ))
        .expect("linked-execution link should persist");
    let imported_snapshot = service
        .import_linear_blocker(blocker_request(
            "dependency-1",
            "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
            "9a860a70-70a3-456f-9944-d3a474446e74",
        ))
        .expect("valid imported blocker should persist");

    assert_eq!(linked_snapshot.external_links.len(), 2);
    assert_eq!(
        linked_snapshot.external_links[1].connection_mode,
        ExternalConnectionMode::LinkedExecution
    );
    assert_eq!(imported_snapshot.dependencies.len(), 1);
    assert_eq!(
        imported_snapshot.dependencies[0].kind,
        DependencyKind::Blocks
    );
    assert!(matches!(
        imported_snapshot.dependencies[0].source,
        DependencySource::Connector { ref connector_id } if connector_id == "linear"
    ));
}

#[test]
fn rejects_unlinked_or_cyclic_linear_blockers_without_mutating_the_graph() {
    let mut service = service();
    create_board(&mut service);
    for work_item_id in ["task-1", "task-2"] {
        service
            .create_work_item(create_work_item_request(work_item_id))
            .expect("task should persist");
    }
    assert!(matches!(
        service.import_linear_blocker(blocker_request(
            "missing",
            "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
            "9a860a70-70a3-456f-9944-d3a474446e74",
        )),
        Err(BoardServiceError::ExternalResourceNotLinked { .. })
    ));
    for (link_id, work_item_id, issue_id) in [
        ("link-1", "task-1", "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad"),
        ("link-2", "task-2", "9a860a70-70a3-456f-9944-d3a474446e74"),
    ] {
        service
            .import_linear_issue(issue_request(
                link_id,
                work_item_id,
                issue_id,
                ExternalConnectionMode::ReadOnly,
            ))
            .expect("link should persist");
    }
    service
        .import_linear_blocker(blocker_request(
            "forward",
            "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
            "9a860a70-70a3-456f-9944-d3a474446e74",
        ))
        .expect("first edge should persist");

    assert!(
        service
            .import_linear_blocker(blocker_request(
                "cycle",
                "9a860a70-70a3-456f-9944-d3a474446e74",
                "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
            ))
            .is_err()
    );
}

#[test]
fn rejects_noncanonical_linear_identity_or_url_before_persisting_a_link() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("task should persist");

    let mut invalid_identifier = issue_request(
        "link-1",
        "task-1",
        "not-a-uuid",
        ExternalConnectionMode::ReadOnly,
    );
    assert!(matches!(
        service.import_linear_issue(invalid_identifier.clone()),
        Err(BoardServiceError::InvalidExternalIdentifier { .. })
    ));
    invalid_identifier.issue_id = "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad".to_owned();
    invalid_identifier.url = "javascript:alert(1)".to_owned();

    assert!(matches!(
        service.import_linear_issue(invalid_identifier),
        Err(BoardServiceError::InvalidExternalUrl)
    ));
    assert!(
        service
            .snapshot(&BoardId::from("board-1"))
            .expect("board should remain readable")
            .external_links
            .is_empty()
    );
}
