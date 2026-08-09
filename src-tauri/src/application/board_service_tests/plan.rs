use super::{create_board, create_work_item_request, service};
use crate::{
    application::{
        BoardServiceError, ConfirmPlanRequest, CreateBoardRequest, ProposePlanRequest,
        ProposedPlanDependencyRequest, ProposedPlanWorkItemRequest,
    },
    domain::{BoardId, DependencyId, DependencyKind, WorkItemId, WorkItemState},
    persistence::{BoardStoreError, SqliteEventStore},
};
use tempfile::TempDir;

#[test]
fn confirms_a_saved_plan_once_and_materializes_its_tasks_and_dependencies() {
    let mut service = service();
    create_board(&mut service);

    let preview = service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    assert_eq!(
        preview.preview.critical_path,
        vec![
            WorkItemId::from("foundation"),
            WorkItemId::from("interface")
        ]
    );
    assert_eq!(
        preview.preview.parallel_stages,
        vec![
            vec![WorkItemId::from("foundation")],
            vec![WorkItemId::from("interface")]
        ]
    );
    assert!(preview.confirmation.is_none());

    let snapshot = service
        .confirm_plan(confirm_request("Daniel"))
        .expect("confirmed plan should materialize");
    assert_eq!(snapshot.work_items.len(), 2);
    assert!(
        snapshot
            .work_items
            .iter()
            .all(|entry| entry.work_item.state == WorkItemState::Inbox)
    );
    assert_eq!(snapshot.dependencies.len(), 1);
    assert_eq!(
        snapshot.dependencies[0].id,
        DependencyId::from("foundation-interface")
    );
    let confirmed = service
        .board_plan("board-1")
        .expect("plan should load")
        .expect("plan should exist");
    assert_eq!(confirmed.confirmation.unwrap().confirmed_by, "Daniel");
}

#[test]
fn repeats_a_matching_plan_confirmation_without_duplicate_board_records() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    service
        .confirm_plan(confirm_request("Daniel"))
        .expect("initial confirmation should materialize");

    let retried_snapshot = service
        .confirm_plan(confirm_request("Daniel"))
        .expect("matching confirmation should be idempotent");
    assert_eq!(retried_snapshot.work_items.len(), 2);
    assert_eq!(retried_snapshot.dependencies.len(), 1);
}

#[test]
fn rejects_a_mismatched_confirmation_without_materializing_any_plan_record() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    let mut mismatched_confirmation = confirm_request("Daniel");
    mismatched_confirmation.plan_id = "other-plan".to_owned();

    assert!(matches!(
        service.confirm_plan(mismatched_confirmation),
        Err(BoardServiceError::PlanConfirmation(
            crate::orchestration::PlanConfirmationError::PlanIdMismatch { .. }
        ))
    ));
    let snapshot = service
        .snapshot(&BoardId::from("board-1"))
        .expect("board should remain readable");
    assert!(snapshot.work_items.is_empty());
    assert!(snapshot.dependencies.is_empty());
}

#[test]
fn replaces_an_unconfirmed_plan_without_materializing_its_superseded_tasks() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("initial proposal should be saved");
    let mut replacement = proposal();
    replacement.plan_id = "plan-2".to_owned();
    replacement.work_items[0].title = "Revised foundation".to_owned();
    service
        .propose_plan(replacement)
        .expect("unconfirmed plan should be replaceable");

    let pending_plan = service
        .board_plan("board-1")
        .expect("plan should load")
        .expect("plan should exist");
    assert_eq!(pending_plan.preview.id, "plan-2".into());
    assert!(
        service
            .snapshot(&BoardId::from("board-1"))
            .expect("board should load")
            .work_items
            .is_empty()
    );

    let mut replacement_confirmation = confirm_request("Daniel");
    replacement_confirmation.plan_id = "plan-2".to_owned();
    let snapshot = service
        .confirm_plan(replacement_confirmation)
        .expect("replacement plan should materialize");
    assert_eq!(snapshot.work_items[0].work_item.title, "Revised foundation");
}

#[test]
fn rejects_a_different_confirmation_after_tasks_were_materialized() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    service
        .confirm_plan(confirm_request("Daniel"))
        .expect("initial confirmation should materialize");

    assert!(matches!(
        service.confirm_plan(confirm_request("Ada")),
        Err(BoardServiceError::Repository(
            BoardStoreError::PlanConfirmationConflict { .. }
        ))
    ));
    assert_eq!(
        service
            .snapshot(&BoardId::from("board-1"))
            .expect("snapshot should load")
            .work_items
            .len(),
        2
    );
}

#[test]
fn rejects_a_cross_board_confirmation_without_materializing_either_board() {
    let mut service = service();
    create_board(&mut service);
    create_second_board(&mut service);
    let mut second_board_proposal = proposal();
    second_board_proposal.plan_id = "plan-2".to_owned();
    second_board_proposal.board_id = "board-2".to_owned();
    service
        .propose_plan(second_board_proposal)
        .expect("second board proposal should be saved");

    let mut cross_board_confirmation = confirm_request("Daniel");
    cross_board_confirmation.plan_id = "plan-2".to_owned();
    assert!(matches!(
        service.confirm_plan(cross_board_confirmation),
        Err(BoardServiceError::PlanNotFound { plan_id }) if plan_id == "plan-2".into()
    ));
    assert!(
        service
            .snapshot(&BoardId::from("board-1"))
            .expect("first board should load")
            .work_items
            .is_empty()
    );
    assert!(
        service
            .snapshot(&BoardId::from("board-2"))
            .expect("second board should load")
            .work_items
            .is_empty()
    );
}

