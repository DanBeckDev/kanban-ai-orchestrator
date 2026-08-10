use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde::Deserialize;

use crate::{
    agent::{ProviderModel, ProviderModelCatalogError},
    domain::AgentEffort,
};

use super::bounded_output;

const CORE_ENTRYPOINT: &str = "node_modules/@cline/core/dist/index.js";
const MAX_CATALOGUE_OUTPUT_BYTES: u64 = 262_144;
const MAX_MODEL_VALUE_LENGTH: usize = 512;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
const CLINE_SDK_QUERY: &str = r#"
import { pathToFileURL } from "node:url";
import path from "node:path";

const packageRoot = process.argv.at(-1);
const coreEntrypoint = path.join(packageRoot, "node_modules", "@cline", "core", "dist", "index.js");
const core = await import(pathToFileURL(coreEntrypoint).href);
const manager = new core.ProviderSettingsManager();
const { providers } = await core.listLocalProviders(manager);
const provider = providers.find(({ id }) => id === "cline");
if (!provider) throw new Error("Cline provider is unavailable");
const { models } = await core.getLocalProviderModels(
  provider.id,
  manager.getProviderConfig(provider.id),
);
const safeModels = models.flatMap(({ id, name, supportsReasoning }) =>
  typeof id === "string" && typeof name === "string"
    ? [{ id, name, supportsReasoning: Boolean(supportsReasoning) }]
    : [],
);
process.stdout.write(JSON.stringify({ models: safeModels }));
"#;

/// Reads model metadata through Cline's installed SDK. The fixed SDK query
/// returns only model identifiers, labels, and reasoning support; it does not
/// create an agent session, prompt Cline, or provide credentials.
pub(super) fn models() -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let package_root = cline_package_root().ok_or(ProviderModelCatalogError::RuntimeUnavailable)?;
    let mut command = cline_sdk_command(&package_root);
    query_sdk_model_list(&mut command, RESPONSE_TIMEOUT)
}

fn cline_sdk_command(package_root: &Path) -> Command {
    let mut command = Command::new("node");
    command.args(["--input-type=module", "--eval", CLINE_SDK_QUERY, "--"]);
    command.arg(package_root);
    command
}

fn query_sdk_model_list(
    command: &mut Command,
    timeout: Duration,
) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    bounded_output::run(command, timeout, MAX_CATALOGUE_OUTPUT_BYTES)
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)
        .and_then(|output| models_from_sdk_output(&output))
}

fn models_from_sdk_output(output: &[u8]) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let catalogue: ClineModelCatalogue = serde_json::from_slice(output)
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    let mut seen_ids = HashSet::new();
    let models = catalogue
        .models
        .into_iter()
        .filter_map(|model| provider_model(model, &mut seen_ids))
        .collect::<Vec<_>>();
    (!models.is_empty())
        .then_some(models)
        .ok_or(ProviderModelCatalogError::RuntimeUnavailable)
}

fn provider_model(model: ClineSdkModel, seen_ids: &mut HashSet<String>) -> Option<ProviderModel> {
    let id = safe_model_value(&model.id)?;
    let label = safe_model_value(&model.name)?;
    seen_ids.insert(id.clone()).then_some(ProviderModel {
        id,
        label,
        efforts: if model.supports_reasoning {
            cline_reasoning_efforts()
        } else {
            Vec::new()
        },
    })
}

fn safe_model_value(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= MAX_MODEL_VALUE_LENGTH
        && !value.chars().any(char::is_control))
    .then(|| value.to_owned())
}

fn cline_reasoning_efforts() -> Vec<AgentEffort> {
    vec![
        AgentEffort::Focused,
        AgentEffort::Balanced,
        AgentEffort::Thorough,
        AgentEffort::ExtraThorough,
    ]
}

fn cline_package_root() -> Option<PathBuf> {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|path| env::split_paths(&path).collect::<Vec<_>>())
        .flat_map(|directory| program_candidates(&directory, "cline"))
        .find(|candidate| executable_file_exists(candidate))
        .and_then(|launcher| launcher.canonicalize().ok())
        .and_then(|launcher| package_root_from_launcher(&launcher))
}

fn package_root_from_launcher(launcher: &Path) -> Option<PathBuf> {
    launcher
        .ancestors()
        .take(6)
        .flat_map(|ancestor| [ancestor.to_owned(), ancestor.join("node_modules/cline")])
        .find(|candidate| is_cline_package_root(candidate))
}

fn is_cline_package_root(candidate: &Path) -> bool {
    candidate.join("package.json").is_file() && candidate.join(CORE_ENTRYPOINT).is_file()
}

