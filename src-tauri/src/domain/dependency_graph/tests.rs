use std::collections::BTreeMap;

use super::{
    DependencyBlockerReason, DependencyContextField, DependencyGraph, DependencyGraphError,
    WorkItemProgress,
};
use crate::domain::{
    Dependency, DependencyId, DependencyKind, DependencySource, SchemaMetadata, WorkItemId,
    WorkItemState,
};

fn id(value: &str) -> WorkItemId {
    WorkItemId::from(value)
}

fn progress(state: WorkItemState, evidence: bool, review: bool) -> WorkItemProgress {
    WorkItemProgress {
        state,
        completion_evidence_accepted: evidence,
        review_accepted: review,
    }
}

fn dependency(
    id_value: &str,
    upstream: &str,
    downstream: &str,
    kind: DependencyKind,
) -> Dependency {
    Dependency {
        schema: SchemaMetadata::current(),
        id: DependencyId::from(id_value),
        upstream_work_item_id: id(upstream),
        downstream_work_item_id: id(downstream),
        kind,
        source: DependencySource::Orchestrator,
        reason: "The downstream task needs the upstream result.".to_owned(),
        owner: "planner".to_owned(),
        next_action: "Complete the upstream task.".to_owned(),
        created_by: "orchestrator".to_owned(),
        created_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn graph(work_item_ids: &[&str]) -> DependencyGraph {
    DependencyGraph::new(work_item_ids.iter().map(|value| id(value)))
}

#[test]
fn blocks_on_incomplete_evidence_and_unaccepted_review() {
    let mut graph = graph(&["implementation", "review", "release"]);
    graph
        .add_dependency(dependency(
            "implementation-release",
            "implementation",
            "release",
            DependencyKind::Blocks,
        ))
        .expect("dependency should register");
    graph
        .add_dependency(dependency(
            "review-release",
            "review",
            "release",
            DependencyKind::ReviewRequired,
        ))
        .expect("dependency should register");

    let progress_by_work_item = BTreeMap::from([
        (
            id("implementation"),
            progress(WorkItemState::Done, false, false),
        ),
        (id("review"), progress(WorkItemState::Review, true, false)),
    ]);
    let eligibility = graph
        .evaluate_eligibility(&id("release"), &progress_by_work_item)
        .expect("registered item should evaluate");

    assert_eq!(
        eligibility
            .hard_blockers
            .iter()
            .map(|blocker| blocker.reason)
            .collect::<Vec<_>>(),
        vec![
            DependencyBlockerReason::CompletionEvidenceNotAccepted,
            DependencyBlockerReason::ReviewNotAccepted
        ]
    );
    assert!(!eligibility.is_eligible());
}

#[test]
fn identifies_missing_progress_as_a_blocker_and_soft_edges_as_advice() {
    let mut graph = graph(&["upstream", "advice", "downstream"]);
    graph
        .add_dependency(dependency(
            "hard",
            "upstream",
            "downstream",
            DependencyKind::Blocks,
        ))
        .expect("dependency should register");
    graph
        .add_dependency(dependency(
            "soft",
            "advice",
            "downstream",
            DependencyKind::Soft,
        ))
        .expect("dependency should register");

    let eligibility = graph
        .evaluate_eligibility(&id("downstream"), &BTreeMap::new())
        .expect("registered item should evaluate");
    assert_eq!(
        eligibility.hard_blockers[0].reason,
        DependencyBlockerReason::UpstreamProgressUnavailable
    );
    assert_eq!(eligibility.advisories[0].id, DependencyId::from("soft"));
}

#[test]
fn finds_ready_unblocked_items_and_critical_path() {
    let mut graph = graph(&["api", "database", "domain", "worker"]);
    graph
        .add_dependency(dependency(
            "api-domain",
            "api",
            "domain",
            DependencyKind::Blocks,
        ))
        .expect("dependency should register");
    graph
        .add_dependency(dependency(
            "domain-worker",
            "domain",
            "worker",
            DependencyKind::ReviewRequired,
        ))
        .expect("dependency should register");
    graph
        .add_dependency(dependency(
            "database-worker",
            "database",
            "worker",
            DependencyKind::Blocks,
        ))
        .expect("dependency should register");
    let progress_by_work_item = BTreeMap::from([
        (id("api"), progress(WorkItemState::Ready, false, false)),
        (id("database"), progress(WorkItemState::Ready, false, false)),
        (id("domain"), progress(WorkItemState::Ready, false, false)),
        (id("worker"), progress(WorkItemState::Ready, false, false)),
    ]);

    assert_eq!(
        graph.dependency_safe_ready_work_items(&progress_by_work_item),
        vec![id("api"), id("database")]
    );
    assert_eq!(
        graph.critical_path(),
        vec![id("api"), id("domain"), id("worker")]
    );
}

#[test]
fn preserves_non_hard_cycles_as_nonblocking_relationships() {
    let mut graph = graph(&["a", "b"]);
    graph
        .add_dependency(dependency("a-b", "a", "b", DependencyKind::Contract))
        .expect("contract edge should register");
    graph
        .add_dependency(dependency("b-a", "b", "a", DependencyKind::Soft))
        .expect("soft edge should register");

    assert!(
        graph
            .evaluate_eligibility(&id("a"), &BTreeMap::new())
            .expect("registered item should evaluate")
            .is_eligible()
    );
    assert_eq!(graph.critical_path(), vec![id("b")]);
}

#[test]
fn rejects_invalid_edges_and_explains_cycles() {
    let mut graph = graph(&["a", "b", "c"]);
    assert_eq!(
        graph.add_dependency(dependency(
            "unknown",
            "missing",
            "a",
            DependencyKind::Blocks
        )),
        Err(DependencyGraphError::UnknownWorkItem {
            work_item_id: id("missing")
        })
    );
    let mut missing_context = dependency("missing-context", "a", "b", DependencyKind::Blocks);
    missing_context.reason.clear();
    assert_eq!(
        graph.add_dependency(missing_context),
        Err(DependencyGraphError::MissingHardDependencyContext {
            dependency_id: DependencyId::from("missing-context"),
            field: DependencyContextField::Reason
        })
    );
    assert_eq!(
        graph.add_dependency(dependency("self", "a", "a", DependencyKind::Blocks)),
        Err(DependencyGraphError::SelfDependency {
            dependency_id: DependencyId::from("self"),
            work_item_id: id("a")
        })
    );
    graph
        .add_dependency(dependency("a-b", "a", "b", DependencyKind::Blocks))
        .expect("edge should register");
    graph
        .add_dependency(dependency("b-c", "b", "c", DependencyKind::ReviewRequired))
        .expect("edge should register");

    let error = graph
        .add_dependency(dependency("c-a", "c", "a", DependencyKind::Blocks))
        .expect_err("cycle must be rejected");
    assert!(matches!(
        error,
        DependencyGraphError::HardDependencyCycle { .. }
    ));
    assert_eq!(
        error.to_string(),
        "hard dependency c-a would create cycle c -> a -> b -> c; remove it or reverse it to a -> c"
    );
}

#[test]
fn rejects_duplicate_edge_ids_and_unknown_queries() {
    let mut graph = graph(&["a", "b"]);
    graph
        .add_dependency(dependency("a-b", "a", "b", DependencyKind::Blocks))
        .expect("edge should register");

    assert_eq!(
        graph.add_dependency(dependency("a-b", "a", "b", DependencyKind::Blocks)),
        Err(DependencyGraphError::DuplicateDependencyId {
            dependency_id: DependencyId::from("a-b")
        })
    );
    assert_eq!(
        graph.upstream_dependencies(&id("missing")),
        Err(DependencyGraphError::UnknownWorkItem {
            work_item_id: id("missing")
        })
    );
    assert_eq!(
        graph
            .downstream_dependencies(&id("a"))
            .expect("registered item should query")
            .len(),
        1
    );
}
