use std::path::Path;

use tauri::State;

use crate::{
    application::{
        BoardPlan, GeneratePlanRequest, SaveProjectAgentSettingsRequest, generated_plan_request,
    },
    domain::ProjectAgentSettings,
    orchestration::{PlannerProfile, ProcessPlanGenerator},
};

use crate::desktop::{BoardDaemonState, error_message, lock_service};

#[tauri::command]
pub(crate) fn save_planner_profile(
    state: State<'_, BoardDaemonState>,
    profile: PlannerProfile,
) -> Result<PlannerProfile, String> {
    lock_service(&state)?
        .save_planner_profile(profile)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn planner_profiles(
    state: State<'_, BoardDaemonState>,
) -> Result<Vec<PlannerProfile>, String> {
    lock_service(&state)?
        .planner_profiles()
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn save_project_agent_settings(
    state: State<'_, BoardDaemonState>,
    request: SaveProjectAgentSettingsRequest,
) -> Result<ProjectAgentSettings, String> {
    lock_service(&state)?
        .save_project_agent_settings(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn project_agent_settings(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<Option<ProjectAgentSettings>, String> {
    lock_service(&state)?
        .project_agent_settings_for_board(&board_id)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn generate_plan(
    state: State<'_, BoardDaemonState>,
    request: GeneratePlanRequest,
) -> Result<BoardPlan, String> {
    let context = lock_service(&state)?
        .planner_context(&request.board_id, &request.planner_profile_name)
        .map_err(error_message)?;
    let draft = ProcessPlanGenerator::generate(
        &context.profile,
        Path::new(&context.repository_path),
        &request.goal,
    )
    .map_err(error_message)?;
    let proposal =
        generated_plan_request(&request, &context.profile.name, draft).map_err(error_message)?;
    lock_service(&state)?
        .propose_plan(proposal)
        .map_err(error_message)
}
