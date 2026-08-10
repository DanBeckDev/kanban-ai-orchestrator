#![cfg(unix)]

use std::time::Duration;

use tempfile::TempDir;

use crate::{
    domain::SupervisionAction,
    orchestration::{
        BoardSupervisionInput, BoardSupervisionInputError, ProcessBoardSupervisionError,
        ProcessBoardSupervisor, SupervisionCandidate, SupervisionWorkItem,
        SupervisorRecommendationError, bounded_summary,
    },
};

use super::PlannerProfile;

#[test]
fn accepts_only_safe_candidate_recommendations_and_redacts_secret_like_text() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let recommendation = ProcessBoardSupervisor::recommend(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"task-1\",\"recommendation\":\"Prepare ghp_verysecret.\",\"rationale\":\"token supersecret\"}'",
        ),
        repository.path(),
        &input(),
    )
    .expect("typed recommendation should parse");

    assert_eq!(recommendation.action, SupervisionAction::PrepareWork);
    assert!(recommendation.recommendation.contains("[redacted]"));
    assert!(recommendation.rationale.contains("[redacted]"));
    assert!(!recommendation.recommendation.contains("ghp_verysecret"));
}

#[test]
fn rejects_an_action_or_work_item_that_the_daemon_did_not_offer() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let result = ProcessBoardSupervisor::recommend(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"start_work\",\"workItemId\":\"task-2\",\"recommendation\":\"Start it.\",\"rationale\":\"It is ready.\"}'",
        ),
        repository.path(),
        &input(),
    );

    assert!(matches!(
        result,
        Err(ProcessBoardSupervisionError::InvalidRecommendation(
            SupervisorRecommendationError::UnsupportedCandidate { .. }
        ))
    ));
}

#[test]
fn rejects_malformed_or_empty_organiser_summaries_before_recording_them() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let malformed = ProcessBoardSupervisor::recommend(
        &profile("cat >/dev/null; printf '%s' '{not-json}'"),
        repository.path(),
        &input(),
    );
    assert!(matches!(
        malformed,
        Err(ProcessBoardSupervisionError::InvalidOutput)
    ));

    let empty_summary = ProcessBoardSupervisor::recommend(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"task-1\",\"recommendation\":\" \",\"rationale\":\"\\n\"}'",
        ),
        repository.path(),
        &input(),
    );
    assert!(matches!(
        empty_summary,
        Err(ProcessBoardSupervisionError::InvalidRecommendation(
            SupervisorRecommendationError::MissingSummary
        ))
    ));
}

#[test]
fn redacts_bearer_tokens_and_bounds_organiser_text() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let recommendation = ProcessBoardSupervisor::recommend(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"task-1\",\"recommendation\":\"Authorization: Bearer copied-secret\",\"rationale\":\"token copied-value\"}'",
        ),
        repository.path(),
        &input(),
    )
    .expect("safe recommendation should parse");

    assert_eq!(
        recommendation.recommendation,
        "[redacted] [redacted] [redacted]"
    );
    assert_eq!(recommendation.rationale, "[redacted] [redacted]");
    assert_eq!(bounded_summary("one\n two\tthree"), "one two three");
    assert_eq!(bounded_summary(&"x".repeat(700)).chars().count(), 600);
}

