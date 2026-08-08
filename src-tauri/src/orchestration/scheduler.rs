use std::{collections::BTreeMap, error::Error, fmt};

use crate::{
    domain::{
        DependencyGraph, PlanId, PolicyDecision, PolicyDecisionId, WorkItem, WorkItemId,
        WorkItemProgress,
    },
    policy::{
        AuthorizedAction, PolicyAction, PolicyAuditStore, PolicyGate, PolicyRequest, PolicyUsage,
    },
};

use super::{
    PlanConfirmation, PlanConfirmationError, PlanPreview, PlanProposal, PlanProposalError,
    plan::{ValidatedPlan, validate_plan},
};

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
        let ValidatedPlan {
            preview,
            dependency_graph,
            work_items,
        } = validate_plan(proposal)?;

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
