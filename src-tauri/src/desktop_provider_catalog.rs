use crate::agent::{
    AgentProfileKind, AgentProviderAvailability, InstalledProviderRuntimeClient,
    ProviderModelCatalog, ProviderModelCatalogService, discover_native_agent_providers,
};
use crate::desktop::error_message;

pub(crate) type LocalProviderModelCatalog =
    ProviderModelCatalogService<InstalledProviderRuntimeClient>;

pub(crate) fn provider_model_catalog_service() -> LocalProviderModelCatalog {
    ProviderModelCatalogService::new(InstalledProviderRuntimeClient)
}

#[tauri::command]
pub(crate) fn agent_provider_availability() -> Vec<AgentProviderAvailability> {
    discover_native_agent_providers()
}

#[tauri::command]
pub(crate) fn provider_model_catalog(
    provider_kind: AgentProfileKind,
) -> Result<ProviderModelCatalog, String> {
    provider_model_catalog_service()
        .catalog(provider_kind)
        .map_err(error_message)
}
