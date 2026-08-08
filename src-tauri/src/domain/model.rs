use std::fmt;

use serde::{Deserialize, Serialize};

pub type SchemaVersion = u16;

pub const CURRENT_SCHEMA_VERSION: SchemaVersion = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMetadata {
    pub version: SchemaVersion,
}

impl SchemaMetadata {
    pub const fn current() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
        }
    }

    pub const fn is_current(self) -> bool {
        self.version == CURRENT_SCHEMA_VERSION
    }
}

pub trait VersionedSchema {
    fn schema(&self) -> SchemaMetadata;

    fn uses_current_schema(&self) -> bool {
        self.schema().is_current()
    }
}

macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

domain_id!(ProjectId);
domain_id!(BoardId);
domain_id!(WorkItemId);
domain_id!(ExecutionId);
domain_id!(EvidenceId);
domain_id!(PolicyDecisionId);
domain_id!(PlanId);
domain_id!(ExternalLinkId);
domain_id!(DependencyId);
domain_id!(WorkItemEventId);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema: SchemaMetadata,
    pub id: ProjectId,
    pub name: String,
    pub repository_path: String,
    pub base_ref: String,
    pub policy_set_id: String,
}

impl VersionedSchema for Project {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub schema: SchemaMetadata,
    pub id: BoardId,
    pub project_id: ProjectId,
    pub name: String,
}

impl VersionedSchema for Board {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Inbox,
    Planned,
    Ready,
    Running,
    AwaitingInput,
    Review,
    Done,
    Blocked,
    Failed,
    Cancelled,
    Interrupted,
}

