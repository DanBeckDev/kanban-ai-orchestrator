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
    run_process(
        &profile.name,
        &profile.program,
        &profile.arguments,
        repository_path,
        input,
        max_runtime,
    )
}

pub(crate) fn run_process(
    profile_name: &str,
    program: &str,
    arguments: &[String],
    repository_path: &Path,
    input: &[u8],
    max_runtime: Duration,
) -> Result<Vec<u8>, BoundedProcessError> {
    run_process_observed(
        profile_name,
        program,
        arguments,
        repository_path,
        input,
        max_runtime,
        |_| {},
    )
}

pub(crate) fn run_process_observed<F>(
    profile_name: &str,
    program: &str,
    arguments: &[String],
    repository_path: &Path,
    input: &[u8],
    max_runtime: Duration,
    on_output_line: F,
) -> Result<Vec<u8>, BoundedProcessError>
where
    F: Fn(&[u8]) + Send + 'static,
{
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(repository_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command.stdin(Stdio::piped());
    let mut child = command.spawn().map_err(|_| BoundedProcessError::Launch {
        profile_name: profile_name.to_owned(),
    })?;
    write_input(&mut child, input)?;
    let stdout = take_stdout(&mut child)?;
    read_process_output(&mut child, stdout, max_runtime, on_output_line)
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

fn read_process_output<F>(
    child: &mut Child,
    stdout: ChildStdout,
    max_runtime: Duration,
    on_output_line: F,
) -> Result<Vec<u8>, BoundedProcessError>
where
    F: Fn(&[u8]) + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    if thread::Builder::new()
        .name("bounded-process-output".to_owned())
        .spawn(move || {
            let _ = sender.send(read_bounded_output(stdout, on_output_line));
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

fn read_bounded_output(
    mut stdout: impl Read,
    on_output_line: impl Fn(&[u8]),
) -> Result<Vec<u8>, BoundedProcessError> {
    let mut output = Vec::new();
    let mut pending = Vec::new();
    let mut buffer = [0; 4096];
    let buffer_length = buffer.len() as u64;

    loop {
        let remaining = MAX_DIRECT_PROCESS_OUTPUT_BYTES
            .saturating_add(1)
            .saturating_sub(output.len() as u64);
        if remaining == 0 {
            return Err(BoundedProcessError::OutputTooLarge);
        }
        let read = stdout
            .read(&mut buffer[..remaining.min(buffer_length) as usize])
            .map_err(|_| BoundedProcessError::Output)?;
        if read == 0 {
            break;
        }
        output.extend_from_slice(&buffer[..read]);
        pending.extend_from_slice(&buffer[..read]);
        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<_> = pending.drain(..=newline).collect();
            on_output_line(trim_line(&line));
        }
    }

    if !pending.is_empty() {
        on_output_line(trim_line(&pending));
    }
    Ok(output)
}

fn trim_line(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use std::{
        io::Cursor,
        sync::{Arc, Mutex},
    };

    use super::read_bounded_output;

    #[test]
    fn observes_complete_output_lines_without_changing_the_captured_output() {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let observer = observed.clone();

        let output = read_bounded_output(Cursor::new(b"first\r\nsecond\nfinal"), move |line| {
            observer
                .lock()
                .expect("observed lines should remain available")
                .push(String::from_utf8(line.to_vec()).expect("line should be UTF-8"));
        })
        .expect("output should be read");

        assert_eq!(output, b"first\r\nsecond\nfinal");
        assert_eq!(
            *observed
                .lock()
                .expect("observed lines should remain available"),
            ["first", "second", "final"]
        );
    }
}
