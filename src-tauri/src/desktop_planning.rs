use std::path::Path;

use tauri::State;

use crate::{
    agent::NormalizedAgentEventKind,
    application::{
        BoardPlan, GeneratePlanRequest, PlannerContext, SaveProjectAgentSettingsRequest,
        generated_plan_request,
    },
    domain::ProjectAgentSettings,
    orchestration::{PlannerActivitySink, PlannerProfile, ProcessPlanGenerator},
};

use crate::desktop::{BoardDaemonState, error_message, lock_service};
use crate::desktop_planning_activity::{activate, complete};

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
    let _planning_guard = state
        .planning_gate
        .lock()
        .map_err(|_| "the planning service stopped unexpectedly".to_owned())?;
    let activity_sink = activate(state.inner(), &request.board_id)?;
    let result = (|| {
        let context = lock_service(&state)?
            .planner_context(&request.board_id, &request.planner_profile_name)
            .map_err(error_message)?;
        generate_and_propose(&state, &request, &context, activity_sink.clone())
    })();
    activity_sink(match &result {
        Ok(_) => NormalizedAgentEventKind::Completed {
            summary: "Your ticket proposal is ready to review.".to_owned(),
        },
        Err(_) => NormalizedAgentEventKind::Failed {
            reason: "Kanban could not save a reviewable ticket proposal.".to_owned(),
        },
    });
    complete(state.inner(), &request.board_id);
    result
}

fn generate_and_propose(
    state: &State<'_, BoardDaemonState>,
    request: &GeneratePlanRequest,
    context: &PlannerContext,
    activity_sink: PlannerActivitySink,
) -> Result<BoardPlan, String> {
    let draft = ProcessPlanGenerator::generate_with_preferences_and_activity(
        &context.profile,
        Path::new(&context.repository_path),
        &request.goal,
        &context.model,
        context.effort,
        activity_sink,
    )
    .map_err(error_message)?;
    let proposal =
        generated_plan_request(request, &context.profile.name, draft).map_err(error_message)?;
    lock_service(state)?
        .propose_plan(proposal)
        .map_err(error_message)
}
