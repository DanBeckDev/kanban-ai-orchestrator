use std::{error::Error, fmt, path::Path, time::Duration};

use crate::domain::{AgentEffort, AgentModelPreference};

use super::{
    PlannerProfile, PlannerProfileError, TicketEffectInput, TicketEffectRecommendation,
    TicketEffectRecommendationError,
    bounded_process::{BoundedProcessError, MAX_DIRECT_PROCESS_OUTPUT_BYTES},
    native_profile_process::{ProfileProcessError, run_json_profile},
};

const MAX_TICKET_EFFECT_INPUT_BYTES: usize = 16_384;
const MAX_TICKET_EFFECT_RUNTIME: Duration = Duration::from_secs(30);

pub struct ProcessTicketEffectAdvisor;

impl ProcessTicketEffectAdvisor {
    pub fn advise(
        profile: &PlannerProfile,
        repository_path: &Path,
        input: &TicketEffectInput,
    ) -> Result<TicketEffectRecommendation, ProcessTicketEffectError> {
        Self::advise_with_preferences(
            profile,
            repository_path,
            input,
            &AgentModelPreference::ProviderDefault,
            AgentEffort::ProviderDefault,
        )
    }

    pub fn advise_with_preferences(
        profile: &PlannerProfile,
        repository_path: &Path,
        input: &TicketEffectInput,
        model: &AgentModelPreference,
        effort: AgentEffort,
    ) -> Result<TicketEffectRecommendation, ProcessTicketEffectError> {
        profile
            .validate()
            .map_err(ProcessTicketEffectError::Profile)?;
        let encoded = serde_json::to_vec(input).map_err(ProcessTicketEffectError::InputEncoding)?;
        if encoded.len() > MAX_TICKET_EFFECT_INPUT_BYTES {
            return Err(ProcessTicketEffectError::InputTooLarge);
        }
        let output = run_json_profile(
            profile,
            repository_path,
            model,
            effort,
            &encoded,
            MAX_TICKET_EFFECT_RUNTIME,
        )
        .map_err(map_profile_process_error)?;
        let mut recommendation: TicketEffectRecommendation =
            serde_json::from_slice(&output).map_err(|_| ProcessTicketEffectError::InvalidOutput)?;
        recommendation
            .validate_against(input)
            .map_err(ProcessTicketEffectError::InvalidRecommendation)?;
        Ok(recommendation)
    }
}

fn map_profile_process_error(error: ProfileProcessError) -> ProcessTicketEffectError {
    match error {
        ProfileProcessError::Process(error) => map_process_error(error),
        ProfileProcessError::InvalidNativeOutput => ProcessTicketEffectError::InvalidOutput,
    }
}

fn map_process_error(error: BoundedProcessError) -> ProcessTicketEffectError {
    match error {
        BoundedProcessError::Launch { profile_name } => {
            ProcessTicketEffectError::ProcessLaunch { profile_name }
        }
        BoundedProcessError::MissingStandardInput => ProcessTicketEffectError::MissingStandardInput,
        BoundedProcessError::Input => ProcessTicketEffectError::ProcessInput,
        BoundedProcessError::MissingStandardOutput => {
            ProcessTicketEffectError::MissingStandardOutput
        }
        BoundedProcessError::Reader => ProcessTicketEffectError::ProcessReader,
        BoundedProcessError::Output => ProcessTicketEffectError::ProcessOutput,
        BoundedProcessError::OutputTooLarge => ProcessTicketEffectError::OutputTooLarge,
        BoundedProcessError::Wait => ProcessTicketEffectError::ProcessWait,
        BoundedProcessError::TimedOut => ProcessTicketEffectError::ProcessTimedOut,
        BoundedProcessError::Exited { exit_code } => {
            ProcessTicketEffectError::ProcessExited { exit_code }
        }
    }
}

#[derive(Debug)]
pub enum ProcessTicketEffectError {
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
    InvalidRecommendation(TicketEffectRecommendationError),
}

impl fmt::Display for ProcessTicketEffectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "invalid organiser profile: {error}"),
            Self::InputEncoding(_) => {
                formatter.write_str("could not encode the safe ticket context")
            }
            Self::InputTooLarge => {
                formatter.write_str("safe ticket context exceeds the 16384-byte limit")
            }
            Self::ProcessLaunch { profile_name } => write!(
                formatter,
                "could not start organiser profile {profile_name}"
            ),
            Self::MissingStandardInput => {
                formatter.write_str("organiser process did not expose standard input")
            }
            Self::ProcessInput => {
                formatter.write_str("could not send the ticket request to the organiser")
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
                "organiser process exited without a ticket recommendation{}",
                exit_code
                    .map(|code| format!(" (code {code})"))
                    .unwrap_or_default()
            ),
            Self::InvalidOutput => {
                formatter.write_str("organiser returned an invalid ticket recommendation")
            }
            Self::InvalidRecommendation(error) => {
                write!(formatter, "ticket recommendation was rejected: {error}")
            }
        }
    }
}

impl Error for ProcessTicketEffectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::InputEncoding(error) => Some(error),
            Self::InvalidRecommendation(error) => Some(error),
            _ => None,
        }
    }
}
