use serde::{Deserialize, Serialize};

use super::ProjectId;

/// A provider-neutral effort preference. Provider adapters decide how (or whether)
/// to express the preference in their native protocol.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEffort {
    #[default]
    ProviderDefault,
    Focused,
    Balanced,
    Thorough,
    ExtraThorough,
    Maximum,
    Ultra,
}

/// A safe model preference for a project role. Native adapters remain responsible
/// for translating an explicit model name into their own invocation contract.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "name")]
pub enum AgentModelPreference {
    #[default]
    ProviderDefault,
    Named(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganiserDefaults {
    pub planner_profile_name: String,
    #[serde(default)]
    pub model: AgentModelPreference,
    #[serde(default)]
    pub effort: AgentEffort,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketWorkerDefaults {
    pub agent_profile_name: String,
    #[serde(default)]
    pub model: AgentModelPreference,
    #[serde(default)]
    pub effort: AgentEffort,
}

/// Durable project-scoped choices for the two distinct AI roles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAgentSettings {
    pub project_id: ProjectId,
    #[serde(default)]
    pub organiser: Option<OrganiserDefaults>,
    #[serde(default)]
    pub ticket_worker: Option<TicketWorkerDefaults>,
}
