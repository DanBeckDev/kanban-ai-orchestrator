#![cfg(unix)]

use std::error::Error;

use tempfile::TempDir;

use crate::{
    domain::{TicketEffectAction, TicketEffectOutcome},
    orchestration::{
        PlannerProfileError, ProcessTicketEffectAdvisor, ProcessTicketEffectError,
        TicketEffectEvidence, TicketEffectInput, TicketEffectInputError,
        TicketEffectRecommendationError, TicketEffectTask,
    },
};

use super::PlannerProfile;

#[test]
fn accepts_a_complete_refinement_and_redacts_persistable_text() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let recommendation = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"refine_specification\",\"recommendation\":\"Use ghp_secret.\",\"rationale\":\"token copied-value\",\"proposal\":{\"title\":\"Improve setup\",\"description\":\"Explain the safe first step.\",\"acceptanceCriteria\":[\"The setup can be understood.\"]}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::RefineSpecification),
    )
    .expect("complete refinement should parse");

    assert_eq!(
        recommendation.proposal.title.as_deref(),
        Some("Improve setup")
    );
    assert!(recommendation.recommendation.contains("[redacted]"));
    assert!(recommendation.rationale.contains("[redacted]"));
}

#[test]
fn rejects_an_unrequested_action_and_unsafe_proposal_shape() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let wrong_action = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_start\",\"recommendation\":\"Start it.\",\"rationale\":\"Ready.\",\"proposal\":{}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::ExplainEvidence),
    );
    assert!(matches!(
        wrong_action,
        Err(ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::UnexpectedAction
        ))
    ));

    let unexpected_proposal = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_start\",\"recommendation\":\"Start it.\",\"rationale\":\"Ready.\",\"proposal\":{\"workerGuidance\":\"rm -rf\"}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::PrepareStart),
    );
    assert!(matches!(
        unexpected_proposal,
        Err(ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::UnexpectedProposal
        ))
    ));

    let unknown_proposal_field = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_start\",\"recommendation\":\"Start it.\",\"rationale\":\"Ready.\",\"proposal\":{\"extra\":\"unbounded instruction\"}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::PrepareStart),
    );
    assert!(matches!(
        unknown_proposal_field,
        Err(ProcessTicketEffectError::InvalidOutput)
    ));
}

#[test]
fn requires_the_action_specific_guidance_or_evidence_explanation() {
    let repository = TempDir::new().expect("temporary repository should exist");
    let missing_guidance = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"give_worker_guidance\",\"recommendation\":\"Guide it.\",\"rationale\":\"It needs context.\",\"proposal\":{}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::GiveWorkerGuidance),
    );
    assert!(matches!(
        missing_guidance,
        Err(ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::MissingGuidance
        ))
    ));

    let explanation = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"explain_evidence\",\"recommendation\":\"Review the failure.\",\"rationale\":\"A check failed.\",\"proposal\":{\"evidenceExplanation\":\"The failing check needs a correction.\"}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::ExplainEvidence),
    )
    .expect("evidence explanation should parse");
    assert_eq!(
        explanation.proposal.evidence_explanation.as_deref(),
        Some("The failing check needs a correction.")
    );
}

#[test]
fn rejects_empty_prompts_and_process_failures_without_exposing_output() {
    assert!(matches!(
        TicketEffectInput::new(TicketEffectAction::ExplainEvidence, " ", task(), Vec::new(),),
        Err(TicketEffectInputError::MissingPrompt)
    ));
    let repository = TempDir::new().expect("temporary repository should exist");
    assert!(matches!(
        ProcessTicketEffectAdvisor::advise(
            &profile("cat >/dev/null; exit 7"),
            repository.path(),
            &input(TicketEffectAction::ExplainEvidence),
        ),
        Err(ProcessTicketEffectError::ProcessExited { exit_code: Some(7) })
    ));
    assert_eq!(
        ProcessTicketEffectError::InvalidOutput.to_string(),
        "organiser returned an invalid ticket recommendation"
    );
}

