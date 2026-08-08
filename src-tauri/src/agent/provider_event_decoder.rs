use serde_json::Value;

use super::{NormalizedAgentEvent, NormalizedAgentEventKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeEventProtocol {
    Codex,
    ClaudeCode,
}

pub(super) struct NativeEventDecoder {
    protocol: NativeEventProtocol,
    next_sequence: u64,
}

impl NativeEventDecoder {
    pub(super) fn new(protocol: NativeEventProtocol) -> Self {
        Self {
            protocol,
            next_sequence: 1,
        }
    }

    pub(super) fn decode_line(&mut self, line: &[u8]) -> Result<Vec<NormalizedAgentEvent>, String> {
        let event: Value = serde_json::from_slice(line)
            .map_err(|error| format!("invalid provider JSON event: {error}"))?;
        let event_type = required_string(&event, "type", "provider event type")?;
        let kinds = match self.protocol {
            NativeEventProtocol::Codex => codex_event_kinds(event_type, &event),
            NativeEventProtocol::ClaudeCode => claude_event_kinds(event_type, &event),
        };
        Ok(kinds.into_iter().map(|kind| self.event(kind)).collect())
    }

    fn event(&mut self, kind: NormalizedAgentEventKind) -> NormalizedAgentEvent {
        let event = NormalizedAgentEvent {
            sequence: self.next_sequence,
            kind,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        event
    }
}

fn codex_event_kinds(event_type: &str, event: &Value) -> Vec<NormalizedAgentEventKind> {
    match event_type {
        "turn.started" => vec![activity("Codex started a turn.")],
        "item.started" => vec![activity("Codex started a tool or message item.")],
        "item.completed" => vec![activity("Codex completed a tool or message item.")],
        "turn.completed" => completed_with_usage(
            event.get("usage"),
            "Codex completed the task and is ready for review.",
        ),
        "turn.failed" | "error" => vec![failed("Codex reported a failed turn.")],
        _ => Vec::new(),
    }
}

fn claude_event_kinds(event_type: &str, event: &Value) -> Vec<NormalizedAgentEventKind> {
    match event_type {
        "system" if event.get("subtype").and_then(Value::as_str) == Some("init") => {
            vec![activity("Claude Code started a session.")]
        }
        "assistant" => vec![activity("Claude Code produced an agent message.")],
        "result" if event.get("is_error").and_then(Value::as_bool) == Some(true) => {
            vec![failed("Claude Code reported a failed result.")]
        }
        "result" => completed_with_usage(
            event.get("usage"),
            "Claude Code completed the task and is ready for review.",
        ),
        "error" => vec![failed("Claude Code reported an error.")],
        _ => Vec::new(),
    }
}

fn completed_with_usage(usage: Option<&Value>, summary: &str) -> Vec<NormalizedAgentEventKind> {
    let mut kinds = usage
        .and_then(usage_updated)
        .into_iter()
        .collect::<Vec<_>>();
    kinds.push(NormalizedAgentEventKind::Completed {
        summary: summary.to_owned(),
    });
    kinds
}

fn usage_updated(usage: &Value) -> Option<NormalizedAgentEventKind> {
    let input_tokens = unsigned(usage, "input_tokens")?;
    let output_tokens = unsigned(usage, "output_tokens")?;
    Some(NormalizedAgentEventKind::UsageUpdated {
        input_tokens,
        output_tokens,
        cost_micros: cost_micros(usage),
    })
}

fn unsigned(value: &Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

fn cost_micros(usage: &Value) -> Option<u64> {
    let dollars = usage
        .get("total_cost_usd")
        .or_else(|| usage.get("cost_usd"))?
        .as_f64()?;
    (dollars.is_finite() && dollars >= 0.0 && dollars <= u64::MAX as f64 / 1_000_000.0)
        .then(|| (dollars * 1_000_000.0).round() as u64)
}

fn required_string<'a>(
    value: &'a Value,
    field: &str,
    description: &str,
) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{description} is required"))
}

fn activity(summary: &str) -> NormalizedAgentEventKind {
    NormalizedAgentEventKind::Activity {
        summary: summary.to_owned(),
    }
}

fn failed(reason: &str) -> NormalizedAgentEventKind {
    NormalizedAgentEventKind::Failed {
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeEventDecoder, NativeEventProtocol};
    use crate::agent::NormalizedAgentEventKind;

    #[test]
    fn maps_codex_lifecycle_and_usage_without_retaining_agent_text() {
        let mut decoder = NativeEventDecoder::new(NativeEventProtocol::Codex);
        let item = decoder
            .decode_line(
                br#"{"type":"item.completed","item":{"type":"agent_message","text":"secret"}}"#,
            )
            .expect("Codex event should decode");
        let completed = decoder
            .decode_line(
                br#"{"type":"turn.completed","usage":{"input_tokens":21,"output_tokens":8}}"#,
            )
            .expect("Codex completion should decode");

        assert!(matches!(
            item[0].kind,
            NormalizedAgentEventKind::Activity { ref summary }
                if summary == "Codex completed a tool or message item."
        ));
        assert!(matches!(
            completed[0].kind,
            NormalizedAgentEventKind::UsageUpdated {
                input_tokens: 21,
                output_tokens: 8,
                cost_micros: None,
            }
        ));
        assert!(matches!(
            completed[1].kind,
            NormalizedAgentEventKind::Completed { ref summary }
                if summary == "Codex completed the task and is ready for review."
        ));
        assert_eq!(completed[1].sequence, 3);
    }

    #[test]
    fn maps_claude_result_cost_and_failure_without_retaining_result_text() {
        let mut decoder = NativeEventDecoder::new(NativeEventProtocol::ClaudeCode);
        let completed = decoder
            .decode_line(
                br#"{"type":"result","result":"secret","usage":{"input_tokens":11,"output_tokens":4,"total_cost_usd":0.000015}}"#,
            )
            .expect("Claude result should decode");
        let failed = decoder
            .decode_line(br#"{"type":"result","is_error":true,"result":"secret"}"#)
            .expect("Claude failed result should decode");

        assert!(matches!(
            completed[0].kind,
            NormalizedAgentEventKind::UsageUpdated {
                input_tokens: 11,
                output_tokens: 4,
                cost_micros: Some(15),
            }
        ));
        assert!(matches!(
            completed[1].kind,
            NormalizedAgentEventKind::Completed { ref summary }
                if summary == "Claude Code completed the task and is ready for review."
        ));
        assert!(matches!(
            failed[0].kind,
            NormalizedAgentEventKind::Failed { ref reason }
                if reason == "Claude Code reported a failed result."
        ));
    }

    #[test]
    fn rejects_malformed_or_typeless_native_events() {
        let mut decoder = NativeEventDecoder::new(NativeEventProtocol::Codex);

        assert!(decoder.decode_line(b"not JSON").is_err());
        assert!(decoder.decode_line(br#"{"usage":{}}"#).is_err());
    }

    #[test]
    fn maps_safe_progress_and_failure_events_without_replaying_provider_text() {
        let mut codex = NativeEventDecoder::new(NativeEventProtocol::Codex);
        let mut claude = NativeEventDecoder::new(NativeEventProtocol::ClaudeCode);

        assert_eq!(
            codex
                .decode_line(br#"{"type":"turn.started"}"#)
                .expect("Codex turn start should decode")[0]
                .kind,
            NormalizedAgentEventKind::Activity {
                summary: "Codex started a turn.".to_owned(),
            }
        );
        assert_eq!(
            codex
                .decode_line(br#"{"type":"turn.failed","message":"secret"}"#)
                .expect("Codex turn failure should decode")[0]
                .kind,
            NormalizedAgentEventKind::Failed {
                reason: "Codex reported a failed turn.".to_owned(),
            }
        );
        assert!(
            codex
                .decode_line(br#"{"type":"unrecognized","secret":"secret"}"#)
                .expect("unknown Codex events should be ignored")
                .is_empty()
        );
        assert!(
            claude
                .decode_line(br#"{"type":"system","subtype":"other","secret":"secret"}"#)
                .expect("unknown Claude system events should be ignored")
                .is_empty()
        );
        assert_eq!(
            claude
                .decode_line(br#"{"type":"assistant","message":"secret"}"#)
                .expect("Claude progress should decode")[0]
                .kind,
            NormalizedAgentEventKind::Activity {
                summary: "Claude Code produced an agent message.".to_owned(),
            }
        );
        assert_eq!(
            claude
                .decode_line(br#"{"type":"error","message":"secret"}"#)
                .expect("Claude errors should decode")[0]
                .kind,
            NormalizedAgentEventKind::Failed {
                reason: "Claude Code reported an error.".to_owned(),
            }
        );
    }

    #[test]
    fn completes_when_usage_is_missing_or_incomplete() {
        let mut codex = NativeEventDecoder::new(NativeEventProtocol::Codex);
        let mut claude = NativeEventDecoder::new(NativeEventProtocol::ClaudeCode);

        assert_eq!(
            codex
                .decode_line(br#"{"type":"turn.completed"}"#)
                .expect("Codex completion should decode"),
            vec![super::NormalizedAgentEvent {
                sequence: 1,
                kind: NormalizedAgentEventKind::Completed {
                    summary: "Codex completed the task and is ready for review.".to_owned(),
                },
            }]
        );
        assert_eq!(
            claude
                .decode_line(br#"{"type":"result","usage":{"input_tokens":4,"cost_usd":0.00002}}"#,)
                .expect("Claude completion should decode"),
            vec![super::NormalizedAgentEvent {
                sequence: 1,
                kind: NormalizedAgentEventKind::Completed {
                    summary: "Claude Code completed the task and is ready for review.".to_owned(),
                },
            }]
        );
    }
}
