use tauri::State;

use crate::{
    application::BoardSnapshot,
    desktop::{BoardDaemonState, error_message},
};

#[tauri::command]
pub(crate) fn coordinate_board(
    state: State<'_, BoardDaemonState>,
    board_id: String,
    agent_profile_name: String,
) -> Result<BoardSnapshot, String> {
    state
        .runtime
        .coordinate_board(&board_id, &agent_profile_name)
        .map_err(error_message)
}
