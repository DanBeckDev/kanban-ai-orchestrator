#![cfg(unix)]

use std::time::Duration;

use tempfile::TempDir;

use crate::orchestration::{
    MAX_PLANNER_GOAL_BYTES, PlanDraftError, PlannerProfile, ProcessPlanGenerationError,
    ProcessPlanGenerator,
};

fn profile(script: &str) -> PlannerProfile {
    PlannerProfile {
        name: "test planner".to_owned(),
        program: "sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    }
}

#[test]
fn reads_one_bounded_plan_payload_from_a_direct_planner_process() {
    let temporary_directory = TempDir::new().expect("temporary repository should exist");
    let planner = profile(
        "cat >/dev/null; printf '%s' '{\"workItems\":[{\"key\":\"foundation\",\"title\":\"Foundation\",\"description\":\"Create the contract.\",\"acceptanceCriteria\":[\"Contract tests pass.\"]}],\"dependencies\":[],\"unresolvedAssumptions\":[]}'",
    );

    let draft = ProcessPlanGenerator::generate(
        &planner,
        temporary_directory.path(),
        "Create a dependable foundation.",
    )
    .expect("planner draft should parse");

    assert_eq!(draft.work_items[0].key, "foundation");
    assert!(draft.work_items[0].requires_human_review);
}

#[test]
fn rejects_malformed_or_oversized_planner_output_before_it_can_reach_the_board() {
    let temporary_directory = TempDir::new().expect("temporary repository should exist");
    let malformed = ProcessPlanGenerator::generate(
        &profile("cat >/dev/null; printf '%s' '{not-json}'"),
        temporary_directory.path(),
        "Create a plan.",
    );
    assert!(matches!(
        malformed,
        Err(ProcessPlanGenerationError::InvalidOutput)
    ));

    let invalid_draft = ProcessPlanGenerator::generate(
        &profile("cat >/dev/null; printf '%s' '{\"workItems\":[]}'"),
        temporary_directory.path(),
        "Create a plan.",
    );
    assert!(matches!(
        invalid_draft,
        Err(ProcessPlanGenerationError::InvalidDraft(
            PlanDraftError::EmptyWorkItems
        ))
    ));

    let oversized = ProcessPlanGenerator::generate(
        &profile("cat >/dev/null; head -c 65537 /dev/zero"),
        temporary_directory.path(),
        "Create a plan.",
    );
    assert!(matches!(
        oversized,
        Err(ProcessPlanGenerationError::OutputTooLarge)
    ));
}

#[test]
fn rejects_invalid_goals_or_profile_process_failures_with_actionable_errors() {
    let temporary_directory = TempDir::new().expect("temporary repository should exist");
    let blank_goal =
        ProcessPlanGenerator::generate(&profile("cat >/dev/null"), temporary_directory.path(), " ");
    assert!(matches!(
        blank_goal,
        Err(ProcessPlanGenerationError::BlankGoal)
    ));
    let oversized_goal = ProcessPlanGenerator::generate(
        &profile("cat >/dev/null"),
        temporary_directory.path(),
        &"a".repeat(MAX_PLANNER_GOAL_BYTES + 1),
    );
    assert!(matches!(
        oversized_goal,
        Err(ProcessPlanGenerationError::GoalTooLarge)
    ));
    let unavailable_profile = PlannerProfile {
        name: "unavailable planner".to_owned(),
        program: "planner-command-that-does-not-exist".to_owned(),
        arguments: Vec::new(),
    };
    assert!(matches!(
        ProcessPlanGenerator::generate(
            &unavailable_profile,
            temporary_directory.path(),
            "Create a plan.",
        ),
        Err(ProcessPlanGenerationError::ProcessLaunch { .. })
    ));
    let failed_process = ProcessPlanGenerator::generate(
        &profile("cat >/dev/null; exit 7"),
        temporary_directory.path(),
        "Create a plan.",
    );
    assert!(matches!(
        failed_process,
        Err(ProcessPlanGenerationError::ProcessExited { exit_code: Some(7) })
    ));
}

#[test]
fn terminates_a_slow_direct_child_at_the_configured_deadline() {
    let temporary_directory = TempDir::new().expect("temporary repository should exist");

    let result = ProcessPlanGenerator::generate_with_runtime(
        &profile("cat >/dev/null; sleep 1"),
        temporary_directory.path(),
        "Create a plan.",
        Duration::from_millis(20),
    );

    assert!(matches!(
        result,
        Err(ProcessPlanGenerationError::ProcessTimedOut)
    ));
}
