use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::domain::{Dependency, DependencyId, DependencyKind, WorkItemId};

use super::{
    DependencyBlocker, DependencyBlockerReason, DependencyContextField, DependencyEligibility,
    DependencyGraphError, WorkItemProgress,
};

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
        self.dependencies_for(work_item_id, |dependency| {
            dependency.downstream_work_item_id == *work_item_id
        })
    }

    pub fn downstream_dependencies(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Dependency>, DependencyGraphError> {
        self.dependencies_for(work_item_id, |dependency| {
            dependency.upstream_work_item_id == *work_item_id
        })
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

        longest_paths
            .into_values()
            .max_by_key(|path| path.len())
            .unwrap_or_default()
    }

    fn dependencies_for(
        &self,
        work_item_id: &WorkItemId,
        predicate: impl Fn(&Dependency) -> bool,
    ) -> Result<Vec<Dependency>, DependencyGraphError> {
        self.require_registered_work_item(work_item_id)?;
        Ok(self
            .dependencies
            .values()
            .filter(|dependency| predicate(dependency))
            .cloned()
            .collect())
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
        self.work_item_ids
            .contains(work_item_id)
            .then_some(())
            .ok_or_else(|| DependencyGraphError::UnknownWorkItem {
                work_item_id: work_item_id.clone(),
            })
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
        let reason = match dependency.kind {
            DependencyKind::Blocks if !progress.has_accepted_completion() => {
                Some(DependencyBlockerReason::CompletionEvidenceNotAccepted)
            }
            DependencyKind::ReviewRequired if !progress.has_accepted_review() => {
                Some(DependencyBlockerReason::ReviewNotAccepted)
            }
            DependencyKind::Blocks
            | DependencyKind::ReviewRequired
            | DependencyKind::Contract
            | DependencyKind::Soft => None,
        };
        reason.map(|reason| DependencyBlocker {
            dependency: dependency.clone(),
            reason,
        })
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
        let mut incoming_counts = self
            .work_item_ids
            .iter()
            .cloned()
            .map(|id| (id, 0_usize))
            .collect::<BTreeMap<_, _>>();
        for dependency in self
            .dependencies
            .values()
            .filter(|dependency| dependency.kind.is_hard())
        {
            *incoming_counts
                .get_mut(&dependency.downstream_work_item_id)
                .expect("all dependency endpoints are registered graph work items") += 1;
        }
        let mut available = incoming_counts
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>();
        let mut ordered = Vec::with_capacity(self.work_item_ids.len());
        while let Some(work_item_id) = available.pop_first() {
            ordered.push(work_item_id.clone());
            for dependency in self.outgoing_hard_dependencies(&work_item_id) {
                let count = incoming_counts
                    .get_mut(&dependency.downstream_work_item_id)
                    .expect("all dependency endpoints are registered graph work items");
                *count -= 1;
                if *count == 0 {
                    available.insert(dependency.downstream_work_item_id.clone());
                }
            }
        }
        debug_assert_eq!(ordered.len(), self.work_item_ids.len());
        ordered
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
