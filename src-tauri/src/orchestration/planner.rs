use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::domain::{DependencyKind, WorkItemBudget};

pub const MAX_PLANNER_GOAL_BYTES: usize = 8_000;
pub const MAX_PLAN_WORK_ITEMS: usize = 50;
pub const MAX_PLAN_DEPENDENCIES: usize = 100;
pub const MAX_PLAN_ASSUMPTIONS: usize = 50;

/// A direct executable that consumes one planner input JSON object and returns one plan draft JSON object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerProfile {
    pub name: String,
    pub program: String,
    pub arguments: Vec<String>,
}

impl PlannerProfile {
    pub fn validate(&self) -> Result<(), PlannerProfileError> {
        validate_required(&self.name, "planner profile name")?;
        validate_required(&self.program, "planner program")?;
        if self
            .arguments
            .iter()
            .any(|argument| argument.contains('\0'))
        {
            return Err(PlannerProfileError::ArgumentContainsNull);
        }
        Ok(())
    }
}

/// The only model-produced data accepted by the plan-generation boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanDraft {
    pub work_items: Vec<PlanDraftWorkItem>,
    #[serde(default)]
    pub dependencies: Vec<PlanDraftDependency>,
    #[serde(default)]
    pub unresolved_assumptions: Vec<String>,
}

impl PlanDraft {
    pub fn validate(&self) -> Result<(), PlanDraftError> {
        if self.work_items.is_empty() {
            return Err(PlanDraftError::EmptyWorkItems);
        }
        if self.work_items.len() > MAX_PLAN_WORK_ITEMS {
            return Err(PlanDraftError::TooManyWorkItems);
        }
        if self.dependencies.len() > MAX_PLAN_DEPENDENCIES {
            return Err(PlanDraftError::TooManyDependencies);
        }
        if self.unresolved_assumptions.len() > MAX_PLAN_ASSUMPTIONS {
            return Err(PlanDraftError::TooManyAssumptions);
        }

        let keys = self.work_item_keys()?;
        for dependency in &self.dependencies {
            dependency.validate(&keys)?;
        }
        if self
            .unresolved_assumptions
            .iter()
            .any(|assumption| assumption.trim().is_empty())
        {
            return Err(PlanDraftError::BlankAssumption);
        }
        Ok(())
    }

