use super::{
    Board, BoardId, Dependency, DependencyId, DependencyKind, DependencySource, Evidence,
    EvidenceId, EvidenceKind, EvidenceResult, Execution, ExecutionId, ExecutionStatus,
    ExecutionUsage, ExternalLink, ExternalLinkId, ExternalLinkProvenance, PolicyAction,
    PolicyDecision, PolicyDecisionId, PolicyDecisionKind, Project, ProjectId, ProtectedGitAction,
    SchemaMetadata, ToolScope, VersionedSchema, WorkItem, WorkItemBudget, WorkItemId,
    WorkItemState,
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

#[test]
fn execution_lifecycle_keeps_terminal_attempts_distinct_from_recoverable_progress() {
    assert!(ExecutionStatus::Completed.is_terminal());
    assert!(ExecutionStatus::Failed.is_terminal());
    assert!(!ExecutionStatus::AwaitingInput.is_terminal());
    assert!(ExecutionStatus::Pending.allows_transition_to(ExecutionStatus::Running));
    assert!(ExecutionStatus::Running.allows_transition_to(ExecutionStatus::AwaitingReview));
    assert!(ExecutionStatus::AwaitingInput.allows_transition_to(ExecutionStatus::Running));
    assert!(!ExecutionStatus::Completed.allows_transition_to(ExecutionStatus::Running));
}
