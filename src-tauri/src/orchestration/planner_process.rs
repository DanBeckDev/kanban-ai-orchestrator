use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdout, Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

use super::{
    MAX_PLANNER_GOAL_BYTES, PlanDraft, PlanDraftError, PlannerProfile, PlannerProfileError,
};

const MAX_PLANNER_OUTPUT_BYTES: u64 = 65_536;
const MAX_PLANNER_RUNTIME: Duration = Duration::from_secs(45);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct ProcessPlanGenerator;

impl ProcessPlanGenerator {
    pub fn generate(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        Self::generate_with_runtime(profile, repository_path, goal, MAX_PLANNER_RUNTIME)
    }

    pub(super) fn generate_with_runtime(
        profile: &PlannerProfile,
        repository_path: &Path,
        goal: &str,
        max_runtime: Duration,
    ) -> Result<PlanDraft, ProcessPlanGenerationError> {
        profile
            .validate()
            .map_err(ProcessPlanGenerationError::Profile)?;
        validate_goal(goal)?;
        let input = serde_json::to_vec(&PlannerInput::new(goal))
            .map_err(ProcessPlanGenerationError::InputEncoding)?;
        let output = run_planner_process(profile, repository_path, &input, max_runtime)?;
        parse_plan_draft(&output)
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

fn run_planner_process(
    profile: &PlannerProfile,
    repository_path: &Path,
    input: &[u8],
    max_runtime: Duration,
) -> Result<Vec<u8>, ProcessPlanGenerationError> {
    let mut child = Command::new(&profile.program)
        .args(&profile.arguments)
        .current_dir(repository_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ProcessPlanGenerationError::ProcessLaunch {
            profile_name: profile.name.clone(),
        })?;
    write_input(&mut child, input)?;
    let stdout = take_stdout(&mut child)?;
    read_process_output(&mut child, stdout, max_runtime)
}

fn write_input(child: &mut Child, input: &[u8]) -> Result<(), ProcessPlanGenerationError> {
    let Some(mut stdin) = child.stdin.take() else {
        terminate(child);
        return Err(ProcessPlanGenerationError::MissingStandardInput);
    };
    if stdin
        .write_all(input)
        .and_then(|()| stdin.write_all(b"\n"))
        .is_err()
    {
        terminate(child);
        return Err(ProcessPlanGenerationError::ProcessInput);
    }
    Ok(())
}

fn take_stdout(child: &mut Child) -> Result<ChildStdout, ProcessPlanGenerationError> {
    child.stdout.take().ok_or_else(|| {
        terminate(child);
        ProcessPlanGenerationError::MissingStandardOutput
    })
}

fn read_process_output(
    child: &mut Child,
    stdout: ChildStdout,
    max_runtime: Duration,
) -> Result<Vec<u8>, ProcessPlanGenerationError> {
    let (sender, receiver) = mpsc::channel();
    if thread::Builder::new()
        .name("planner-output".to_owned())
        .spawn(move || {
            let _ = sender.send(read_bounded_output(stdout));
        })
        .is_err()
    {
        terminate(child);
        return Err(ProcessPlanGenerationError::ProcessReader);
    }
    wait_for_process_output(child, receiver, max_runtime)
}

fn wait_for_process_output(
    child: &mut Child,
    receiver: mpsc::Receiver<Result<Vec<u8>, ProcessPlanGenerationError>>,
    max_runtime: Duration,
) -> Result<Vec<u8>, ProcessPlanGenerationError> {
    let deadline = Instant::now() + max_runtime;
    let mut output = None;
    loop {
        if output.is_none() {
            match receiver.try_recv() {
                Ok(Ok(result)) => output = Some(result),
                Ok(Err(error)) => {
                    terminate(child);
                    return Err(error);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    terminate(child);
                    return Err(ProcessPlanGenerationError::ProcessOutput);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(_) => {
                terminate(child);
                return Err(ProcessPlanGenerationError::ProcessWait);
            }
        };
        if let Some(status) = status {
            return completed_output(output, receiver, deadline, status);
        }
        if Instant::now() >= deadline {
            terminate(child);
            return Err(ProcessPlanGenerationError::ProcessTimedOut);
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn completed_output(
    output: Option<Vec<u8>>,
    receiver: mpsc::Receiver<Result<Vec<u8>, ProcessPlanGenerationError>>,
    deadline: Instant,
    status: ExitStatus,
) -> Result<Vec<u8>, ProcessPlanGenerationError> {
    if !status.success() {
        return Err(ProcessPlanGenerationError::ProcessExited {
            exit_code: status.code(),
        });
    }
    if let Some(output) = output {
        return Ok(output);
    }
    let remaining = deadline.saturating_duration_since(Instant::now());
    receiver
        .recv_timeout(remaining)
        .map_err(|_| ProcessPlanGenerationError::ProcessOutput)?
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_bounded_output(stdout: impl Read) -> Result<Vec<u8>, ProcessPlanGenerationError> {
    let mut output = Vec::new();
    stdout
        .take(MAX_PLANNER_OUTPUT_BYTES + 1)
        .read_to_end(&mut output)
        .map_err(|_| ProcessPlanGenerationError::ProcessOutput)?;
    if output.len() as u64 > MAX_PLANNER_OUTPUT_BYTES {
        Err(ProcessPlanGenerationError::OutputTooLarge)
    } else {
        Ok(output)
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
                "planner response exceeds the {MAX_PLANNER_OUTPUT_BYTES}-byte limit"
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
