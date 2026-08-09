use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::AgentProfileKind;

/// A known native agent executable and whether it can be found without running it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProviderAvailability {
    pub kind: AgentProfileKind,
    pub label: &'static str,
    pub program: &'static str,
    pub installed: bool,
}

pub fn discover_native_agent_providers() -> Vec<AgentProviderAvailability> {
    discover_native_agent_providers_in_path(env::var_os("PATH"))
}

fn discover_native_agent_providers_in_path(
    path: Option<OsString>,
) -> Vec<AgentProviderAvailability> {
    known_providers()
        .into_iter()
        .map(|(kind, label, program)| AgentProviderAvailability {
            kind,
            label,
            program,
            installed: program_exists(program, path.as_deref()),
        })
        .collect()
}

fn known_providers() -> [(AgentProfileKind, &'static str, &'static str); 3] {
    [
        (AgentProfileKind::CodexCli, "Codex", "codex"),
        (AgentProfileKind::ClaudeCode, "Claude Code", "claude"),
        (AgentProfileKind::ClinePassCli, "Cline", "cline"),
    ]
}

fn program_exists(program: &str, path: Option<&std::ffi::OsStr>) -> bool {
    path.into_iter()
        .flat_map(env::split_paths)
        .flat_map(|directory| program_candidates(&directory, program))
        .any(|candidate| executable_file_exists(&candidate))
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

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;

    use super::{discover_native_agent_providers, discover_native_agent_providers_in_path};

    #[test]
    fn detects_only_known_programs_present_on_the_supplied_path() {
        let directory = TempDir::new().expect("temporary directory should be created");
        let program = if cfg!(windows) { "codex.exe" } else { "codex" };
        let executable = directory.path().join(program);
        fs::write(&executable, "").expect("fixture should be created");
        #[cfg(unix)]
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
            .expect("fixture should be executable");

        let providers =
            discover_native_agent_providers_in_path(Some(OsString::from(directory.path())));

        assert_eq!(providers.len(), 3);
        assert!(providers[0].installed);
        assert!(!providers[1].installed);
        assert!(!providers[2].installed);
    }

    #[test]
    fn returns_not_installed_when_path_is_not_available() {
        let providers = discover_native_agent_providers_in_path(None);

        assert!(providers.iter().all(|provider| !provider.installed));
    }

    #[cfg(unix)]
    #[test]
    fn ignores_a_non_executable_file_with_a_known_agent_name() {
        let directory = TempDir::new().expect("temporary directory should be created");
        let candidate = directory.path().join("codex");
        fs::write(&candidate, "").expect("fixture should be created");
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o644))
            .expect("fixture permissions should be set");

        let providers =
            discover_native_agent_providers_in_path(Some(OsString::from(directory.path())));

        assert!(!providers[0].installed);
    }

    #[test]
    fn reports_all_supported_native_agent_choices() {
        let providers = discover_native_agent_providers();

        assert_eq!(providers.len(), 3);
        assert_eq!(providers[0].label, "Codex");
        assert_eq!(providers[1].label, "Claude Code");
        assert_eq!(providers[2].label, "Cline");
    }
}
