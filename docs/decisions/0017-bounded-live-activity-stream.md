# ADR 0017: Expose bounded live activity separately from durable board state

- Status: Accepted
- Date: 2026-08-09

## Context

An agent's current progress is useful while a task runs, but a provider transcript is untrusted, potentially large, and may contain sensitive context. Putting that output in a board snapshot or SQLite would make opening a card dependent on an unbounded payload, blur durable decision history with transient output, and turn a UI convenience into a privacy-retention promise.

## Decision

- The execution monitor records safe normalized event summaries in an in-memory activity stream after the existing durable lifecycle update succeeds. It starts no additional runtime, worker, or thread for an activity event.
- A stream keeps at most 128 entries for one execution. Each summary is UTF-8-safe and capped at 1,024 bytes. Usage events are already visible as structured execution usage and do not enter the human-readable activity feed.
- The local command returns at most 32 entries after a caller-supplied event sequence. The React card fetches pages asynchronously and virtualizes fixed-height rows, so rendering work is bounded even for the complete retained stream.
- Active feeds move to a bounded recent collection when their execution stops. The collection retains at most 32 completed executions, then evicts the oldest feed. App restart also discards activity feeds.
- Board snapshots and SQLite retain only durable lifecycle, evidence, and decision-history facts. Raw provider transcripts, prompt context, and terminal output are never activity-stream inputs or durable board fields.

## Consequences

- A card can show short-lived, provider-neutral progress without making task state or review evidence depend on a terminal renderer.
- Users can inspect a just-finished execution's safe activity summary, but activity is intentionally not a recovery artifact. Durable execution status, evidence, workspace identity, and decision history remain the restart-safe record.
- This imposes explicit retention and payload boundaries. A future searchable transcript feature would require a separate consent, encryption, privacy, and storage decision rather than extending this API silently.
