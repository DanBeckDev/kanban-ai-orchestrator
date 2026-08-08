use crate::{
    desktop::LocalBoardService,
    desktop_execution_runtime_support::ExecutionRuntimeError,
    domain::{MaterializedWorkItem, PolicyDecisionId, Project},
    policy::{PolicyAction, PolicyGate, PolicyLimits, PolicyRequest, PolicySet, PolicyUsage},
};

pub(crate) fn authorize_execution_start(
    service: &mut LocalBoardService,
    project: &Project,
    work_item: &MaterializedWorkItem,
    execution_id: &str,
    decided_at: String,
) -> Result<(), ExecutionRuntimeError> {
    let active_execution_count = service
        .active_execution_count_for_project(&project.id)
        .map_err(ExecutionRuntimeError::Board)?;
    let authorization = service
        .authorize_execution_start(
            &policy_gate(&project.policy_set_id)?,
            PolicyRequest {
                decision_id: PolicyDecisionId::from(
                    format!("execution-start-{execution_id}").as_str(),
                ),
                project_id: project.id.clone(),
                work_item_id: Some(work_item.work_item.id.clone()),
                actor: "board-user".to_owned(),
                action: PolicyAction::StartExecution,
                usage: PolicyUsage {
                    active_execution_count,
                    ..PolicyUsage::default()
                },
                work_item_budget: Some(work_item.work_item.budget.clone()),
                protected_git_approval: None,
                decided_at,
            },
        )
        .map_err(ExecutionRuntimeError::PolicyAudit)?;
    authorization
        .authorized_action()
        .is_some()
        .then_some(())
        .ok_or_else(|| ExecutionRuntimeError::PolicyDenied {
            reason: authorization.decision().reason.clone(),
        })
}

fn policy_gate(policy_set_id: &str) -> Result<PolicyGate, ExecutionRuntimeError> {
    if policy_set_id != "standard" {
        return Err(ExecutionRuntimeError::UnsupportedPolicySet {
            policy_set_id: policy_set_id.to_owned(),
        });
    }
    Ok(PolicyGate::new(PolicySet {
        limits: PolicyLimits {
            max_parallel_executions: 1,
            ..PolicyLimits::default()
        },
        ..PolicySet::default()
    }))
}
