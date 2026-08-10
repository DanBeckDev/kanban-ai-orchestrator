use serde_json::Value;

use super::{NormalizedAgentEvent, NormalizedAgentEventKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeEventProtocol {
    Codex,
    ClaudeCode,
    ClinePass,
}

pub(crate) struct NativeEventDecoder {
    protocol: NativeEventProtocol,
    next_sequence: u64,
}

impl NativeEventDecoder {
    pub(crate) fn new(protocol: NativeEventProtocol) -> Self {
        Self {
            protocol,
            next_sequence: 1,
        }
    }

    pub(crate) fn decode_line(&mut self, line: &[u8]) -> Result<Vec<NormalizedAgentEvent>, String> {
        let event: Value = serde_json::from_slice(line)
            .map_err(|error| format!("invalid provider JSON event: {error}"))?;
        let event_type = required_string(&event, "type", "provider event type")?;
        let kinds = match self.protocol {
            NativeEventProtocol::Codex => Ok(codex_event_kinds(event_type, &event)),
            NativeEventProtocol::ClaudeCode => Ok(claude_event_kinds(event_type, &event)),
            NativeEventProtocol::ClinePass => cline_pass_event_kinds(event_type, &event),
        }?;
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

fn cline_pass_event_kinds(
    event_type: &str,
    event: &Value,
) -> Result<Vec<NormalizedAgentEventKind>, String> {
    if event_type == "error" {
        return Ok(vec![failed("ClinePass reported an error.")]);
    }
    if event_type != "agent_event" {
        return Ok(Vec::new());
    }

    let agent_event = event
        .get("event")
        .ok_or_else(|| "ClinePass agent event is required".to_owned())?;
    let agent_event_type = required_string(agent_event, "type", "ClinePass agent event type")?;
    Ok(match agent_event_type {
        "iteration_start" => vec![activity("ClinePass started an iteration.")],
        "iteration_end" => vec![activity("ClinePass completed an iteration.")],
        "usage" => agent_event
            .get("usage")
            .and_then(cline_pass_usage_updated)
            .into_iter()
            .collect(),
        "done" => cline_pass_completed_with_usage(agent_event),
        "error" => vec![failed("ClinePass reported an error.")],
        _ => Vec::new(),
    })
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

fn cline_pass_completed_with_usage(event: &Value) -> Vec<NormalizedAgentEventKind> {
    let mut kinds = event
        .get("usage")
        .and_then(cline_pass_usage_updated)
        .into_iter()
        .collect::<Vec<_>>();
    kinds.push(NormalizedAgentEventKind::Completed {
        summary: "ClinePass completed the task and is ready for review.".to_owned(),
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

fn cline_pass_usage_updated(usage: &Value) -> Option<NormalizedAgentEventKind> {
    let input_tokens = unsigned_any(usage, &["input_tokens", "inputTokens"])?;
    let output_tokens = unsigned_any(usage, &["output_tokens", "outputTokens"])?;
    Some(NormalizedAgentEventKind::UsageUpdated {
        input_tokens,
        output_tokens,
        cost_micros: cost_micros_any(usage),
    })
}

fn unsigned(value: &Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

fn unsigned_any(value: &Value, fields: &[&str]) -> Option<u64> {
    fields.iter().find_map(|field| unsigned(value, field))
}

fn cost_micros(usage: &Value) -> Option<u64> {
    cost_micros_for_fields(usage, &["total_cost_usd", "cost_usd"])
}

fn cost_micros_any(usage: &Value) -> Option<u64> {
    cost_micros_for_fields(
        usage,
        &[
            "total_cost_usd",
            "cost_usd",
            "totalCostUsd",
            "costUsd",
            "cost",
        ],
    )
}

fn cost_micros_for_fields(usage: &Value, fields: &[&str]) -> Option<u64> {
    let dollars = fields.iter().find_map(|field| usage.get(field)?.as_f64())?;
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
