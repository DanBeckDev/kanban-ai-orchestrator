use tauri::State;

use crate::agent::{
    AgentProfileKind, AgentProviderAvailability, KeyringProviderCatalogCredentialStore,
    ProviderModelCatalog, ProviderModelCatalogService, ReqwestProviderModelCatalogClient,
    SaveProviderCatalogCredentialRequest, discover_native_agent_providers,
};
use crate::desktop::{BoardDaemonState, error_message};

pub(crate) type LocalProviderModelCatalog = ProviderModelCatalogService<
    KeyringProviderCatalogCredentialStore,
    ReqwestProviderModelCatalogClient,
>;

pub(crate) fn provider_model_catalog_service() -> LocalProviderModelCatalog {
    ProviderModelCatalogService::new(
        KeyringProviderCatalogCredentialStore,
        ReqwestProviderModelCatalogClient::new(),
    )
}

#[tauri::command]
pub(crate) fn agent_provider_availability() -> Vec<AgentProviderAvailability> {
    discover_native_agent_providers()
}

#[tauri::command]
pub(crate) fn provider_model_catalog(
    state: State<'_, BoardDaemonState>,
    provider_kind: AgentProfileKind,
) -> Result<ProviderModelCatalog, String> {
    state
        .provider_model_catalog
        .catalog(provider_kind)
        .map_err(error_message)
}

#[tauri::command]
pub(crate) fn save_provider_catalog_credential(
    state: State<'_, BoardDaemonState>,
    request: SaveProviderCatalogCredentialRequest,
) -> Result<ProviderModelCatalog, String> {
    state
        .provider_model_catalog
        .save_credential_and_catalog(request)
        .map_err(error_message)
}
