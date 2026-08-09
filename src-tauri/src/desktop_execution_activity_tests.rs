use crate::{
    agent::{NormalizedAgentEvent, NormalizedAgentEventKind},
    desktop_execution_activity::{ExecutionActivityKind, ExecutionActivityStreams},
};

fn activity(sequence: u64, summary: impl Into<String>) -> NormalizedAgentEvent {
    NormalizedAgentEvent {
        sequence,
        kind: NormalizedAgentEventKind::Activity {
            summary: summary.into(),
        },
    }
}

#[test]
fn activity_pages_are_bounded_and_continue_from_the_supplied_cursor() {
    let mut streams = ExecutionActivityStreams::default();
    streams.activate("execution-1");
    for sequence in 1..=40 {
        streams.record(
            "execution-1",
            &activity(sequence, format!("step {sequence}")),
            "2026-08-09T00:00:00Z",
        );
    }

    let first_page = streams.page("execution-1", None);
    assert_eq!(first_page.chunks.len(), 32);
    assert_eq!(
        first_page.chunks.first().map(|chunk| chunk.sequence),
        Some(1)
    );
    assert_eq!(
        first_page.chunks.last().map(|chunk| chunk.sequence),
        Some(32)
    );
    assert!(first_page.has_more);

    let second_page = streams.page("execution-1", Some(32));
    assert_eq!(second_page.chunks.len(), 8);
    assert_eq!(
        second_page.chunks.first().map(|chunk| chunk.sequence),
        Some(33)
    );
    assert!(!second_page.has_more);
}

#[test]
fn activity_buffers_keep_only_the_most_recent_128_chunks() {
    let mut streams = ExecutionActivityStreams::default();
    streams.activate("execution-1");
    for sequence in 1..=140 {
        streams.record(
            "execution-1",
            &activity(sequence, "progress"),
            "2026-08-09T00:00:00Z",
        );
    }

    let page = streams.page("execution-1", None);
    assert_eq!(page.chunks.len(), 32);
    assert_eq!(page.chunks.first().map(|chunk| chunk.sequence), Some(13));
    assert!(page.has_more);
}

#[test]
fn activity_summaries_are_utf8_safe_and_limited_to_one_kibibyte() {
    let mut streams = ExecutionActivityStreams::default();
    streams.activate("execution-1");
    streams.record(
        "execution-1",
        &activity(1, "🐳".repeat(400)),
        "2026-08-09T00:00:00Z",
    );

    let chunk = streams.page("execution-1", None).chunks.remove(0);
    assert_eq!(chunk.kind, ExecutionActivityKind::Activity);
    assert!(chunk.summary.len() <= 1_024);
    assert!(chunk.summary.ends_with('…'));
    assert!(chunk.summary.is_char_boundary(chunk.summary.len()));
}

#[test]
fn usage_events_do_not_enter_the_human_readable_activity_feed() {
    let mut streams = ExecutionActivityStreams::default();
    streams.activate("execution-1");
    streams.record(
        "execution-1",
        &NormalizedAgentEvent {
            sequence: 1,
            kind: NormalizedAgentEventKind::UsageUpdated {
                input_tokens: 10,
                output_tokens: 20,
                cost_micros: None,
            },
        },
        "2026-08-09T00:00:00Z",
    );

    assert!(streams.page("execution-1", None).chunks.is_empty());
}

#[test]
fn completed_streams_remain_available_for_a_bounded_recent_history() {
    let mut streams = ExecutionActivityStreams::default();
    for index in 0..=32 {
        let execution_id = format!("execution-{index}");
        streams.activate(&execution_id);
        streams.record(
            &execution_id,
            &activity(1, "finished"),
            "2026-08-09T00:00:00Z",
        );
        streams.complete(&execution_id);
    }

    assert!(streams.page("execution-0", None).chunks.is_empty());
    assert_eq!(streams.page("execution-32", None).chunks.len(), 1);
}
