use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt,
};

use super::{Dependency, DependencyId, DependencyKind, WorkItemId, WorkItemState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkItemProgress {
    pub state: WorkItemState,
    pub completion_evidence_accepted: bool,
    pub review_accepted: bool,
}

impl WorkItemProgress {
    fn has_accepted_completion(self) -> bool {
        self.state == WorkItemState::Done && self.completion_evidence_accepted
    }

    fn has_accepted_review(self) -> bool {
        self.review_accepted && matches!(self.state, WorkItemState::Review | WorkItemState::Done)
    }

    fn is_ready_to_start(self) -> bool {
        self.state == WorkItemState::Ready
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyBlockerReason {
    UpstreamProgressUnavailable,
    CompletionEvidenceNotAccepted,
    ReviewNotAccepted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyContextField {
    Reason,
    Owner,
    NextAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyBlocker {
    pub dependency: Dependency,
    pub reason: DependencyBlockerReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEligibility {
    pub hard_blockers: Vec<DependencyBlocker>,
    pub advisories: Vec<Dependency>,
}

impl DependencyEligibility {
    pub const fn is_eligible(&self) -> bool {
        self.hard_blockers.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DependencyGraph {
    work_item_ids: BTreeSet<WorkItemId>,
    dependencies: BTreeMap<DependencyId, Dependency>,
}

impl DependencyGraph {
    pub fn new(work_item_ids: impl IntoIterator<Item = WorkItemId>) -> Self {
        Self {
            work_item_ids: work_item_ids.into_iter().collect(),
            dependencies: BTreeMap::new(),
        }
    }

    pub fn add_dependency(&mut self, dependency: Dependency) -> Result<(), DependencyGraphError> {
        self.require_registered_work_items(&dependency)?;
        self.reject_invalid_dependency(&dependency)?;
        self.dependencies.insert(dependency.id.clone(), dependency);
        Ok(())
    }

    pub fn upstream_dependencies(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Dependency>, DependencyGraphError> {
        self.require_registered_work_item(work_item_id)?;
        Ok(self
            .dependencies
            .values()
            .filter(|dependency| dependency.downstream_work_item_id == *work_item_id)
            .cloned()
            .collect())
    }

    pub fn downstream_dependencies(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Dependency>, DependencyGraphError> {
        self.require_registered_work_item(work_item_id)?;
        Ok(self
            .dependencies
            .values()
            .filter(|dependency| dependency.upstream_work_item_id == *work_item_id)
            .cloned()
            .collect())
    }

    pub fn evaluate_eligibility(
        &self,
        work_item_id: &WorkItemId,
        progress_by_work_item: &BTreeMap<WorkItemId, WorkItemProgress>,
    ) -> Result<DependencyEligibility, DependencyGraphError> {
        let dependencies = self.upstream_dependencies(work_item_id)?;
        let hard_blockers = dependencies
            .iter()
            .filter(|dependency| dependency.kind.is_hard())
            .filter_map(|dependency| {
                Self::hard_dependency_blocker(dependency, progress_by_work_item)
            })
            .collect();
        let advisories = dependencies
            .into_iter()
            .filter(|dependency| !dependency.kind.is_hard())
            .collect();

        Ok(DependencyEligibility {
            hard_blockers,
            advisories,
        })
    }

    pub fn dependency_safe_ready_work_items(
        &self,
        progress_by_work_item: &BTreeMap<WorkItemId, WorkItemProgress>,
    ) -> Vec<WorkItemId> {
        self.work_item_ids
            .iter()
            .filter(|work_item_id| {
                progress_by_work_item
                    .get(*work_item_id)
                    .is_some_and(|progress| progress.is_ready_to_start())
            })
            .filter(|work_item_id| {
                self.evaluate_eligibility(work_item_id, progress_by_work_item)
                    .expect("every graph work item can be evaluated")
                    .is_eligible()
            })
            .cloned()
            .collect()
    }

    pub fn critical_path(&self) -> Vec<WorkItemId> {
        let mut longest_paths = self
            .work_item_ids
            .iter()
            .cloned()
            .map(|work_item_id| (work_item_id.clone(), vec![work_item_id]))
            .collect::<BTreeMap<_, _>>();

        for work_item_id in self.hard_topological_order() {
            let path_to_work_item = longest_paths
                .get(&work_item_id)
                .cloned()
                .expect("all graph work items have a critical-path entry");

            for dependency in self.outgoing_hard_dependencies(&work_item_id) {
                let mut candidate_path = path_to_work_item.clone();
                candidate_path.push(dependency.downstream_work_item_id.clone());
                let current_path = longest_paths
                    .get(&dependency.downstream_work_item_id)
                    .expect("all graph work items have a critical-path entry");

                if candidate_path.len() > current_path.len() {
                    longest_paths
                        .insert(dependency.downstream_work_item_id.clone(), candidate_path);
                }
            }
        }

        // Planning has no duration estimates yet, so each hard dependency has equal weight.
        longest_paths
            .into_values()
            .max_by_key(|path| path.len())
            .unwrap_or_default()
    }

    fn require_registered_work_items(
        &self,
        dependency: &Dependency,
    ) -> Result<(), DependencyGraphError> {
        self.require_registered_work_item(&dependency.upstream_work_item_id)?;
        self.require_registered_work_item(&dependency.downstream_work_item_id)
    }

    fn require_registered_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<(), DependencyGraphError> {
        if self.work_item_ids.contains(work_item_id) {
            Ok(())
        } else {
            Err(DependencyGraphError::UnknownWorkItem {
                work_item_id: work_item_id.clone(),
            })
        }
    }

    fn reject_invalid_dependency(
        &self,
        dependency: &Dependency,
    ) -> Result<(), DependencyGraphError> {
        if self.dependencies.contains_key(&dependency.id) {
            return Err(DependencyGraphError::DuplicateDependencyId {
                dependency_id: dependency.id.clone(),
            });
        }

        if dependency.upstream_work_item_id == dependency.downstream_work_item_id {
            return Err(DependencyGraphError::SelfDependency {
                dependency_id: dependency.id.clone(),
                work_item_id: dependency.upstream_work_item_id.clone(),
            });
        }

        if let Some(field) = missing_hard_dependency_context(dependency) {
            return Err(DependencyGraphError::MissingHardDependencyContext {
                dependency_id: dependency.id.clone(),
                field,
            });
        }

        if dependency.kind.is_hard()
            && let Some(path) = self.hard_path(
                &dependency.downstream_work_item_id,
                &dependency.upstream_work_item_id,
            )
        {
            let mut cycle = Vec::with_capacity(path.len() + 1);
            cycle.push(dependency.upstream_work_item_id.clone());
            cycle.extend(path);
            return Err(DependencyGraphError::HardDependencyCycle {
                dependency_id: dependency.id.clone(),
                cycle,
                suggested_upstream_work_item_id: dependency.downstream_work_item_id.clone(),
                suggested_downstream_work_item_id: dependency.upstream_work_item_id.clone(),
            });
        }

        Ok(())
    }

    fn hard_dependency_blocker(
        dependency: &Dependency,
        progress_by_work_item: &BTreeMap<WorkItemId, WorkItemProgress>,
    ) -> Option<DependencyBlocker> {
        let Some(progress) = progress_by_work_item.get(&dependency.upstream_work_item_id) else {
            return Some(DependencyBlocker {
                dependency: dependency.clone(),
                reason: DependencyBlockerReason::UpstreamProgressUnavailable,
            });
        };

        let is_satisfied = match dependency.kind {
            DependencyKind::Blocks => progress.has_accepted_completion(),
            DependencyKind::ReviewRequired => progress.has_accepted_review(),
            DependencyKind::Contract | DependencyKind::Soft => true,
        };

        if is_satisfied {
            None
        } else {
            Some(DependencyBlocker {
                dependency: dependency.clone(),
                reason: match dependency.kind {
                    DependencyKind::Blocks => {
                        DependencyBlockerReason::CompletionEvidenceNotAccepted
                    }
                    DependencyKind::ReviewRequired => DependencyBlockerReason::ReviewNotAccepted,
                    DependencyKind::Contract | DependencyKind::Soft => {
                        unreachable!("only hard dependencies are evaluated as blockers")
                    }
                },
            })
        }
    }

    fn hard_path(&self, start: &WorkItemId, target: &WorkItemId) -> Option<Vec<WorkItemId>> {
        let mut parents = BTreeMap::from([(start.clone(), None)]);
        let mut candidates = VecDeque::from([start.clone()]);

        while let Some(work_item_id) = candidates.pop_front() {
            if &work_item_id == target {
                return Some(path_from_parents(&parents, target));
            }

            for dependency in self.outgoing_hard_dependencies(&work_item_id) {
                let downstream_work_item_id = dependency.downstream_work_item_id.clone();
                if parents.contains_key(&downstream_work_item_id) {
                    continue;
                }

                parents.insert(downstream_work_item_id.clone(), Some(work_item_id.clone()));
                candidates.push_back(downstream_work_item_id);
            }
        }

        None
    }

    fn hard_topological_order(&self) -> Vec<WorkItemId> {
        let mut incoming_hard_dependency_counts = self
            .work_item_ids
            .iter()
            .cloned()
            .map(|work_item_id| (work_item_id, 0_usize))
            .collect::<BTreeMap<_, _>>();

        for dependency in self
            .dependencies
            .values()
            .filter(|dependency| dependency.kind.is_hard())
        {
            let count = incoming_hard_dependency_counts
                .get_mut(&dependency.downstream_work_item_id)
                .expect("all dependency endpoints are registered graph work items");
            *count += 1;
        }

        let mut work_items_without_prerequisites = incoming_hard_dependency_counts
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(work_item_id, _)| work_item_id.clone())
            .collect::<BTreeSet<_>>();
        let mut ordered_work_items = Vec::with_capacity(self.work_item_ids.len());

        while let Some(work_item_id) = work_items_without_prerequisites.pop_first() {
            ordered_work_items.push(work_item_id.clone());

            for dependency in self.outgoing_hard_dependencies(&work_item_id) {
                let count = incoming_hard_dependency_counts
                    .get_mut(&dependency.downstream_work_item_id)
                    .expect("all dependency endpoints are registered graph work items");
                *count -= 1;
                if *count == 0 {
                    work_items_without_prerequisites
                        .insert(dependency.downstream_work_item_id.clone());
                }
            }
        }

        debug_assert_eq!(ordered_work_items.len(), self.work_item_ids.len());
        ordered_work_items
    }

    fn outgoing_hard_dependencies(
        &self,
        work_item_id: &WorkItemId,
    ) -> impl Iterator<Item = &Dependency> {
        self.dependencies.values().filter(move |dependency| {
            dependency.kind.is_hard() && dependency.upstream_work_item_id == *work_item_id
        })
    }
}

fn path_from_parents(
    parents: &BTreeMap<WorkItemId, Option<WorkItemId>>,
    target: &WorkItemId,
) -> Vec<WorkItemId> {
    let mut path = vec![target.clone()];
    let mut current = target;

    while let Some(Some(parent)) = parents.get(current) {
        path.push(parent.clone());
        current = parent;
    }

    path.reverse();
    path
}

fn missing_hard_dependency_context(dependency: &Dependency) -> Option<DependencyContextField> {
    if !dependency.kind.is_hard() {
        return None;
    }

    if dependency.reason.trim().is_empty() {
        Some(DependencyContextField::Reason)
    } else if dependency.owner.trim().is_empty() {
        Some(DependencyContextField::Owner)
    } else if dependency.next_action.trim().is_empty() {
        Some(DependencyContextField::NextAction)
    } else {
        None
    }
}

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
                dependency_context_field_name(*field)
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
            } => {
                let cycle_path = cycle
                    .iter()
                    .map(|work_item_id| work_item_id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(
                    formatter,
                    "hard dependency {} would create cycle {}; remove it or reverse it to {} -> {}",
                    dependency_id.0,
                    cycle_path,
                    suggested_upstream_work_item_id.0,
                    suggested_downstream_work_item_id.0
                )
            }
        }
    }
}

impl Error for DependencyGraphError {}

fn dependency_context_field_name(field: DependencyContextField) -> &'static str {
    match field {
        DependencyContextField::Reason => "reason",
        DependencyContextField::Owner => "owner",
        DependencyContextField::NextAction => "next action",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        DependencyBlockerReason, DependencyContextField, DependencyGraph, DependencyGraphError,
        WorkItemProgress,
    };
    use crate::domain::{
        Dependency, DependencyId, DependencyKind, DependencySource, SchemaMetadata, WorkItemId,
        WorkItemState,
    };

    fn work_item_id(value: &str) -> WorkItemId {
        WorkItemId::from(value)
    }

    fn progress(
        state: WorkItemState,
        completion_evidence_accepted: bool,
        review_accepted: bool,
    ) -> WorkItemProgress {
        WorkItemProgress {
            state,
            completion_evidence_accepted,
            review_accepted,
        }
    }

    fn dependency(
        id: &str,
        upstream_work_item_id: &str,
        downstream_work_item_id: &str,
        kind: DependencyKind,
    ) -> Dependency {
        Dependency {
            schema: SchemaMetadata::current(),
            id: DependencyId::from(id),
            upstream_work_item_id: work_item_id(upstream_work_item_id),
            downstream_work_item_id: work_item_id(downstream_work_item_id),
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
        DependencyGraph::new(work_item_ids.iter().map(|id| work_item_id(id)))
    }

    #[test]
    fn requires_every_incoming_block_dependency_to_have_accepted_completion_evidence() {
        let mut graph = graph(&["schema", "api", "worker"]);
        graph
            .add_dependency(dependency(
                "schema-worker",
                "schema",
                "worker",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");
        graph
            .add_dependency(dependency(
                "api-worker",
                "api",
                "worker",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");
        let progress_by_work_item = BTreeMap::from([
            (
                work_item_id("schema"),
                progress(WorkItemState::Done, true, false),
            ),
            (
                work_item_id("api"),
                progress(WorkItemState::Done, false, false),
            ),
        ]);

        let eligibility = graph
            .evaluate_eligibility(&work_item_id("worker"), &progress_by_work_item)
            .expect("registered work item should be evaluated");

        assert!(!eligibility.is_eligible());
        assert_eq!(eligibility.hard_blockers.len(), 1);
        assert_eq!(
            eligibility.hard_blockers[0].reason,
            DependencyBlockerReason::CompletionEvidenceNotAccepted
        );
        assert_eq!(
            eligibility.hard_blockers[0].dependency.id,
            DependencyId::from("api-worker")
        );
    }

    #[test]
    fn requires_accepted_review_for_review_required_dependencies() {
        let mut graph = graph(&["review", "worker"]);
        graph
            .add_dependency(dependency(
                "review-worker",
                "review",
                "worker",
                DependencyKind::ReviewRequired,
            ))
            .expect("registered hard dependency should be accepted");
        let progress_by_work_item = BTreeMap::from([(
            work_item_id("review"),
            progress(WorkItemState::Review, true, false),
        )]);

        let eligibility = graph
            .evaluate_eligibility(&work_item_id("worker"), &progress_by_work_item)
            .expect("registered work item should be evaluated");

        assert_eq!(
            eligibility.hard_blockers,
            vec![super::DependencyBlocker {
                dependency: dependency(
                    "review-worker",
                    "review",
                    "worker",
                    DependencyKind::ReviewRequired,
                ),
                reason: DependencyBlockerReason::ReviewNotAccepted,
            }]
        );
        let accepted_progress = BTreeMap::from([(
            work_item_id("review"),
            progress(WorkItemState::Review, true, true),
        )]);
        assert!(
            graph
                .evaluate_eligibility(&work_item_id("worker"), &accepted_progress)
                .expect("registered work item should be evaluated")
                .is_eligible()
        );
    }

    #[test]
    fn reports_missing_upstream_progress_as_a_blocker() {
        let mut graph = graph(&["upstream", "downstream"]);
        graph
            .add_dependency(dependency(
                "upstream-downstream",
                "upstream",
                "downstream",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");

        let eligibility = graph
            .evaluate_eligibility(&work_item_id("downstream"), &BTreeMap::new())
            .expect("registered work item should be evaluated");

        assert_eq!(
            eligibility.hard_blockers[0].reason,
            DependencyBlockerReason::UpstreamProgressUnavailable
        );
    }

    #[test]
    fn surfaces_contract_and_soft_dependencies_as_non_blocking_advisories() {
        let mut graph = graph(&["contract", "soft", "worker"]);
        graph
            .add_dependency(dependency(
                "contract-worker",
                "contract",
                "worker",
                DependencyKind::Contract,
            ))
            .expect("registered contract dependency should be accepted");
        graph
            .add_dependency(dependency(
                "soft-worker",
                "soft",
                "worker",
                DependencyKind::Soft,
            ))
            .expect("registered soft dependency should be accepted");

        let eligibility = graph
            .evaluate_eligibility(&work_item_id("worker"), &BTreeMap::new())
            .expect("registered work item should be evaluated");

        assert!(eligibility.is_eligible());
        assert_eq!(
            eligibility
                .advisories
                .iter()
                .map(|dependency| dependency.kind)
                .collect::<Vec<_>>(),
            vec![DependencyKind::Contract, DependencyKind::Soft]
        );
    }

    #[test]
    fn returns_dependency_safe_ready_work_items_and_their_graph_neighbours() {
        let mut graph = graph(&["schema", "api", "worker"]);
        graph
            .add_dependency(dependency(
                "schema-worker",
                "schema",
                "worker",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");
        graph
            .add_dependency(dependency(
                "api-worker",
                "api",
                "worker",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");
        let progress_by_work_item = BTreeMap::from([
            (
                work_item_id("schema"),
                progress(WorkItemState::Ready, false, false),
            ),
            (
                work_item_id("api"),
                progress(WorkItemState::Ready, false, false),
            ),
            (
                work_item_id("worker"),
                progress(WorkItemState::Ready, false, false),
            ),
        ]);

        assert_eq!(
            graph.dependency_safe_ready_work_items(&progress_by_work_item),
            vec![work_item_id("api"), work_item_id("schema")]
        );
        assert_eq!(
            graph
                .upstream_dependencies(&work_item_id("worker"))
                .expect("registered work item should expose upstream dependencies")
                .iter()
                .map(|dependency| dependency.id.clone())
                .collect::<Vec<_>>(),
            vec![
                DependencyId::from("api-worker"),
                DependencyId::from("schema-worker")
            ]
        );
        assert_eq!(
            graph
                .downstream_dependencies(&work_item_id("schema"))
                .expect("registered work item should expose downstream dependencies")
                .iter()
                .map(|dependency| dependency.downstream_work_item_id.clone())
                .collect::<Vec<_>>(),
            vec![work_item_id("worker")]
        );
    }

    #[test]
    fn calculates_the_longest_hard_dependency_path() {
        let mut graph = graph(&["api", "database", "domain", "worker"]);
        graph
            .add_dependency(dependency(
                "api-domain",
                "api",
                "domain",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");
        graph
            .add_dependency(dependency(
                "domain-worker",
                "domain",
                "worker",
                DependencyKind::ReviewRequired,
            ))
            .expect("registered hard dependency should be accepted");
        graph
            .add_dependency(dependency(
                "database-worker",
                "database",
                "worker",
                DependencyKind::Blocks,
            ))
            .expect("registered hard dependency should be accepted");

        assert_eq!(
            graph.critical_path(),
            vec![
                work_item_id("api"),
                work_item_id("domain"),
                work_item_id("worker")
            ]
        );
    }

    #[test]
    fn permits_non_hard_cycles_without_treating_them_as_scheduling_prerequisites() {
        let mut graph = graph(&["contract-a", "contract-b"]);
        graph
            .add_dependency(dependency(
                "a-b-contract",
                "contract-a",
                "contract-b",
                DependencyKind::Contract,
            ))
            .expect("contract dependency should be accepted");
        graph
            .add_dependency(dependency(
                "b-a-soft",
                "contract-b",
                "contract-a",
                DependencyKind::Soft,
            ))
            .expect("soft dependency should be accepted");

        assert_eq!(graph.critical_path(), vec![work_item_id("contract-b")]);
    }

    #[test]
    fn rejects_invalid_dependency_insertions_with_actionable_errors() {
        let mut graph = graph(&["a", "b", "c"]);
        assert_eq!(
            graph.add_dependency(dependency(
                "unknown",
                "missing",
                "a",
                DependencyKind::Blocks,
            )),
            Err(DependencyGraphError::UnknownWorkItem {
                work_item_id: work_item_id("missing"),
            })
        );
        let mut missing_reason = dependency("missing-reason", "a", "b", DependencyKind::Blocks);
        missing_reason.reason = "  ".to_owned();
        assert_eq!(
            graph.add_dependency(missing_reason),
            Err(DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-reason"),
                field: DependencyContextField::Reason,
            })
        );
        let mut missing_owner = dependency("missing-owner", "a", "b", DependencyKind::Blocks);
        missing_owner.owner.clear();
        assert_eq!(
            graph.add_dependency(missing_owner),
            Err(DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-owner"),
                field: DependencyContextField::Owner,
            })
        );
        let mut missing_next_action =
            dependency("missing-next-action", "a", "b", DependencyKind::Blocks);
        missing_next_action.next_action.clear();
        assert_eq!(
            graph.add_dependency(missing_next_action),
            Err(DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-next-action"),
                field: DependencyContextField::NextAction,
            })
        );
        assert_eq!(
            graph.add_dependency(dependency("self", "a", "a", DependencyKind::Blocks)),
            Err(DependencyGraphError::SelfDependency {
                dependency_id: DependencyId::from("self"),
                work_item_id: work_item_id("a"),
            })
        );
        graph
            .add_dependency(dependency("a-b", "a", "b", DependencyKind::Blocks))
            .expect("first registered dependency should be accepted");
        assert_eq!(
            graph.add_dependency(dependency("a-b", "a", "c", DependencyKind::Blocks)),
            Err(DependencyGraphError::DuplicateDependencyId {
                dependency_id: DependencyId::from("a-b"),
            })
        );
    }

    #[test]
    fn rejects_hard_cycles_with_the_existing_path_and_a_safe_alternative() {
        let mut graph = graph(&["a", "b", "c"]);
        graph
            .add_dependency(dependency("a-b", "a", "b", DependencyKind::Blocks))
            .expect("first hard dependency should be accepted");
        graph
            .add_dependency(dependency("b-c", "b", "c", DependencyKind::ReviewRequired))
            .expect("second hard dependency should be accepted");

        assert_eq!(
            graph.add_dependency(dependency("c-a", "c", "a", DependencyKind::Blocks)),
            Err(DependencyGraphError::HardDependencyCycle {
                dependency_id: DependencyId::from("c-a"),
                cycle: vec![
                    work_item_id("c"),
                    work_item_id("a"),
                    work_item_id("b"),
                    work_item_id("c")
                ],
                suggested_upstream_work_item_id: work_item_id("a"),
                suggested_downstream_work_item_id: work_item_id("c"),
            })
        );
        assert_eq!(
            DependencyGraphError::HardDependencyCycle {
                dependency_id: DependencyId::from("c-a"),
                cycle: vec![
                    work_item_id("c"),
                    work_item_id("a"),
                    work_item_id("b"),
                    work_item_id("c")
                ],
                suggested_upstream_work_item_id: work_item_id("a"),
                suggested_downstream_work_item_id: work_item_id("c"),
            }
            .to_string(),
            "hard dependency c-a would create cycle c -> a -> b -> c; remove it or reverse it to a -> c"
        );
    }

    #[test]
    fn formats_the_remaining_validation_errors_for_people() {
        assert_eq!(
            DependencyGraphError::UnknownWorkItem {
                work_item_id: work_item_id("missing"),
            }
            .to_string(),
            "work item missing is not registered in this dependency graph"
        );
        assert_eq!(
            DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-reason"),
                field: DependencyContextField::Reason,
            }
            .to_string(),
            "hard dependency missing-reason requires a non-empty reason"
        );
        assert_eq!(
            DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-owner"),
                field: DependencyContextField::Owner,
            }
            .to_string(),
            "hard dependency missing-owner requires a non-empty owner"
        );
        assert_eq!(
            DependencyGraphError::MissingHardDependencyContext {
                dependency_id: DependencyId::from("missing-next-action"),
                field: DependencyContextField::NextAction,
            }
            .to_string(),
            "hard dependency missing-next-action requires a non-empty next action"
        );
        assert_eq!(
            DependencyGraphError::DuplicateDependencyId {
                dependency_id: DependencyId::from("duplicate"),
            }
            .to_string(),
            "dependency id duplicate already exists"
        );
        assert_eq!(
            DependencyGraphError::SelfDependency {
                dependency_id: DependencyId::from("self"),
                work_item_id: work_item_id("a"),
            }
            .to_string(),
            "dependency self cannot make work item a depend on itself"
        );
    }
}
