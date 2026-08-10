use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::domain::SupervisionAction;

pub const MAX_SUPERVISION_WORK_ITEMS: usize = 100;
pub const MAX_SUPERVISION_DEPENDENCIES: usize = 200;
pub const MAX_SUPERVISION_ACTIVITY: usize = 50;
pub const MAX_SUPERVISION_EVIDENCE: usize = 50;
pub const MAX_SUPERVISION_CANDIDATES: usize = 100;
pub const MAX_SUPERVISION_SUMMARY_CHARS: usize = 600;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSupervisionInput {
    pub work_items: Vec<SupervisionWorkItem>,
    pub dependencies: Vec<SupervisionDependency>,
    pub activity: Vec<SupervisionActivity>,
    pub evidence: Vec<SupervisionEvidence>,
    pub candidate_actions: Vec<SupervisionCandidate>,
    output_contract: &'static str,
}

impl BoardSupervisionInput {
    pub fn new(
        work_items: Vec<SupervisionWorkItem>,
        dependencies: Vec<SupervisionDependency>,
        activity: Vec<SupervisionActivity>,
        evidence: Vec<SupervisionEvidence>,
        candidate_actions: Vec<SupervisionCandidate>,
    ) -> Result<Self, BoardSupervisionInputError> {
        for (count, maximum, field) in [
            (work_items.len(), MAX_SUPERVISION_WORK_ITEMS, "work items"),
            (
                dependencies.len(),
                MAX_SUPERVISION_DEPENDENCIES,
                "dependencies",
            ),
            (activity.len(), MAX_SUPERVISION_ACTIVITY, "activity entries"),
            (evidence.len(), MAX_SUPERVISION_EVIDENCE, "evidence entries"),
            (
                candidate_actions.len(),
                MAX_SUPERVISION_CANDIDATES,
                "candidate actions",
            ),
        ] {
            if count > maximum {
                return Err(BoardSupervisionInputError::TooManyEntries { field, maximum });
            }
        }
        if candidate_actions.is_empty() {
            return Err(BoardSupervisionInputError::NoCandidateActions);
        }
        Ok(Self {
            work_items,
            dependencies,
            activity,
            evidence,
            candidate_actions,
            output_contract: "Return exactly one JSON object with action, workItemId, recommendation, and rationale. action and workItemId must exactly match one candidateActions entry. recommendation and rationale must be short plain-language summaries. Do not use Markdown, logs, credentials, or fields outside this contract.",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionWorkItem {
    pub id: String,
    pub title: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionDependency {
    pub upstream_work_item_id: String,
    pub downstream_work_item_id: String,
    pub kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionActivity {
    pub work_item_id: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionEvidence {
    pub work_item_id: String,
    pub kind: String,
    pub result: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionCandidate {
    pub action: SupervisionAction,
    pub work_item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupervisorRecommendation {
    pub action: SupervisionAction,
    pub work_item_id: String,
    pub recommendation: String,
    pub rationale: String,
}

impl SupervisorRecommendation {
    pub fn validate_against(
        &mut self,
        input: &BoardSupervisionInput,
    ) -> Result<(), SupervisorRecommendationError> {
        self.recommendation = sanitized_summary(&self.recommendation);
        self.rationale = sanitized_summary(&self.rationale);
        if self.recommendation.is_empty() || self.rationale.is_empty() {
            return Err(SupervisorRecommendationError::MissingSummary);
        }
        let supported = input.candidate_actions.iter().any(|candidate| {
            candidate.action == self.action && candidate.work_item_id == self.work_item_id
        });
        if supported {
            Ok(())
        } else {
            Err(SupervisorRecommendationError::UnsupportedCandidate {
                action: self.action,
                work_item_id: self.work_item_id.clone(),
            })
        }
    }
}

pub fn bounded_summary(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_SUPERVISION_SUMMARY_CHARS)
        .collect()
}

fn sanitized_summary(value: &str) -> String {
    let words = bounded_summary(value)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut sanitized = Vec::new();
    let mut redact_next = false;
    for word in words {
        let lower = word.to_ascii_lowercase();
        let labelled_secret = [
            "api_key",
            "apikey",
            "authorization",
            "token",
            "password",
            "secret",
        ]
        .iter()
        .any(|label| lower.starts_with(label));
        let bearer_label = lower == "bearer" || lower == "authorization:";
        if redact_next || looks_like_credential(&word) || labelled_secret {
            sanitized.push("[redacted]".to_owned());
            redact_next =
                (labelled_secret && !word.contains(':') && !word.contains('=')) || bearer_label;
        } else {
            sanitized.push(word);
        }
    }
    sanitized.join(" ")
}

pub fn redacted_summary(value: &str) -> String {
    sanitized_summary(value)
}

fn looks_like_credential(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.starts_with("ghp_")
        || lower.starts_with("github_pat_")
        || lower.starts_with("bearer")
        || word.len() >= 32
            && word
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
}

#[derive(Debug, Eq, PartialEq)]
pub enum BoardSupervisionInputError {
    TooManyEntries { field: &'static str, maximum: usize },
    NoCandidateActions,
}

impl fmt::Display for BoardSupervisionInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyEntries { field, maximum } => {
                write!(
                    formatter,
                    "supervision input has more than {maximum} {field}"
                )
            }
            Self::NoCandidateActions => {
                formatter.write_str("supervision needs at least one safe candidate action")
            }
        }
    }
}

impl Error for BoardSupervisionInputError {}

#[derive(Debug, Eq, PartialEq)]
pub enum SupervisorRecommendationError {
    MissingSummary,
    UnsupportedCandidate {
        action: SupervisionAction,
        work_item_id: String,
    },
}

impl fmt::Display for SupervisorRecommendationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSummary => formatter
                .write_str("organiser response needs a concise recommendation and rationale"),
            Self::UnsupportedCandidate {
                action,
                work_item_id,
            } => write!(
                formatter,
                "organiser selected unsupported action {action:?} for work item {work_item_id}"
            ),
        }
    }
}

impl Error for SupervisorRecommendationError {}
