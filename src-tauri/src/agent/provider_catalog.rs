use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::domain::AgentEffort;

use super::{AgentProfileKind, provider_catalog_response::catalog_from_responses};

/// A provider-returned model that the native adapter can offer in Settings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub id: String,
    pub label: String,
    pub efforts: Vec<AgentEffort>,
}

/// The safe state of a provider's account-specific model catalogue.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderModelCatalogStatus {
    Disconnected,
    Ready,
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
    fn disconnected(provider_kind: AgentProfileKind) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::Disconnected,
            models: Vec::new(),
        }
    }

    pub(crate) fn ready(provider_kind: AgentProfileKind, models: Vec<ProviderModel>) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::Ready,
            models,
        }
    }

    fn unavailable(provider_kind: AgentProfileKind) -> Self {
        Self {
            provider_kind,
            status: ProviderModelCatalogStatus::Unavailable,
            models: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderCatalogCredentialRequest {
    pub provider_kind: AgentProfileKind,
    pub api_key: String,
}

pub trait ProviderModelCatalogCredentialStore {
    fn load(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<Option<String>, ProviderModelCatalogError>;
    fn save(
        &self,
        provider_kind: AgentProfileKind,
        api_key: &str,
    ) -> Result<(), ProviderModelCatalogError>;
}

pub trait ProviderModelCatalogClient {
    fn list_models(
        &self,
        provider_kind: AgentProfileKind,
        api_key: &str,
    ) -> Result<Vec<String>, ProviderModelCatalogError>;
}

pub struct ProviderModelCatalogService<CredentialStore, Client> {
    credential_store: CredentialStore,
    client: Client,
}

impl<CredentialStore, Client> ProviderModelCatalogService<CredentialStore, Client>
where
    CredentialStore: ProviderModelCatalogCredentialStore,
    Client: ProviderModelCatalogClient,
{
    pub fn new(credential_store: CredentialStore, client: Client) -> Self {
        Self {
            credential_store,
            client,
        }
    }

    pub fn catalog(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<ProviderModelCatalog, ProviderModelCatalogError> {
        validate_native_provider(provider_kind)?;
        let Some(api_key) = self.credential_store.load(provider_kind)? else {
            return Ok(ProviderModelCatalog::disconnected(provider_kind));
        };
        self.catalog_with_key(provider_kind, &api_key)
    }

    pub fn save_credential_and_catalog(
        &self,
        request: SaveProviderCatalogCredentialRequest,
    ) -> Result<ProviderModelCatalog, ProviderModelCatalogError> {
        validate_native_provider(request.provider_kind)?;
        validate_api_key(&request.api_key)?;
        let catalog = self.catalog_with_key(request.provider_kind, &request.api_key)?;
        if catalog.status == ProviderModelCatalogStatus::Ready {
            self.credential_store
                .save(request.provider_kind, &request.api_key)?;
        }
        Ok(catalog)
    }

    fn catalog_with_key(
        &self,
        provider_kind: AgentProfileKind,
        api_key: &str,
    ) -> Result<ProviderModelCatalog, ProviderModelCatalogError> {
        match self.client.list_models(provider_kind, api_key) {
            Ok(responses) => Ok(catalog_from_responses(provider_kind, &responses)
                .unwrap_or_else(|_| ProviderModelCatalog::unavailable(provider_kind))),
            Err(ProviderModelCatalogError::RequestFailed) => {
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

fn validate_api_key(api_key: &str) -> Result<(), ProviderModelCatalogError> {
    if api_key.trim().is_empty() || api_key.contains('\0') || api_key.len() > 1024 {
        Err(ProviderModelCatalogError::InvalidCredential)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderModelCatalogError {
    CredentialStore,
    InvalidCredential,
    RequestFailed,
    UnsupportedProvider,
}

impl fmt::Display for ProviderModelCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CredentialStore => {
                formatter.write_str("Kanban could not access this device's secure credential store")
            }
            Self::InvalidCredential => {
                formatter.write_str("enter a valid provider API key, then try again")
            }
            Self::RequestFailed => formatter.write_str(
                "Kanban could not load provider models. Check the API key and connection, then try again",
            ),
            Self::UnsupportedProvider => {
                formatter.write_str("this provider cannot load an account model catalogue")
            }
        }
    }
}

impl Error for ProviderModelCatalogError {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{
        AgentProfileKind, ProviderModelCatalogClient, ProviderModelCatalogCredentialStore,
        ProviderModelCatalogError, ProviderModelCatalogService, ProviderModelCatalogStatus,
        SaveProviderCatalogCredentialRequest,
    };

    #[derive(Clone, Default)]
    struct FakeCredentialStore {
        keys: Rc<RefCell<Vec<(AgentProfileKind, String)>>>,
    }

    impl ProviderModelCatalogCredentialStore for FakeCredentialStore {
        fn load(
            &self,
            provider_kind: AgentProfileKind,
        ) -> Result<Option<String>, ProviderModelCatalogError> {
            Ok(self
                .keys
                .borrow()
                .iter()
                .find_map(|(kind, key)| (*kind == provider_kind).then(|| key.clone())))
        }

        fn save(
            &self,
            provider_kind: AgentProfileKind,
            api_key: &str,
        ) -> Result<(), ProviderModelCatalogError> {
            let mut keys = self.keys.borrow_mut();
            keys.retain(|(kind, _)| *kind != provider_kind);
            keys.push((provider_kind, api_key.to_owned()));
            Ok(())
        }
    }

    struct FakeClient {
        response: Result<Vec<String>, ProviderModelCatalogError>,
    }

    impl ProviderModelCatalogClient for FakeClient {
        fn list_models(
            &self,
            _: AgentProfileKind,
            _: &str,
        ) -> Result<Vec<String>, ProviderModelCatalogError> {
            self.response.clone()
        }
    }

    #[test]
    fn reports_disconnected_without_calling_a_provider() {
        let service = ProviderModelCatalogService::new(
            FakeCredentialStore::default(),
            FakeClient {
                response: Err(ProviderModelCatalogError::RequestFailed),
            },
        );

        let catalog = service
            .catalog(AgentProfileKind::CodexCli)
            .expect("catalog state should load");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Disconnected);
        assert!(catalog.models.is_empty());
    }

    #[test]
    fn saves_a_key_then_returns_provider_models_without_echoing_the_key() {
        let service = ProviderModelCatalogService::new(
            FakeCredentialStore::default(),
            FakeClient {
                response: Ok(vec![r#"{"data":[{"id":"gpt-5-codex"}]}"#.to_owned()]),
            },
        );

        let catalog = service
            .save_credential_and_catalog(SaveProviderCatalogCredentialRequest {
                provider_kind: AgentProfileKind::CodexCli,
                api_key: "secret-key".to_owned(),
            })
            .expect("catalog should load");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Ready);
        assert_eq!(catalog.models[0].id, "gpt-5-codex");
        assert!(!format!("{catalog:?}").contains("secret-key"));
    }

    #[test]
    fn turns_provider_and_response_failures_into_a_retryable_catalogue_state() {
        let service = ProviderModelCatalogService::new(
            FakeCredentialStore {
                keys: Rc::new(RefCell::from(vec![(
                    AgentProfileKind::ClaudeCode,
                    "key".to_owned(),
                )])),
            },
            FakeClient {
                response: Err(ProviderModelCatalogError::RequestFailed),
            },
        );

        let catalog = service
            .catalog(AgentProfileKind::ClaudeCode)
            .expect("failure state should be safe to render");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Unavailable);
        assert!(catalog.models.is_empty());
    }

    #[test]
    fn does_not_store_a_key_when_the_provider_cannot_load_its_catalogue() {
        let credentials = FakeCredentialStore::default();
        let service = ProviderModelCatalogService::new(
            credentials.clone(),
            FakeClient {
                response: Err(ProviderModelCatalogError::RequestFailed),
            },
        );

        let catalog = service
            .save_credential_and_catalog(SaveProviderCatalogCredentialRequest {
                provider_kind: AgentProfileKind::ClinePassCli,
                api_key: "new-key".to_owned(),
            })
            .expect("failure state should be safe to render");

        assert_eq!(catalog.status, ProviderModelCatalogStatus::Unavailable);
        assert!(credentials.keys.borrow().is_empty());
    }

    #[test]
    fn rejects_an_empty_key_and_non_native_provider_before_persisting() {
        let service = ProviderModelCatalogService::new(
            FakeCredentialStore::default(),
            FakeClient {
                response: Ok(vec!["{}".to_owned()]),
            },
        );

        assert_eq!(
            service
                .save_credential_and_catalog(SaveProviderCatalogCredentialRequest {
                    provider_kind: AgentProfileKind::CodexCli,
                    api_key: " ".to_owned(),
                })
                .expect_err("empty key should fail"),
            ProviderModelCatalogError::InvalidCredential
        );
        assert_eq!(
            service
                .catalog(AgentProfileKind::StructuredProcess)
                .expect_err("structured bridge has no provider API"),
            ProviderModelCatalogError::UnsupportedProvider
        );
    }
}
