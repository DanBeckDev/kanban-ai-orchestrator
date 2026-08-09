use std::cell::{Cell, RefCell};

use chrono::{Duration, Utc};
use url::Url;

use super::{
    AuthorizationCodeExchange, LinearConnectionStatus, LinearCredentialStore, LinearCredentials,
    LinearOAuthConfiguration, LinearOAuthError, LinearOAuthService, LinearRequestCredentials,
    LinearTokenClient, resolve_request_credentials,
};

mod callback;
mod comment_scope;

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

struct FailingTokenClient;

impl LinearTokenClient for FailingTokenClient {
    fn exchange_authorization_code(
        &self,
        _exchange: AuthorizationCodeExchange,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        Err(LinearOAuthError::TokenExchange("unavailable".to_owned()))
    }

    fn refresh_access_token(
        &self,
        _credentials: &LinearCredentials,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        Err(LinearOAuthError::TokenExchange("unavailable".to_owned()))
    }
}

struct FailingCredentialStore;

impl LinearCredentialStore for FailingCredentialStore {
    fn clear(&self) -> Result<(), LinearOAuthError> {
        Err(LinearOAuthError::CredentialStore("clear failed".to_owned()))
    }

    fn load(&self) -> Result<Option<LinearCredentials>, LinearOAuthError> {
        Err(LinearOAuthError::CredentialStore("load failed".to_owned()))
    }

    fn save(&self, _credentials: &LinearCredentials) -> Result<(), LinearOAuthError> {
        Err(LinearOAuthError::CredentialStore("save failed".to_owned()))
    }
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
fn requests_targeted_comment_access_only_after_an_existing_connection() {
    let mut existing = credentials(
        "client-id".to_owned(),
        "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        Utc::now() + Duration::hours(1),
    );
    existing.scopes.push("comments:create".to_owned());
    let store = MemoryCredentialStore {
        credentials: RefCell::new(Some(existing)),
    };
    let mut service = LinearOAuthService::new(store);

    let authorization_url = service
        .begin_comment_access()
        .expect("existing connection should be re-authorizable for comments");
    let parsed_url = Url::parse(&authorization_url).expect("authorization URL should be valid");
    let parameters = parsed_url.query_pairs().collect::<Vec<_>>();

    assert!(
        parameters
            .iter()
            .any(|(name, value)| name == "scope" && value == "read,comments:create")
    );
    assert!(
        service
            .comments_are_authorized()
            .expect("credentials should remain readable")
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
    let existing = match service
        .credentials_for_request()
        .expect("expired connection should be readable")
    {
        LinearRequestCredentials::RequiresRefresh(credentials) => credentials,
        LinearRequestCredentials::AccessToken(_) => {
            panic!("expired connection should request a refresh")
        }
    };
    let (access_token, refreshed) = resolve_request_credentials(
        LinearRequestCredentials::RequiresRefresh(existing),
        &token_client,
    )
    .expect("expired token should refresh");
    service
        .record_credentials(refreshed.expect("refresh should produce replacement credentials"))
        .expect("refreshed credentials should be stored");

    assert_eq!(
        access_token, "access-token",
        "refresh should return the replacement access token"
    );
    assert_eq!(token_client.refreshes.get(), 1);
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

    let request_credentials = service
        .credentials_for_request()
        .expect("valid credentials should remain readable");
    let token_client = FakeTokenClient::default();
    let (access_token, refreshed) = resolve_request_credentials(request_credentials, &token_client)
        .expect("valid credentials should not require refresh");
    assert_eq!(access_token, "access-token");
    assert!(refreshed.is_none());
    assert_eq!(token_client.refreshes.get(), 0);
    assert!(matches!(
        service.connection_status().expect("status should load"),
        LinearConnectionStatus::Connected { .. }
    ));
}

#[test]
fn surfaces_a_refresh_failure_without_discarding_the_stored_credentials() {
    let existing = credentials(
        "client-id".to_owned(),
        "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        Utc::now() - Duration::minutes(1),
    );

    assert!(matches!(
        resolve_request_credentials(
            LinearRequestCredentials::RequiresRefresh(existing),
            &FailingTokenClient,
        ),
        Err(LinearOAuthError::TokenExchange(error)) if error == "unavailable"
    ));
}

#[test]
fn surfaces_secure_credential_store_failures_at_the_operation_that_needs_them() {
    let credentials = credentials(
        "client-id".to_owned(),
        "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        Utc::now() + Duration::hours(24),
    );

    assert_eq!(
        LinearOAuthService::new(FailingCredentialStore).record_credentials(credentials),
        Err(LinearOAuthError::CredentialStore("save failed".to_owned()))
    );
    assert!(matches!(
        LinearOAuthService::new(FailingCredentialStore).credentials_for_request(),
        Err(LinearOAuthError::CredentialStore(error)) if error == "load failed"
    ));
    assert_eq!(
        LinearOAuthService::new(FailingCredentialStore).forget_local_credentials(),
        Err(LinearOAuthError::CredentialStore("clear failed".to_owned()))
    );
}

#[test]
fn reports_no_refresh_credential_when_no_linear_account_is_connected() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());

    assert!(matches!(
        service.credentials_for_request(),
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
