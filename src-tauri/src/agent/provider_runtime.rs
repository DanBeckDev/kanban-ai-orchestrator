use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::AgentEffort;

use super::{
    AgentProfileKind, ProviderModel, ProviderModelCatalogClient, ProviderModelCatalogError,
};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Uses an installed agent's own local protocol instead of a second API account.
pub(crate) struct InstalledProviderRuntimeClient;

impl ProviderModelCatalogClient for InstalledProviderRuntimeClient {
    fn list_models(
        &self,
        provider_kind: AgentProfileKind,
    ) -> Result<Option<Vec<ProviderModel>>, ProviderModelCatalogError> {
        match provider_kind {
            AgentProfileKind::CodexCli => codex_models().map(Some),
            AgentProfileKind::ClaudeCode | AgentProfileKind::ClinePassCli => Ok(None),
            AgentProfileKind::StructuredProcess => {
                Err(ProviderModelCatalogError::UnsupportedProvider)
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn codex_models() -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let mut command = Command::new("codex");
    command.args(["app-server", "--stdio"]);
    query_codex_model_list(&mut command)
}

fn query_codex_model_list(
    command: &mut Command,
) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    let result = query_model_list(&mut child);
    finish_process(&mut child);
    result
}

fn query_model_list(child: &mut Child) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or(ProviderModelCatalogError::RuntimeUnavailable)?;
    let stdout = child
        .stdout
        .take()
        .ok_or(ProviderModelCatalogError::RuntimeUnavailable)?;
    let messages = read_messages(stdout);

    write_request(
        &mut stdin,
        1,
        "initialize",
        json!({
            "clientInfo": {
                "name": "Kanban AI Orchestrator",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }),
    )?;
    wait_for_response(&messages, 1)?;
    write_notification(&mut stdin, "initialized", json!({}))?;
    write_request(&mut stdin, 2, "model/list", json!({}))?;
    let response = wait_for_response(&messages, 2)?;
    models_from_response(response)
}

fn read_messages(stdout: ChildStdout) -> Receiver<Result<Value, ProviderModelCatalogError>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                let _ = sender.send(Err(ProviderModelCatalogError::RuntimeUnavailable));
                return;
            };
            let Ok(message) = serde_json::from_str::<Value>(&line) else {
                let _ = sender.send(Err(ProviderModelCatalogError::RuntimeUnavailable));
                return;
            };
            if message.get("id").is_none() {
                continue;
            }
            if sender.send(Ok(message)).is_err() {
                return;
            }
        }
    });
    receiver
}

fn write_request(
    stdin: &mut ChildStdin,
    id: u8,
    method: &str,
    params: Value,
) -> Result<(), ProviderModelCatalogError> {
    write_message(
        stdin,
        json!({ "id": id, "method": method, "params": params }),
    )
}

fn write_notification(
    stdin: &mut ChildStdin,
    method: &str,
    params: Value,
) -> Result<(), ProviderModelCatalogError> {
    write_message(stdin, json!({ "method": method, "params": params }))
}

fn write_message(stdin: &mut ChildStdin, message: Value) -> Result<(), ProviderModelCatalogError> {
    serde_json::to_writer(&mut *stdin, &message)
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    stdin
        .write_all(b"\n")
        .and_then(|()| stdin.flush())
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)
}

fn wait_for_response(
    messages: &Receiver<Result<Value, ProviderModelCatalogError>>,
    id: u8,
) -> Result<Value, ProviderModelCatalogError> {
    loop {
        let message = messages
            .recv_timeout(RESPONSE_TIMEOUT)
            .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)??;
        if message.get("id") != Some(&json!(id)) {
            continue;
        }
        return message
            .get("result")
            .cloned()
            .ok_or(ProviderModelCatalogError::RuntimeUnavailable);
    }
}

fn finish_process(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn models_from_response(response: Value) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let response: CodexModelList = serde_json::from_value(response)
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    Ok(response
        .data
        .into_iter()
        .filter(|model| !model.hidden)
        .map(model_from_codex)
        .collect())
}

fn model_from_codex(model: CodexModel) -> ProviderModel {
    ProviderModel {
        id: model.model,
        label: model.display_name,
        efforts: neutral_efforts(model.supported_reasoning_efforts),
    }
}

fn neutral_efforts(efforts: Vec<CodexReasoningEffort>) -> Vec<AgentEffort> {
    let mut supported = Vec::new();
    for effort in efforts {
        let neutral = match effort.reasoning_effort.as_str() {
            "low" => AgentEffort::Focused,
            "medium" => AgentEffort::Balanced,
            "high" => AgentEffort::Thorough,
            _ => continue,
        };
        if !supported.contains(&neutral) {
            supported.push(neutral);
        }
    }
    supported
}

#[derive(Deserialize)]
struct CodexModelList {
    data: Vec<CodexModel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexModel {
    display_name: String,
    #[serde(default)]
    hidden: bool,
    model: String,
    #[serde(default)]
    supported_reasoning_efforts: Vec<CodexReasoningEffort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexReasoningEffort {
    reasoning_effort: String,
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::process::Command;

    use serde_json::json;

    use super::{
        AgentEffort, AgentProfileKind, InstalledProviderRuntimeClient, ProviderModelCatalogClient,
        models_from_response, query_codex_model_list,
    };

    #[cfg(unix)]
    #[test]
    fn reads_a_model_list_after_json_rpc_initialisation() {
        let mut command = Command::new("sh");
        command.args([
            "-c",
            r#"
            while IFS= read -r line; do
              case "$line" in
                *'"id":1'*) printf '%s\n' '{"id":1,"result":{}}' ;;
                *'"id":2'*)
                  printf '%s\n' '{"id":2,"result":{"data":[{"model":"gpt-5.6","displayName":"GPT-5.6","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
                  exit 0
                  ;;
              esac
            done
            "#,
        ]);

        let models = query_codex_model_list(&mut command).expect("model list should load");

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gpt-5.6");
        assert_eq!(models[0].efforts, vec![AgentEffort::Balanced]);
    }

    #[test]
    fn maps_only_visible_models_and_supported_efforts() {
        let models = models_from_response(json!({
            "data": [
                {
                    "model": "gpt-5.6",
                    "displayName": "GPT-5.6",
                    "hidden": false,
                    "supportedReasoningEfforts": [
                        { "reasoningEffort": "low" },
                        { "reasoningEffort": "high" },
                        { "reasoningEffort": "xhigh" }
                    ]
                },
                {
                    "model": "hidden-model",
                    "displayName": "Hidden model",
                    "hidden": true,
                    "supportedReasoningEfforts": []
                }
            ]
        }))
        .expect("response should parse");

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gpt-5.6");
        assert_eq!(
            models[0].efforts,
            vec![AgentEffort::Focused, AgentEffort::Thorough]
        );
    }

    #[test]
    fn rejects_an_unexpected_runtime_response() {
        assert!(models_from_response(json!({ "notData": [] })).is_err());
    }

    #[test]
    fn defers_to_clients_that_do_not_expose_a_catalogue_protocol() {
        let client = InstalledProviderRuntimeClient;

        assert_eq!(client.list_models(AgentProfileKind::ClaudeCode), Ok(None));
        assert_eq!(client.list_models(AgentProfileKind::ClinePassCli), Ok(None));
    }
}