    fn work_item_keys(&self) -> Result<BTreeSet<&str>, PlanDraftError> {
        let mut keys = BTreeSet::new();
        for work_item in &self.work_items {
            work_item.validate()?;
            if !keys.insert(work_item.key.as_str()) {
                return Err(PlanDraftError::DuplicateWorkItemKey {
                    key: work_item.key.clone(),
                });
            }
        }
        Ok(keys)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanDraftWorkItem {
    pub key: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub budget: PlanDraftBudget,
    #[serde(default = "requires_human_review")]
    pub requires_human_review: bool,
}

/// The planner-specific budget payload. It deliberately does not reuse the permissive domain
/// deserializer so unknown model fields cannot cross the planner boundary unnoticed.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanDraftBudget {
    pub max_agent_turns: Option<u32>,
    pub max_duration_seconds: Option<u64>,
    pub max_cost_micros: Option<u64>,
}

impl From<PlanDraftBudget> for WorkItemBudget {
    fn from(budget: PlanDraftBudget) -> Self {
        Self {
            max_agent_turns: budget.max_agent_turns,
            max_duration_seconds: budget.max_duration_seconds,
            max_cost_micros: budget.max_cost_micros,
        }
    }
}

impl PlanDraftWorkItem {
    fn validate(&self) -> Result<(), PlanDraftError> {
        validate_draft_required(&self.key, "work item key")?;
        validate_draft_required(&self.title, "work item title")?;
        validate_draft_required(&self.description, "work item description")?;
        if self.acceptance_criteria.is_empty()
            || self
                .acceptance_criteria
                .iter()
                .any(|criterion| criterion.trim().is_empty())
        {
            return Err(PlanDraftError::InvalidAcceptanceCriteria {
                key: self.key.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanDraftDependency {
    pub upstream_key: String,
    pub downstream_key: String,
    pub kind: DependencyKind,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
}

impl PlanDraftDependency {
    fn validate(&self, work_item_keys: &BTreeSet<&str>) -> Result<(), PlanDraftError> {
        validate_draft_required(&self.upstream_key, "dependency upstream key")?;
        validate_draft_required(&self.downstream_key, "dependency downstream key")?;
        validate_draft_required(&self.reason, "dependency reason")?;
        validate_draft_required(&self.owner, "dependency owner")?;
        validate_draft_required(&self.next_action, "dependency next action")?;
        if !work_item_keys.contains(self.upstream_key.as_str()) {
            return Err(PlanDraftError::UnknownDependencyWorkItemKey {
                key: self.upstream_key.clone(),
            });
        }
        if !work_item_keys.contains(self.downstream_key.as_str()) {
            return Err(PlanDraftError::UnknownDependencyWorkItemKey {
                key: self.downstream_key.clone(),
            });
        }
        Ok(())
    }
}

fn requires_human_review() -> bool {
    true
}

fn validate_required(value: &str, field: &'static str) -> Result<(), PlannerProfileError> {
    if value.trim().is_empty() {
        Err(PlannerProfileError::MissingRequiredField { field })
    } else if value.contains('\0') {
        Err(PlannerProfileError::FieldContainsNull { field })
    } else {
        Ok(())
    }
}

fn validate_draft_required(value: &str, field: &'static str) -> Result<(), PlanDraftError> {
    if value.trim().is_empty() {
        Err(PlanDraftError::MissingRequiredField { field })
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannerProfileError {
    MissingRequiredField { field: &'static str },
    FieldContainsNull { field: &'static str },
    ArgumentContainsNull,
}

impl fmt::Display for PlannerProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::FieldContainsNull { field } => {
                write!(formatter, "{field} cannot contain a null character")
            }
            Self::ArgumentContainsNull => {
                formatter.write_str("planner arguments cannot contain a null character")
            }
        }
    }
}

impl Error for PlannerProfileError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanDraftError {
    EmptyWorkItems,
    TooManyWorkItems,
    TooManyDependencies,
    TooManyAssumptions,
    MissingRequiredField { field: &'static str },
    DuplicateWorkItemKey { key: String },
    UnknownDependencyWorkItemKey { key: String },
    InvalidAcceptanceCriteria { key: String },
    BlankAssumption,
}

impl fmt::Display for PlanDraftError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWorkItems => {
                formatter.write_str("planner output must include at least one work item")
            }
            Self::TooManyWorkItems => write!(
                formatter,
                "planner output exceeds the {MAX_PLAN_WORK_ITEMS}-work-item limit"
            ),
            Self::TooManyDependencies => write!(
                formatter,
                "planner output exceeds the {MAX_PLAN_DEPENDENCIES}-dependency limit"
            ),
            Self::TooManyAssumptions => write!(
                formatter,
                "planner output exceeds the {MAX_PLAN_ASSUMPTIONS}-assumption limit"
            ),
            Self::MissingRequiredField { field } => {
                write!(formatter, "planner output requires {field}")
            }
            Self::DuplicateWorkItemKey { key } => {
                write!(formatter, "planner output repeats work item key {key}")
            }
            Self::UnknownDependencyWorkItemKey { key } => write!(
                formatter,
                "planner output references unknown work item key {key}"
            ),
            Self::InvalidAcceptanceCriteria { key } => write!(
                formatter,
                "planner work item {key} requires non-empty acceptance criteria"
            ),
            Self::BlankAssumption => {
                formatter.write_str("planner output contains a blank unresolved assumption")
            }
        }
    }
}

impl Error for PlanDraftError {}
