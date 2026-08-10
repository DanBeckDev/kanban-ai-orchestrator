mod bounded_output;
mod claude_code;
mod cline;
mod codex;

use super::{
    AgentProfileKind, ProviderModel, ProviderModelCatalogClient, ProviderModelCatalogError,
};

/// Uses an installed agent's own local protocol instead of a second API account.
pub(crate) struct InstalledProviderRuntimeClient;

impl ProviderModelCatalogClient for InstalledProviderRuntimeClient {
    fn list_models(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<Option<Vec<ProviderModel>>, ProviderModelCatalogError> {
        match provider_kind {
            AgentProfileKind::CodexCli => codex::models().map(Some),
            AgentProfileKind::ClaudeCode => claude_code::models().map(Some),
            AgentProfileKind::ClinePassCli => cline::models().map(Some),
            AgentProfileKind::StructuredProcess => {
                Err(ProviderModelCatalogError::UnsupportedProvider)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentProfileKind, InstalledProviderRuntimeClient, ProviderModelCatalogClient};

    #[test]
    fn rejects_the_structured_process_bridge() {
        let client = InstalledProviderRuntimeClient;

        assert_eq!(
            client.list_models(AgentProfileKind::StructuredProcess),
            Err(super::ProviderModelCatalogError::UnsupportedProvider)
        );
    }
}