#[test]
fn rejects_invalid_profiles_unavailable_processes_and_excessive_context() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let invalid_profile = PlannerProfile {
        name: " ".to_owned(),
        program: "sh".to_owned(),
        arguments: Vec::new(),
    };
    assert!(matches!(
        ProcessBoardSupervisor::recommend(&invalid_profile, repository.path(), &input()),
        Err(ProcessBoardSupervisionError::Profile(_))
    ));
    let unavailable_profile = PlannerProfile {
        name: "unavailable organiser".to_owned(),
        program: "organiser-command-that-does-not-exist".to_owned(),
        arguments: Vec::new(),
    };
    assert!(matches!(
        ProcessBoardSupervisor::recommend(&unavailable_profile, repository.path(), &input()),
        Err(ProcessBoardSupervisionError::ProcessLaunch { .. })
    ));
    assert!(matches!(
        ProcessBoardSupervisor::recommend(
            &profile("cat >/dev/null; exit 7"),
            repository.path(),
            &input()
        ),
        Err(ProcessBoardSupervisionError::ProcessExited { exit_code: Some(7) })
    ));

    let large_input = BoardSupervisionInput::new(
        (0..100)
            .map(|index| SupervisionWorkItem {
                id: format!("task-{index}"),
                title: "a".repeat(600),
                state: "inbox".to_owned(),
            })
            .collect(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SupervisionCandidate {
            action: SupervisionAction::PrepareWork,
            work_item_id: "task-1".to_owned(),
        }],
    )
    .expect("input entries remain within the count limit");
    assert!(matches!(
        ProcessBoardSupervisor::recommend(
            &profile("cat >/dev/null"),
            repository.path(),
            &large_input
        ),
        Err(ProcessBoardSupervisionError::InputTooLarge)
    ));
}

#[test]
fn enforces_process_deadlines_output_limits_and_safe_input_cardinality() {
    let repository = TempDir::new().expect("temporary repository should exist");
    assert!(matches!(
        ProcessBoardSupervisor::recommend_with_runtime(
            &profile("cat >/dev/null; sleep 1"),
            repository.path(),
            &input(),
            Duration::from_millis(20),
        ),
        Err(ProcessBoardSupervisionError::ProcessTimedOut)
    ));
    assert!(matches!(
        ProcessBoardSupervisor::recommend(
            &profile("cat >/dev/null; head -c 65537 /dev/zero"),
            repository.path(),
            &input(),
        ),
        Err(ProcessBoardSupervisionError::OutputTooLarge)
    ));
    assert!(matches!(
        BoardSupervisionInput::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        Err(BoardSupervisionInputError::NoCandidateActions)
    ));
    assert!(matches!(
        BoardSupervisionInput::new(
            vec![
                SupervisionWorkItem {
                    id: "overflow".to_owned(),
                    title: "too many".to_owned(),
                    state: "inbox".to_owned(),
                };
                101
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![SupervisionCandidate {
                action: SupervisionAction::PrepareWork,
                work_item_id: "task-1".to_owned(),
            }],
        ),
        Err(BoardSupervisionInputError::TooManyEntries {
            field: "work items",
            maximum: 100
        })
    ));
}

#[test]
fn explains_rejected_organiser_operations_without_echoing_process_output() {
    assert_eq!(
        ProcessBoardSupervisionError::InputTooLarge.to_string(),
        "safe organiser context exceeds the 32768-byte limit"
    );
    assert_eq!(
        ProcessBoardSupervisionError::ProcessLaunch {
            profile_name: "organiser".to_owned(),
        }
        .to_string(),
        "could not start organiser profile organiser"
    );
    assert_eq!(
        ProcessBoardSupervisionError::ProcessTimedOut.to_string(),
        "organiser process exceeded the 30-second limit"
    );
    assert_eq!(
        ProcessBoardSupervisionError::ProcessExited { exit_code: None }.to_string(),
        "organiser process exited without a recommendation"
    );
    assert_eq!(
        ProcessBoardSupervisionError::ProcessExited { exit_code: Some(7) }.to_string(),
        "organiser process exited without a recommendation (code 7)"
    );
    assert_eq!(
        BoardSupervisionInputError::NoCandidateActions.to_string(),
        "supervision needs at least one safe candidate action"
    );
    assert_eq!(
        BoardSupervisionInputError::TooManyEntries {
            field: "work items",
            maximum: 100,
        }
        .to_string(),
        "supervision input has more than 100 work items"
    );
}

fn profile(script: &str) -> PlannerProfile {
    PlannerProfile {
        name: "test organiser".to_owned(),
        program: "sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    }
}

fn input() -> BoardSupervisionInput {
    BoardSupervisionInput::new(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SupervisionCandidate {
            action: SupervisionAction::PrepareWork,
            work_item_id: "task-1".to_owned(),
        }],
    )
    .expect("safe candidate input should construct")
}
