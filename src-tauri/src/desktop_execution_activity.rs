use std::collections::{BTreeMap, VecDeque};

use serde::Serialize;

use crate::agent::{NormalizedAgentEvent, NormalizedAgentEventKind};

const MAX_ACTIVITY_CHUNKS_PER_EXECUTION: usize = 128;
const MAX_ACTIVITY_SUMMARY_BYTES: usize = 1_024;
const MAX_ACTIVITY_CHUNKS_PER_PAGE: usize = 32;
const MAX_COMPLETED_ACTIVITY_STREAMS: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExecutionActivityKind {
    Activity,
    ApprovalRequested,
    AwaitingInput,
    AwaitingReview,
    Completed,
    Failed,
    Interrupted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExecutionActivityChunk {
    pub(crate) sequence: u64,
    pub(crate) kind: ExecutionActivityKind,
    pub(crate) summary: String,
    pub(crate) recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExecutionActivityPage {
    pub(crate) chunks: Vec<ExecutionActivityChunk>,
    pub(crate) has_more: bool,
}

#[derive(Default)]
pub(crate) struct ExecutionActivityStreams {
    active: BTreeMap<String, ExecutionActivityStream>,
    completed: VecDeque<(String, ExecutionActivityStream)>,
}

impl ExecutionActivityStreams {
    pub(crate) fn activate(&mut self, execution_id: &str) {
        self.remove_completed(execution_id);
        self.active
            .insert(execution_id.to_owned(), ExecutionActivityStream::default());
    }

    pub(crate) fn record(
        &mut self,
        execution_id: &str,
        event: &NormalizedAgentEvent,
        recorded_at: &str,
    ) {
        if let Some(stream) = self.active.get_mut(execution_id) {
            stream.record(event, recorded_at);
        }
    }

    pub(crate) fn complete(&mut self, execution_id: &str) {
        let Some(stream) = self.active.remove(execution_id) else {
            return;
        };
        self.remove_completed(execution_id);
        self.completed.push_back((execution_id.to_owned(), stream));
        while self.completed.len() > MAX_COMPLETED_ACTIVITY_STREAMS {
            self.completed.pop_front();
        }
    }

    pub(crate) fn page(
        &self,
        execution_id: &str,
        after_sequence: Option<u64>,
    ) -> ExecutionActivityPage {
        self.active
            .get(execution_id)
            .or_else(|| self.completed_stream(execution_id))
            .map(|stream| stream.page(after_sequence))
            .unwrap_or_else(ExecutionActivityPage::empty)
    }

    fn completed_stream(&self, execution_id: &str) -> Option<&ExecutionActivityStream> {
        self.completed
            .iter()
            .rev()
            .find(|(id, _)| id == execution_id)
            .map(|(_, stream)| stream)
    }

    fn remove_completed(&mut self, execution_id: &str) {
        self.completed.retain(|(id, _)| id != execution_id);
    }
}

impl ExecutionActivityPage {
    fn empty() -> Self {
        Self {
            chunks: Vec::new(),
            has_more: false,
        }
    }
}

#[derive(Default)]
struct ExecutionActivityStream {
    chunks: VecDeque<ExecutionActivityChunk>,
}

impl ExecutionActivityStream {
    fn record(&mut self, event: &NormalizedAgentEvent, recorded_at: &str) {
        let Some((kind, summary)) = activity_details(&event.kind) else {
            return;
        };
        self.chunks.push_back(ExecutionActivityChunk {
            sequence: event.sequence,
            kind,
            summary: truncate_summary(summary),
            recorded_at: recorded_at.to_owned(),
        });
        if self.chunks.len() > MAX_ACTIVITY_CHUNKS_PER_EXECUTION {
            self.chunks.pop_front();
        }
    }

    fn page(&self, after_sequence: Option<u64>) -> ExecutionActivityPage {
        let chunks: Vec<_> = self
            .chunks
            .iter()
            .filter(|chunk| after_sequence.is_none_or(|sequence| chunk.sequence > sequence))
            .cloned()
            .collect();
        let has_more = chunks.len() > MAX_ACTIVITY_CHUNKS_PER_PAGE;
        ExecutionActivityPage {
            chunks: chunks
                .into_iter()
                .take(MAX_ACTIVITY_CHUNKS_PER_PAGE)
                .collect(),
            has_more,
        }
    }
}

fn activity_details(event: &NormalizedAgentEventKind) -> Option<(ExecutionActivityKind, &str)> {
    match event {
        NormalizedAgentEventKind::Activity { summary } => {
            Some((ExecutionActivityKind::Activity, summary))
        }
        NormalizedAgentEventKind::ApprovalRequested { question } => {
            Some((ExecutionActivityKind::ApprovalRequested, question))
        }
        NormalizedAgentEventKind::AwaitingInput { question } => {
            Some((ExecutionActivityKind::AwaitingInput, question))
        }
        NormalizedAgentEventKind::AwaitingReview { summary } => {
            Some((ExecutionActivityKind::AwaitingReview, summary))
        }
        NormalizedAgentEventKind::Completed { summary } => {
            Some((ExecutionActivityKind::Completed, summary))
        }
        NormalizedAgentEventKind::Failed { reason } => {
            Some((ExecutionActivityKind::Failed, reason))
        }
        NormalizedAgentEventKind::Interrupted { reason } => {
            Some((ExecutionActivityKind::Interrupted, reason))
        }
        NormalizedAgentEventKind::UsageUpdated { .. } => None,
    }
}

fn truncate_summary(summary: &str) -> String {
    if summary.len() <= MAX_ACTIVITY_SUMMARY_BYTES {
        return summary.to_owned();
    }
    let content_limit = MAX_ACTIVITY_SUMMARY_BYTES - "…".len();
    let mut end = 0;
    for (index, character) in summary.char_indices() {
        let next_end = index + character.len_utf8();
        if next_end > content_limit {
            break;
        }
        end = next_end;
    }
    format!("{}…", &summary[..end])
}
