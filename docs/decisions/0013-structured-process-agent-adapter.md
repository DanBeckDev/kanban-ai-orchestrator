# ADR 0013: Use a bounded structured-process adapter as the provider boundary

- Status: Accepted
- Date: 2026-08-08

## Context

The core accepts normalized agent lifecycle events, but a provider-neutral desktop also needs a concrete outer boundary for local agent CLIs and future provider wrappers. Terminal prose is neither a reliable completion signal nor a safe process protocol. Unbounded stdout can also freeze or exhaust the desktop process.

## Decision

- A configured process adapter starts a declared executable with structured argument values, never through a shell string. It uses the assigned workspace as its current directory and supplies the task brief over standard input, which avoids operating-system command-line length limits.
- The adapter reads only newline-delimited JSON lifecycle events from standard output. Each line has a monotonic `sequence` and a flattened normalized event shape, for example `{"sequence":1,"type":"activity","summary":"Inspecting the repository"}`.
- A single event line is limited to 64 KiB and a session retains at most 1,000 events. Malformed, out-of-order, oversized, or excessive output becomes one normalized `failed` event and stops the child process.
- Standard error is not retained by this adapter. Provider wrappers must publish selected safe lifecycle summaries instead of leaking raw transcripts or secrets into durable board data.
- The generic adapter exposes structured streaming but honestly reports feedback, resume, and process-tree interruption as unsupported. It may not claim that killing a direct child cancels its descendants. Platform-specific adapters can add those capabilities only with a tested process-tree implementation.

## Consequences

- Codex CLI, Claude Code, and other providers can use small wrappers that translate their native protocols into this one event stream without changing the daemon state machine.
- A provider `completed` event still requests only `Review`; `Done` remains subject to evidence and human-review rules.
- The initial adapter is a safe capability boundary, not the complete execution controller. Durable session checkpoints, worktree launch authorization, and platform process-tree cancellation remain separate execution-layer work.
