use std::fmt::Debug;

use crate::{
    application::BoardSnapshot,
    domain::{BoardSupervision, EvidenceResult, SupervisionAction, WorkItemState},
    orchestration::{
        BoardSupervisionInput, SupervisionActivity, SupervisionCandidate as InputCandidate,
        SupervisionDependency, SupervisionEvidence, SupervisionWorkItem, bounded_summary,
    },
};

use crate::desktop_execution_runtime_support::ExecutionRuntimeError;

#[derive(Clone, Debug)]
pub(super) struct SupervisionCandidate {
    pub(super) action: SupervisionAction,
    pub(super) work_item_id: String,
    pub(super) expected_sequence: u64,
    pub(super) recommendation: String,
    pub(super) rationale: String,
}

#[cfg(test)]
pub(super) fn next_candidate(
    snapshot: &BoardSnapshot,
    supervision: &BoardSupervision,
) -> Option<SupervisionCandidate> {
    candidates(snapshot, supervision).into_iter().next()
}

pub(super) fn candidates(
    snapshot: &BoardSnapshot,
    supervision: &BoardSupervision,
) -> Vec<SupervisionCandidate> {
    let mut candidates = Vec::new();
    for item in &snapshot.work_items {
        let action = match item.work_item.state {
            WorkItemState::Inbox => Some(SupervisionAction::PrepareWork),
            WorkItemState::Planned if dependencies_are_complete(snapshot, &item.work_item.id.0) => {
                Some(SupervisionAction::MakeWorkReady)
            }
            WorkItemState::Review if needs_correction(snapshot, &item.work_item.id.0) => {
                Some(SupervisionAction::ReturnForCorrection)
            }
            WorkItemState::Failed | WorkItemState::Interrupted
                if retry_is_available(snapshot, &item.work_item.id.0, supervision) =>
            {
                Some(SupervisionAction::RetryWork)
            }
            WorkItemState::Ready if dependencies_are_complete(snapshot, &item.work_item.id.0) => {
                Some(SupervisionAction::StartWork)
            }
            _ => None,
        };
        if let Some(action) = action.filter(|action| supervision.permitted_actions.contains(action))
        {
            candidates.push(candidate(item, action));
        }
    }
    candidates
}

pub(super) fn organiser_input(
    snapshot: &BoardSnapshot,
    candidates: &[SupervisionCandidate],
) -> Result<BoardSupervisionInput, ExecutionRuntimeError> {
    BoardSupervisionInput::new(
        snapshot
            .work_items
            .iter()
            .map(|item| SupervisionWorkItem {
                id: item.work_item.id.0.clone(),
                title: bounded_summary(&item.work_item.title),
                state: normalized_name(&item.work_item.state),
            })
            .collect(),
        snapshot
            .dependencies
            .iter()
            .map(|dependency| SupervisionDependency {
                upstream_work_item_id: dependency.upstream_work_item_id.0.clone(),
                downstream_work_item_id: dependency.downstream_work_item_id.0.clone(),
                kind: normalized_name(&dependency.kind),
            })
            .collect(),
        snapshot
            .activity
            .iter()
            .rev()
            .take(crate::orchestration::MAX_SUPERVISION_ACTIVITY)
            .map(|activity| SupervisionActivity {
                work_item_id: activity.work_item_id.0.clone(),
                summary: bounded_summary(&activity.summary),
            })
            .collect(),
        snapshot
            .evidence
            .iter()
            .rev()
            .take(crate::orchestration::MAX_SUPERVISION_EVIDENCE)
            .map(|evidence| SupervisionEvidence {
                work_item_id: evidence.work_item_id.0.clone(),
                kind: normalized_name(&evidence.kind),
                result: normalized_name(&evidence.result),
                summary: bounded_summary(&evidence.summary),
            })
            .collect(),
        candidates
            .iter()
            .map(|candidate| InputCandidate {
                action: candidate.action,
                work_item_id: candidate.work_item_id.clone(),
            })
            .collect(),
    )
    .map_err(ExecutionRuntimeError::SupervisorInput)
}

pub(super) fn dependencies_are_complete(snapshot: &BoardSnapshot, work_item_id: &str) -> bool {
    snapshot
        .dependencies
        .iter()
        .filter(|dependency| {
            dependency.downstream_work_item_id.0 == work_item_id && dependency.kind.is_hard()
        })
        .all(|dependency| {
            snapshot.work_items.iter().any(|item| {
                item.work_item.id == dependency.upstream_work_item_id
                    && item.work_item.state == WorkItemState::Done
            })
        })
}

fn needs_correction(snapshot: &BoardSnapshot, work_item_id: &str) -> bool {
    snapshot.evidence.iter().any(|evidence| {
        evidence.work_item_id.0 == work_item_id && evidence.result == EvidenceResult::Failed
    })
}

fn execution_attempt_count(snapshot: &BoardSnapshot, work_item_id: &str) -> u32 {
    snapshot
        .executions
        .iter()
        .filter(|execution| execution.work_item_id.0 == work_item_id)
        .count() as u32
}

fn retry_is_available(
    snapshot: &BoardSnapshot,
    work_item_id: &str,
    supervision: &BoardSupervision,
) -> bool {
    let attempts = execution_attempt_count(snapshot, work_item_id);
    attempts > 0 && attempts <= supervision.limits.max_retries_per_work_item
}

fn candidate(
    item: &crate::domain::MaterializedWorkItem,
    action: SupervisionAction,
) -> SupervisionCandidate {
    let (recommendation, rationale) = match action {
        SupervisionAction::PrepareWork => (
            "Prepare this confirmed task for dependency scheduling.",
            "The task is confirmed but has not entered the dependency schedule.",
        ),
        SupervisionAction::MakeWorkReady => (
            "Make this task ready for a worker.",
            "All required upstream tasks are complete.",
        ),
        SupervisionAction::StartWork => (
            "Start this dependency-ready task with the selected ticket worker.",
            "All hard blockers are complete and the task is ready for execution.",
        ),
        SupervisionAction::RetryWork => (
            "Retry this recoverable task once within the recorded limit.",
            "The previous worker attempt ended without a completed review outcome.",
        ),
        SupervisionAction::ReturnForCorrection => (
            "Return this reviewed task to the worker for correction.",
            "Recorded review evidence has failed and the task remains reviewable.",
        ),
    };
    SupervisionCandidate {
        action,
        work_item_id: item.work_item.id.0.clone(),
        expected_sequence: item.last_event_sequence,
        recommendation: recommendation.to_owned(),
        rationale: rationale.to_owned(),
    }
}

fn normalized_name(value: &impl Debug) -> String {
    let mut normalized = String::new();
    for (index, character) in format!("{value:?}").chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            normalized.push('_');
        }
        normalized.push(character.to_ascii_lowercase());
    }
    normalized
}
