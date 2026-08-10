use std::{
    io::Read,
    process::{Child, ChildStdout, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Runs a non-interactive local command with no stderr capture and a bounded
/// stdout response. The caller owns parsing the response.
pub(super) fn run(
    command: &mut Command,
    timeout: Duration,
    maximum_output_bytes: u64,
) -> Result<Vec<u8>, ()> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ())?;
    let result = read_output(&mut child, timeout, maximum_output_bytes);
    finish_process(&mut child);
    result
}

fn read_output(
    child: &mut Child,
    timeout: Duration,
    maximum_output_bytes: u64,
) -> Result<Vec<u8>, ()> {
    let stdout = child.stdout.take().ok_or(())?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(read_bounded_output(stdout, maximum_output_bytes));
    });
    wait_for_output(child, receiver, timeout)
}

fn wait_for_output(
    child: &mut Child,
    receiver: mpsc::Receiver<Result<Vec<u8>, ()>>,
    timeout: Duration,
) -> Result<Vec<u8>, ()> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().map_err(|_| ())? {
            if !status.success() {
                return Err(());
            }
            return receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| ())?;
        }
        if Instant::now() >= deadline {
            return Err(());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn read_bounded_output(mut stdout: ChildStdout, maximum_output_bytes: u64) -> Result<Vec<u8>, ()> {
    let mut output = Vec::new();
    let bytes_read = stdout
        .by_ref()
        .take(maximum_output_bytes + 1)
        .read_to_end(&mut output)
        .map_err(|_| ())?;
    (bytes_read as u64 <= maximum_output_bytes)
        .then_some(output)
        .ok_or(())
}

fn finish_process(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