#[test]
fn enforces_bounded_input_and_the_action_specific_proposal_vocabulary() {
    assert!(matches!(
        TicketEffectInput::new(
            TicketEffectAction::ExplainEvidence,
            "Explain the evidence.",
            task(),
            (0..21)
                .map(|_| TicketEffectEvidence {
                    kind: "check".to_owned(),
                    result: "failed".to_owned(),
                    summary: "A check failed.".to_owned(),
                })
                .collect(),
        ),
        Err(TicketEffectInputError::TooMuchEvidence)
    ));
    assert!(TicketEffectAction::RefineSpecification.requires_user_decision_in_manual_mode());
    assert!(!TicketEffectAction::ExplainEvidence.requires_user_decision_in_manual_mode());
    assert!(TicketEffectOutcome::Applied.is_terminal());
    assert!(!TicketEffectOutcome::AwaitingApproval.is_terminal());

    let repository = TempDir::new().expect("temporary repository should exist");
    let complete_start = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_start\",\"recommendation\":\"Start it.\",\"rationale\":\"It is ready.\",\"proposal\":{}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::PrepareStart),
    )
    .expect("an action without a content proposal should parse");
    assert_eq!(complete_start.proposal, Default::default());

    let incomplete_refinement = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"refine_specification\",\"recommendation\":\"Clarify it.\",\"rationale\":\"Details are missing.\",\"proposal\":{\"title\":\"Only a title\",\"description\":\"Still missing criteria.\",\"acceptanceCriteria\":[]}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::RefineSpecification),
    );
    assert!(matches!(
        incomplete_refinement,
        Err(ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::IncompleteRefinement
        ))
    ));

    let malformed_profile = ProcessTicketEffectAdvisor::advise(
        &PlannerProfile {
            name: " ".to_owned(),
            program: "sh".to_owned(),
            arguments: Vec::new(),
        },
        repository.path(),
        &input(TicketEffectAction::ExplainEvidence),
    );
    assert!(matches!(
        malformed_profile,
        Err(ProcessTicketEffectError::Profile(_))
    ));
}

#[test]
fn rejects_every_incomplete_or_cross_action_proposal_field() {
    for proposal in [
        "{}",
        "{\"title\":\"Title\"}",
        "{\"title\":\"Title\",\"description\":\"Description\",\"acceptanceCriteria\":[]}",
        "{\"title\":\"Title\",\"description\":\"Description\",\"acceptanceCriteria\":[\"one\",\"two\",\"three\",\"four\",\"five\",\"six\",\"seven\",\"eight\",\"nine\",\"ten\",\"eleven\",\"twelve\",\"thirteen\"]}",
        "{\"title\":\"Title\",\"description\":\"Description\",\"acceptanceCriteria\":[\"one\"],\"workerGuidance\":\"extra\"}",
        "{\"title\":\"Title\",\"description\":\"Description\",\"acceptanceCriteria\":[\"one\"],\"evidenceExplanation\":\"extra\"}",
        "{\"title\":\"Title\",\"description\":\"Description\",\"acceptanceCriteria\":[\" \"]}",
    ] {
        assert!(matches!(
            advise_proposal(TicketEffectAction::RefineSpecification, proposal),
            Err(ProcessTicketEffectError::InvalidRecommendation(
                TicketEffectRecommendationError::IncompleteRefinement
            ))
        ));
    }

    for proposal in [
        "{}",
        "{\"workerGuidance\":\" \"}",
        "{\"workerGuidance\":\"Guide\",\"title\":\"extra\"}",
        "{\"workerGuidance\":\"Guide\",\"description\":\"extra\"}",
        "{\"workerGuidance\":\"Guide\",\"acceptanceCriteria\":[\"extra\"]}",
        "{\"workerGuidance\":\"Guide\",\"evidenceExplanation\":\"extra\"}",
    ] {
        assert!(matches!(
            advise_proposal(TicketEffectAction::GiveWorkerGuidance, proposal),
            Err(ProcessTicketEffectError::InvalidRecommendation(
                TicketEffectRecommendationError::MissingGuidance
            ))
        ));
    }

    for proposal in [
        "{}",
        "{\"evidenceExplanation\":\" \"}",
        "{\"evidenceExplanation\":\"Explain\",\"title\":\"extra\"}",
        "{\"evidenceExplanation\":\"Explain\",\"description\":\"extra\"}",
        "{\"evidenceExplanation\":\"Explain\",\"acceptanceCriteria\":[\"extra\"]}",
        "{\"evidenceExplanation\":\"Explain\",\"workerGuidance\":\"extra\"}",
    ] {
        assert!(matches!(
            advise_proposal(TicketEffectAction::ExplainEvidence, proposal),
            Err(ProcessTicketEffectError::InvalidRecommendation(
                TicketEffectRecommendationError::MissingExplanation
            ))
        ));
    }
}

