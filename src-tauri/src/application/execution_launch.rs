use std::{error::Error, fmt};

use crate::{
    agent::StartAgentRequest,
    domain::{
        Execution, ExecutionId, ExecutionRole, ExecutionStatus, MaterializedWorkItem, WorkItemState,
    },
    workspace::{WorkspaceAssignment, WorkspaceError, WorkspaceManager},
};

pub struct ExecutionLaunchPreparation {
    execution_id: String,
    request: StartAgentRequest,
}

impl ExecutionLaunchPreparation {
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    pub fn request(&self) -> &StartAgentRequest {
        &self.request
    }
}

pub fn prepare_execution_launch(
    execution: &Execution,
    work_item: &MaterializedWorkItem,
    manager: &WorkspaceManager,
    assignment: &WorkspaceAssignment,
    adapter_name: &str,
    task_brief: &str,
) -> Result<ExecutionLaunchPreparation, ExecutionLaunchError> {
    validate_required(adapter_name, "adapter name")?;
    validate_required(task_brief, "task brief")?;

    if execution.status != ExecutionStatus::Pending {
        return Err(ExecutionLaunchError::ExecutionNotPending {
            execution_id: execution.id.clone(),
            status: execution.status,
        });
    }
    if execution.adapter_name != adapter_name {
        return Err(ExecutionLaunchError::AdapterNameMismatch {
            execution_id: execution.id.clone(),
            expected: execution.adapter_name.clone(),
            configured: adapter_name.to_owned(),
        });
    }
    ensure_work_item_is_ready_for(execution.role, work_item)?;
    manager
        .verify_execution_workspace(execution, assignment)
        .map_err(ExecutionLaunchError::Workspace)?;

    Ok(ExecutionLaunchPreparation {
        execution_id: execution.id.0.clone(),
        request: StartAgentRequest {
            work_item_id: execution.work_item_id.0.clone(),
            workspace_path: execution.workspace_path.clone(),
            task_brief: task_brief.to_owned(),
        },
    })
}

fn ensure_work_item_is_ready_for(
    role: ExecutionRole,
    work_item: &MaterializedWorkItem,
) -> Result<(), ExecutionLaunchError> {
    let expected_state = match role {
        ExecutionRole::Implementation => WorkItemState::Ready,
        ExecutionRole::IndependentReview => WorkItemState::Review,
    };
    if work_item.work_item.state == expected_state {
        Ok(())
    } else {
        Err(ExecutionLaunchError::WorkItemNotReady {
            work_item_id: work_item.work_item.id.clone(),
            state: work_item.work_item.state,
        })
    }
}

fn validate_required(value: &str, field: &'static str) -> Result<(), ExecutionLaunchError> {
    if value.trim().is_empty() {
        Err(ExecutionLaunchError::MissingRequiredField { field })
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ExecutionLaunchError {
    Workspace(WorkspaceError),
    MissingRequiredField {
        field: &'static str,
    },
    ExecutionNotPending {
        execution_id: ExecutionId,
        status: ExecutionStatus,
    },
    AdapterNameMismatch {
        execution_id: ExecutionId,
        expected: String,
        configured: String,
    },
    WorkItemNotReady {
        work_item_id: crate::domain::WorkItemId,
        state: WorkItemState,
    },
}

impl fmt::Display for ExecutionLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Workspace(error) => write!(formatter, "workspace launch check failed: {error}"),
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::ExecutionNotPending {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {} cannot launch because it is {status:?}",
                execution_id.0
            ),
            Self::AdapterNameMismatch {
                execution_id,
                expected,
                configured,
            } => write!(
                formatter,
                "execution {} expects adapter {expected}, not configured adapter {configured}",
                execution_id.0
            ),
            Self::WorkItemNotReady {
                work_item_id,
                state,
            } => write!(
                formatter,
                "work item {} cannot launch because it is {state:?}",
                work_item_id.0
            ),
        }
    }
}

impl Error for ExecutionLaunchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Workspace(error) => Some(error),
            Self::MissingRequiredField { .. }
            | Self::ExecutionNotPending { .. }
            | Self::AdapterNameMismatch { .. }
            | Self::WorkItemNotReady { .. } => None,
        }
    }
}
