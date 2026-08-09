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
        AddDependencyRequest, BoardLibraryEntry, BoardPlan, BoardService, BoardSnapshot,
        ConfirmPlanRequest, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
        ImportLinearBlockerRequest, ImportLinearIssueRequest, ProposePlanRequest,
        RecordCleanCodeReviewRequest, RecordReviewCheckRequest, RecordReviewDecisionRequest,
        StartExecutionRequest, TransitionWorkItemRequest,
    },
    desktop_daemon_lock::DaemonLock,
    desktop_execution_runtime::ExecutionRuntime,
    domain::{BoardId, Project, WorkItemState},
    linear::{
        KeyringCredentialStore, LinearConnectionStatus, LinearIssueReader, LinearIssueSummary,
        LinearOAuthConfiguration, LinearOAuthService, ReqwestLinearCommentPublisher,
        ReqwestLinearGraphQlTransport, ReqwestLinearTokenClient, resolve_request_credentials,
    },
    persistence::SqliteEventStore,
};

pub(crate) type LocalBoardService = BoardService<SqliteEventStore>;
pub(crate) type LocalLinearOAuthService = LinearOAuthService<KeyringCredentialStore>;
type LocalLinearCommentPublisher = ReqwestLinearCommentPublisher;
type LocalLinearIssueReader = LinearIssueReader<ReqwestLinearGraphQlTransport>;

pub(crate) struct BoardDaemonState {
    _daemon_lock: DaemonLock,
    linear_comment_publisher: LocalLinearCommentPublisher,
    linear_issue_reader: LocalLinearIssueReader,
    linear_oauth: Arc<Mutex<LocalLinearOAuthService>>,
    linear_token_client: Arc<ReqwestLinearTokenClient>,
    service: Arc<Mutex<LocalBoardService>>,
    runtime: ExecutionRuntime,
}

impl BoardDaemonState {
    fn open(data_directory: &Path) -> Result<Self, DesktopBootstrapError> {
        fs::create_dir_all(data_directory)?;
        let daemon_lock = DaemonLock::acquire(data_directory)?;
        let database_path = data_directory.join("kanban-ai-orchestrator.sqlite");
        let mut store = SqliteEventStore::open(database_path)?;
        store.recover_connector_outbox_deliveries()?;
        let service = Arc::new(Mutex::new(LocalBoardService::new(store)));
        Ok(Self {
            _daemon_lock: daemon_lock,
            linear_comment_publisher: ReqwestLinearCommentPublisher::new(),
            linear_issue_reader: LocalLinearIssueReader::new(ReqwestLinearGraphQlTransport::new()),
            linear_oauth: Arc::new(Mutex::new(LinearOAuthService::new(KeyringCredentialStore))),
            linear_token_client: Arc::new(ReqwestLinearTokenClient::new()),
            runtime: ExecutionRuntime::new(service.clone(), data_directory.join("workspaces")),
            service,
        })
    }
}

#[path = "desktop_linear_sync.rs"]
pub(crate) mod linear_sync;

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
pub(crate) fn board_library(
    state: State<'_, BoardDaemonState>,
) -> Result<Vec<BoardLibraryEntry>, String> {
    lock_service(&state)?.board_library().map_err(error_message)
}

