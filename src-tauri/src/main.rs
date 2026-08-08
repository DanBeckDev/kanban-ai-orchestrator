#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() {
    kanban_ai_orchestrator_lib::run();
}
