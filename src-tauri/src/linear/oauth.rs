use std::{error::Error, fmt};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::{Host, Url};
use uuid::Uuid;

const LINEAR_AUTHORIZE_URL: &str = "https://linear.app/oauth/authorize";
const REFRESH_SAFETY_WINDOW: Duration = Duration::minutes(1);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearOAuthConfiguration {
    pub client_id: String,
    pub redirect_uri: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum LinearConnectionStatus {
    Disconnected,
    AwaitingAuthorization,
    Connected {
        expires_at: String,
        scopes: Vec<String>,
    },
    Failed {
        message: String,
    },
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearCredentials {
    pub client_id: String,
    pub redirect_uri: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct AuthorizationCodeExchange {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_verifier: String,
}

pub trait LinearCredentialStore {
    fn clear(&self) -> Result<(), LinearOAuthError>;
    fn load(&self) -> Result<Option<LinearCredentials>, LinearOAuthError>;
    fn save(&self, credentials: &LinearCredentials) -> Result<(), LinearOAuthError>;
}

pub trait LinearTokenClient {
    fn exchange_authorization_code(
        &self,
        exchange: AuthorizationCodeExchange,
    ) -> Result<LinearCredentials, LinearOAuthError>;
    fn refresh_access_token(
        &self,
        credentials: &LinearCredentials,
    ) -> Result<LinearCredentials, LinearOAuthError>;
}

pub struct LinearOAuthService<Store> {
    credential_store: Store,
    pending: Option<PendingAuthorization>,
    status: LinearConnectionStatus,
}

#[derive(Clone, Debug)]
struct PendingAuthorization {
    configuration: LinearOAuthConfiguration,
    code_verifier: String,
    state: String,
}

impl<Store> LinearOAuthService<Store>
where
    Store: LinearCredentialStore,
{
    pub fn new(credential_store: Store) -> Self {
        Self {
            credential_store,
            pending: None,
            status: LinearConnectionStatus::Disconnected,
        }
    }

    pub fn begin(
        &mut self,
        configuration: LinearOAuthConfiguration,
    ) -> Result<String, LinearOAuthError> {
        validate_configuration(&configuration)?;
        if self.pending.is_some() {
            return Err(LinearOAuthError::AuthenticationAlreadyPending);
        }

        let code_verifier = secure_value();
        let state = secure_value();
        let authorization_url = authorization_url(&configuration, &state, &code_verifier)?;
        self.pending = Some(PendingAuthorization {
            configuration,
            code_verifier,
            state,
        });
        self.status = LinearConnectionStatus::AwaitingAuthorization;
        Ok(authorization_url)
    }

    pub fn authorization_code_exchange(
        &mut self,
        code: &str,
        state: &str,
    ) -> Result<AuthorizationCodeExchange, LinearOAuthError> {
        if code.trim().is_empty() {
            return Err(LinearOAuthError::MissingAuthorizationCode);
        }
        let pending = self
            .pending
            .as_ref()
            .ok_or(LinearOAuthError::AuthenticationNotPending)?;
        if state != pending.state {
            return Err(LinearOAuthError::AuthorizationStateMismatch);
        }

        let pending = self
            .pending
            .take()
            .expect("pending Linear authorization was checked above");
        Ok(AuthorizationCodeExchange {
            code: code.to_owned(),
            client_id: pending.configuration.client_id,
            redirect_uri: pending.configuration.redirect_uri,
            code_verifier: pending.code_verifier,
        })
    }

    pub fn record_credentials(
        &mut self,
        credentials: LinearCredentials,
    ) -> Result<LinearConnectionStatus, LinearOAuthError> {
        self.credential_store.save(&credentials)?;
        self.status = connected_status(&credentials);
        Ok(self.status.clone())
    }

    pub fn connection_status(&mut self) -> Result<LinearConnectionStatus, LinearOAuthError> {
        if self.pending.is_some() || matches!(self.status, LinearConnectionStatus::Failed { .. }) {
            return Ok(self.status.clone());
        }
        self.status = self
            .credential_store
            .load()?
            .map(|credentials| connected_status(&credentials))
            .unwrap_or(LinearConnectionStatus::Disconnected);
        Ok(self.status.clone())
    }

    pub fn credentials_needing_refresh(
        &mut self,
    ) -> Result<Option<LinearCredentials>, LinearOAuthError> {
        let existing = self
            .credential_store
            .load()?
            .ok_or(LinearOAuthError::NotConnected)?;
        if existing.expires_at > Utc::now() + REFRESH_SAFETY_WINDOW {
            self.status = connected_status(&existing);
            return Ok(None);
        }
        Ok(Some(existing))
    }

    pub fn forget_local_credentials(&mut self) -> Result<(), LinearOAuthError> {
        self.credential_store.clear()?;
        self.pending = None;
        self.status = LinearConnectionStatus::Disconnected;
        Ok(())
    }

    pub fn record_failure(&mut self, error: LinearOAuthError) {
        self.pending = None;
        self.status = LinearConnectionStatus::Failed {
            message: error.to_string(),
        };
    }
}

pub(crate) fn loopback_callback_address(
    configuration: &LinearOAuthConfiguration,
) -> Result<std::net::SocketAddr, LinearOAuthError> {
    let redirect_url = validated_redirect_url(configuration)?;
    let host = match redirect_url.host() {
        Some(Host::Ipv4(address)) => std::net::IpAddr::V4(address),
        Some(Host::Ipv6(address)) => std::net::IpAddr::V6(address),
        _ => return Err(LinearOAuthError::InvalidRedirectUri),
    };
    let port = redirect_url
        .port()
        .ok_or(LinearOAuthError::InvalidRedirectUri)?;
    Ok(std::net::SocketAddr::new(host, port))
}

pub(crate) fn loopback_callback_path(
    configuration: &LinearOAuthConfiguration,
) -> Result<String, LinearOAuthError> {
    Ok(validated_redirect_url(configuration)?.path().to_owned())
}

fn authorization_url(
    configuration: &LinearOAuthConfiguration,
    state: &str,
    code_verifier: &str,
) -> Result<String, LinearOAuthError> {
    let mut url = Url::parse(LINEAR_AUTHORIZE_URL).expect("Linear authorization URL is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &configuration.client_id)
        .append_pair("redirect_uri", &configuration.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "read")
        .append_pair("state", state)
        .append_pair("code_challenge", &pkce_challenge(code_verifier))
        .append_pair("code_challenge_method", "S256");
    Ok(url.into())
}

fn validate_configuration(
    configuration: &LinearOAuthConfiguration,
) -> Result<(), LinearOAuthError> {
    if configuration.client_id.trim().is_empty() {
        return Err(LinearOAuthError::MissingClientId);
    }
    validated_redirect_url(configuration).map(|_| ())
}

fn validated_redirect_url(
    configuration: &LinearOAuthConfiguration,
) -> Result<Url, LinearOAuthError> {
    let redirect_url = Url::parse(&configuration.redirect_uri)
        .map_err(|_| LinearOAuthError::InvalidRedirectUri)?;
    let is_loopback_ip = match redirect_url.host() {
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        _ => false,
    };
    if redirect_url.scheme() != "http"
        || !is_loopback_ip
        || redirect_url.port().is_none()
        || redirect_url.path() == "/"
        || redirect_url.query().is_some()
        || redirect_url.fragment().is_some()
        || !redirect_url.username().is_empty()
        || redirect_url.password().is_some()
    {
        return Err(LinearOAuthError::InvalidRedirectUri);
    }
    Ok(redirect_url)
}

fn connected_status(credentials: &LinearCredentials) -> LinearConnectionStatus {
    LinearConnectionStatus::Connected {
        expires_at: credentials.expires_at.to_rfc3339(),
        scopes: credentials.scopes.clone(),
    }
}

fn secure_value() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn pkce_challenge(code_verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinearOAuthError {
    AuthenticationAlreadyPending,
    AuthenticationNotPending,
    AuthorizationStateMismatch,
    BrowserLaunch(String),
    Callback(String),
    ConnectorUnavailable,
    CredentialStore(String),
    InvalidRedirectUri,
    MissingAuthorizationCode,
    MissingClientId,
    NotConnected,
    TokenExchange(String),
}

impl fmt::Display for LinearOAuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationAlreadyPending => {
                formatter.write_str("a Linear authorization is already pending")
            }
            Self::AuthenticationNotPending => {
                formatter.write_str("no Linear authorization is pending")
            }
            Self::AuthorizationStateMismatch => {
                formatter.write_str("Linear authorization state did not match the pending request")
            }
            Self::BrowserLaunch(error) => write!(
                formatter,
                "could not open the authorization browser: {error}"
            ),
            Self::Callback(error) => {
                write!(formatter, "Linear authorization callback failed: {error}")
            }
            Self::ConnectorUnavailable => {
                formatter.write_str("the local Linear connector stopped unexpectedly")
            }
            Self::CredentialStore(error) => {
                write!(formatter, "secure credential storage failed: {error}")
            }
            Self::InvalidRedirectUri => formatter.write_str(
                "Linear redirect URI must be an HTTP loopback URL with an explicit port and path",
            ),
            Self::MissingAuthorizationCode => {
                formatter.write_str("Linear callback did not include an authorization code")
            }
            Self::MissingClientId => formatter.write_str("Linear OAuth client ID is required"),
            Self::NotConnected => formatter.write_str("no Linear account is connected"),
            Self::TokenExchange(error) => {
                write!(formatter, "Linear token exchange failed: {error}")
            }
        }
    }
}

impl Error for LinearOAuthError {}

#[cfg(test)]
mod tests;
