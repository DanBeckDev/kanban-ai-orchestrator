use std::fmt;

use serde::{Deserialize, Serialize};

use super::{PolicyDecisionId, ProjectId, SchemaMetadata, VersionedSchema, WorkItemId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecisionKind {
    Allow,
    Deny,
    ApprovalRequired,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolScope {
    ReadAssignedWorkspace,
    WriteAssignedWorkspace,
    RunProjectChecks,
    NetworkAccess,
}

impl fmt::Display for ToolScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::ReadAssignedWorkspace => "read_assigned_workspace",
            Self::WriteAssignedWorkspace => "write_assigned_workspace",
            Self::RunProjectChecks => "run_project_checks",
            Self::NetworkAccess => "network_access",
        };

        formatter.write_str(name)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedGitAction {
    Commit,
    Push,
    Merge,
    ForcePush,
    DeleteBranch,
}

impl fmt::Display for ProtectedGitAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Commit => "commit",
            Self::Push => "push",
            Self::Merge => "merge",
            Self::ForcePush => "force_push",
            Self::DeleteBranch => "delete_branch",
        };

        formatter.write_str(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicyAction {
    StartExecution,
    Tool { scope: ToolScope },
    ProtectedGit { action: ProtectedGitAction },
}

impl fmt::Display for PolicyAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StartExecution => formatter.write_str("start_execution"),
            Self::Tool { scope } => write!(formatter, "tool:{scope}"),
            Self::ProtectedGit { action } => write!(formatter, "protected_git:{action}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyDecision {
    pub schema: SchemaMetadata,
    pub id: PolicyDecisionId,
    pub project_id: ProjectId,
    pub work_item_id: Option<WorkItemId>,
    #[serde(default)]
    pub action: Option<PolicyAction>,
    pub decision: PolicyDecisionKind,
    pub actor: String,
    pub input_summary: String,
    pub outcome_summary: String,
    pub reason: String,
    pub decided_at: String,
}

impl VersionedSchema for PolicyDecision {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
