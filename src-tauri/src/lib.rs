#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod domain;
pub mod persistence;

mod foundation;

pub use foundation::FoundationSummary;

#[tauri::command]
fn foundation_summary() -> FoundationSummary {
    FoundationSummary::new()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![foundation_summary])
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
