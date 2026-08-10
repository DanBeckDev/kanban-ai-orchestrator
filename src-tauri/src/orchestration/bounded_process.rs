use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdout, Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use super::PlannerProfile;

pub(crate) const MAX_DIRECT_PROCESS_OUTPUT_BYTES: u64 = 65_536;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub(crate) enum BoundedProcessError {
    Launch { profile_name: String },
    MissingStandardInput,
    Input,
    MissingStandardOutput,
    Reader,
    Output,
    OutputTooLarge,
    Wait,
    TimedOut,
    Exited { exit_code: Option<i32> },
}

pub(crate) fn run_direct_json_process(
    profile: &PlannerProfile,
    repository_path: &Path,
    input: &[u8],
    max_runtime: Duration,
) -> Result<Vec<u8>, BoundedProcessError> {
    let mut child = Command::new(&profile.program)
        .args(&profile.arguments)
        .current_dir(repository_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| BoundedProcessError::Launch {
            profile_name: profile.name.clone(),
        })?;
    write_input(&mut child, input)?;
    let stdout = take_stdout(&mut child)?;
    read_process_output(&mut child, stdout, max_runtime)
}

fn write_input(child: &mut Child, input: &[u8]) -> Result<(), BoundedProcessError> {
    let Some(mut stdin) = child.stdin.take() else {
        terminate(child);
        return Err(BoundedProcessError::MissingStandardInput);
    };
    if stdin
        .write_all(input)
        .and_then(|()| stdin.write_all(b"\n"))
        .is_err()
    {
        terminate(child);
        return Err(BoundedProcessError::Input);
    }
    Ok(())
}

fn take_stdout(child: &mut Child) -> Result<ChildStdout, BoundedProcessError> {
    child.stdout.take().ok_or_else(|| {
        terminate(child);
        BoundedProcessError::MissingStandardOutput
    })
}

fn read_process_output(
    child: &mut Child,
    stdout: ChildStdout,
    max_runtime: Duration,
) -> Result<Vec<u8>, BoundedProcessError> {
    let (sender, receiver) = mpsc::channel();
    if thread::Builder::new()
        .name("bounded-process-output".to_owned())
        .spawn(move || {
            let _ = sender.send(read_bounded_output(stdout));
        })
        .is_err()
    {
        terminate(child);
        return Err(BoundedProcessError::Reader);
    }
    wait_for_process_output(child, receiver, max_runtime)
}

fn wait_for_process_output(
    child: &mut Child,
    receiver: mpsc::Receiver<Result<Vec<u8>, BoundedProcessError>>,
    max_runtime: Duration,
) -> Result<Vec<u8>, BoundedProcessError> {
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
                    return Err(BoundedProcessError::Output);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let status = child.try_wait().map_err(|_| BoundedProcessError::Wait)?;
        if let Some(status) = status {
            return completed_output(output, receiver, deadline, status);
        }
        if Instant::now() >= deadline {
            terminate(child);
            return Err(BoundedProcessError::TimedOut);
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn completed_output(
    output: Option<Vec<u8>>,
    receiver: mpsc::Receiver<Result<Vec<u8>, BoundedProcessError>>,
    deadline: Instant,
    status: ExitStatus,
) -> Result<Vec<u8>, BoundedProcessError> {
    if !status.success() {
        return Err(BoundedProcessError::Exited {
            exit_code: status.code(),
        });
    }
    if let Some(output) = output {
        return Ok(output);
    }
    receiver
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|_| BoundedProcessError::Output)?
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_bounded_output(stdout: impl Read) -> Result<Vec<u8>, BoundedProcessError> {
    let mut output = Vec::new();
    stdout
        .take(MAX_DIRECT_PROCESS_OUTPUT_BYTES + 1)
        .read_to_end(&mut output)
        .map_err(|_| BoundedProcessError::Output)?;
    if output.len() as u64 > MAX_DIRECT_PROCESS_OUTPUT_BYTES {
        Err(BoundedProcessError::OutputTooLarge)
    } else {
        Ok(output)
    }
}
