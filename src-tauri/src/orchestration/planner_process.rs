use std::{error::Error, fmt, path::Path, time::Duration};

use serde::Serialize;

use crate::{
    agent::NormalizedAgentEventKind,
    domain::{AgentEffort, AgentModelPreference},
};

use super::{
    MAX_PLANNER_GOAL_BYTES, PlanDraft, PlanDraftError, PlannerProfile, PlannerProfileError,
    bounded_process::{BoundedProcessError, MAX_DIRECT_PROCESS_OUTPUT_BYTES},
    native_profile_process::{
        PlannerActivitySink, ProfileProcessError, run_json_profile_with_activity,
    },
};

const MAX_PLANNER_RUNTIME: Duration = Duration::from_secs(45);

pub struct ProcessPlanGenerator;

impl ProcessPlanGenerator {
    pub fn generate(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        Self::generate_with_preferences(
            profile,
            repository_path,
            goal,
            &AgentModelPreference::ProviderDefault,
            AgentEffort::ProviderDefault,
        )
    }

    pub fn generate_with_preferences(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        model: &AgentModelPreference,
        effort: AgentEffort,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        Self::generate_with_preferences_and_activity(
            profile,
            repository_path,
            goal,
            model,
            effort,
            std::sync::Arc::new(|_| {}),
        )
    }

    pub(crate) fn generate_with_preferences_and_activity(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        model: &AgentModelPreference,
        effort: AgentEffort,
        activity_sink: PlannerActivitySink,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        let result = Self::generate_with_preferences_and_runtime_and_activity(
            profile,
            repository_path,
            goal,
            model,
            effort,
            activity_sink.clone(),
            MAX_PLANNER_RUNTIME,
        );
        activity_sink(match &result {
            Ok(_) => NormalizedAgentEventKind::Activity {
                summary: "The planner prepared a ticket proposal for Kanban to check.".to_owned(),
            },
            Err(_) => NormalizedAgentEventKind::Failed {
                reason: "The planner did not produce a reviewable proposal.".to_owned(),
            },
        });
        result
    }

    #[cfg(test)]
    fn generate_with_preferences_and_runtime(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        model: &AgentModelPreference,
        effort: AgentEffort,
        max_runtime: Duration,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        Self::generate_with_preferences_and_runtime_and_activity(
            profile,
            repository_path,
            goal,
            model,
            effort,
            std::sync::Arc::new(|_| {}),
            max_runtime,
        )
    }

    #[cfg(test)]
    pub(super) fn generate_with_runtime(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        max_runtime: Duration,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        Self::generate_with_preferences_and_runtime(
            profile,
            repository_path,
            goal,
            &AgentModelPreference::ProviderDefault,
            AgentEffort::ProviderDefault,
            max_runtime,
        )
    }

    fn generate_with_preferences_and_runtime_and_activity(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        model: &AgentModelPreference,
        effort: AgentEffort,
        activity_sink: PlannerActivitySink,
        max_runtime: Duration,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        profile
            .validate()
            .map_err(ProcessPlanGenerationError::Profile)?;
        validate_goal(goal)?;
        activity_sink(NormalizedAgentEventKind::Activity {
            summary: "Kanban is preparing the planning request.".to_owned(),
        });
        let input = serde_json::to_vec(&PlannerInput::new(goal))
            .map_err(ProcessPlanGenerationError::InputEncoding)?;
        let output = run_json_profile_with_activity(
            profile,
            repository_path,
            model,
            effort,
            &input,
            activity_sink.clone(),
            max_runtime,
        )
        .map_err(map_profile_process_error)?;
        activity_sink(NormalizedAgentEventKind::Activity {
            summary: "Kanban is checking the proposed tickets.".to_owned(),
        });
        parse_plan_draft(&output)
    }
}

