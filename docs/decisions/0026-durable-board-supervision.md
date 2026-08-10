# ADR 0026: Persist autonomous supervision as daemon-owned board state

- Status: Accepted
- Date: 2026-08-10

## Context

Autonomous coordination was previously represented by a browser preference and
an effect that invoked the daemon when the visible board changed. That makes
the UI an accidental scheduler: closing a window, reopening a board, or
receiving the same event twice can change whether work starts. It also leaves
no durable explanation of what authority was enabled, which organiser and
worker defaults were selected, or why a coordination action was attempted.

The accepted product requirements require bounded autonomy. The organiser may
reason over normalized board facts and propose a small set of management
actions, but a local daemon must still check state, limits, policy, and durable
audit immediately before any side effect.

BookCtx — *AI Agents: The Definitive Guide*, Nicole Koenigstein, “From LLMs to
Agents: The Foundational Blueprint” (chunk 1), describes bounded autonomy as
explicit state, events, guards, actions, safeguards, and termination. The
following design is a project-specific application of that model, not a claim
that an organiser's text is authoritative.

## Decision

- Store one board-supervision record in SQLite. It captures the mode, a
  snapshot of the selected organiser and ticket-worker defaults, the bounded
  concurrency/retry limits, permitted coordination actions, the named local
  actor, pause details, and a monotonically increasing revision.
- Store every supervision decision separately before an effect. Each record
  contains only a typed action, an affected work item when applicable, a
  concise recommendation and rationale, the policy result, outcome, expected
  work-item version, and idempotency key. Raw provider output, credentials,
  commands, worktree paths, and chain-of-thought are outside the record.
- The daemon serializes supervision attempts independently from worker launch.
  It rejects a stale expected work-item version, reuses a matching durable
  decision on duplicate delivery, and reconciles unfinished decisions before
  making another attempt.
- Manual mode may record an organiser recommendation but cannot perform its
  action. Autonomous mode may prepare dependency-safe work, start a ready
  worker, perform one bounded retry, or return work for correction only when
  the persisted scope and existing policy allow it. It cannot mark work Done,
  install/authenticate a provider, relax policy, perform protected Git, or
  write to an external connector.
- The frontend reads and changes this daemon-owned record. It contains no
  local-storage coordination mode and no timer/effect that schedules work.

## Consequences

- Automation survives a UI reconnect and can explain a start, denial, pause,
  retry, correction, stale decision, or recovery from durable data.
- The selected organiser runs through the same provider-neutral, bounded direct
  process boundary as planning. It receives only the normalized input contract
  and can return one strict typed recommendation. A future native organiser
  adapter may improve the bridge, but it cannot expand the persisted action
  vocabulary or bypass daemon guards.
- Changing an automation limit or action requires an explicit, reviewable
  board-record update rather than a frontend deployment or a hidden browser
  preference.