#[test]
fn rejects_replacing_a_confirmed_plan_without_mutating_the_materialized_board() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    service
        .confirm_plan(confirm_request("Daniel"))
        .expect("proposal should materialize");
    let mut replacement = proposal();
    replacement.plan_id = "plan-2".to_owned();

    assert!(matches!(
        service.propose_plan(replacement),
        Err(BoardServiceError::Repository(
            BoardStoreError::PlanAlreadyExists { .. }
        ))
    ));
    let snapshot = service
        .snapshot(&BoardId::from("board-1"))
        .expect("board should load");
    assert_eq!(snapshot.work_items.len(), 2);
    assert_eq!(snapshot.dependencies.len(), 1);
}

#[test]
fn rejects_a_duplicate_plan_id_for_a_different_board_without_saving_it() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("first proposal should be saved");
    create_second_board(&mut service);
    let mut conflicting_proposal = proposal();
    conflicting_proposal.board_id = "board-2".to_owned();

    assert!(matches!(
        service.propose_plan(conflicting_proposal),
        Err(BoardServiceError::Repository(BoardStoreError::PlanIdConflict { plan_id }))
            if plan_id == "plan-1".into()
    ));
    assert!(
        service
            .board_plan("board-2")
            .expect("second board plan should load")
            .is_none()
    );
}

#[test]
fn preserves_pending_and_confirmed_plan_evidence_across_database_reopen() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("board.sqlite");
    let mut service = crate::application::BoardService::new(
        SqliteEventStore::open(&database_path).expect("store should open"),
    );
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    drop(service);

    let mut reopened_service = crate::application::BoardService::new(
        SqliteEventStore::open(&database_path).expect("store should reopen"),
    );
    assert!(
        reopened_service
            .board_plan("board-1")
            .expect("pending plan should load")
            .expect("plan should exist")
            .confirmation
            .is_none()
    );
    reopened_service
        .confirm_plan(confirm_request("Daniel"))
        .expect("reopened plan should confirm");
    drop(reopened_service);

    let reopened_service = crate::application::BoardService::new(
        SqliteEventStore::open(&database_path).expect("store should reopen again"),
    );
    let plan = reopened_service
        .board_plan("board-1")
        .expect("confirmed plan should load")
        .expect("plan should exist");
    assert_eq!(plan.confirmation.unwrap().confirmed_by, "Daniel");
    assert_eq!(
        reopened_service
            .snapshot(&BoardId::from("board-1"))
            .expect("snapshot should load")
            .work_items
            .len(),
        2
    );
}

#[test]
fn rejects_a_plan_target_collision_without_partially_materializing_the_plan() {
    let mut service = service();
    create_board(&mut service);
    service
        .propose_plan(proposal())
        .expect("proposal should be saved");
    service
        .create_work_item(create_work_item_request("foundation"))
        .expect("manual task should be created");

    assert!(matches!(
        service.confirm_plan(confirm_request("Daniel")),
        Err(BoardServiceError::Repository(BoardStoreError::PlanWorkItemAlreadyExists {
            work_item_id,
        })) if work_item_id == WorkItemId::from("foundation")
    ));
    let snapshot = service
        .snapshot(&BoardId::from("board-1"))
        .expect("board should remain readable");
    assert_eq!(snapshot.work_items.len(), 1);
    assert!(snapshot.dependencies.is_empty());
}

#[test]
fn rejects_a_proposal_with_an_incomplete_task_specification() {
    let mut service = service();
    create_board(&mut service);
    let mut invalid_proposal = proposal();
    invalid_proposal.work_items[0].title = " ".to_owned();

    assert!(matches!(
        service.propose_plan(invalid_proposal),
        Err(BoardServiceError::MissingRequiredField {
            field: "plan work item title"
        })
    ));
}

fn proposal() -> ProposePlanRequest {
    ProposePlanRequest {
        plan_id: "plan-1".to_owned(),
        board_id: "board-1".to_owned(),
        proposed_by: "orchestrator".to_owned(),
        proposed_at: "2026-08-08T21:20:00Z".to_owned(),
        work_items: vec![work_item("foundation"), work_item("interface")],
        dependencies: vec![ProposedPlanDependencyRequest {
            dependency_id: "foundation-interface".to_owned(),
            upstream_work_item_id: "foundation".to_owned(),
            downstream_work_item_id: "interface".to_owned(),
            kind: DependencyKind::Blocks,
            reason: "The interface depends on the foundation contract.".to_owned(),
            owner: "orchestrator".to_owned(),
            next_action: "Complete the foundation work item.".to_owned(),
        }],
        unresolved_assumptions: vec!["The base branch is available locally.".to_owned()],
    }
}

fn work_item(id: &str) -> ProposedPlanWorkItemRequest {
    ProposedPlanWorkItemRequest {
        work_item_id: id.to_owned(),
        title: format!("Implement {id}"),
        description: format!("Deliver the {id} task."),
        acceptance_criteria: vec![format!("{id} behavior is verified")],
        budget: Default::default(),
        requires_human_review: false,
    }
}

fn confirm_request(confirmed_by: &str) -> ConfirmPlanRequest {
    ConfirmPlanRequest {
        board_id: "board-1".to_owned(),
        plan_id: "plan-1".to_owned(),
        confirmed_by: confirmed_by.to_owned(),
        confirmed_at: "2026-08-08T21:21:00Z".to_owned(),
    }
}

fn create_second_board(service: &mut crate::application::BoardService<SqliteEventStore>) {
    service
        .create_board(CreateBoardRequest {
            board_id: "board-2".to_owned(),
            project_id: "project-1".to_owned(),
            name: "Second board".to_owned(),
        })
        .expect("second board should be created");
}
