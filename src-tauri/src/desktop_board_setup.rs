use tauri::State;

use crate::{
    application::{BoardSnapshot, CreateLocalBoardRequest},
    desktop::{BoardDaemonState, error_message, lock_service},
    workspace::{
        GitHubRepositoryCloneRequest, RepositorySetup,
        clone_github_repository as clone_github_repository_in_workspace,
        inspect_project_repository, validate_project_repository,
    },
};

#[tauri::command]
pub(crate) fn inspect_repository(repository_path: String) -> Result<RepositorySetup, String> {
    inspect_project_repository(repository_path).map_err(error_message)
}

#[tauri::command]
pub(crate) fn clone_github_repository(
    request: GitHubRepositoryCloneRequest,
) -> Result<RepositorySetup, String> {
    clone_github_repository_in_workspace(request).map_err(error_message)
}

#[tauri::command]
pub(crate) fn create_local_board(
    state: State<'_, BoardDaemonState>,
    request: CreateLocalBoardRequest,
) -> Result<BoardSnapshot, String> {
    let repository =
        validate_project_repository(&request.repository_path, request.base_ref.as_deref())
            .map_err(error_message)?;
    let request = CreateLocalBoardRequest {
        repository_path: repository.repository_path,
        base_ref: Some(repository.base_ref),
        ..request
    };

    lock_service(&state)?
        .create_local_board(request)
        .map_err(error_message)
}
