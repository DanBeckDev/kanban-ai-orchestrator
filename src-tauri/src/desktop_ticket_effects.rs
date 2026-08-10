use tauri::State;

use crate::{
    application::{BoardSnapshot, ResolveTicketEffectRequest, TicketEffectPromptRequest},
    desktop::{BoardDaemonState, error_message},
    domain::TicketEffect,
};

#[tauri::command]
pub(crate) fn request_ticket_effect(
    state: State<'_, BoardDaemonState>,
    request: TicketEffectPromptRequest,
) -> Result<TicketEffect, String> {
    state
        .runtime
        .request_ticket_effect(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn resolve_ticket_effect(
    state: State<'_, BoardDaemonState>,
    request: ResolveTicketEffectRequest,
) -> Result<BoardSnapshot, String> {
    state
        .runtime
        .resolve_ticket_effect(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn ticket_effects(
    state: State<'_, BoardDaemonState>,
    work_item_id: String,
) -> Result<Vec<TicketEffect>, String> {
    state
        .runtime
        .ticket_effects_for_work_item(&work_item_id)
        .map_err(error_message)
}
