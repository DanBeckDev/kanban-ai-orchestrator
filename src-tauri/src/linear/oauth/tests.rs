use std::cell::{Cell, RefCell};

use chrono::{Duration, Utc};
use url::Url;

use super::{
    AuthorizationCodeExchange, LinearConnectionStatus, LinearCredentialStore, LinearCredentials,
    LinearOAuthConfiguration, LinearOAuthError, LinearOAuthService, LinearTokenClient,
};

#[derive(Default)]
struct MemoryCredentialStore {
    credentials: RefCell<Option<LinearCredentials>>,
}

impl LinearCredentialStore for MemoryCredentialStore {
    fn clear(&self) -> Result<(), LinearOAuthError> {
        self.credentials.replace(None);
        Ok(())
    }

    fn load(&self) -> Result<Option<LinearCredentials>, LinearOAuthError> {
        Ok(self.credentials.borrow().clone())
    }

    fn save(&self, credentials: &LinearCredentials) -> Result<(), LinearOAuthError> {
        self.credentials.replace(Some(credentials.clone()));
        Ok(())
    }
}

#[derive(Default)]
struct FakeTokenClient {
    refreshes: Cell<u8>,
}

impl LinearTokenClient for FakeTokenClient {
    fn exchange_authorization_code(
        &self,
        exchange: AuthorizationCodeExchange,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        assert_eq!(exchange.code, "authorization-code");
        assert!(!exchange.code_verifier.is_empty());
        Ok(credentials(
            exchange.client_id,
            exchange.redirect_uri,
            Utc::now() + Duration::hours(24),
        ))
    }

    fn refresh_access_token(
        &self,
        existing: &LinearCredentials,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        self.refreshes.set(self.refreshes.get() + 1);
        Ok(credentials(
            existing.client_id.clone(),
            existing.redirect_uri.clone(),
            Utc::now() + Duration::hours(24),
        ))
    }
}

fn configuration() -> LinearOAuthConfiguration {
    LinearOAuthConfiguration {
        client_id: "client-id".to_owned(),
        redirect_uri: "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
    }
}

fn credentials(
    client_id: String,
    redirect_uri: String,
    expires_at: chrono::DateTime<Utc>,
) -> LinearCredentials {
    LinearCredentials {
        client_id,
        redirect_uri,
        access_token: "access-token".to_owned(),
        refresh_token: "refresh-token".to_owned(),
        expires_at,
        scopes: vec!["read".to_owned()],
    }
}

fn authorization_state(authorization_url: &str) -> String {
    Url::parse(authorization_url)
        .expect("authorization URL should be valid")
        .query_pairs()
        .find(|(name, _)| name == "state")
        .expect("state should be present")
        .1
        .into_owned()
}

#[test]
fn begins_a_read_only_pkce_authorization_and_marks_it_awaiting() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());

    let authorization_url = service
        .begin(configuration())
        .expect("authorization should begin");
    let parsed_url = Url::parse(&authorization_url).expect("authorization URL should be valid");
    let parameters = parsed_url.query_pairs().collect::<Vec<_>>();

    assert_eq!(parsed_url.scheme(), "https");
    assert_eq!(parsed_url.host_str(), Some("linear.app"));
    assert!(
        parameters
            .iter()
            .any(|(name, value)| name == "scope" && value == "read")
    );
    assert!(
        parameters
            .iter()
            .any(|(name, value)| name == "code_challenge_method" && value == "S256")
    );
    assert_eq!(
        service.connection_status().expect("status should load"),
        LinearConnectionStatus::AwaitingAuthorization
    );
}

#[test]
fn rejects_a_callback_when_the_state_does_not_match_the_pending_authorization() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    service
        .begin(configuration())
        .expect("authorization should begin");

    assert!(matches!(
        service.authorization_code_exchange("authorization-code", "wrong-state"),
        Err(LinearOAuthError::AuthorizationStateMismatch)
    ));
}

#[test]
fn rejects_a_second_authorization_before_the_pending_one_is_resolved() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    service
        .begin(configuration())
        .expect("first authorization should begin");

    assert_eq!(
        service.begin(configuration()),
        Err(LinearOAuthError::AuthenticationAlreadyPending)
    );
}

#[test]
fn rejects_a_callback_without_an_authorization_code() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    service
        .begin(configuration())
        .expect("authorization should begin");

    assert!(matches!(
        service.authorization_code_exchange(" ", "state"),
        Err(LinearOAuthError::MissingAuthorizationCode)
    ));
}

#[test]
fn rejects_a_callback_when_no_authorization_is_pending() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());

    assert!(matches!(
        service.authorization_code_exchange("authorization-code", "state"),
        Err(LinearOAuthError::AuthenticationNotPending)
    ));
}