#[cfg(windows)]
fn program_candidates(directory: &Path, program: &str) -> Vec<PathBuf> {
    [".exe", ".cmd", ".bat"]
        .into_iter()
        .map(|extension| directory.join(format!("{program}{extension}")))
        .collect()
}

#[cfg(not(windows))]
fn program_candidates(directory: &Path, program: &str) -> Vec<PathBuf> {
    vec![directory.join(program)]
}

fn executable_file_exists(candidate: &Path) -> bool {
    let Ok(metadata) = candidate.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[derive(Deserialize)]
struct ClineModelCatalogue {
    models: Vec<ClineSdkModel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClineSdkModel {
    id: String,
    name: String,
    #[serde(default)]
    supports_reasoning: bool,
}

#[cfg(test)]
mod tests {
    use std::{fs, process::Command, time::Duration};

    use tempfile::TempDir;

    use super::{
        AgentEffort, CLINE_SDK_QUERY, ProviderModelCatalogError, cline_sdk_command,
        models_from_sdk_output, package_root_from_launcher, query_sdk_model_list,
    };

    #[test]
    fn maps_c_line_sdk_models_and_only_supported_thinking_levels() {
        let models = models_from_sdk_output(
            br#"{"models":[{"id":"~anthropic/claude-opus-latest","name":"Claude Opus Latest","supportsReasoning":true},{"id":"openai/gpt-4o","name":"GPT-4o","supportsReasoning":false}]}"#,
        )
        .expect("a Cline SDK catalogue should parse");

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "~anthropic/claude-opus-latest");
        assert_eq!(
            models[0].efforts,
            [
                AgentEffort::Focused,
                AgentEffort::Balanced,
                AgentEffort::Thorough,
                AgentEffort::ExtraThorough,
            ]
        );
        assert!(models[1].efforts.is_empty());
    }

    #[test]
    fn rejects_an_empty_or_unsafe_c_line_sdk_catalogue() {
        assert!(models_from_sdk_output(br#"{"models":[]}"#).is_err());
        assert!(
            models_from_sdk_output(
                br#"{"models":[{"id":"\n","name":"Unsafe","supportsReasoning":true}]}"#,
            )
            .is_err()
        );
    }

    #[test]
    fn finds_the_sdk_from_an_installed_cline_package_layout() {
        let directory = TempDir::new().expect("temporary directory should be created");
        let package_root = directory.path().join("node_modules/cline");
        let launcher = package_root.join("bin/cline");
        fs::create_dir_all(package_root.join("node_modules/@cline/core/dist"))
            .expect("Cline SDK fixture should be created");
        fs::create_dir_all(launcher.parent().expect("launcher should have a parent"))
            .expect("Cline launcher directory should be created");
        fs::write(package_root.join("package.json"), "{}")
            .expect("Cline package manifest should be created");
        fs::write(
            package_root.join("node_modules/@cline/core/dist/index.js"),
            "",
        )
        .expect("Cline SDK entrypoint should be created");
        fs::write(&launcher, "").expect("Cline launcher should be created");

        assert_eq!(package_root_from_launcher(&launcher), Some(package_root));
    }

    #[test]
    fn fixes_the_sdk_query_and_passes_the_verified_package_root_as_data() {
        let command = cline_sdk_command(std::path::Path::new("/verified/cline"));
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(arguments[0], "--input-type=module");
        assert_eq!(arguments[1], "--eval");
        assert_eq!(arguments[3], "--");
        assert_eq!(arguments[4], "/verified/cline");
        assert!(CLINE_SDK_QUERY.contains("getLocalProviderModels"));
        assert!(!CLINE_SDK_QUERY.contains("--key"));
    }

    #[cfg(unix)]
    #[test]
    fn accepts_a_bounded_sdk_catalogue_response() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf '%s' \"$CLINE_CATALOGUE\""]);
        command.env(
            "CLINE_CATALOGUE",
            r#"{"models":[{"id":"openai/gpt-5.6","name":"GPT-5.6","supportsReasoning":true}]}"#,
        );

        let models = query_sdk_model_list(&mut command, Duration::from_secs(1))
            .expect("SDK catalogue should load");

        assert_eq!(models[0].id, "openai/gpt-5.6");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_failed_sdk_catalogue_process() {
        let mut command = Command::new("sh");
        command.args(["-c", "exit 1"]);

        assert_eq!(
            query_sdk_model_list(&mut command, Duration::from_secs(1)),
            Err(ProviderModelCatalogError::RuntimeUnavailable)
        );
    }
}
