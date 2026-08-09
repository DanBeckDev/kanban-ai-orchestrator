use std::{
    io::Read,
    process::Child,
    sync::{Arc, Mutex},
};

use super::provider_event_decoder::{NativeEventDecoder, NativeEventProtocol};
use super::{NormalizedAgentEvent, NormalizedAgentEventKind};

const MAX_EVENT_LINE_BYTES: usize = 64 * 1024;
const MAX_RETAINED_EVENTS: usize = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcessEventProtocol {
    Normalized,
    Codex,
    ClaudeCode,
    ClinePass,
}

enum ProcessEventDecoder {
    NormalizedJsonl,
    Native(NativeEventDecoder),
}

impl ProcessEventProtocol {
    fn decoder(self) -> ProcessEventDecoder {
        match self {
            Self::Normalized => ProcessEventDecoder::NormalizedJsonl,
            Self::Codex => {
                ProcessEventDecoder::Native(NativeEventDecoder::new(NativeEventProtocol::Codex))
            }
            Self::ClaudeCode => ProcessEventDecoder::Native(NativeEventDecoder::new(
                NativeEventProtocol::ClaudeCode,
            )),
            Self::ClinePass => {
                ProcessEventDecoder::Native(NativeEventDecoder::new(NativeEventProtocol::ClinePass))
            }
        }
    }
}

impl ProcessEventDecoder {
    fn decode_line(&mut self, line: &[u8]) -> Result<Vec<NormalizedAgentEvent>, String> {
        match self {
            Self::NormalizedJsonl => serde_json::from_slice(line)
                .map(|event| vec![event])
                .map_err(|error| format!("invalid JSON event: {error}")),
            Self::Native(decoder) => decoder.decode_line(line),
        }
    }
}

pub(super) fn read_events(
    mut stdout: impl Read,
    session_id: &str,
    child: &Arc<Mutex<Child>>,
    events: &Arc<Mutex<Vec<NormalizedAgentEvent>>>,
    protocol: ProcessEventProtocol,
) {
    let mut pending = Vec::new();
    let mut chunk = [0; 4096];
    let mut decoder = protocol.decoder();

    loop {
        let read = match stdout.read(&mut chunk) {
            Ok(read) => read,
            Err(error) => {
                record_reader_failure(
                    session_id,
                    events,
                    &format!("could not read events: {error}"),
                );
                kill_child(child);
                return;
            }
        };
        if read == 0 {
            break;
        }
        pending.extend_from_slice(&chunk[..read]);
        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<_> = pending.drain(..=newline).collect();
            if !record_event_line(&line, session_id, events, &mut decoder) {
                kill_child(child);
                return;
            }
        }
        if pending.len() > MAX_EVENT_LINE_BYTES {
            record_reader_failure(session_id, events, "an event line exceeded 65536 bytes");
            kill_child(child);
            return;
        }
    }

    if !pending.is_empty() && !record_event_line(&pending, session_id, events, &mut decoder) {
        kill_child(child);
    }
}

pub(super) fn kill_child(child: &Arc<Mutex<Child>>) {
    if let Ok(mut child) = child.lock() {
        let _ = child.kill();
    }
}

pub(super) fn has_terminal_event(events: &Arc<Mutex<Vec<NormalizedAgentEvent>>>) -> bool {
    events.lock().is_ok_and(|events| {
        events.iter().any(|event| {
            matches!(
                event.kind,
                NormalizedAgentEventKind::Completed { .. }
                    | NormalizedAgentEventKind::Failed { .. }
                    | NormalizedAgentEventKind::Interrupted { .. }
            )
        })
    })
}

fn record_event_line(
    line: &[u8],
    session_id: &str,
    events: &Arc<Mutex<Vec<NormalizedAgentEvent>>>,
    decoder: &mut ProcessEventDecoder,
) -> bool {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    match decoder.decode_line(line) {
        Ok(decoded_events) => decoded_events
            .into_iter()
            .all(|event| record_event(event, events)),
        Err(reason) => {
            record_reader_failure(session_id, events, &reason);
            false
        }
    }
}

fn record_event(
    event: NormalizedAgentEvent,
    events: &Arc<Mutex<Vec<NormalizedAgentEvent>>>,
) -> bool {
    let Ok(mut events) = events.lock() else {
        return false;
    };
    let expected_sequence = events.last().map_or(1, |previous_event| {
        previous_event.sequence.saturating_add(1)
    });
    if events.len() >= MAX_RETAINED_EVENTS.saturating_sub(1) {
        events.push(failed_event(
            expected_sequence,
            "the adapter emitted too many retained events",
        ));
        return false;
    }
    if event.sequence != expected_sequence {
        events.push(failed_event(
            expected_sequence,
            &format!(
                "the adapter emitted event sequence {}; expected {expected_sequence}",
                event.sequence
            ),
        ));
        return false;
    }
    events.push(event);
    true
}

fn record_reader_failure(
    session_id: &str,
    events: &Arc<Mutex<Vec<NormalizedAgentEvent>>>,
    reason: &str,
) {
    let Ok(mut events) = events.lock() else {
        return;
    };
    if events.len() >= MAX_RETAINED_EVENTS {
        return;
    }
    let sequence = events.last().map_or(1, |previous_event| {
        previous_event.sequence.saturating_add(1)
    });
    events.push(failed_event(sequence, &format!("{session_id} {reason}")));
}

fn failed_event(sequence: u64, reason: &str) -> NormalizedAgentEvent {
    NormalizedAgentEvent {
        sequence,
        kind: NormalizedAgentEventKind::Failed {
            reason: reason.to_owned(),
        },
    }
}
