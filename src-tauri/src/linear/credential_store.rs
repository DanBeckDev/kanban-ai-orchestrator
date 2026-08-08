use keyring::{Entry, Error as KeyringError};

use super::{LinearCredentialStore, LinearCredentials, LinearOAuthError};

const ACCOUNT: &str = "linear-oauth";
const SERVICE: &str = "Kanban AI Orchestrator";

pub struct KeyringCredentialStore;

impl LinearCredentialStore for KeyringCredentialStore {
    fn clear(&self) -> Result<(), LinearOAuthError> {
        match entry()?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(credential_error(error)),
        }
    }

    fn load(&self) -> Result<Option<LinearCredentials>, LinearOAuthError> {
        match entry()?.get_password() {
            Ok(serialized) => deserialize_credentials(&serialized).map(Some),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(credential_error(error)),
        }
    }

    fn save(&self, credentials: &LinearCredentials) -> Result<(), LinearOAuthError> {
        entry()?
            .set_password(&serialize_credentials(credentials)?)
            .map_err(credential_error)
    }
}

fn entry() -> Result<Entry, LinearOAuthError> {
    Entry::new(SERVICE, ACCOUNT).map_err(credential_error)
}

fn serialize_credentials(credentials: &LinearCredentials) -> Result<String, LinearOAuthError> {
    serde_json::to_string(credentials)
        .map_err(|error| LinearOAuthError::CredentialStore(error.to_string()))
}

fn deserialize_credentials(serialized: &str) -> Result<LinearCredentials, LinearOAuthError> {
    serde_json::from_str(serialized).map_err(|_| {
        LinearOAuthError::CredentialStore("stored Linear credentials are invalid".to_owned())
    })
}

fn credential_error(error: impl std::fmt::Display) -> LinearOAuthError {
    LinearOAuthError::CredentialStore(error.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::{deserialize_credentials, serialize_credentials};
    use crate::linear::LinearCredentials;

    #[test]
    fn serializes_credentials_without_changing_their_expiry_or_scopes() {
        let credentials = LinearCredentials {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
            access_token: "access".to_owned(),
            refresh_token: "refresh".to_owned(),
            expires_at: Utc::now() + Duration::hours(24),
            scopes: vec!["read".to_owned()],
        };

        let restored = deserialize_credentials(
            &serialize_credentials(&credentials).expect("credentials should serialize"),
        )
        .expect("credentials should deserialize");

        assert!(restored == credentials);
    }

    #[test]
    fn rejects_malformed_credentials_without_exposing_the_serialized_value() {
        assert!(deserialize_credentials("not JSON").is_err());
    }
}
