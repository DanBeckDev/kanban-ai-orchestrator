use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

/// A user-approved program that accepts a task brief on stdin and emits normalized JSONL events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    pub name: String,
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
    MissingRequiredField { field: &'static str },
    FieldContainsNull { field: &'static str },
    ArgumentContainsNull,
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
        }
    }
}

impl Error for AgentProfileError {}

#[cfg(test)]
mod tests {
    use super::{AgentProfile, AgentProfileError};

    #[test]
    fn accepts_a_direct_program_and_plain_arguments() {
        assert!(
            AgentProfile {
                name: "structured worker".to_owned(),
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
                program: "agent-worker".to_owned(),
                arguments: vec!["bad\0argument".to_owned()],
            }
            .validate(),
            Err(AgentProfileError::ArgumentContainsNull)
        ));
    }
}