impl WorkItemState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    pub const fn is_recoverable(self) -> bool {
        matches!(self, Self::Blocked | Self::Failed | Self::Interrupted)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemBudget {
    pub max_agent_turns: Option<u32>,
    pub max_duration_seconds: Option<u64>,
    pub max_cost_micros: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub schema: SchemaMetadata,
    pub id: WorkItemId,
    pub board_id: BoardId,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub budget: WorkItemBudget,
    pub state: WorkItemState,
    pub requires_human_review: bool,
}

impl VersionedSchema for WorkItem {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Blocks,
    ReviewRequired,
    Contract,
    Soft,
}

impl DependencyKind {
    pub const fn is_hard(self) -> bool {
        matches!(self, Self::Blocks | Self::ReviewRequired)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DependencySource {
    User,
    Orchestrator,
    Connector { connector_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub schema: SchemaMetadata,
    pub id: DependencyId,
    pub upstream_work_item_id: WorkItemId,
    pub downstream_work_item_id: WorkItemId,
    pub kind: DependencyKind,
    pub source: DependencySource,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
    pub created_by: String,
    pub created_at: String,
}

impl VersionedSchema for Dependency {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    AwaitingInput,
    AwaitingReview,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_micros: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub schema: SchemaMetadata,
    pub id: ExecutionId,
    pub work_item_id: WorkItemId,
    pub adapter_name: String,
    pub status: ExecutionStatus,
    pub session_id: Option<String>,
    pub workspace_path: String,
    pub usage: ExecutionUsage,
    pub last_event_sequence: u64,
}

impl VersionedSchema for Execution {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Check,
    Commit,
    PullRequest,
    CompletionReport,
    ReviewDecision,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceResult {
    Recorded,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub schema: SchemaMetadata,
    pub id: EvidenceId,
    pub work_item_id: WorkItemId,
    pub kind: EvidenceKind,
    pub result: EvidenceResult,
    pub summary: String,
    pub recorded_at: String,
}

impl VersionedSchema for Evidence {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalLinkProvenance {
    Imported,
    UserLinked,
    Synchronized,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLink {
    pub schema: SchemaMetadata,
    pub id: ExternalLinkId,
    pub work_item_id: WorkItemId,
    pub connector_id: String,
    pub provenance: ExternalLinkProvenance,
    pub external_id: String,
    pub url: String,
}

impl VersionedSchema for ExternalLink {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Board, BoardId, Dependency, DependencyId, DependencyKind, DependencySource, Evidence,
        EvidenceId, EvidenceKind, EvidenceResult, Execution, ExecutionId, ExecutionStatus,
        ExecutionUsage, ExternalLink, ExternalLinkId, ExternalLinkProvenance, PolicyAction,
        PolicyDecision, PolicyDecisionId, PolicyDecisionKind, Project, ProjectId,
        ProtectedGitAction, SchemaMetadata, ToolScope, VersionedSchema, WorkItem, WorkItemBudget,
        WorkItemId, WorkItemState,
    };

    #[test]
    fn versioned_domain_records_start_at_the_current_schema() {
        let project_id = ProjectId::from("project-1");
        let board_id = BoardId::from("board-1");
        let work_item_id = WorkItemId::from("work-item-1");

        let project = Project {
            schema: SchemaMetadata::current(),
            id: project_id.clone(),
            name: "Desktop app".to_owned(),
            repository_path: "/projects/desktop-app".to_owned(),
            base_ref: "main".to_owned(),
            policy_set_id: "standard".to_owned(),
        };
        let board = Board {
            schema: SchemaMetadata::current(),
            id: board_id.clone(),
            project_id: project_id.clone(),
            name: "MVP".to_owned(),
        };
        let work_item = WorkItem {
            schema: SchemaMetadata::current(),
            id: work_item_id.clone(),
            board_id,
            title: "Build the core".to_owned(),
            description: "Implement the durable domain layer.".to_owned(),
            acceptance_criteria: vec!["State transitions are guarded.".to_owned()],
            budget: WorkItemBudget {
                max_agent_turns: Some(20),
                max_duration_seconds: Some(1_800),
                max_cost_micros: Some(5_000_000),
            },
            state: WorkItemState::Inbox,
            requires_human_review: true,
        };
        let dependency = Dependency {
            schema: SchemaMetadata::current(),
            id: DependencyId::from("dependency-1"),
            upstream_work_item_id: work_item_id.clone(),
            downstream_work_item_id: WorkItemId::from("work-item-2"),
            kind: DependencyKind::Blocks,
            source: DependencySource::Orchestrator,
            reason: "The domain state must exist before scheduling.".to_owned(),
            owner: "orchestrator".to_owned(),
            next_action: "Finish the upstream task.".to_owned(),
            created_by: "planner".to_owned(),
            created_at: "2026-08-08T00:00:00Z".to_owned(),
        };
        let execution = Execution {
            schema: SchemaMetadata::current(),
            id: ExecutionId::from("execution-1"),
            work_item_id: work_item_id.clone(),
            adapter_name: "fake-agent".to_owned(),
            status: ExecutionStatus::Pending,
            session_id: None,
            workspace_path: "/projects/desktop-app/.worktrees/core".to_owned(),
            usage: ExecutionUsage {
                input_tokens: 1_000,
                output_tokens: 500,
                cost_micros: Some(100_000),
            },
            last_event_sequence: 4,
        };
        let evidence = Evidence {
            schema: SchemaMetadata::current(),
            id: EvidenceId::from("evidence-1"),
            work_item_id: work_item_id.clone(),
            kind: EvidenceKind::Check,
            result: EvidenceResult::Passed,
            summary: "All checks passed.".to_owned(),
            recorded_at: "2026-08-08T00:00:00Z".to_owned(),
        };
        let policy_decision = PolicyDecision {
            schema: SchemaMetadata::current(),
            id: PolicyDecisionId::from("policy-decision-1"),
            project_id,
            work_item_id: Some(work_item_id.clone()),
            action: Some(PolicyAction::ProtectedGit {
                action: ProtectedGitAction::Push,
            }),
            decision: PolicyDecisionKind::ApprovalRequired,
            actor: "user-1".to_owned(),
            input_summary: "Request to push main".to_owned(),
            outcome_summary: "Awaiting user approval".to_owned(),
            reason: "Pushes need approval.".to_owned(),
            decided_at: "2026-08-08T00:00:00Z".to_owned(),
        };
        let external_link = ExternalLink {
            schema: SchemaMetadata::current(),
            id: ExternalLinkId::from("external-link-1"),
            work_item_id,
            connector_id: "linear".to_owned(),
            provenance: ExternalLinkProvenance::Imported,
            external_id: "LIN-1".to_owned(),
            url: "https://linear.app/example/issue/LIN-1".to_owned(),
        };

        assert!(project.uses_current_schema());
        assert!(board.uses_current_schema());
        assert!(work_item.uses_current_schema());
        assert!(dependency.uses_current_schema());
        assert!(execution.uses_current_schema());
        assert!(evidence.uses_current_schema());
        assert!(policy_decision.uses_current_schema());
        assert!(external_link.uses_current_schema());
    }

    #[test]
    fn serialized_records_preserve_the_schema_version_for_future_migrations() {
        let work_item = WorkItem {
            schema: SchemaMetadata::current(),
            id: WorkItemId::from("work-item-1"),
            board_id: BoardId::from("board-1"),
            title: "Build the core".to_owned(),
            description: "Implement the durable domain layer.".to_owned(),
            acceptance_criteria: vec![],
            budget: WorkItemBudget::default(),
            state: WorkItemState::Inbox,
            requires_human_review: false,
        };

        let serialized = serde_json::to_value(work_item).expect("work item should serialize");

        assert_eq!(serialized["schema"]["version"], 1);
        assert_eq!(serialized["state"], "inbox");
    }

    #[test]
    fn policy_actions_are_typed_and_legacy_policy_decisions_remain_readable() {
        let legacy_decision: PolicyDecision = serde_json::from_value(serde_json::json!({
            "schema": { "version": 1 },
            "id": "policy-decision-1",
            "projectId": "project-1",
            "workItemId": "work-item-1",
            "decision": "allow",
            "actor": "user-1",
            "inputSummary": "A historical summary.",
            "outcomeSummary": "Execution proceeded.",
            "reason": "The previous policy allowed it.",
            "decidedAt": "2026-08-08T00:00:00Z"
        }))
        .expect("legacy policy decision should deserialize");

        assert_eq!(legacy_decision.action, None);
        assert_eq!(
            PolicyAction::Tool {
                scope: ToolScope::RunProjectChecks,
            }
            .to_string(),
            "tool:run_project_checks"
        );
        assert_eq!(
            [
                ProtectedGitAction::Commit,
                ProtectedGitAction::Push,
                ProtectedGitAction::Merge,
                ProtectedGitAction::ForcePush,
                ProtectedGitAction::DeleteBranch,
            ]
            .map(|action| action.to_string()),
            ["commit", "push", "merge", "force_push", "delete_branch"]
        );
    }

    #[test]
    fn serialized_dependencies_keep_connector_provenance_provider_neutral() {
        let dependency = Dependency {
            schema: SchemaMetadata::current(),
            id: DependencyId::from("dependency-1"),
            upstream_work_item_id: WorkItemId::from("upstream"),
            downstream_work_item_id: WorkItemId::from("downstream"),
            kind: DependencyKind::Blocks,
            source: DependencySource::Connector {
                connector_id: "linear".to_owned(),
            },
            reason: "The downstream task consumes the upstream API.".to_owned(),
            owner: "platform-team".to_owned(),
            next_action: "Publish the API contract.".to_owned(),
            created_by: "connector-sync".to_owned(),
            created_at: "2026-08-08T00:00:00Z".to_owned(),
        };

        let serialized = serde_json::to_value(dependency).expect("dependency should serialize");

        assert_eq!(serialized["schema"]["version"], 1);
        assert_eq!(serialized["kind"], "blocks");
        assert_eq!(serialized["source"]["kind"], "connector");
        assert_eq!(serialized["source"]["connector_id"], "linear");
    }

    #[test]
    fn state_categories_keep_recovery_states_distinct_from_terminal_states() {
        assert!(WorkItemState::Done.is_terminal());
        assert!(WorkItemState::Cancelled.is_terminal());
        assert!(!WorkItemState::Failed.is_terminal());
        assert!(WorkItemState::Blocked.is_recoverable());
        assert!(WorkItemState::Failed.is_recoverable());
        assert!(WorkItemState::Interrupted.is_recoverable());
        assert!(!WorkItemState::Review.is_recoverable());
        assert!(!SchemaMetadata { version: 0 }.is_current());
    }
}
