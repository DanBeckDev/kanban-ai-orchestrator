#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use tauri::Manager;

pub mod agent;
pub mod application;
pub mod domain;
pub mod orchestration;
pub mod persistence;
pub mod policy;
pub mod workspace;

mod desktop;
mod foundation;

pub use foundation::FoundationSummary;

#[tauri::command]
fn foundation_summary() -> FoundationSummary {
    FoundationSummary::new()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run() {
    tauri::Builder::default()
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
            desktop::create_work_item,
            desktop::add_dependency,
            desktop::transition_work_item,
            desktop::record_execution,
            desktop::record_evidence,
            desktop::update_execution,
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