#[tauri::command]
pub(crate) fn open_board(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .open_board(&board_id)
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
pub(crate) fn propose_plan(
    state: State<'_, BoardDaemonState>,
    request: ProposePlanRequest,
) -> Result<BoardPlan, String> {
    lock_service(&state)?
        .propose_plan(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn board_plan(
    state: State<'_, BoardDaemonState>,
    board_id: String,
) -> Result<Option<BoardPlan>, String> {
    lock_service(&state)?
        .board_plan(&board_id)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn confirm_plan(
    state: State<'_, BoardDaemonState>,
    request: ConfirmPlanRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .confirm_plan(request)
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
pub(crate) fn execution_activity(
    state: State<'_, BoardDaemonState>,
    execution_id: String,
    after_sequence: Option<u64>,
) -> Result<crate::desktop_execution_activity::ExecutionActivityPage, String> {
    state
        .runtime
        .activity_page(&execution_id, after_sequence)
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
pub(crate) fn record_review_decision(
    state: State<'_, BoardDaemonState>,
    request: RecordReviewDecisionRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .record_review_decision(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn record_clean_code_review(
    state: State<'_, BoardDaemonState>,
    request: RecordCleanCodeReviewRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .record_clean_code_review(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn begin_linear_oauth(
    app_handle: AppHandle,
    state: State<'_, BoardDaemonState>,
    configuration: LinearOAuthConfiguration,
) -> Result<LinearConnectionStatus, String> {
    linear_sync::begin_linear_authorization(app_handle, state, configuration.clone(), |oauth| {
        oauth.begin(configuration)
    })
}

#[tauri::command]
pub(crate) fn linear_connection_status(
    state: State<'_, BoardDaemonState>,
) -> Result<LinearConnectionStatus, String> {
    lock_linear_oauth(&state)?
        .connection_status()
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn linear_assigned_issues(
    state: State<'_, BoardDaemonState>,
) -> Result<Vec<LinearIssueSummary>, String> {
    let access_token = linear_access_token(&state)?;
    state
        .linear_issue_reader
        .assigned_issues(&access_token)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn import_linear_issue(
    state: State<'_, BoardDaemonState>,
    request: ImportLinearIssueRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .import_linear_issue(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn import_linear_blocker(
    state: State<'_, BoardDaemonState>,
    request: ImportLinearBlockerRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .import_linear_blocker(request)
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

pub(crate) fn lock_service<'state, 'daemon>(
    state: &'state State<'daemon, BoardDaemonState>,
) -> Result<MutexGuard<'state, LocalBoardService>, String> {
    state
        .service
        .lock()
        .map_err(|_| "the local board daemon stopped unexpectedly".to_owned())
}

fn lock_linear_oauth<'state, 'daemon>(
    state: &'state State<'daemon, BoardDaemonState>,
) -> Result<MutexGuard<'state, LocalLinearOAuthService>, String> {
    lock_linear_oauth_state(state.inner())
}

fn linear_access_token(state: &BoardDaemonState) -> Result<String, String> {
    let request_credentials = lock_linear_oauth_state(state)?
        .credentials_for_request()
        .map_err(error_message)?;
    let (access_token, refreshed_credentials) =
        resolve_request_credentials(request_credentials, state.linear_token_client.as_ref())
            .map_err(error_message)?;
    if let Some(refreshed_credentials) = refreshed_credentials {
        lock_linear_oauth_state(state)?
            .record_credentials(refreshed_credentials)
            .map_err(error_message)?;
    }
    Ok(access_token)
}

fn lock_linear_oauth_state(
    state: &BoardDaemonState,
) -> Result<MutexGuard<'_, LocalLinearOAuthService>, String> {
    state
        .linear_oauth
        .lock()
        .map_err(|_| "the local Linear connector stopped unexpectedly".to_owned())
}

pub(crate) fn error_message(error: impl fmt::Display) -> String {
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
    LockDaemon(crate::desktop_daemon_lock::DaemonLockError),
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
            Self::LockDaemon(error) => {
                write!(formatter, "local board daemon cannot start: {error}")
            }
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
            Self::LockDaemon(error) => Some(error),
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

impl From<crate::desktop_daemon_lock::DaemonLockError> for DesktopBootstrapError {
    fn from(error: crate::desktop_daemon_lock::DaemonLockError) -> Self {
        Self::LockDaemon(error)
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

    use super::{BoardDaemonState, DesktopBootstrapError, ensure_user_owned_transition};

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
    fn refuses_a_second_daemon_for_the_same_local_data_directory() {
        let temporary_directory = TempDir::new().expect("temporary directory should be created");
        let data_directory = temporary_directory.path().join("local-board");
        let daemon =
            BoardDaemonState::open(&data_directory).expect("first local board daemon should open");

        assert!(matches!(
            BoardDaemonState::open(&data_directory),
            Err(DesktopBootstrapError::LockDaemon(_))
        ));

        drop(daemon);
    }

    #[test]
    fn keeps_agent_owned_states_out_of_the_general_transition_command() {
        assert!(ensure_user_owned_transition(WorkItemState::Review).is_ok());
        assert!(ensure_user_owned_transition(WorkItemState::Running).is_err());
        assert!(ensure_user_owned_transition(WorkItemState::AwaitingInput).is_err());
    }
}
