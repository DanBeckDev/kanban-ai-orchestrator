use std::{
    io::Read,
    process::{Child, ChildStdout, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crate::domain::AgentEffort;

use super::super::{ProviderModel, ProviderModelCatalogError};

const MAX_HELP_OUTPUT_BYTES: u64 = 65_536;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Reads only Claude Code's own documented CLI capability metadata. It does not
/// create a session, read credentials, or contact a provider endpoint.
pub(super) fn models() -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let mut command = Command::new("claude");
    command.arg("--help");
    query_model_capabilities(&mut command, RESPONSE_TIMEOUT)
}

fn query_model_capabilities(
    command: &mut Command,
    timeout: Duration,
) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    let result = read_help_output(&mut child, timeout).and_then(|output| {
        String::from_utf8(output)
            .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)
            .and_then(|help| models_from_help(&help))
    });
    finish_process(&mut child);
    result
}

fn read_help_output(
    child: &mut Child,
    timeout: Duration,
) -> Result<Vec<u8>, ProviderModelCatalogError> {
    let stdout = child
        .stdout
        .take()
        .ok_or(ProviderModelCatalogError::RuntimeUnavailable)?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(read_bounded_output(stdout));
    });
    wait_for_output(child, receiver, timeout)
}

fn wait_for_output(
    child: &mut Child,
    receiver: mpsc::Receiver<Result<Vec<u8>, ProviderModelCatalogError>>,
    timeout: Duration,
) -> Result<Vec<u8>, ProviderModelCatalogError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?
        {
            if !status.success() {
                return Err(ProviderModelCatalogError::RuntimeUnavailable);
            }
            return receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
        }
        if Instant::now() >= deadline {
            return Err(ProviderModelCatalogError::RuntimeUnavailable);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn read_bounded_output(mut stdout: ChildStdout) -> Result<Vec<u8>, ProviderModelCatalogError> {
    let mut output = Vec::new();
    let bytes_read = stdout
        .by_ref()
        .take(MAX_HELP_OUTPUT_BYTES + 1)
        .read_to_end(&mut output)
        .map_err(|_| ProviderModelCatalogError::RuntimeUnavailable)?;
    if bytes_read as u64 > MAX_HELP_OUTPUT_BYTES {
        return Err(ProviderModelCatalogError::RuntimeUnavailable);
    }
    Ok(output)
}

fn finish_process(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn models_from_help(help: &str) -> Result<Vec<ProviderModel>, ProviderModelCatalogError> {
    let efforts = option_values(help, "--effort <level>")
        .and_then(parenthesized_values)
        .into_iter()
        .flat_map(|values| values.split(','))
        .filter_map(|value| claude_effort(value.trim()))
        .collect::<Vec<_>>();
    let aliases = option_values(help, "--model <model>")
        .and_then(parenthesized_values)
        .map(quoted_values)
        .unwrap_or_default();
    if aliases.is_empty() || efforts.is_empty() {
        return Err(ProviderModelCatalogError::RuntimeUnavailable);
    }
    Ok(aliases
        .into_iter()
        .map(|alias| ProviderModel {
            id: alias.to_owned(),
            label: format!("Claude {}", title_case(alias)),
            efforts: efforts.clone(),
        })
        .collect())
}

fn claude_effort(name: &str) -> Option<AgentEffort> {
    match name {
        "low" => Some(AgentEffort::Focused),
        "medium" => Some(AgentEffort::Balanced),
        "high" => Some(AgentEffort::Thorough),
        "xhigh" => Some(AgentEffort::ExtraThorough),
        "max" => Some(AgentEffort::Maximum),
        _ => None,
    }
}

fn option_values<'help>(help: &'help str, option: &str) -> Option<&'help str> {
    let values = help.get(help.find(option)? + option.len()..)?;
    let end = values.find("\n  --").unwrap_or(values.len());
    values.get(..end)
}

fn parenthesized_values(values: &str) -> Option<&str> {
    let start = values.find('(')? + 1;
    let end = values.get(start..)?.find(')')? + start;
    values.get(start..end)
}

fn quoted_values(values: &str) -> Vec<&str> {
    let mut quoted = Vec::new();
    let mut remainder = values;
    while let Some(start) = remainder.find('\'') {
        let value = remainder.get(start + 1..).unwrap_or_default();
        let Some(end) = value.find('\'') else {
            break;
        };
        if let Some(alias) = value.get(..end).filter(|alias| !alias.is_empty()) {
            quoted.push(alias);
        }
        remainder = value.get(end + 1..).unwrap_or_default();
    }
    quoted
}

fn title_case(alias: &str) -> String {
    let mut characters = alias.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::process::Command;
    #[cfg(unix)]
    use std::time::Duration;

    use super::{
        AgentEffort, ProviderModelCatalogError, models_from_help, option_values,
        parenthesized_values, query_model_capabilities, quoted_values,
    };

    const HELP: &str = r#"
  --effort <level>                      Effort level for the current session
                                        (low, medium, high, xhigh, max)
  --model <model>                       Model for the current session. Provide
                                        an alias for the latest model (e.g.
                                        'fable', 'opus', or 'sonnet') or a
                                        model's full name (e.g. 'claude-fable-5').
  --output-format <format>              Specify output format.
"#;

    #[test]
    fn reads_cli_advertised_model_aliases_and_efforts() {
        assert_eq!(
            option_values(HELP, "--effort <level>").and_then(parenthesized_values),
            Some("low, medium, high, xhigh, max")
        );
        assert_eq!(
            option_values(HELP, "--model <model>")
                .and_then(parenthesized_values)
                .map(quoted_values)
                .unwrap_or_default(),
            ["fable", "opus", "sonnet"]
        );
        let models = models_from_help(HELP).expect("help output should parse");

        assert_eq!(
            models
                .iter()
                .map(|model| model.id.as_str())
                .collect::<Vec<_>>(),
            ["fable", "opus", "sonnet"]
        );
        assert_eq!(models[0].label, "Claude Fable");
        assert_eq!(
            models[0].efforts,
            vec![
                AgentEffort::Focused,
                AgentEffort::Balanced,
                AgentEffort::Thorough,
                AgentEffort::ExtraThorough,
                AgentEffort::Maximum,
            ]
        );
    }

    #[test]
    fn rejects_help_that_does_not_advertise_selectable_capabilities() {
        assert!(models_from_help("--model <model> no aliases").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn reads_bounded_cli_capability_output() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf '%s' \"$CLAUDE_HELP\""]);
        command.env("CLAUDE_HELP", HELP);

        let models = query_model_capabilities(&mut command, Duration::from_secs(1))
            .expect("help should load");

        assert_eq!(models.len(), 3);
        assert_eq!(models[2].id, "sonnet");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_failed_or_oversized_help_processes() {
        let mut failed = Command::new("sh");
        failed.args(["-c", "exit 1"]);
        assert_eq!(
            query_model_capabilities(&mut failed, Duration::from_secs(1)),
            Err(ProviderModelCatalogError::RuntimeUnavailable)
        );

        let mut oversized = Command::new("sh");
        oversized.args(["-c", "head -c 65537 /dev/zero"]);
        assert_eq!(
            query_model_capabilities(&mut oversized, Duration::from_secs(1)),
            Err(ProviderModelCatalogError::RuntimeUnavailable)
        );
    }

    #[cfg(unix)]
    #[test]
    fn terminates_a_help_process_that_exceeds_its_time_budget() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 1"]);

        assert_eq!(
            query_model_capabilities(&mut command, Duration::from_millis(1)),
            Err(ProviderModelCatalogError::RuntimeUnavailable)
        );
    }
}
