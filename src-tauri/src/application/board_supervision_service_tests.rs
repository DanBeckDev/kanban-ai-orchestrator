use crate::{
    agent::{AgentProfile, AgentProfileKind},
    application::{
        BoardSupervisionServiceError, ConfigureBoardSupervisionRequest,
        SaveProjectAgentSettingsRequest,
        board_service_tests::{create_board, service},
    },
    domain::{
        AgentEffort, AgentModelPreference, BoardSupervisionMode, OrganiserDefaults,
        TicketWorkerDefaults,
    },
    orchestration::PlannerProfile,
};

#[test]
fn requires_configured_organiser_and_ticket_worker_before_automation_is_saved() {
    let mut service = service();
    create_board(&mut service);

    assert!(matches!(
        service.configure_board_supervision(request(BoardSupervisionMode::Autonomous)),
        Err(BoardSupervisionServiceError::OrganiserNotConfigured)
    ));

    save_organiser(&mut service);
    service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: Some(organiser()),
            ticket_worker: None,
        })
        .expect("organiser setting should save");

    assert!(matches!(
        service.configure_board_supervision(request(BoardSupervisionMode::Autonomous)),
        Err(BoardSupervisionServiceError::TicketWorkerNotConfigured)
    ));
}

#[test]
fn snapshots_role_defaults_and_records_a_named_pause_with_a_new_revision() {
    let mut service = service();
    create_board(&mut service);
    save_organiser(&mut service);
    save_worker(&mut service);
    service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: Some(organiser()),
            ticket_worker: Some(worker()),
        })
        .expect("role settings should save");

    let enabled = service
        .configure_board_supervision(request(BoardSupervisionMode::Autonomous))
        .expect("automation should save");
    let paused = service
        .configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: "board-1".to_owned(),
            mode: BoardSupervisionMode::Manual,
            configured_by: "Alex".to_owned(),
            configured_at: "2026-08-10T10:01:00Z".to_owned(),
        })
        .expect("pause should save");

    assert_eq!(enabled.revision, 1);
    assert_eq!(enabled.organiser, organiser());
    assert_eq!(enabled.ticket_worker, worker());
    assert_eq!(paused.revision, 2);
    assert_eq!(paused.paused_by.as_deref(), Some("Alex"));
    assert_eq!(paused.paused_at.as_deref(), Some("2026-08-10T10:01:00Z"));
    assert_eq!(
        service
            .board_supervision("board-1")
            .expect("supervision should load"),
        Some(paused)
    );
}

#[test]
fn rejects_invalid_configuration_identity_and_unknown_boards() {
    let mut service = service();

    assert!(matches!(
        service.configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: " ".to_owned(),
            mode: BoardSupervisionMode::Manual,
            configured_by: "Alex".to_owned(),
            configured_at: "2026-08-10T10:00:00Z".to_owned(),
        }),
        Err(BoardSupervisionServiceError::MissingRequiredField { field: "board id" })
    ));
    assert!(matches!(
        service.configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: "board-1".to_owned(),
            mode: BoardSupervisionMode::Manual,
            configured_by: "Alex\0".to_owned(),
            configured_at: "2026-08-10T10:00:00Z".to_owned(),
        }),
        Err(BoardSupervisionServiceError::FieldContainsNull {
            field: "configured by"
        })
    ));
    assert!(matches!(
        service.configure_board_supervision(request(BoardSupervisionMode::Manual)),
        Err(BoardSupervisionServiceError::BoardNotFound { .. })
    ));
    assert!(matches!(
        service.board_supervision("\0"),
        Err(BoardSupervisionServiceError::FieldContainsNull { field: "board id" })
    ));
}

#[test]
fn keeps_saved_role_defaults_when_settings_are_later_unavailable() {
    let mut service = service();
    create_board(&mut service);
    save_organiser(&mut service);
    save_worker(&mut service);
    service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: Some(organiser()),
            ticket_worker: Some(worker()),
        })
        .expect("role settings should save");
    let first = service
        .configure_board_supervision(request(BoardSupervisionMode::Autonomous))
        .expect("initial configuration should save");
    service
        .save_project_agent_settings(SaveProjectAgentSettingsRequest {
            board_id: "board-1".to_owned(),
            organiser: None,
            ticket_worker: None,
        })
        .expect("role settings should clear");

    let second = service
        .configure_board_supervision(request(BoardSupervisionMode::Manual))
        .expect("saved defaults should allow a safe pause");

    assert_eq!(second.organiser, first.organiser);
    assert_eq!(second.ticket_worker, first.ticket_worker);
    assert_eq!(second.revision, 2);
}

fn request(mode: BoardSupervisionMode) -> ConfigureBoardSupervisionRequest {
    ConfigureBoardSupervisionRequest {
        board_id: "board-1".to_owned(),
        mode,
        configured_by: "Alex".to_owned(),
        configured_at: "2026-08-10T10:00:00Z".to_owned(),
    }
}

fn save_organiser(
    service: &mut crate::application::BoardService<crate::persistence::SqliteEventStore>,
) {
    service
        .save_planner_profile(PlannerProfile {
            name: "organiser".to_owned(),
            kind: AgentProfileKind::CodexCli,
            program: "codex".to_owned(),
            arguments: Vec::new(),
        })
        .expect("organiser profile should save");
}

fn save_worker(
    service: &mut crate::application::BoardService<crate::persistence::SqliteEventStore>,
) {
    service
        .save_agent_profile(AgentProfile {
            name: "worker".to_owned(),
            kind: AgentProfileKind::CodexCli,
            program: "codex".to_owned(),
            arguments: Vec::new(),
        })
        .expect("worker profile should save");
}

fn organiser() -> OrganiserDefaults {
    OrganiserDefaults {
        planner_profile_name: "organiser".to_owned(),
        model: AgentModelPreference::Named("organiser-model".to_owned()),
        effort: AgentEffort::Thorough,
    }
}

fn worker() -> TicketWorkerDefaults {
    TicketWorkerDefaults {
        agent_profile_name: "worker".to_owned(),
        model: AgentModelPreference::Named("worker-model".to_owned()),
        effort: AgentEffort::Balanced,
    }
}
