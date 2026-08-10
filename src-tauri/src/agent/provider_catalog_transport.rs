use std::time::Duration;

use reqwest::{Url, blocking::Client};
use serde_json::Value;

use super::{
    AgentProfileKind,
    provider_catalog::{ProviderModelCatalogClient, ProviderModelCatalogError},
};

const ANTHROPIC_MODELS_URL: &str = "https://api.anthropic.com/v1/models";
const CLINE_MODELS_URL: &str = "https://api.cline.bot/api/v1/models";
const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";
const MAX_CATALOG_PAGES: usize = 20;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct ReqwestProviderModelCatalogClient {
    client: Client,
}

impl ReqwestProviderModelCatalogClient {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for ReqwestProviderModelCatalogClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderModelCatalogClient for ReqwestProviderModelCatalogClient {
    fn list_models(
        &self,
        provider_kind: AgentProfileKind,
        api_key: &str,
    ) -> Result<Vec<String>, ProviderModelCatalogError> {
        if provider_kind == AgentProfileKind::ClaudeCode {
            return list_anthropic_models(&self.client, api_key);
        }
        fetch(request_for(&self.client, provider_kind, api_key)?).map(|response| vec![response])
    }
}

fn list_anthropic_models(
    client: &Client,
    api_key: &str,
) -> Result<Vec<String>, ProviderModelCatalogError> {
    let mut pages = Vec::new();
    let mut after_id = None;
    for _ in 0..MAX_CATALOG_PAGES {
        let response = fetch(anthropic_request(client, api_key, after_id.as_deref())?)?;
        after_id = next_anthropic_page(&response)?;
        pages.push(response);
        if after_id.is_none() {
            return Ok(pages);
        }
    }
    Err(ProviderModelCatalogError::RequestFailed)
}

fn anthropic_request(
    client: &Client,
    api_key: &str,
    after_id: Option<&str>,
) -> Result<reqwest::blocking::RequestBuilder, ProviderModelCatalogError> {
    let mut url =
        Url::parse(ANTHROPIC_MODELS_URL).map_err(|_| ProviderModelCatalogError::RequestFailed)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("limit", "100");
        if let Some(after_id) = after_id {
            query.append_pair("after_id", after_id);
        }
    }
    Ok(client
        .get(url)
        .header("anthropic-version", "2023-06-01")
        .header("x-api-key", api_key))
}

fn fetch(request: reqwest::blocking::RequestBuilder) -> Result<String, ProviderModelCatalogError> {
    request
        .timeout(REQUEST_TIMEOUT)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(reqwest::blocking::Response::text)
        .map_err(|_| ProviderModelCatalogError::RequestFailed)
}

fn next_anthropic_page(response: &str) -> Result<Option<String>, ProviderModelCatalogError> {
    let envelope = serde_json::from_str::<Value>(response)
        .map_err(|_| ProviderModelCatalogError::RequestFailed)?;
    if !envelope
        .get("has_more")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(None);
    }
    envelope
        .get("last_id")
        .and_then(Value::as_str)
        .filter(|identifier| !identifier.trim().is_empty())
        .map(|identifier| Some(identifier.to_owned()))
        .ok_or(ProviderModelCatalogError::RequestFailed)
}

fn request_for(
    client: &Client,
    provider_kind: AgentProfileKind,
    api_key: &str,
) -> Result<reqwest::blocking::RequestBuilder, ProviderModelCatalogError> {
    match provider_kind {
        AgentProfileKind::CodexCli => Ok(client.get(OPENAI_MODELS_URL).bearer_auth(api_key)),
        AgentProfileKind::ClaudeCode => anthropic_request(client, api_key, None),
        AgentProfileKind::ClinePassCli => Ok(client.get(CLINE_MODELS_URL).bearer_auth(api_key)),
        AgentProfileKind::StructuredProcess => Err(ProviderModelCatalogError::UnsupportedProvider),
    }
}

#[cfg(test)]
mod tests {
    use super::{ANTHROPIC_MODELS_URL, CLINE_MODELS_URL, OPENAI_MODELS_URL, next_anthropic_page};

    #[test]
    fn uses_documented_provider_model_endpoints() {
        assert_eq!(OPENAI_MODELS_URL, "https://api.openai.com/v1/models");
        assert_eq!(ANTHROPIC_MODELS_URL, "https://api.anthropic.com/v1/models");
        assert_eq!(CLINE_MODELS_URL, "https://api.cline.bot/api/v1/models");
    }

    #[test]
    fn follows_anthropic_pagination_only_when_a_valid_cursor_is_present() {
        assert_eq!(
            next_anthropic_page(r#"{"has_more":true,"last_id":"claude-3"}"#),
            Ok(Some("claude-3".to_owned()))
        );
        assert_eq!(next_anthropic_page(r#"{"has_more":false}"#), Ok(None));
        assert!(next_anthropic_page(r#"{"has_more":true}"#).is_err());
    }
}
