use std::{error::Error, fmt, path::Path, time::Duration};

use super::{
    BoardSupervisionInput, PlannerProfile, PlannerProfileError, SupervisorRecommendation,
    SupervisorRecommendationError,
    bounded_process::{
        BoundedProcessError, MAX_DIRECT_PROCESS_OUTPUT_BYTES, run_direct_json_process,
    },
};

const MAX_SUPERVISOR_RUNTIME: Duration = Duration::from_secs(30);
const MAX_SUPERVISION_INPUT_BYTES: usize = 32_768;

pub struct ProcessBoardSupervisor;

impl ProcessBoardSupervisor {
    pub fn recommend(
        profile: &PlannerProfile,
        repository_path: &Path,
        input: &BoardSupervisionInput,
    ) -> Result<SupervisorRecommendation, ProcessBoardSupervisionError> {
        Self::recommend_with_runtime(profile, repository_path, input, MAX_SUPERVISOR_RUNTIME)
    }

    pub(super) fn recommend_with_runtime(
        profile: &PlannerProfile,
        repository_path: &Path,
        input: &BoardSupervisionInput,
        max_runtime: Duration,
    ) -> Result<SupervisorRecommendation, ProcessBoardSupervisionError> {
        profile
            .validate()
            .map_err(ProcessBoardSupervisionError::Profile)?;
        let encoded =
            serde_json::to_vec(input).map_err(ProcessBoardSupervisionError::InputEncoding)?;
        if encoded.len() > MAX_SUPERVISION_INPUT_BYTES {
            return Err(ProcessBoardSupervisionError::InputTooLarge);
        }
        let output = run_direct_json_process(profile, repository_path, &encoded, max_runtime)
            .map_err(map_process_error)?;
        parse_recommendation(&output, input)
    }
}

fn parse_recommendation(
    output: &[u8],
    input: &BoardSupervisionInput,
) -> Result<SupervisorRecommendation, ProcessBoardSupervisionError> {
    let mut recommendation: SupervisorRecommendation =
        serde_json::from_slice(output).map_err(|_| ProcessBoardSupervisionError::InvalidOutput)?;
    recommendation
        .validate_against(input)
        .map_err(ProcessBoardSupervisionError::InvalidRecommendation)?;
    Ok(recommendation)
}

fn map_process_error(error: BoundedProcessError) -> ProcessBoardSupervisionError {
    match error {
        BoundedProcessError::Launch { profile_name } => {
            ProcessBoardSupervisionError::ProcessLaunch { profile_name }
        }
        BoundedProcessError::MissingStandardInput => {
            ProcessBoardSupervisionError::MissingStandardInput
        }
        BoundedProcessError::Input => ProcessBoardSupervisionError::ProcessInput,
        BoundedProcessError::MissingStandardOutput => {
            ProcessBoardSupervisionError::MissingStandardOutput
        }
        BoundedProcessError::Reader => ProcessBoardSupervisionError::ProcessReader,
        BoundedProcessError::Output => ProcessBoardSupervisionError::ProcessOutput,
        BoundedProcessError::OutputTooLarge => ProcessBoardSupervisionError::OutputTooLarge,
        BoundedProcessError::Wait => ProcessBoardSupervisionError::ProcessWait,
        BoundedProcessError::TimedOut => ProcessBoardSupervisionError::ProcessTimedOut,
        BoundedProcessError::Exited { exit_code } => {
            ProcessBoardSupervisionError::ProcessExited { exit_code }
        }
    }
}

#[derive(Debug)]
pub enum ProcessBoardSupervisionError {
    Profile(PlannerProfileError),
    InputEncoding(serde_json::Error),
    InputTooLarge,
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
    InvalidRecommendation(SupervisorRecommendationError),
}

impl fmt::Display for ProcessBoardSupervisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "invalid organiser profile: {error}"),
            Self::InputEncoding(_) => {
                formatter.write_str("could not encode the safe organiser context")
            }
            Self::InputTooLarge => write!(
                formatter,
                "safe organiser context exceeds the {MAX_SUPERVISION_INPUT_BYTES}-byte limit"
            ),
            Self::ProcessLaunch { profile_name } => write!(
                formatter,
                "could not start organiser profile {profile_name}"
            ),
            Self::MissingStandardInput => {
                formatter.write_str("organiser process did not expose standard input")
            }
            Self::ProcessInput => {
                formatter.write_str("could not send the board assessment to the organiser")
            }
            Self::MissingStandardOutput => {
                formatter.write_str("organiser process did not expose standard output")
            }
            Self::ProcessReader => {
                formatter.write_str("could not start the organiser response reader")
            }
            Self::ProcessOutput => formatter.write_str("could not read the organiser response"),
            Self::OutputTooLarge => write!(
                formatter,
                "organiser response exceeds the {MAX_DIRECT_PROCESS_OUTPUT_BYTES}-byte limit"
            ),
            Self::ProcessWait => formatter.write_str("could not wait for the organiser process"),
            Self::ProcessTimedOut => {
                formatter.write_str("organiser process exceeded the 30-second limit")
            }
            Self::ProcessExited { exit_code } => write!(
                formatter,
                "organiser process exited without a recommendation{}",
                exit_code
                    .map(|code| format!(" (code {code})"))
                    .unwrap_or_default()
            ),
            Self::InvalidOutput => {
                formatter.write_str("organiser returned an invalid recommendation payload")
            }
            Self::InvalidRecommendation(error) => {
                write!(formatter, "organiser recommendation was rejected: {error}")
            }
        }
    }
}

impl Error for ProcessBoardSupervisionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::InputEncoding(error) => Some(error),
            Self::InvalidRecommendation(error) => Some(error),
            _ => None,
        }
    }
}
