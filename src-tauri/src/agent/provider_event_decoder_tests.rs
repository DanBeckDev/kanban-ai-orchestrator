use super::{
    NormalizedAgentEvent, NormalizedAgentEventKind,
    provider_event_decoder::{NativeEventDecoder, NativeEventProtocol},
};

#[test]
fn maps_codex_lifecycle_and_usage_without_retaining_agent_text() {
    let mut decoder = NativeEventDecoder::new(NativeEventProtocol::Codex);
    let item = decoder
        .decode_line(
            br#"{"type":"item.completed","item":{"type":"agent_message","text":"secret"}}"#,
        )
        .expect("Codex event should decode");
    let completed = decoder
        .decode_line(br#"{"type":"turn.completed","usage":{"input_tokens":21,"output_tokens":8}}"#)
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
fn maps_cline_pass_progress_and_usage_without_retaining_provider_text() {
    let mut decoder = NativeEventDecoder::new(NativeEventProtocol::ClinePass);
    let progress = decoder
        .decode_line(
            br#"{"type":"agent_event","event":{"type":"iteration_start","text":"secret"}}"#,
        )
        .expect("ClinePass progress should decode");
    let usage = decoder
        .decode_line(
            br#"{"type":"agent_event","event":{"type":"usage","usage":{"inputTokens":13,"outputTokens":5,"cost":0.000015}}}"#,
        )
        .expect("ClinePass usage should decode");

    assert_eq!(
        progress[0].kind,
        NormalizedAgentEventKind::Activity {
            summary: "ClinePass started an iteration.".to_owned(),
        }
    );
    assert!(matches!(
        usage[0].kind,
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens: 13,
            output_tokens: 5,
            cost_micros: Some(15),
        }
    ));
}

#[test]
fn maps_cline_pass_completion_with_usage() {
    let mut decoder = NativeEventDecoder::new(NativeEventProtocol::ClinePass);
    let events = decoder
        .decode_line(
            br#"{"type":"agent_event","event":{"type":"done","result":"secret","usage":{"input_tokens":8,"output_tokens":3,"costUsd":0.00001}}}"#,
        )
        .expect("ClinePass completion should decode");

    assert!(matches!(
        events[0].kind,
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens: 8,
            output_tokens: 3,
            cost_micros: Some(10),
        }
    ));
    assert!(matches!(
        events[1].kind,
        NormalizedAgentEventKind::Completed { ref summary }
            if summary == "ClinePass completed the task and is ready for review."
    ));
}

#[test]
fn maps_cline_pass_failure_and_ignores_unrecognized_events() {
    let mut decoder = NativeEventDecoder::new(NativeEventProtocol::ClinePass);
    let failure = decoder
        .decode_line(br#"{"type":"error","message":"secret"}"#)
        .expect("ClinePass top-level errors should decode");
    let ignored = decoder
        .decode_line(br#"{"type":"agent_event","event":{"type":"unrecognized","text":"secret"}}"#)
        .expect("unknown ClinePass events should be ignored");

    assert_eq!(
        failure[0].kind,
        NormalizedAgentEventKind::Failed {
            reason: "ClinePass reported an error.".to_owned(),
        }
    );
    assert!(ignored.is_empty());
}

#[test]
fn rejects_cline_pass_agent_events_without_a_structured_payload() {
    let mut decoder = NativeEventDecoder::new(NativeEventProtocol::ClinePass);

    assert!(decoder.decode_line(br#"{"type":"agent_event"}"#).is_err());
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
        vec![NormalizedAgentEvent {
            sequence: 1,
            kind: NormalizedAgentEventKind::Completed {
                summary: "Codex completed the task and is ready for review.".to_owned(),
            },
        }]
    );
    assert_eq!(
        claude
            .decode_line(br#"{"type":"result","usage":{"input_tokens":4,"cost_usd":0.00002}}"#)
            .expect("Claude completion should decode"),
        vec![NormalizedAgentEvent {
            sequence: 1,
            kind: NormalizedAgentEventKind::Completed {
                summary: "Claude Code completed the task and is ready for review.".to_owned(),
            },
        }]
    );
}