#[test]
fn accepts_every_empty_proposal_action_and_rejects_blank_summaries() {
    for action in [
        TicketEffectAction::PrepareStart,
        TicketEffectAction::PrepareRestart,
        TicketEffectAction::ReturnForCorrection,
        TicketEffectAction::RecoverInterrupted,
    ] {
        assert!(advise_proposal(action, "{}").is_ok());
    }
    let repository = TempDir::new().expect("temporary repository should exist");
    let blank_recommendation = ProcessTicketEffectAdvisor::advise(
        &profile(
            "cat >/dev/null; printf '%s' '{\"action\":\"prepare_start\",\"recommendation\":\" \",\"rationale\":\"Ready.\",\"proposal\":{}}'",
        ),
        repository.path(),
        &input(TicketEffectAction::PrepareStart),
    );
    assert!(matches!(
        blank_recommendation,
        Err(ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::MissingSummary
        ))
    ));
}

#[test]
fn process_errors_keep_operator_messages_safe_and_only_expose_wrapped_causes() {
    let encoding_error = serde_json::from_str::<serde_json::Value>("not json")
        .expect_err("invalid JSON should construct an encoding error");
    let errors = vec![
        ProcessTicketEffectError::Profile(PlannerProfileError::MissingRequiredField {
            field: "planner profile name",
        }),
        ProcessTicketEffectError::InputEncoding(encoding_error),
        ProcessTicketEffectError::InputTooLarge,
        ProcessTicketEffectError::ProcessLaunch {
            profile_name: "safe profile".to_owned(),
        },
        ProcessTicketEffectError::MissingStandardInput,
        ProcessTicketEffectError::ProcessInput,
        ProcessTicketEffectError::MissingStandardOutput,
        ProcessTicketEffectError::ProcessReader,
        ProcessTicketEffectError::ProcessOutput,
        ProcessTicketEffectError::OutputTooLarge,
        ProcessTicketEffectError::ProcessWait,
        ProcessTicketEffectError::ProcessTimedOut,
        ProcessTicketEffectError::ProcessExited { exit_code: None },
        ProcessTicketEffectError::ProcessExited { exit_code: Some(7) },
        ProcessTicketEffectError::InvalidOutput,
        ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::MissingGuidance,
        ),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(!error.to_string().contains("not json"));
    }
    assert!(
        Error::source(&ProcessTicketEffectError::Profile(
            PlannerProfileError::ArgumentContainsNull,
        ))
        .is_some()
    );
    assert!(
        Error::source(&ProcessTicketEffectError::InvalidRecommendation(
            TicketEffectRecommendationError::MissingSummary,
        ))
        .is_some()
    );
    assert!(Error::source(&ProcessTicketEffectError::ProcessTimedOut).is_none());
}

fn advise_proposal(
    action: TicketEffectAction,
    proposal: &str,
) -> Result<crate::orchestration::TicketEffectRecommendation, ProcessTicketEffectError> {
    let repository = TempDir::new().expect("temporary repository should exist");
    ProcessTicketEffectAdvisor::advise(
        &profile(&format!(
            "cat >/dev/null; printf '%s' '{{\"action\":\"{}\",\"recommendation\":\"Prepare it.\",\"rationale\":\"The task is bounded.\",\"proposal\":{proposal}}}'",
            action_name(action),
        )),
        repository.path(),
        &input(action),
    )
}

fn action_name(action: TicketEffectAction) -> &'static str {
    match action {
        TicketEffectAction::RefineSpecification => "refine_specification",
        TicketEffectAction::GiveWorkerGuidance => "give_worker_guidance",
        TicketEffectAction::PrepareStart => "prepare_start",
        TicketEffectAction::PrepareRestart => "prepare_restart",
        TicketEffectAction::ExplainEvidence => "explain_evidence",
        TicketEffectAction::ReturnForCorrection => "return_for_correction",
        TicketEffectAction::RecoverInterrupted => "recover_interrupted",
    }
}

fn profile(script: &str) -> PlannerProfile {
    PlannerProfile {
        name: "test organiser".to_owned(),
        program: "sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    }
}

fn input(action: TicketEffectAction) -> TicketEffectInput {
    TicketEffectInput::new(
        action,
        "Please prepare this safely.",
        task(),
        vec![TicketEffectEvidence {
            kind: "check".to_owned(),
            result: "failed".to_owned(),
            summary: "A check failed.".to_owned(),
        }],
    )
    .expect("ticket effect input should construct")
}

fn task() -> TicketEffectTask {
    TicketEffectTask {
        title: "Improve setup".to_owned(),
        description: "The first-run setup needs a clear path.".to_owned(),
        acceptance_criteria: vec!["The setup is clear.".to_owned()],
        state: "inbox".to_owned(),
    }
}
