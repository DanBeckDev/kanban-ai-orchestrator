use crate::{
    agent::{AgentProfile, AgentProfileKind},
    application::{
        ProjectAgentSettingsError, ProposePlanRequest, ProposedPlanWorkItemRequest,
        SaveProjectAgentSettingsRequest,
        board_service_tests::{create_board, create_work_item_request, service},
    },
    domain::{AgentEffort, AgentModelPreference, OrganiserDefaults, TicketWorkerDefaults},
    orchestration::PlannerProfile,
    persistence::SqliteEventStore,
};
use tempfile::TempDir;

#[test]
fn stores_project_scoped_role_defaults_and_uses_the_worker_for_new_tasks() {
    let mut service = service();
    create_board(&mut service);
    save_profiles(&mut service);

    let saved = service
        .save_project_agent_settings(settings_request())
        .expect("settings should save");
    let snapshot = service
        .create_work_item(create_work_item_request("manual-task"))
        .expect("manual task should be created");

    assert_eq!(
        saved
            .organiser
            .as_ref()
            .expect("organiser should be saved")
            .effort,
        AgentEffort::Thorough
    );
    assert_eq!(
        snapshot.work_items[0]
            .work_item
            .assigned_agent_profile_name
            .as_deref(),
        Some("ticket-worker")
    );
    assert_eq!(
        service
            .project_agent_settings_for_board("board-1")
            .expect("settings should load"),
        Some(saved)
    );
}

#[test]
fn rejects_role_defaults_that_reference_unavailable_profiles() {
    let mut service = service();
    create_board(&mut service);
    let request = SaveProjectAgentSettingsRequest {
        board_id: "board-1".to_owned(),
        organiser: Some(OrganiserDefaults {
            planner_profile_name: "missing organiser".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        }),
        ticket_worker: None,
    };

    assert!(matches!(
        service.save_project_agent_settings(request),
        Err(ProjectAgentSettingsError::OrganiserProfileNotFound { profile_name })
            if profile_name == "missing organiser"
    ));
}

#[test]
fn rejects_a_ticket_worker_profile_that_has_not_been_saved() {
    let mut service = service();
    create_board(&mut service);

    let request = SaveProjectAgentSettingsRequest {
        board_id: "board-1".to_owned(),
        organiser: None,
        ticket_worker: Some(TicketWorkerDefaults {
            agent_profile_name: "missing worker".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        }),
    };

    assert!(matches!(
        service.save_project_agent_settings(request),
        Err(ProjectAgentSettingsError::TicketWorkerProfileNotFound { profile_name })
            if profile_name == "missing worker"
    ));
}

#[test]
fn permits_a_project_to_clear_both_role_defaults() {
    let mut service = service();
    create_board(&mut service);

    let saved = service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: None,
            ticket_worker: None,
        })
        .expect("empty role settings should save");

    assert!(saved.organiser.is_none());
    assert!(saved.ticket_worker.is_none());
}

#[test]
fn validates_named_model_preferences_before_persisting_role_defaults() {
    let mut service = service();
    create_board(&mut service);
    save_profiles(&mut service);

    let mut valid = settings_request();
    valid.ticket_worker = Some(TicketWorkerDefaults {
        agent_profile_name: "ticket-worker".to_owned(),
        model: AgentModelPreference::Named("worker-model".to_owned()),
        effort: AgentEffort::Balanced,
    });
    assert_eq!(
        service
            .save_project_agent_settings(valid)
            .expect("a named worker model should save")
            .ticket_worker
            .expect("ticket worker should be retained")
            .model,
        AgentModelPreference::Named("worker-model".to_owned())
    );

    let mut blank_model = settings_request();
    blank_model.organiser = Some(OrganiserDefaults {
        planner_profile_name: "organiser".to_owned(),
        model: AgentModelPreference::Named(" ".to_owned()),
        effort: AgentEffort::Thorough,
    });
    assert!(matches!(
        service.save_project_agent_settings(blank_model),
        Err(ProjectAgentSettingsError::MissingRequiredField { field }) if field == "model"
    ));

    let mut null_model = settings_request();
    null_model.ticket_worker = Some(TicketWorkerDefaults {
        agent_profile_name: "ticket-worker".to_owned(),
        model: AgentModelPreference::Named("worker\0model".to_owned()),
        effort: AgentEffort::Balanced,
    });
    assert!(matches!(
        service.save_project_agent_settings(null_model),
        Err(ProjectAgentSettingsError::FieldContainsNull { field }) if field == "model"
    ));

    let mut long_model = settings_request();
    long_model.ticket_worker = Some(TicketWorkerDefaults {
        agent_profile_name: "ticket-worker".to_owned(),
        model: AgentModelPreference::Named("m".repeat(129)),
        effort: AgentEffort::Balanced,
    });
    assert!(matches!(
        service.save_project_agent_settings(long_model),
        Err(ProjectAgentSettingsError::ModelNameTooLong)
    ));
}

