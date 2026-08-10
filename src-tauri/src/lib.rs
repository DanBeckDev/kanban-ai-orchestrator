#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use tauri::Manager;

pub mod agent;
pub mod application;
pub mod domain;
pub mod linear;
pub mod orchestration;
pub mod persistence;
pub mod policy;
pub mod workspace;

mod desktop;
mod desktop_board_setup;
mod desktop_daemon_lock;
mod desktop_execution_activity;
mod desktop_execution_policy;
mod desktop_execution_runtime;
mod desktop_execution_runtime_control;
mod desktop_execution_runtime_events;
mod desktop_execution_runtime_review;
mod desktop_execution_runtime_support;
mod desktop_planning;
mod foundation;

pub use foundation::FoundationSummary;

#[tauri::command]
fn foundation_summary() -> FoundationSummary {
    FoundationSummary::new()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let daemon = desktop::open_daemon(app.handle())
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            app.manage(daemon);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            foundation_summary,
            desktop::create_project,
            desktop::create_board,
            desktop_board_setup::inspect_repository,
            desktop_board_setup::clone_github_repository,
            desktop_board_setup::create_local_board,
            desktop::board_library,
            desktop::open_board,
            desktop::create_work_item,
            desktop::add_dependency,
            desktop::propose_plan,
            desktop::board_plan,
            desktop::confirm_plan,
            desktop::transition_work_item,
            desktop::save_agent_profile,
            desktop::agent_profiles,
            desktop::agent_provider_availability,
            desktop_planning::save_planner_profile,
            desktop_planning::planner_profiles,
            desktop_planning::save_project_agent_settings,
            desktop_planning::project_agent_settings,
            desktop_planning::generate_plan,
            desktop::start_execution,
            desktop::coordination::configure_board_supervision,
            desktop::coordination::board_supervision,
            desktop::coordination::supervision_decisions,
            desktop::coordination::coordinate_board,
            desktop::ticket_effects::request_ticket_effect,
            desktop::ticket_effects::resolve_ticket_effect,
            desktop::ticket_effects::ticket_effects,
            desktop::stop_execution,
            desktop::execution_activity,
            desktop::record_review_check,
            desktop::record_review_decision,
            desktop::record_clean_code_review,
            desktop::begin_linear_oauth,
            desktop::linear_sync::begin_linear_comment_access,
            desktop::linear_connection_status,
            desktop::linear_assigned_issues,
            desktop::import_linear_issue,
            desktop::import_linear_blocker,
            desktop::linear_sync::queue_linear_comment,
            desktop::linear_sync::observe_linear_shared_field,
            desktop::linear_sync::sync_linear_shared_fields,
            desktop::linear_sync::deliver_linear_comment,
            desktop::board_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("the Tauri application should start");
}

#[cfg(test)]
mod tests {
    use super::{FoundationSummary, foundation_summary};

    #[test]
    fn foundation_command_returns_the_expected_summary() {
        assert_eq!(foundation_summary(), FoundationSummary::new());
    }
}

#[cfg(test)]
mod desktop_execution_activity_tests;

#[cfg(test)]
mod desktop_execution_runtime_tests;
