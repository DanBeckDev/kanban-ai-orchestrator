use std::sync::{Arc, Mutex};

use chrono::{SecondsFormat, Utc};
use tauri::State;

use crate::{
    agent::NormalizedAgentEvent,
    desktop::{BoardDaemonState, error_message},
    desktop_execution_activity::{ExecutionActivityPage, ExecutionActivityStreams},
    orchestration::PlannerActivitySink,
};

pub(crate) fn activate(
    state: &BoardDaemonState,
    board_id: &str,
) -> Result<PlannerActivitySink, String> {
    state
        .planning_activity
        .lock()
        .map_err(|_| "the planning activity stream stopped unexpectedly".to_owned())?
        .activate(board_id);
    Ok(sink(state.planning_activity.clone(), board_id.to_owned()))
}

pub(crate) fn complete(state: &BoardDaemonState, board_id: &str) {
    if let Ok(mut streams) = state.planning_activity.lock() {
        streams.complete(board_id);
    }
}

#[tauri::command]
pub(crate) fn planning_activity(
    state: State<'_, BoardDaemonState>,
    board_id: String,
    after_sequence: Option<u64>,
) -> Result<ExecutionActivityPage, String> {
    state
        .planning_activity
        .lock()
        .map_err(|_| "the planning activity stream stopped unexpectedly".to_owned())
        .map(|streams| streams.page(&board_id, after_sequence))
        .map_err(error_message)
}

fn sink(streams: Arc<Mutex<ExecutionActivityStreams>>, board_id: String) -> PlannerActivitySink {
    let next_sequence = Arc::new(Mutex::new(1_u64));
    Arc::new(move |kind| {
        let Ok(mut next_sequence) = next_sequence.lock() else {
            return;
        };
        let event = NormalizedAgentEvent {
            sequence: *next_sequence,
            kind,
        };
        *next_sequence = next_sequence.saturating_add(1);
        let recorded_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        if let Ok(mut streams) = streams.lock() {
            streams.record(&board_id, &event, &recorded_at);
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    use crate::{
        agent::NormalizedAgentEventKind, desktop_execution_activity::ExecutionActivityStreams,
    };

    use super::sink;

    #[test]
    fn planner_activity_stream_keeps_safe_events_after_completion() {
        let streams = Arc::new(Mutex::new(ExecutionActivityStreams::default()));
        streams
            .lock()
            .expect("stream should remain available")
            .activate("board-1");
        let sink = sink(streams.clone(), "board-1".to_owned());
        sink(NormalizedAgentEventKind::Activity {
            summary: "Kanban is preparing the planning request.".to_owned(),
        });
        sink(NormalizedAgentEventKind::Completed {
            summary: "Your ticket proposal is ready to review.".to_owned(),
        });
        streams
            .lock()
            .expect("stream should remain available")
            .complete("board-1");

        let page = streams
            .lock()
            .expect("stream should remain available")
            .page("board-1", None);
        assert_eq!(page.chunks.len(), 2);
        assert_eq!(page.chunks[0].sequence, 1);
        assert_eq!(page.chunks[1].sequence, 2);
        assert_eq!(
            page.chunks[1].summary,
            "Your ticket proposal is ready to review."
        );
    }

    #[test]
    fn planner_activity_sink_ignores_a_poisoned_stream() {
        let streams = Arc::new(Mutex::new(ExecutionActivityStreams::default()));
        let lock_holder = streams.clone();
        let result = thread::spawn(move || {
            let _lock = lock_holder.lock().expect("stream lock should be available");
            panic!("simulate a failed activity recorder");
        })
        .join();
        assert!(result.is_err());

        let sink = sink(streams, "board-1".to_owned());
        sink(NormalizedAgentEventKind::Activity {
            summary: "Kanban is preparing the planning request.".to_owned(),
        });
    }
}
