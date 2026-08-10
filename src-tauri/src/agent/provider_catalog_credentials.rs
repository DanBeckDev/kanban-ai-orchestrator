use keyring::{Entry, Error as KeyringError};

use super::{
    AgentProfileKind,
    provider_catalog::{ProviderModelCatalogCredentialStore, ProviderModelCatalogError},
};

const SERVICE: &str = "Kanban AI Orchestrator";

pub(crate) struct KeyringProviderCatalogCredentialStore;

impl ProviderModelCatalogCredentialStore for KeyringProviderCatalogCredentialStore {
    fn load(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<Option<String>, ProviderModelCatalogError> {
        match entry(provider_kind)?.get_password() {
            Ok(api_key) => Ok(Some(api_key)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err(ProviderModelCatalogError::CredentialStore),
        }
    }

    fn save(
        &self,
        provider_kind: AgentProfileKind,
        api_key: &str,
    ) -> Result<(), ProviderModelCatalogError> {
        entry(provider_kind)?
            .set_password(api_key)
            .map_err(|_| ProviderModelCatalogError::CredentialStore)
    }
}

fn entry(provider_kind: AgentProfileKind) -> Result<Entry, ProviderModelCatalogError> {
    Entry::new(SERVICE, &account(provider_kind))
        .map_err(|_| ProviderModelCatalogError::CredentialStore)
}

fn account(provider_kind: AgentProfileKind) -> String {
    let provider = match provider_kind {
        AgentProfileKind::CodexCli => "codex",
        AgentProfileKind::ClaudeCode => "claude-code",
        AgentProfileKind::ClinePassCli => "cline",
        AgentProfileKind::StructuredProcess => "unsupported",
    };
    format!("provider-model-catalog/{provider}")
}

#[cfg(test)]
mod tests {
    use super::account;
    use crate::agent::AgentProfileKind;

    #[test]
    fn gives_each_provider_a_separate_keychain_account() {
        assert_eq!(
            account(AgentProfileKind::CodexCli),
            "provider-model-catalog/codex"
        );
        assert_eq!(
            account(AgentProfileKind::ClaudeCode),
            "provider-model-catalog/claude-code"
        );
        assert_eq!(
            account(AgentProfileKind::ClinePassCli),
            "provider-model-catalog/cline"
        );
    }
}
