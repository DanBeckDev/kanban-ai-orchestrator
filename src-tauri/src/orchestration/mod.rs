use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        Dependency, DependencyGraph, DependencyGraphError, PlanId, PolicyDecision,
        PolicyDecisionId, ProjectId, WorkItem, WorkItemBudget, WorkItemId, WorkItemProgress,
    },
    policy::{
        AuthorizedAction, PolicyAction, PolicyAuditStore, PolicyGate, PolicyRequest, PolicyUsage,
    },
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
    pub acceptance_criteria: Vec<String>,
    pub budget: WorkItemBudget,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerTick {
    pub actor: String,
    pub decided_at: String,
    pub active_repository_execution_count: u32,
    pub repository_execution_capacity: u32,
    pub progress_by_work_item: BTreeMap<WorkItemId, WorkItemProgress>,
    pub usage_by_work_item: BTreeMap<WorkItemId, PolicyUsage>,
    pub decision_ids: BTreeMap<WorkItemId, PolicyDecisionId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledLaunch {
    pub work_item_id: WorkItemId,
    pub authorization: AuthorizedAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDeferredWorkItem {
    pub work_item_id: WorkItemId,
    pub decision: PolicyDecision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryDeferredWorkItem {
    pub work_item_id: WorkItemId,
    pub active_execution_count: u32,
    pub execution_capacity: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchedulerResult {
    pub launches: Vec<ScheduledLaunch>,
    pub deferred_by_policy: Vec<PolicyDeferredWorkItem>,
    pub deferred_by_repository_capacity: Vec<RepositoryDeferredWorkItem>,
}

pub struct DaemonScheduler {
    preview: PlanPreview,
    dependency_graph: DependencyGraph,
    work_items: BTreeMap<WorkItemId, WorkItem>,
    confirmation: Option<PlanConfirmation>,
}

impl DaemonScheduler {
    pub fn propose(proposal: PlanProposal) -> Result<Self, PlanProposalError> {
        reject_blank_plan_identity(&proposal)?;
        let work_items = work_items_by_id(&proposal.work_items)?;
        reject_blank_assumptions(&proposal.unresolved_assumptions)?;
        let dependency_graph = dependency_graph(&work_items, &proposal.dependencies)?;
        let parallel_stages = parallel_stages(&work_items, &proposal.dependencies);
        let mut dependencies = proposal.dependencies;
        dependencies.sort_by(|left, right| left.id.cmp(&right.id));
        let work_item_previews = work_items
            .values()
            .map(|work_item| PlanWorkItemPreview {
                id: work_item.id.clone(),
                title: work_item.title.clone(),
                acceptance_criteria: work_item.acceptance_criteria.clone(),
                budget: work_item.budget.clone(),
            })
            .collect::<Vec<_>>();
        let preview = PlanPreview {
            id: proposal.id,
            project_id: proposal.project_id,
            work_items: work_item_previews,
            dependencies,
            critical_path: dependency_graph.critical_path(),
            parallel_stages,
            budget: budget_summary(&work_items),
            unresolved_assumptions: proposal.unresolved_assumptions,
        };

        Ok(Self {
            preview,
            dependency_graph,
            work_items,
            confirmation: None,
        })
    }

    pub fn preview(&self) -> &PlanPreview {
        &self.preview
    }

    pub fn confirmation(&self) -> Option<&PlanConfirmation> {
        self.confirmation.as_ref()
    }

    pub fn confirm(&mut self, confirmation: PlanConfirmation) -> Result<(), PlanConfirmationError> {
        if confirmation.plan_id != self.preview.id {
            return Err(PlanConfirmationError::PlanIdMismatch {
                expected_plan_id: self.preview.id.clone(),
                received_plan_id: confirmation.plan_id,
            });
        }
        if confirmation.confirmed_by.trim().is_empty() {
            return Err(PlanConfirmationError::MissingConfirmedBy);
        }
        if confirmation.confirmed_at.trim().is_empty() {
            return Err(PlanConfirmationError::MissingConfirmedAt);
        }

        self.confirmation = Some(confirmation);
        Ok(())
    }

    pub fn schedule<AuditStore: PolicyAuditStore>(
        &self,
        tick: SchedulerTick,
        policy_gate: &PolicyGate,
        audit_store: &mut AuditStore,
    ) -> Result<SchedulerResult, SchedulerError<AuditStore::Error>> {
        if self.confirmation.is_none() {
            return Err(SchedulerError::PlanNotConfirmed {
                plan_id: self.preview.id.clone(),
            });
        }

        let candidates = self
            .dependency_graph
            .dependency_safe_ready_work_items(&tick.progress_by_work_item);
        if tick.active_repository_execution_count < tick.repository_execution_capacity {
            require_policy_decision_ids(&candidates, &tick.decision_ids)?;
        }

        let mut result = SchedulerResult::default();
        let mut active_execution_count = tick.active_repository_execution_count;
        for work_item_id in candidates {
            if active_execution_count >= tick.repository_execution_capacity {
                result
                    .deferred_by_repository_capacity
                    .push(RepositoryDeferredWorkItem {
                        work_item_id,
                        active_execution_count,
                        execution_capacity: tick.repository_execution_capacity,
                    });
                continue;
            }
            let work_item = self
                .work_items
                .get(&work_item_id)
                .expect("the dependency graph only returns proposed work items");
            let mut usage = tick
                .usage_by_work_item
                .get(&work_item_id)
                .copied()
                .unwrap_or_default();
            usage.active_execution_count = active_execution_count;
            let decision_id = tick
                .decision_ids
                .get(&work_item_id)
                .cloned()
                .ok_or_else(|| SchedulerError::MissingPolicyDecisionId {
                    work_item_id: work_item_id.clone(),
                })?;
            let authorization = policy_gate
                .authorize_and_record(
                    PolicyRequest {
                        decision_id,
                        project_id: self.preview.project_id.clone(),
                        work_item_id: Some(work_item_id.clone()),
                        actor: tick.actor.clone(),
                        action: PolicyAction::StartExecution,
                        usage,
                        work_item_budget: Some(work_item.budget.clone()),
                        protected_git_approval: None,
                        decided_at: tick.decided_at.clone(),
                    },
                    audit_store,
                )
                .map_err(SchedulerError::PolicyAudit)?;

            if let Some(authorized_action) = authorization.authorized_action().cloned() {
                result.launches.push(ScheduledLaunch {
                    work_item_id,
                    authorization: authorized_action,
                });
                active_execution_count = active_execution_count.saturating_add(1);
            } else {
                result.deferred_by_policy.push(PolicyDeferredWorkItem {
                    work_item_id,
                    decision: authorization.decision().clone(),
                });
            }
        }

        Ok(result)
    }
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

fn require_policy_decision_ids<AuditError>(
    candidates: &[WorkItemId],
    decision_ids: &BTreeMap<WorkItemId, PolicyDecisionId>,
) -> Result<(), SchedulerError<AuditError>> {
    let mut work_item_id_by_decision_id = BTreeMap::new();

    for work_item_id in candidates {
        let decision_id = decision_ids.get(work_item_id).ok_or_else(|| {
            SchedulerError::MissingPolicyDecisionId {
                work_item_id: work_item_id.clone(),
            }
        })?;
        if decision_id.0.trim().is_empty() {
            return Err(SchedulerError::BlankPolicyDecisionId {
                work_item_id: work_item_id.clone(),
            });
        }
        if let Some(first_work_item_id) =
            work_item_id_by_decision_id.insert(decision_id.clone(), work_item_id.clone())
        {
            return Err(SchedulerError::DuplicatePolicyDecisionId {
                decision_id: decision_id.clone(),
                first_work_item_id,
                second_work_item_id: work_item_id.clone(),
            });
        }
    }

    Ok(())
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

#[derive(Debug)]
pub enum SchedulerError<AuditError> {
    PlanNotConfirmed {
        plan_id: PlanId,
    },
    MissingPolicyDecisionId {
        work_item_id: WorkItemId,
    },
    BlankPolicyDecisionId {
        work_item_id: WorkItemId,
    },
    DuplicatePolicyDecisionId {
        decision_id: PolicyDecisionId,
        first_work_item_id: WorkItemId,
        second_work_item_id: WorkItemId,
    },
    PolicyAudit(AuditError),
}

impl<AuditError: fmt::Display> fmt::Display for SchedulerError<AuditError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanNotConfirmed { plan_id } => write!(
                formatter,
                "plan {} must be confirmed before the daemon schedules work",
                plan_id.0
            ),
            Self::MissingPolicyDecisionId { work_item_id } => write!(
                formatter,
                "daemon scheduling requires a policy-decision id for work item {}",
                work_item_id.0
            ),
            Self::BlankPolicyDecisionId { work_item_id } => write!(
                formatter,
                "daemon scheduling requires a non-blank policy-decision id for work item {}",
                work_item_id.0
            ),
            Self::DuplicatePolicyDecisionId {
                decision_id,
                first_work_item_id,
                second_work_item_id,
            } => write!(
                formatter,
                "policy-decision id {} cannot authorize both work item {} and {}",
                decision_id.0, first_work_item_id.0, second_work_item_id.0
            ),
            Self::PolicyAudit(error) => write!(formatter, "policy audit failed: {error}"),
        }
    }
}

impl<AuditError: Error + 'static> Error for SchedulerError<AuditError> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PolicyAudit(error) => Some(error),
            Self::PlanNotConfirmed { .. }
            | Self::MissingPolicyDecisionId { .. }
            | Self::BlankPolicyDecisionId { .. }
            | Self::DuplicatePolicyDecisionId { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
