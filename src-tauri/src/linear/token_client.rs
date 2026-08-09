use chrono::{Duration, Utc};
use serde::Deserialize;

use super::oauth::AuthorizationCodeExchange;
use super::{LinearCredentials, LinearOAuthError, LinearTokenClient};

const LINEAR_TOKEN_URL: &str = "https://api.linear.app/oauth/token";

pub struct ReqwestLinearTokenClient {
    client: reqwest::blocking::Client,
    token_url: String,
}

impl ReqwestLinearTokenClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            token_url: LINEAR_TOKEN_URL.to_owned(),
        }
    }

    fn request_token(
        &self,
        parameters: &[(&str, &str)],
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        let response = self
            .client
            .post(&self.token_url)
            .form(parameters)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| LinearOAuthError::TokenExchange(error.to_string()))?;
        let response = response
            .json::<TokenResponse>()
            .map_err(|error| LinearOAuthError::TokenExchange(error.to_string()))?;
        credentials_from_response(response, client_id, redirect_uri)
    }
}

impl Default for ReqwestLinearTokenClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearTokenClient for ReqwestLinearTokenClient {
    fn exchange_authorization_code(
        &self,
        exchange: AuthorizationCodeExchange,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        let parameters = [
            ("code", exchange.code.as_str()),
            ("client_id", exchange.client_id.as_str()),
            ("redirect_uri", exchange.redirect_uri.as_str()),
            ("code_verifier", exchange.code_verifier.as_str()),
            ("grant_type", "authorization_code"),
        ];
        self.request_token(&parameters, &exchange.client_id, &exchange.redirect_uri)
    }

    fn refresh_access_token(
        &self,
        credentials: &LinearCredentials,
    ) -> Result<LinearCredentials, LinearOAuthError> {
        let parameters = [
            ("refresh_token", credentials.refresh_token.as_str()),
            ("client_id", credentials.client_id.as_str()),
            ("grant_type", "refresh_token"),
        ];
        self.request_token(
            &parameters,
            &credentials.client_id,
            &credentials.redirect_uri,
        )
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    #[serde(default)]
    scope: Option<TokenScope>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TokenScope {
    List(Vec<String>),
    Text(String),
}

fn credentials_from_response(
    response: TokenResponse,
    client_id: &str,
    redirect_uri: &str,
) -> Result<LinearCredentials, LinearOAuthError> {
    if response.access_token.trim().is_empty() || response.refresh_token.trim().is_empty() {
        return Err(LinearOAuthError::TokenExchange(
            "Linear returned an incomplete token response".to_owned(),
        ));
    }
    let expires_in = i64::try_from(response.expires_in).map_err(|_| {
        LinearOAuthError::TokenExchange("Linear returned an unsupported token expiry".to_owned())
    })?;
    let scopes = match response.scope {
        Some(TokenScope::List(scopes)) => scopes,
        Some(TokenScope::Text(scopes)) => scopes
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        None => vec!["read".to_owned()],
    };
    Ok(LinearCredentials {
        client_id: client_id.to_owned(),
        redirect_uri: redirect_uri.to_owned(),
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_at: Utc::now() + Duration::seconds(expires_in),
        scopes,
    })
}

#[cfg(test)]
mod tests {
    use super::{ReqwestLinearTokenClient, TokenResponse, TokenScope, credentials_from_response};
    use crate::linear::{AuthorizationCodeExchange, LinearOAuthError, LinearTokenClient};

    #[test]
    fn preserves_the_current_and_legacy_scope_response_shapes() {
        let text_scopes = credentials_from_response(
            TokenResponse {
                access_token: "access".to_owned(),
                expires_in: 86_399,
                refresh_token: "refresh".to_owned(),
                scope: Some(TokenScope::Text("read write".to_owned())),
            },
            "client-id",
            "http://127.0.0.1:38471/linear/oauth/callback",
        )
        .expect("string scope should parse");
        let list_scopes = credentials_from_response(
            TokenResponse {
                access_token: "access".to_owned(),
                expires_in: 86_399,
                refresh_token: "refresh".to_owned(),
                scope: Some(TokenScope::List(vec!["read".to_owned()])),
            },
            "client-id",
            "http://127.0.0.1:38471/linear/oauth/callback",
        )
        .expect("legacy list scope should parse");

        assert_eq!(text_scopes.scopes, ["read", "write"]);
        assert_eq!(list_scopes.scopes, ["read"]);
    }

    #[test]
    fn rejects_token_responses_without_both_credentials() {
        for (access_token, refresh_token) in [("", "refresh"), ("access", "")] {
            assert!(
                credentials_from_response(
                    TokenResponse {
                        access_token: access_token.to_owned(),
                        expires_in: 86_399,
                        refresh_token: refresh_token.to_owned(),
                        scope: None,
                    },
                    "client-id",
                    "http://127.0.0.1:38471/linear/oauth/callback",
                )
                .is_err()
            );
        }
    }

    #[test]
    fn defaults_an_omitted_scope_and_rejects_an_unsupported_expiry() {
        let credentials = credentials_from_response(
            TokenResponse {
                access_token: "access".to_owned(),
                expires_in: 3_600,
                refresh_token: "refresh".to_owned(),
                scope: None,
            },
            "client-id",
            "http://127.0.0.1:38471/linear/oauth/callback",
        )
        .expect("a scope omission should retain the requested read scope");
        assert_eq!(credentials.scopes, ["read"]);
        assert!(
            credentials_from_response(
                TokenResponse {
                    access_token: "access".to_owned(),
                    expires_in: u64::MAX,
                    refresh_token: "refresh".to_owned(),
                    scope: None,
                },
                "client-id",
                "http://127.0.0.1:38471/linear/oauth/callback",
            )
            .is_err()
        );
    }

    #[test]
    fn exposes_a_token_request_construction_failure_as_a_connector_error() {
        let client = ReqwestLinearTokenClient {
            client: reqwest::blocking::Client::new(),
            token_url: "http://[::1".to_owned(),
        };

        let result = client.exchange_authorization_code(AuthorizationCodeExchange {
            code: "authorization-code".to_owned(),
            client_id: "client-id".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
            code_verifier: "pkce-verifier".to_owned(),
        });

        assert!(matches!(result, Err(LinearOAuthError::TokenExchange(_))));
    }
}
