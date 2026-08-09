# ADR 0007: Normalized agent lifecycle contract

- Status: Accepted
- Date: 2026-08-08

## Context

Codex, Claude Code, direct APIs, and future local agents expose different session and event protocols. The local daemon needs a stable way to discover capabilities, start or resume an attempt, provide feedback, request cancellation, check health, and ingest progress without making a provider event authoritative task state.

## Decision

- Define a Rust adapter contract around application operations: discovery, start, resume, feedback, interrupt, terminate, structured event streaming, and health checks.
- Normalize lifecycle events with a monotonically increasing adapter sequence. Reject duplicate and out-of-order input before it can change a work item.
- Treat `usage_updated` values as cumulative checkpoints for one execution, never per-event deltas. A checkpoint may stay the same or increase, but may not reduce an already recorded token or cost value.
- Map `approval_requested` and `awaiting_input` to `AwaitingInput`; map `completed` and `awaiting_review` only to `Review`; map failures and interruptions to their distinct recovery states. Activity and usage are informational.
- Route every proposed state through the existing daemon-owned transition guard. No adapter event can request `Done`, and a rejected transition does not consume its event sequence.
- Maintain a deterministic fake adapter as the contract test double. Provider implementations arrive in EXEC-003 and must pass the same lifecycle conformance cases.

## Consequences

- Provider-specific event formats stay at the outer boundary while the core keeps one lifecycle vocabulary.
- An agent's final message cannot bypass review or completion evidence.
- Sequence handling is deterministic and testable without processes, PTYs, or provider credentials.
