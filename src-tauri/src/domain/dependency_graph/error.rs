use std::{error::Error, fmt};

use crate::domain::{DependencyId, WorkItemId};

use super::DependencyContextField;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencyGraphError {
    UnknownWorkItem {
        work_item_id: WorkItemId,
    },
    MissingHardDependencyContext {
        dependency_id: DependencyId,
        field: DependencyContextField,
    },
    DuplicateDependencyId {
        dependency_id: DependencyId,
    },
    SelfDependency {
        dependency_id: DependencyId,
        work_item_id: WorkItemId,
    },
    HardDependencyCycle {
        dependency_id: DependencyId,
        cycle: Vec<WorkItemId>,
        suggested_upstream_work_item_id: WorkItemId,
        suggested_downstream_work_item_id: WorkItemId,
    },
}

impl fmt::Display for DependencyGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownWorkItem { work_item_id } => write!(
                formatter,
                "work item {} is not registered in this dependency graph",
                work_item_id.0
            ),
            Self::MissingHardDependencyContext {
                dependency_id,
                field,
            } => write!(
                formatter,
                "hard dependency {} requires a non-empty {}",
                dependency_id.0,
                context_field_name(*field)
            ),
            Self::DuplicateDependencyId { dependency_id } => {
                write!(
                    formatter,
                    "dependency id {} already exists",
                    dependency_id.0
                )
            }
            Self::SelfDependency {
                dependency_id,
                work_item_id,
            } => write!(
                formatter,
                "dependency {} cannot make work item {} depend on itself",
                dependency_id.0, work_item_id.0
            ),
            Self::HardDependencyCycle {
                dependency_id,
                cycle,
                suggested_upstream_work_item_id,
                suggested_downstream_work_item_id,
            } => write!(
                formatter,
                "hard dependency {} would create cycle {}; remove it or reverse it to {} -> {}",
                dependency_id.0,
                cycle
                    .iter()
                    .map(|work_item_id| work_item_id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> "),
                suggested_upstream_work_item_id.0,
                suggested_downstream_work_item_id.0
            ),
        }
    }
}

impl Error for DependencyGraphError {}

fn context_field_name(field: DependencyContextField) -> &'static str {
    match field {
        DependencyContextField::Reason => "reason",
        DependencyContextField::Owner => "owner",
        DependencyContextField::NextAction => "next action",
    }
}
