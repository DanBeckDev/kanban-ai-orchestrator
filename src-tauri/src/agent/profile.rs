use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use super::AgentCapabilities;

/// Selects the outer protocol that turns a user-approved executable into
/// normalized daemon lifecycle events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentProfileKind {
    #[default]
    StructuredProcess,
    CodexCli,
    ClaudeCode,
}

impl AgentProfileKind {
    pub fn capabilities(self) -> AgentCapabilities {
        AgentCapabilities {
            supports_feedback: false,
            supports_interrupt: false,
            supports_resume: false,
            streams_structured_events: true,
        }
    }

    fn reserves_argument(self, argument: &str) -> bool {
        match self {
            Self::StructuredProcess => false,
            Self::CodexCli => matches_argument(
                argument,
                &[
                    "exec",
                    "resume",
                    "--json",
                    "--sandbox",
                    "-s",
                    "--cd",
                    "-C",
                    "--add-dir",
                    "--dangerously-bypass-approvals-and-sandbox",
                    "--output-last-message",
                    "-o",
                ],
            ),
            Self::ClaudeCode => matches_argument(
                argument,
                &[
                    "--print",
                    "-p",
                    "--output-format",
                    "--input-format",
                    "--verbose",
                    "--resume",
                    "-r",
                    "--continue",
                    "-c",
                    "--permission-mode",
                    "--dangerously-skip-permissions",
                    "--allow-dangerously-skip-permissions",
                    "--allowedTools",
                    "--allowed-tools",
                    "--disallowedTools",
                    "--disallowed-tools",
                    "--add-dir",
                    "--settings",
                    "--mcp-config",
                ],
            ),
        }
    }
}

fn matches_argument(argument: &str, reserved_arguments: &[&str]) -> bool {
    reserved_arguments.iter().any(|reserved| {
        argument == *reserved
            || reserved.starts_with("--")
                && argument
                    .strip_prefix(reserved)
                    .is_some_and(|suffix| suffix.starts_with('='))
    })
}

/// A user-approved executable configuration and its adapter-owned protocol kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    pub name: String,
    #[serde(default)]
    pub kind: AgentProfileKind,
    pub program: String,
    pub arguments: Vec<String>,
}

impl AgentProfile {
    pub fn validate(&self) -> Result<(), AgentProfileError> {
        validate_required(&self.name, "agent profile name")?;
        validate_required(&self.program, "agent program")?;
        if self
            .arguments
            .iter()
            .any(|argument| argument.contains('\0'))
        {
            return Err(AgentProfileError::ArgumentContainsNull);
        }
        if let Some(argument) = self
            .arguments
            .iter()
            .find(|argument| self.kind.reserves_argument(argument))
        {
            return Err(AgentProfileError::ReservedArgument {
                kind: self.kind,
                argument: argument.clone(),
            });
        }
        Ok(())
    }
}

fn validate_required(value: &str, field: &'static str) -> Result<(), AgentProfileError> {
    if value.trim().is_empty() {
        Err(AgentProfileError::MissingRequiredField { field })
    } else if value.contains('\0') {
        Err(AgentProfileError::FieldContainsNull { field })
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentProfileError {
    MissingRequiredField {
        field: &'static str,
    },
    FieldContainsNull {
        field: &'static str,
    },
    ArgumentContainsNull,
    ReservedArgument {
        kind: AgentProfileKind,
        argument: String,
    },
}

impl fmt::Display for AgentProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::FieldContainsNull { field } => {
                write!(formatter, "{field} cannot contain a null character")
            }
            Self::ArgumentContainsNull => {
                formatter.write_str("agent arguments cannot contain a null character")
            }
            Self::ReservedArgument { kind, argument } => write!(
                formatter,
                "agent argument {argument} is reserved by the {kind:?} adapter"
            ),
        }
    }
}

impl Error for AgentProfileError {}

#[cfg(test)]
mod tests {
    use super::{AgentProfile, AgentProfileError, AgentProfileKind};

    #[test]
    fn accepts_a_direct_program_and_plain_arguments() {
        assert!(
            AgentProfile {
                name: "structured worker".to_owned(),
                kind: AgentProfileKind::StructuredProcess,
                program: "agent-worker".to_owned(),
                arguments: vec!["--jsonl".to_owned()],
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn rejects_missing_values_and_null_arguments() {
        assert!(matches!(
            AgentProfile {
                name: " ".to_owned(),
                kind: AgentProfileKind::StructuredProcess,
                program: "agent-worker".to_owned(),
                arguments: Vec::new(),
            }
            .validate(),
            Err(AgentProfileError::MissingRequiredField {
                field: "agent profile name"
            })
        ));
        assert!(matches!(
            AgentProfile {
                name: "worker".to_owned(),
                kind: AgentProfileKind::StructuredProcess,
                program: "agent-worker".to_owned(),
                arguments: vec!["bad\0argument".to_owned()],
            }
            .validate(),
            Err(AgentProfileError::ArgumentContainsNull)
        ));
    }

    #[test]
    fn defaults_legacy_profiles_to_the_structured_process_protocol() {
        let profile: AgentProfile =
            serde_json::from_str(r#"{"name":"worker","program":"agent-worker","arguments":[]}"#)
                .expect("legacy profile should deserialize");

        assert_eq!(profile.kind, AgentProfileKind::StructuredProcess);
        assert!(profile.kind.capabilities().streams_structured_events);
    }

    #[test]
    fn reserves_native_protocol_and_permission_arguments() {
        assert!(matches!(
            AgentProfile {
                name: "codex".to_owned(),
                kind: AgentProfileKind::CodexCli,
                program: "codex".to_owned(),
                arguments: vec!["--sandbox=read-only".to_owned()],
            }
            .validate(),
            Err(AgentProfileError::ReservedArgument { .. })
        ));
        assert!(matches!(
            AgentProfile {
                name: "claude".to_owned(),
                kind: AgentProfileKind::ClaudeCode,
                program: "claude".to_owned(),
                arguments: vec!["--permission-mode".to_owned()],
            }
            .validate(),
            Err(AgentProfileError::ReservedArgument { .. })
        ));
    }
}
