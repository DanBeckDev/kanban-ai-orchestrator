use tauri::{AppHandle, Manager};

use crate::desktop::{BoardDaemonState, DesktopBootstrapError};

pub(crate) fn open_daemon(
    app_handle: &AppHandle,
) -> Result<BoardDaemonState, DesktopBootstrapError> {
    let data_directory = app_handle.path().app_data_dir()?.join("local-board");
    BoardDaemonState::open(&data_directory)
}