fn map_profile_process_error(error: ProfileProcessError) -> ProcessPlanGenerationError {
    match error {
        ProfileProcessError::Process(error) => map_process_error(error),
        ProfileProcessError::InvalidNativeOutput
        | ProfileProcessError::UnsupportedNativePreference => {
            ProcessPlanGenerationError::InvalidOutput
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlannerInput<'a> {
    goal: &'a str,
    output_contract: &'static str,
}

impl<'a> PlannerInput<'a> {
    fn new(goal: &'a str) -> Self {
        Self {
            goal,
            output_contract: "Return exactly one JSON object with workItems, dependencies, and unresolvedAssumptions. Each work item requires key, title, description, acceptanceCriteria, optional budget, and optional requiresHumanReview. Dependencies refer to work-item keys using upstreamKey and downstreamKey, and require kind, reason, owner, and nextAction. Do not use Markdown or include any fields outside this contract.",
        }
    }
}

fn validate_goal(goal: &str) -> Result<(), ProcessPlanGenerationError> {
    if goal.trim().is_empty() {
        Err(ProcessPlanGenerationError::BlankGoal)
    } else if goal.len() > MAX_PLANNER_GOAL_BYTES {
        Err(ProcessPlanGenerationError::GoalTooLarge)
    } else {
        Ok(())
    }
}

fn parse_plan_draft(output: &[u8]) -> Result<PlanDraft, ProcessPlanGenerationError> {
    let draft: PlanDraft =
        serde_json::from_slice(output).map_err(|_| ProcessPlanGenerationError::InvalidOutput)?;
    draft
        .validate()
        .map_err(ProcessPlanGenerationError::InvalidDraft)?;
    Ok(draft)
}

fn map_process_error(error: BoundedProcessError) -> ProcessPlanGenerationError {
    match error {
        BoundedProcessError::Launch { profile_name } => {
            ProcessPlanGenerationError::ProcessLaunch { profile_name }
        }
        BoundedProcessError::MissingStandardInput => {
            ProcessPlanGenerationError::MissingStandardInput
        }
        BoundedProcessError::Input => ProcessPlanGenerationError::ProcessInput,
        BoundedProcessError::MissingStandardOutput => {
            ProcessPlanGenerationError::MissingStandardOutput
        }
        BoundedProcessError::Reader => ProcessPlanGenerationError::ProcessReader,
        BoundedProcessError::Output => ProcessPlanGenerationError::ProcessOutput,
        BoundedProcessError::OutputTooLarge => ProcessPlanGenerationError::OutputTooLarge,
        BoundedProcessError::Wait => ProcessPlanGenerationError::ProcessWait,
        BoundedProcessError::TimedOut => ProcessPlanGenerationError::ProcessTimedOut,
        BoundedProcessError::Exited { exit_code } => {
            ProcessPlanGenerationError::ProcessExited { exit_code }
        }
    }
}

#[derive(Debug)]
pub enum ProcessPlanGenerationError {
    Profile(PlannerProfileError),
    BlankGoal,
    GoalTooLarge,
    InputEncoding(serde_json::Error),
    ProcessLaunch { profile_name: String },
    MissingStandardInput,
    ProcessInput,
    MissingStandardOutput,
    ProcessReader,
    ProcessOutput,
    OutputTooLarge,
    ProcessWait,
    ProcessTimedOut,
    ProcessExited { exit_code: Option<i32> },
    InvalidOutput,
    InvalidDraft(PlanDraftError),
}

impl fmt::Display for ProcessPlanGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "invalid planner profile: {error}"),
            Self::BlankGoal => formatter.write_str("a planning goal is required"),
            Self::GoalTooLarge => write!(
                formatter,
                "planning goal exceeds the {MAX_PLANNER_GOAL_BYTES}-byte limit"
            ),
            Self::InputEncoding(_) => formatter.write_str("could not encode the planner input"),
            Self::ProcessLaunch { profile_name } => {
                write!(formatter, "could not start planner profile {profile_name}")
            }
            Self::MissingStandardInput => {
                formatter.write_str("planner process did not expose standard input")
            }
            Self::ProcessInput => {
                formatter.write_str("could not send the planning request to the planner process")
            }
            Self::MissingStandardOutput => {
                formatter.write_str("planner process did not expose standard output")
            }
            Self::ProcessReader => {
                formatter.write_str("could not start the planner response reader")
            }
            Self::ProcessOutput => formatter.write_str("could not read the planner response"),
            Self::OutputTooLarge => write!(
                formatter,
                "planner response exceeds the {MAX_DIRECT_PROCESS_OUTPUT_BYTES}-byte limit"
            ),
            Self::ProcessWait => formatter.write_str("could not wait for the planner process"),
            Self::ProcessTimedOut => {
                formatter.write_str("planner process exceeded the 45-second limit")
            }
            Self::ProcessExited { exit_code } => write!(
                formatter,
                "planner process exited without a plan{}",
                exit_code
                    .map(|code| format!(" (code {code})"))
                    .unwrap_or_default()
            ),
            Self::InvalidOutput => formatter.write_str("planner returned an invalid plan payload"),
            Self::InvalidDraft(error) => {
                write!(formatter, "planner returned an invalid plan draft: {error}")
            }
        }
    }
}

impl Error for ProcessPlanGenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::InputEncoding(error) => Some(error),
            Self::InvalidDraft(error) => Some(error),
            _ => None,
        }
    }
}
