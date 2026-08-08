use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use tauri::{AppHandle, Manager, State};

use crate::{
    agent::AgentProfile,
    application::{
        AddDependencyRequest, BoardService, BoardSnapshot, CreateBoardRequest,
        CreateProjectRequest, CreateWorkItemRequest, RecordReviewCheckRequest,
        StartExecutionRequest, TransitionWorkItemRequest,
    },
    desktop_execution_runtime::ExecutionRuntime,
    domain::{BoardId, Project, WorkItemState},
    persistence::SqliteEventStore,
};

pub(crate) type LocalBoardService = BoardService<SqliteEventStore>;

pub(crate) struct BoardDaemonState {
    service: Arc<Mutex<LocalBoardService>>,
    runtime: ExecutionRuntime,
}

impl BoardDaemonState {
    fn open(data_directory: &Path) -> Result<Self, DesktopBootstrapError> {
        fs::create_dir_all(data_directory)?;
        let database_path = data_directory.join("kanban-ai-orchestrator.sqlite");
        let service = Arc::new(Mutex::new(LocalBoardService::new(SqliteEventStore::open(
            database_path,
        )?)));
        Ok(Self {
            runtime: ExecutionRuntime::new(service.clone(), data_directory.join("workspaces")),
            service,
        })
    }
}

pub(crate) fn open_daemon(
    app_handle: &AppHandle,
) -> Result<BoardDaemonState, DesktopBootstrapError> {
    BoardDaemonState::open(&local_data_directory(app_handle)?)
}

fn local_data_directory(app_handle: &AppHandle) -> Result<PathBuf, DesktopBootstrapError> {
    Ok(app_handle.path().app_data_dir()?.join("local-board"))
}

#[tauri::command]
pub(crate) fn create_project(
    state: State<'_, BoardDaemonState>,
    request: CreateProjectRequest,
) -> Result<Project, String> {
    lock_service(&state)?
        .create_project(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn create_board(
    state: State<'_, BoardDaemonState>,
    request: CreateBoardRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .create_board(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn create_work_item(
    state: State<'_, BoardDaemonState>,
    request: CreateWorkItemRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .create_work_item(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn add_dependency(
    state: State<'_, BoardDaemonState>,
    request: AddDependencyRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .add_dependency(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn transition_work_item(
    state: State<'_, BoardDaemonState>,
    request: TransitionWorkItemRequest,
) -> Result<BoardSnapshot, String> {
    ensure_user_owned_transition(request.next_state)?;
    lock_service(&state)?
        .transition_work_item(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn save_agent_profile(
    state: State<'_, BoardDaemonState>,
    profile: AgentProfile,
) -> Result<AgentProfile, String> {
    lock_service(&state)?
        .save_agent_profile(profile)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn agent_profiles(
    state: State<'_, BoardDaemonState>,
) -> Result<Vec<AgentProfile>, String> {
    lock_service(&state)?
        .agent_profiles()
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn start_execution(
    state: State<'_, BoardDaemonState>,
    request: StartExecutionRequest,
) -> Result<BoardSnapshot, String> {
    state.runtime.start(request).map_err(error_message)
}

#[tauri::command]
pub(crate) fn stop_execution(
    state: State<'_, BoardDaemonState>,
    execution_id: String,
) -> Result<BoardSnapshot, String> {
    state
        .runtime
        .request_stop(&execution_id)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn record_review_check(
    state: State<'_, BoardDaemonState>,
    request: RecordReviewCheckRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .record_review_check(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn board_snapshot(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .snapshot(&BoardId::from(board_id.as_str()))
        .map_err(error_message)
}

fn lock_service<'state, 'daemon>(
    state: &'state State<'daemon, BoardDaemonState>,
) -> Result<MutexGuard<'state, LocalBoardService>, String> {
    state
        .service
        .lock()
        .map_err(|_| "the local board daemon stopped unexpectedly".to_owned())
}

fn error_message(error: impl fmt::Display) -> String {
    error.to_string()
}

fn ensure_user_owned_transition(next_state: WorkItemState) -> Result<(), String> {
    if matches!(
        next_state,
        WorkItemState::Running | WorkItemState::AwaitingInput
    ) {
        Err(
            "running and awaiting-input states may only be entered by a live agent execution"
                .to_owned(),
        )
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum DesktopBootstrapError {
    AppDataDirectory(tauri::Error),
    CreateDataDirectory(std::io::Error),
    OpenEventStore(crate::persistence::EventStoreError),
}

impl fmt::Display for DesktopBootstrapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppDataDirectory(error) => {
                write!(formatter, "app data directory is unavailable: {error}")
            }
            Self::CreateDataDirectory(error) => write!(
                formatter,
                "local board data directory cannot be created: {error}"
            ),
            Self::OpenEventStore(error) => {
                write!(formatter, "local board database cannot be opened: {error}")
            }
        }
    }
}

impl Error for DesktopBootstrapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AppDataDirectory(error) => Some(error),
            Self::CreateDataDirectory(error) => Some(error),
            Self::OpenEventStore(error) => Some(error),
        }
    }
}

impl From<tauri::Error> for DesktopBootstrapError {
    fn from(error: tauri::Error) -> Self {
        Self::AppDataDirectory(error)
    }
}

impl From<std::io::Error> for DesktopBootstrapError {
    fn from(error: std::io::Error) -> Self {
        Self::CreateDataDirectory(error)
    }
}

impl From<crate::persistence::EventStoreError> for DesktopBootstrapError {
    fn from(error: crate::persistence::EventStoreError) -> Self {
        Self::OpenEventStore(error)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::domain::WorkItemState;

    use super::{BoardDaemonState, ensure_user_owned_transition};

    #[test]
    fn creates_a_dedicated_directory_for_local_board_state() {
        let temporary_directory = TempDir::new().expect("temporary directory should be created");
        let data_directory = temporary_directory.path().join("local-board");

        BoardDaemonState::open(&data_directory).expect("local board daemon should open");

        assert!(
            data_directory
                .join("kanban-ai-orchestrator.sqlite")
                .exists()
        );
    }

    #[test]
    fn keeps_agent_owned_states_out_of_the_general_transition_command() {
        assert!(ensure_user_owned_transition(WorkItemState::Review).is_ok());
        assert!(ensure_user_owned_transition(WorkItemState::Running).is_err());
        assert!(ensure_user_owned_transition(WorkItemState::AwaitingInput).is_err());
    }
}
