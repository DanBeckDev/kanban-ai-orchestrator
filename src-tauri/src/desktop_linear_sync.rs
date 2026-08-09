use chrono::Utc;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    application::{BoardSnapshot, ObserveLinearSharedFieldRequest, QueueLinearCommentRequest},
    domain::ConnectorSharedField,
    linear::{
        LinearCommentPublisher, LinearConnectionStatus, LinearOAuthConfiguration, LinearOAuthError,
        LinearTokenClient, await_loopback_callback, bind_loopback_callback,
    },
};

use super::{
    BoardDaemonState, LocalBoardService, error_message, linear_access_token, lock_linear_oauth,
    lock_linear_oauth_state, lock_service,
};

#[tauri::command]
pub(crate) fn queue_linear_comment(
    state: State<'_, BoardDaemonState>,
    request: QueueLinearCommentRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .queue_linear_comment(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn observe_linear_shared_field(
    state: State<'_, BoardDaemonState>,
    request: ObserveLinearSharedFieldRequest,
) -> Result<BoardSnapshot, String> {
    lock_service(&state)?
        .observe_linear_shared_field(request)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn sync_linear_shared_fields(
    state: State<'_, BoardDaemonState>,
    external_link_id: String,
) -> Result<BoardSnapshot, String> {
    let access_token = linear_access_token(&state)?;
    let target = lock_service(&state)?
        .linear_issue_sync_target(&external_link_id)
        .map_err(error_message)?;
    let remote = state
        .linear_issue_reader
        .issue_shared_fields(&access_token, &target.issue_id)
        .map_err(error_message)?;
    let mut service = lock_service(&state)?;
    observe_remote_field(
        &mut service,
        &target.external_link_id,
        ConnectorSharedField::Title,
        &remote.title,
        &remote.remote_revision,
    )?;
    observe_remote_field(
        &mut service,
        &target.external_link_id,
        ConnectorSharedField::Description,
        &remote.description,
        &remote.remote_revision,
    )?;
    observe_remote_field(
        &mut service,
        &target.external_link_id,
        ConnectorSharedField::WorkflowState,
        &remote.workflow_state,
        &remote.remote_revision,
    )
}

#[tauri::command]
pub(crate) fn begin_linear_comment_access(
    app_handle: AppHandle,
    state: State<'_, BoardDaemonState>,
) -> Result<LinearConnectionStatus, String> {
    let configuration = lock_linear_oauth(&state)?
        .comment_access_configuration()
        .map_err(error_message)?;
    begin_linear_authorization(app_handle, state, configuration, |oauth| {
        oauth.begin_comment_access()
    })
}

pub(crate) fn begin_linear_authorization(
    app_handle: AppHandle,
    state: State<'_, BoardDaemonState>,
    configuration: LinearOAuthConfiguration,
    begin: impl FnOnce(&mut super::LocalLinearOAuthService) -> Result<String, LinearOAuthError>,
) -> Result<LinearConnectionStatus, String> {
    let listener = bind_loopback_callback(&configuration).map_err(error_message)?;
    let authorization_url = {
        let mut oauth = lock_linear_oauth(&state)?;
        begin(&mut oauth).map_err(error_message)?
    };
    if let Err(error) = app_handle
        .opener()
        .open_url(&authorization_url, None::<&str>)
    {
        let error = LinearOAuthError::BrowserLaunch(error.to_string());
        lock_linear_oauth(&state)?.record_failure(error.clone());
        return Err(error_message(error));
    }
    let oauth_service = state.linear_oauth.clone();
    let token_client = state.linear_token_client.clone();
    std::thread::spawn(move || {
        let outcome = await_loopback_callback(listener, &configuration).and_then(|callback| {
            let exchange = oauth_service
                .lock()
                .map_err(|_| LinearOAuthError::ConnectorUnavailable)?
                .authorization_code_exchange(&callback.code, &callback.state)?;
            let credentials = token_client.exchange_authorization_code(exchange)?;
            oauth_service
                .lock()
                .map_err(|_| LinearOAuthError::ConnectorUnavailable)?
                .record_credentials(credentials)
        });
        if let Err(error) = outcome
            && let Ok(mut oauth) = oauth_service.lock()
        {
            oauth.record_failure(error);
        }
    });
    lock_linear_oauth(&state)?
        .connection_status()
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn deliver_linear_comment(
    state: State<'_, BoardDaemonState>,
    outbox_item_id: String,
) -> Result<BoardSnapshot, String> {
    let access_token = linear_comment_access_token(&state)?;
    let delivery = lock_service(&state)?
        .claim_linear_comment_delivery(&outbox_item_id)
        .map_err(error_message)?;
    match state.linear_comment_publisher.publish_comment(
        &access_token,
        &delivery.issue_id,
        &delivery.body,
    ) {
        Ok(()) => lock_service(&state)?
            .mark_linear_comment_delivered(&outbox_item_id, Utc::now().to_rfc3339())
            .map_err(error_message),
        Err(error) => record_uncertain_delivery(&state, &outbox_item_id, error),
    }
}

fn linear_comment_access_token(state: &BoardDaemonState) -> Result<String, String> {
    if !lock_linear_oauth_state(state)?
        .comments_are_authorized()
        .map_err(error_message)?
    {
        return Err(error_message(LinearOAuthError::MissingCommentScope));
    }
    linear_access_token(state)
}

fn record_uncertain_delivery(
    state: &BoardDaemonState,
    outbox_item_id: &str,
    error: LinearOAuthError,
) -> Result<BoardSnapshot, String> {
    let connector_error = error_message(error);
    match lock_service_state(state)?
        .mark_linear_comment_delivery_uncertain(outbox_item_id)
        .map_err(error_message)
    {
        Ok(_) => Err(format!(
            "Linear did not confirm comment delivery: {connector_error}. The outbox item is marked delivery uncertain and will not retry automatically."
        )),
        Err(record_error) => Err(format!(
            "Linear did not confirm comment delivery: {connector_error}. The local delivery state could not be updated: {record_error}."
        )),
    }
}

fn reconciliation_item_id(
    external_link_id: &str,
    field: ConnectorSharedField,
    remote_revision: &str,
) -> String {
    let field = match field {
        ConnectorSharedField::Title => "title",
        ConnectorSharedField::Description => "description",
        ConnectorSharedField::WorkflowState => "workflow_state",
    };
    format!("linear-reconciliation:{external_link_id}:{field}:{remote_revision}")
}

fn observe_remote_field(
    service: &mut LocalBoardService,
    external_link_id: &str,
    field: ConnectorSharedField,
    remote_value: &str,
    remote_revision: &str,
) -> Result<BoardSnapshot, String> {
    service
        .observe_linear_shared_field(ObserveLinearSharedFieldRequest {
            reconciliation_item_id: reconciliation_item_id(
                external_link_id,
                field,
                remote_revision,
            ),
            external_link_id: external_link_id.to_owned(),
            field,
            remote_value: remote_value.to_owned(),
            remote_revision: remote_revision.to_owned(),
            observed_at: Utc::now().to_rfc3339(),
        })
        .map_err(error_message)
}

fn lock_service_state(
    state: &BoardDaemonState,
) -> Result<std::sync::MutexGuard<'_, LocalBoardService>, String> {
    state
        .service
        .lock()
        .map_err(|_| "the local board daemon stopped unexpectedly".to_owned())
}

#[cfg(test)]
mod tests {
    use super::reconciliation_item_id;
    use crate::domain::ConnectorSharedField;

    #[test]
    fn derives_a_stable_field_specific_reconciliation_identifier() {
        for (field, expected_field) in [
            (ConnectorSharedField::Title, "title"),
            (ConnectorSharedField::Description, "description"),
            (ConnectorSharedField::WorkflowState, "workflow_state"),
        ] {
            assert_eq!(
                reconciliation_item_id("linear-link-1", field, "revision-1"),
                format!("linear-reconciliation:linear-link-1:{expected_field}:revision-1")
            );
        }
    }
}
