use std::collections::BTreeMap;

use serde_json::Value;

use crate::domain::AgentEffort;

use super::{
    AgentProfileKind,
    provider_catalog::{ProviderModel, ProviderModelCatalog},
};

pub(super) fn catalog_from_responses(
    provider_kind: AgentProfileKind,
    responses: &[String],
) -> Result<ProviderModelCatalog, ()> {
    if responses.is_empty() {
        return Err(());
    }
    let mut models = BTreeMap::new();
    for response in responses {
        for model in models_from_response(provider_kind, response)? {
            models.insert(model.id.clone(), model);
        }
    }
    Ok(ProviderModelCatalog::ready(
        provider_kind,
        models.into_values().collect(),
    ))
}

fn models_from_response(
    provider_kind: AgentProfileKind,
    response: &str,
) -> Result<Vec<ProviderModel>, ()> {
    let response = serde_json::from_str::<Value>(response).map_err(|_| ())?;
    Ok(response
        .get("data")
        .and_then(Value::as_array)
        .or_else(|| response.get("models").and_then(Value::as_array))
        .ok_or(())?
        .iter()
        .filter_map(|model| provider_model(provider_kind, model))
        .collect::<Vec<_>>())
}

fn provider_model(provider_kind: AgentProfileKind, model: &Value) -> Option<ProviderModel> {
    let id = model.get("id")?.as_str()?.trim();
    if id.is_empty() {
        return None;
    }
    let label = model
        .get("display_name")
        .or_else(|| model.get("displayName"))
        .or_else(|| model.get("name"))
        .and_then(Value::as_str)
        .filter(|label| !label.trim().is_empty())
        .unwrap_or(id)
        .to_owned();
    Some(ProviderModel {
        id: id.to_owned(),
        label,
        efforts: efforts_for(provider_kind, model),
    })
}

fn efforts_for(provider_kind: AgentProfileKind, model: &Value) -> Vec<AgentEffort> {
    let Some(capability) = effort_capability(provider_kind, model) else {
        return native_efforts(provider_kind);
    };
    let efforts = [
        ("low", AgentEffort::Focused),
        ("medium", AgentEffort::Balanced),
        ("high", AgentEffort::Thorough),
    ]
    .into_iter()
    .filter_map(|(name, effort)| capability_supported(capability, name).then_some(effort))
    .collect::<Vec<_>>();
    if efforts.is_empty() && !capability.is_boolean() && capability.get("supported").is_none() {
        native_efforts(provider_kind)
    } else {
        efforts
    }
}

fn effort_capability(provider_kind: AgentProfileKind, model: &Value) -> Option<&Value> {
    match provider_kind {
        AgentProfileKind::ClaudeCode => model.pointer("/capabilities/effort"),
        AgentProfileKind::ClinePassCli => model
            .get("supportsReasoning")
            .or_else(|| model.get("supports_reasoning")),
        AgentProfileKind::CodexCli | AgentProfileKind::StructuredProcess => None,
    }
}

fn capability_supported(capability: &Value, effort: &str) -> bool {
    if let Some(support) = capability.get(effort) {
        return support
            .get("supported")
            .and_then(Value::as_bool)
            .or_else(|| support.as_bool())
            .unwrap_or(false);
    }
    capability.as_bool().unwrap_or(false)
}

fn native_efforts(provider_kind: AgentProfileKind) -> Vec<AgentEffort> {
    match provider_kind {
        AgentProfileKind::StructuredProcess => Vec::new(),
        AgentProfileKind::CodexCli
        | AgentProfileKind::ClaudeCode
        | AgentProfileKind::ClinePassCli => vec![
            AgentEffort::Focused,
            AgentEffort::Balanced,
            AgentEffort::Thorough,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::catalog_from_responses;
    use crate::{agent::AgentProfileKind, domain::AgentEffort};

    #[test]
    fn reads_openai_models_and_uses_adapter_effort_choices_when_api_omits_them() {
        let catalog = catalog_from_responses(
            AgentProfileKind::CodexCli,
            &[r#"{"data":[{"id":"gpt-5-codex"}]}"#.to_owned()],
        )
        .expect("response should parse");

        assert_eq!(catalog.models[0].label, "gpt-5-codex");
        assert_eq!(catalog.models[0].efforts.len(), 3);
    }

    #[test]
    fn narrows_claude_efforts_to_the_model_capability_returned_by_the_api() {
        let catalog = catalog_from_responses(
            AgentProfileKind::ClaudeCode,
            &[r#"{"data":[{"id":"claude","display_name":"Claude","capabilities":{"effort":{"low":{"supported":true},"medium":{"supported":false},"high":{"supported":true}}}}]}"#.to_owned()],
        )
        .expect("response should parse");

        assert_eq!(catalog.models[0].label, "Claude");
        assert_eq!(
            catalog.models[0].efforts,
            vec![AgentEffort::Focused, AgentEffort::Thorough]
        );
    }

    #[test]
    fn respects_a_cline_model_that_does_not_support_reasoning() {
        let catalog = catalog_from_responses(
            AgentProfileKind::ClinePassCli,
            &[
                r#"{"data":[{"id":"openai/gpt-4o","name":"GPT-4o","supportsReasoning":false}]}"#
                    .to_owned(),
            ],
        )
        .expect("response should parse");

        assert_eq!(catalog.models[0].efforts, Vec::<AgentEffort>::new());
    }

    #[test]
    fn rejects_a_malformed_or_model_less_response() {
        assert!(
            catalog_from_responses(AgentProfileKind::CodexCli, &["not json".to_owned()]).is_err()
        );
        assert!(catalog_from_responses(AgentProfileKind::CodexCli, &["{}".to_owned()]).is_err());
        assert!(catalog_from_responses(AgentProfileKind::CodexCli, &[]).is_err());
    }

    #[test]
    fn combines_paginated_responses_without_repeating_a_model() {
        let catalog = catalog_from_responses(
            AgentProfileKind::ClaudeCode,
            &[
                r#"{"data":[{"id":"claude-1"},{"id":"claude-2"}]}"#.to_owned(),
                r#"{"data":[{"id":"claude-2"},{"id":"claude-3"}]}"#.to_owned(),
            ],
        )
        .expect("responses should parse");

        assert_eq!(
            catalog
                .models
                .into_iter()
                .map(|model| model.id)
                .collect::<Vec<_>>(),
            vec!["claude-1", "claude-2", "claude-3"]
        );
    }
}
