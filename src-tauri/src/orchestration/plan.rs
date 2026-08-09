use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::domain::{
    AgentEffort, AgentModelPreference, Dependency, DependencyGraph, DependencyGraphError, PlanId,
    ProjectId, WorkItem, WorkItemBudget, WorkItemId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanProposal {
    pub id: PlanId,
    pub project_id: ProjectId,
    pub work_items: Vec<WorkItem>,
    pub dependencies: Vec<Dependency>,
    pub unresolved_assumptions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanWorkItemPreview {
    pub id: WorkItemId,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub budget: WorkItemBudget,
    pub requires_human_review: bool,
    pub assigned_agent_profile_name: Option<String>,
    pub assigned_agent_model: AgentModelPreference,
    pub assigned_agent_effort: AgentEffort,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanBudgetSummary {
    pub max_agent_turns: Option<u64>,
    pub max_duration_seconds: Option<u64>,
    pub max_cost_micros: Option<u64>,
    pub work_items_missing_agent_turn_budget: Vec<WorkItemId>,
    pub work_items_missing_duration_budget: Vec<WorkItemId>,
    pub work_items_missing_cost_budget: Vec<WorkItemId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPreview {
    pub id: PlanId,
    pub project_id: ProjectId,
    pub work_items: Vec<PlanWorkItemPreview>,
    pub dependencies: Vec<Dependency>,
    pub critical_path: Vec<WorkItemId>,
    pub parallel_stages: Vec<Vec<WorkItemId>>,
    pub budget: PlanBudgetSummary,
    pub unresolved_assumptions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanConfirmation {
    pub plan_id: PlanId,
    pub confirmed_by: String,
    pub confirmed_at: String,
}

pub(crate) struct ValidatedPlan {
    pub preview: PlanPreview,
    pub dependency_graph: DependencyGraph,
    pub work_items: BTreeMap<WorkItemId, WorkItem>,
}

pub(crate) fn validate_plan(proposal: PlanProposal) -> Result<ValidatedPlan, PlanProposalError> {
    reject_blank_plan_identity(&proposal)?;
    let work_items = work_items_by_id(&proposal.work_items)?;
    reject_blank_assumptions(&proposal.unresolved_assumptions)?;
    let dependency_graph = dependency_graph(&work_items, &proposal.dependencies)?;
    let parallel_stages = parallel_stages(&work_items, &proposal.dependencies);
    let mut dependencies = proposal.dependencies;
    dependencies.sort_by(|left, right| left.id.cmp(&right.id));
    let work_item_previews = proposal
        .work_items
        .iter()
        .map(|work_item| PlanWorkItemPreview {
            id: work_item.id.clone(),
            title: work_item.title.clone(),
            description: work_item.description.clone(),
            acceptance_criteria: work_item.acceptance_criteria.clone(),
            budget: work_item.budget.clone(),
            requires_human_review: work_item.requires_human_review,
            assigned_agent_profile_name: work_item.assigned_agent_profile_name.clone(),
            assigned_agent_model: work_item.assigned_agent_model.clone(),
            assigned_agent_effort: work_item.assigned_agent_effort,
        })
        .collect::<Vec<_>>();

    Ok(ValidatedPlan {
        preview: PlanPreview {
            id: proposal.id,
            project_id: proposal.project_id,
            work_items: work_item_previews,
            dependencies,
            critical_path: dependency_graph.critical_path(),
            parallel_stages,
            budget: budget_summary(&work_items),
            unresolved_assumptions: proposal.unresolved_assumptions,
        },
        dependency_graph,
        work_items,
    })
}

fn work_items_by_id(
    work_items: &[WorkItem],
) -> Result<BTreeMap<WorkItemId, WorkItem>, PlanProposalError> {
    if work_items.is_empty() {
        return Err(PlanProposalError::EmptyPlan);
    }

    let mut by_id = BTreeMap::new();
    for work_item in work_items {
        if by_id
            .insert(work_item.id.clone(), work_item.clone())
            .is_some()
        {
            return Err(PlanProposalError::DuplicateWorkItemId {
                work_item_id: work_item.id.clone(),
            });
        }
    }

    Ok(by_id)
}

fn reject_blank_plan_identity(proposal: &PlanProposal) -> Result<(), PlanProposalError> {
    if proposal.id.0.trim().is_empty() {
        Err(PlanProposalError::MissingPlanId)
    } else if proposal.project_id.0.trim().is_empty() {
        Err(PlanProposalError::MissingProjectId)
    } else {
        Ok(())
    }
}

fn reject_blank_assumptions(assumptions: &[String]) -> Result<(), PlanProposalError> {
    if assumptions
        .iter()
        .any(|assumption| assumption.trim().is_empty())
    {
        Err(PlanProposalError::BlankUnresolvedAssumption)
    } else {
        Ok(())
    }
}

fn dependency_graph(
    work_items: &BTreeMap<WorkItemId, WorkItem>,
    dependencies: &[Dependency],
) -> Result<DependencyGraph, PlanProposalError> {
    let mut graph = DependencyGraph::new(work_items.keys().cloned());
    for dependency in dependencies {
        graph
            .add_dependency(dependency.clone())
            .map_err(PlanProposalError::DependencyGraph)?;
    }

    Ok(graph)
}

fn parallel_stages(
    work_items: &BTreeMap<WorkItemId, WorkItem>,
    dependencies: &[Dependency],
) -> Vec<Vec<WorkItemId>> {
    let hard_dependencies = dependencies
        .iter()
        .filter(|dependency| dependency.kind.is_hard())
        .fold(
            BTreeMap::<WorkItemId, BTreeSet<WorkItemId>>::new(),
            |mut map, dependency| {
                map.entry(dependency.downstream_work_item_id.clone())
                    .or_default()
                    .insert(dependency.upstream_work_item_id.clone());
                map
            },
        );
    let mut unscheduled = work_items.keys().cloned().collect::<BTreeSet<_>>();
    let mut stages = Vec::new();

    while !unscheduled.is_empty() {
        let stage = unscheduled
            .iter()
            .filter(|work_item_id| {
                hard_dependencies
                    .get(*work_item_id)
                    .is_none_or(|upstream_ids| upstream_ids.is_disjoint(&unscheduled))
            })
            .cloned()
            .collect::<Vec<_>>();

        assert!(
            !stage.is_empty(),
            "the validated hard-dependency graph must have a topological stage"
        );
        for work_item_id in &stage {
            unscheduled.remove(work_item_id);
        }
        stages.push(stage);
    }

    stages
}

fn budget_summary(work_items: &BTreeMap<WorkItemId, WorkItem>) -> PlanBudgetSummary {
    let (max_agent_turns, work_items_missing_agent_turn_budget) =
        budget_total(work_items, |budget| budget.max_agent_turns.map(u64::from));
    let (max_duration_seconds, work_items_missing_duration_budget) =
        budget_total(work_items, |budget| budget.max_duration_seconds);
    let (max_cost_micros, work_items_missing_cost_budget) =
        budget_total(work_items, |budget| budget.max_cost_micros);

    PlanBudgetSummary {
        max_agent_turns,
        max_duration_seconds,
        max_cost_micros,
        work_items_missing_agent_turn_budget,
        work_items_missing_duration_budget,
        work_items_missing_cost_budget,
    }
}

fn budget_total(
    work_items: &BTreeMap<WorkItemId, WorkItem>,
    limit: impl Fn(&WorkItemBudget) -> Option<u64>,
) -> (Option<u64>, Vec<WorkItemId>) {
    let missing_work_item_ids = work_items
        .values()
        .filter(|work_item| limit(&work_item.budget).is_none())
        .map(|work_item| work_item.id.clone())
        .collect::<Vec<_>>();
    let total = work_items
        .values()
        .map(|work_item| limit(&work_item.budget).unwrap_or_default())
        .sum::<u64>();

    if missing_work_item_ids.is_empty() {
        (Some(total), Vec::new())
    } else {
        (None, missing_work_item_ids)
    }
}

#[derive(Debug)]
pub enum PlanProposalError {
    MissingPlanId,
    MissingProjectId,
    EmptyPlan,
    DuplicateWorkItemId { work_item_id: WorkItemId },
    BlankUnresolvedAssumption,
    DependencyGraph(DependencyGraphError),
}

impl fmt::Display for PlanProposalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlanId => formatter.write_str("a plan proposal requires a plan id"),
            Self::MissingProjectId => formatter.write_str("a plan proposal requires a project id"),
            Self::EmptyPlan => {
                formatter.write_str("a plan proposal requires at least one work item")
            }
            Self::DuplicateWorkItemId { work_item_id } => {
                write!(
                    formatter,
                    "plan proposal repeats work item {}",
                    work_item_id.0
                )
            }
            Self::BlankUnresolvedAssumption => {
                formatter.write_str("unresolved assumptions must not be blank")
            }
            Self::DependencyGraph(error) => write!(formatter, "invalid plan dependency: {error}"),
        }
    }
}

impl Error for PlanProposalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DependencyGraph(error) => Some(error),
            Self::MissingPlanId
            | Self::MissingProjectId
            | Self::EmptyPlan
            | Self::DuplicateWorkItemId { .. }
            | Self::BlankUnresolvedAssumption => None,
        }
    }
}

#[derive(Debug)]
pub enum PlanConfirmationError {
    PlanIdMismatch {
        expected_plan_id: PlanId,
        received_plan_id: PlanId,
    },
    MissingConfirmedBy,
    MissingConfirmedAt,
}

impl fmt::Display for PlanConfirmationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanIdMismatch {
                expected_plan_id,
                received_plan_id,
            } => write!(
                formatter,
                "confirmation is for plan {} but the daemon proposed {}",
                received_plan_id.0, expected_plan_id.0
            ),
            Self::MissingConfirmedBy => {
                formatter.write_str("plan confirmation requires a user identity")
            }
            Self::MissingConfirmedAt => {
                formatter.write_str("plan confirmation requires a timestamp")
            }
        }
    }
}

impl Error for PlanConfirmationError {}
