use std::{error::Error, fmt};

use serde::Serialize;

use crate::domain::AgentEffort;

use super::AgentProfileKind;

/// A model supplied by an installed agent runtime.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub id: String,
    pub label: String,
    pub efforts: Vec<AgentEffort>,
}

/// Whether the installed runtime can safely supply a model list.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderModelCatalogStatus {
    Ready,
    UsesProviderDefault,
    Unavailable,
}

/// A provider-owned model catalogue exposed through the desktop command boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelCatalog {
    pub provider_kind: AgentProfileKind,
    pub status: ProviderModelCatalogStatus,
    pub models: Vec<ProviderModel>,
}

impl ProviderModelCatalog {
    pub(crate) fn ready(provider_kind: AgentProfileKind, models: Vec<ProviderModel>) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::Ready,
            models,
        }
    }

    pub(crate) fn uses_provider_default(provider_kind: AgentProfileKind) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::UsesProviderDefault,
            models: Vec::new(),
        }
    }

    pub(crate) fn unavailable(provider_kind: AgentProfileKind) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::Unavailable,
            models: Vec::new(),
        }
    }
}

/// Reads safe model metadata from the agent that is already installed locally.
pub trait ProviderModelCatalogClient {
    fn list_models(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<Option<Vec<ProviderModel>>, ProviderModelCatalogError>;
}

pub struct ProviderModelCatalogService<Client> {
    client: Client,
}

impl<Client> ProviderModelCatalogService<Client>
where
    Client: ProviderModelCatalogClient,
{
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn catalog(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<ProviderModelCatalog, ProviderModelCatalogError> {
        validate_native_provider(provider_kind)?;
        match self.client.list_models(provider_kind) {
            Ok(Some(models)) => Ok(ProviderModelCatalog::ready(provider_kind, models)),
            Ok(None) => Ok(ProviderModelCatalog::uses_provider_default(provider_kind)),
            Err(ProviderModelCatalogError::RuntimeUnavailable) => {
                Ok(ProviderModelCatalog::unavailable(provider_kind))
            }
            Err(error) => Err(error),
        }
    }
}

fn validate_native_provider(
    provider_kind: AgentProfileKind,
) -> Result<(), ProviderModelCatalogError> {
    if provider_kind == AgentProfileKind::StructuredProcess {
        Err(ProviderModelCatalogError::UnsupportedProvider)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderModelCatalogError {
    RuntimeUnavailable,
    UnsupportedProvider,
}

impl fmt::Display for ProviderModelCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeUnavailable => formatter.write_str(
                "Kanban could not read model choices from this installed AI. Sign in or update it, then try again",
            ),
            Self::UnsupportedProvider => {
                formatter.write_str("this provider cannot load an installed model catalogue")
            }
        }
    }
}

impl Error for ProviderModelCatalogError {}

#[cfg(test)]
mod tests {
    use super::{
        AgentEffort, AgentProfileKind, ProviderModel, ProviderModelCatalogClient,
        ProviderModelCatalogError, ProviderModelCatalogService, ProviderModelCatalogStatus,
    };

    struct FakeClient {
        response: Result<Option<Vec<ProviderModel>>, ProviderModelCatalogError>,
    }

    impl ProviderModelCatalogClient for FakeClient {
        fn list_models(
            &self,
            _: AgentProfileKind,
        ) -> Result<Option<Vec<ProviderModel>>, ProviderModelCatalogError> {
            self.response.clone()
        }
    }

    #[test]
    fn returns_models_from_the_installed_runtime() {
        let service = ProviderModelCatalogService::new(FakeClient {
            response: Ok(Some(vec![ProviderModel {
                id: "gpt-5.6".to_owned(),
                label: "GPT-5.6".to_owned(),
                efforts: vec![AgentEffort::Balanced],
            }])),
        });

        let catalog = service
            .catalog(AgentProfileKind::CodexCli)
            .expect("catalog should load");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Ready);
        assert_eq!(catalog.models.len(), 1);
    }

    #[test]
    fn preserves_provider_default_when_a_runtime_has_no_catalogue_protocol() {
        let service = ProviderModelCatalogService::new(FakeClient { response: Ok(None) });

        let catalog = service
            .catalog(AgentProfileKind::ClaudeCode)
            .expect("fallback should load");

        assert_eq!(
            catalog.status,
            ProviderModelCatalogStatus::UsesProviderDefault
        );
        assert!(catalog.models.is_empty());
    }

    #[test]
    fn reports_unavailable_without_requesting_credentials() {
        let service = ProviderModelCatalogService::new(FakeClient {
            response: Err(ProviderModelCatalogError::RuntimeUnavailable),
        });

        let catalog = service
            .catalog(AgentProfileKind::CodexCli)
            .expect("unavailable state should be recoverable");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Unavailable);
    }

    #[test]
    fn rejects_the_generic_bridge() {
        let service = ProviderModelCatalogService::new(FakeClient { response: Ok(None) });

        assert_eq!(
            service.catalog(AgentProfileKind::StructuredProcess),
            Err(ProviderModelCatalogError::UnsupportedProvider)
        );
    }
}