#[test]
fn stores_credentials_after_a_matching_pending_authorization() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    let token_client = FakeTokenClient::default();
    let authorization_url = service
        .begin(configuration())
        .expect("authorization should begin");

    let exchange = service
        .authorization_code_exchange(
            "authorization-code",
            &authorization_state(&authorization_url),
        )
        .expect("matching callback should produce an exchange request");
    let credentials = token_client
        .exchange_authorization_code(exchange)
        .expect("authorization code should exchange for credentials");
    let status = service
        .record_credentials(credentials)
        .expect("credentials should be stored after a successful exchange");
    assert!(matches!(status, LinearConnectionStatus::Connected { .. }));
}

#[test]
fn refreshes_an_expired_credential_without_clearing_the_existing_connection_first() {
    let store = MemoryCredentialStore::default();
    store
        .save(&credentials(
            "client-id".to_owned(),
            "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
            Utc::now() - Duration::minutes(1),
        ))
        .expect("expired credential should be stored");
    let token_client = FakeTokenClient::default();
    let mut service = LinearOAuthService::new(store);
    let existing = service
        .credentials_needing_refresh()
        .expect("expired connection should be readable")
        .expect("expired connection should request a refresh");
    let refreshed = token_client
        .refresh_access_token(&existing)
        .expect("expired token should refresh");
    service
        .record_credentials(refreshed)
        .expect("refreshed credentials should be stored");

    assert_eq!(
        token_client.refreshes.get(),
        1,
        "refresh should occur outside the credential service"
    );
    assert_eq!(
        service
            .credential_store
            .load()
            .expect("credential store should remain readable")
            .expect("refreshed credential should remain stored")
            .redirect_uri,
        "http://127.0.0.1:38471/linear/oauth/callback"
    );
}

#[test]
fn keeps_a_valid_credential_without_requesting_a_refresh() {
    let store = MemoryCredentialStore::default();
    store
        .save(&credentials(
            "client-id".to_owned(),
            "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
            Utc::now() + Duration::hours(24),
        ))
        .expect("valid credential should be stored");
    let mut service = LinearOAuthService::new(store);

    assert!(matches!(service.credentials_needing_refresh(), Ok(None)));
    assert!(matches!(
        service.connection_status().expect("status should load"),
        LinearConnectionStatus::Connected { .. }
    ));
}

#[test]
fn reports_no_refresh_credential_when_no_linear_account_is_connected() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());

    assert!(matches!(
        service.credentials_needing_refresh(),
        Err(LinearOAuthError::NotConnected)
    ));
}

#[test]
fn forgets_local_credentials_and_resets_the_connection_status() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    service
        .record_credentials(credentials(
            "client-id".to_owned(),
            "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
            Utc::now() + Duration::hours(24),
        ))
        .expect("credential should be stored");

    service
        .forget_local_credentials()
        .expect("credential should be removed");

    assert_eq!(
        service.connection_status().expect("status should load"),
        LinearConnectionStatus::Disconnected
    );
}

#[test]
fn retains_a_connection_failure_until_the_user_starts_a_new_authorization() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    service.record_failure(LinearOAuthError::Callback("timed out".to_owned()));

    assert!(matches!(
        service.connection_status().expect("status should load"),
        LinearConnectionStatus::Failed { .. }
    ));
}

#[test]
fn rejects_redirect_uris_that_are_not_explicit_loopback_callbacks() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    let configurations = [
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "https://example.com/callback".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://localhost:38471/callback".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1/callback".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1:38471".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/callback?unexpected=true".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/callback#fragment".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://user@127.0.0.1:38471/callback".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "http://user:password@127.0.0.1:38471/callback".to_owned(),
        },
        LinearOAuthConfiguration {
            client_id: "client-id".to_owned(),
            redirect_uri: "not a URL".to_owned(),
        },
    ];

    for configuration in configurations {
        assert_eq!(
            service.begin(configuration),
            Err(LinearOAuthError::InvalidRedirectUri)
        );
    }
}

#[test]
fn requires_a_nonblank_client_id_and_accepts_ipv6_loopback_callbacks() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    assert_eq!(
        service.begin(LinearOAuthConfiguration {
            client_id: " ".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        }),
        Err(LinearOAuthError::MissingClientId)
    );

    assert!(
        service
            .begin(LinearOAuthConfiguration {
                client_id: "client-id".to_owned(),
                redirect_uri: "http://[::1]:38471/linear/oauth/callback".to_owned(),
            })
            .is_ok()
    );
}
