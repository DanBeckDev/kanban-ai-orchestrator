use tauri::State;

use crate::{
    application::{BoardSnapshot, ConfigureBoardSupervisionRequest},
    desktop::{BoardDaemonState, error_message, lock_service},
    domain::{BoardSupervision, BoardSupervisionMode, SupervisionDecision},
};

const LOCAL_ACTOR_FALLBACK: &str = "local-user";

#[tauri::command]
pub(crate) fn configure_board_supervision(
    state: State<'_, BoardDaemonState>,
    board_id: String,
    mode: BoardSupervisionMode,
) -> Result<BoardSupervision, String> {
    lock_service(&state)?
        .configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id,
            mode,
            configured_by: local_actor(),
            configured_at: chrono::Utc::now().to_rfc3339(),
        })
        .map_err(error_message)
}

fn local_actor() -> String {
    actor_from(
        ["USER", "USERNAME"]
            .into_iter()
            .filter_map(|name| std::env::var(name).ok()),
    )
}

fn actor_from(values: impl IntoIterator<Item = String>) -> String {
    values
        .into_iter()
        .find(|value| !value.trim().is_empty() && !value.contains('\0'))
        .unwrap_or_else(|| LOCAL_ACTOR_FALLBACK.to_owned())
}

#[cfg(test)]
mod tests {
    use super::actor_from;

    #[test]
    fn records_an_available_local_user_name_and_keeps_a_safe_fallback() {
        assert_eq!(
            actor_from([" ".to_owned(), "Alex".to_owned()]),
            "Alex".to_owned()
        );
        assert_eq!(actor_from(["\0".to_owned()]), "local-user".to_owned());
    }
}

#[tauri::command]
pub(crate) fn board_supervision(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<Option<BoardSupervision>, String> {
    lock_service(&state)?
        .board_supervision(&board_id)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn supervision_decisions(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<Vec<SupervisionDecision>, String> {
    lock_service(&state)?
        .supervision_decisions(&crate::domain::BoardId::from(board_id.as_str()))
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn coordinate_board(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<BoardSnapshot, String> {
    state
        .runtime
        .coordinate_board(&board_id)
        .map_err(error_message)
}