#[test]
fn assigns_the_project_ticket_worker_to_ai_proposed_tasks() {
    let mut service = service();
    create_board(&mut service);
    save_profiles(&mut service);
    service
        .save_project_agent_settings(settings_request())
        .expect("settings should save");

    let plan = service
        .propose_plan(ProposePlanRequest {
            plan_id: "plan-1".to_owned(),
            board_id: "board-1".to_owned(),
            proposed_by: "organiser".to_owned(),
            proposed_at: "2026-08-10T00:00:00Z".to_owned(),
            work_items: vec![ProposedPlanWorkItemRequest {
                work_item_id: "proposal-task".to_owned(),
                title: "Review the proposal".to_owned(),
                description: "Keep the worker assignment reviewable.".to_owned(),
                acceptance_criteria: vec!["The assignment is visible.".to_owned()],
                budget: Default::default(),
                requires_human_review: true,
                assigned_agent_profile_name: None,
                assigned_agent_model: None,
                assigned_agent_effort: None,
            }],
            dependencies: Vec::new(),
            unresolved_assumptions: Vec::new(),
        })
        .expect("plan should be proposed");

    assert_eq!(
        plan.preview.work_items[0]
            .assigned_agent_profile_name
            .as_deref(),
        Some("ticket-worker")
    );
}

#[test]
fn keeps_role_defaults_after_the_database_is_reopened() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("board.sqlite");
    let mut service = crate::application::BoardService::new(
        SqliteEventStore::open(&database_path).expect("store should open"),
    );
    create_board(&mut service);
    save_profiles(&mut service);
    let expected = service
        .save_project_agent_settings(settings_request())
        .expect("settings should save");
    drop(service);

    let reopened_service = crate::application::BoardService::new(
        SqliteEventStore::open(&database_path).expect("store should reopen"),
    );

    assert_eq!(
        reopened_service
            .project_agent_settings_for_board("board-1")
            .expect("settings should load after reopening"),
        Some(expected)
    );
}

fn save_profiles(
    service: &mut crate::application::BoardService<crate::persistence::SqliteEventStore>,
) {
    service
        .save_planner_profile(PlannerProfile {
            name: "organiser".to_owned(),
            program: "planner-bridge".to_owned(),
            arguments: vec!["--strict-json".to_owned()],
        })
        .expect("organiser profile should save");
    service
        .save_agent_profile(AgentProfile {
            name: "ticket-worker".to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "agent-worker".to_owned(),
            arguments: vec!["--jsonl".to_owned()],
        })
        .expect("ticket worker profile should save");
}

fn settings_request() -> SaveProjectAgentSettingsRequest {
    SaveProjectAgentSettingsRequest {
        board_id: "board-1".to_owned(),
        organiser: Some(OrganiserDefaults {
            planner_profile_name: "organiser".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::Thorough,
        }),
        ticket_worker: Some(TicketWorkerDefaults {
            agent_profile_name: "ticket-worker".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::Balanced,
        }),
    }
}
