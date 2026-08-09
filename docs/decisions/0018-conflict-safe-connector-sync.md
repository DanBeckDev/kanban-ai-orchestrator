# ADR 0018: Use a durable, conflict-safe connector outbox

- Status: Accepted
- Date: 2026-08-09

## Context

Linear-linked work can change in the local board and in Linear at the same time. A connector
must not make an external write while holding the daemon's state lock, silently apply a remote
value over local work, or retry an ambiguous comment delivery until it creates duplicates.
Agent transcripts, command output, secrets, and private diffs are also not valid external-comment
inputs.

## Decision

- Model connector delivery as durable local intent. Every item has a connector-local idempotency
  key, one immutable safe payload, and an explicit state: `pending`, `delivering`, `delivered`,
  or `delivery_uncertain`.
- Claim an item in one short SQLite transaction, release the daemon lock for provider I/O, then
  record the outcome in a second short transaction. Restart converts an in-flight claim to
  `delivery_uncertain`; it never automatically repeats an externally ambiguous write.
- Generate Linear comments only from a bounded, explicitly user-approved public summary and typed
  task facts. The input model has no transcript, terminal-output, secret, or diff field, and the
  formatter rejects common credential and patch markers.
- Record every observed shared-field comparison as an immutable reconciliation outcome. A mismatch
  is `needs_resolution` and preserves both values; connector input cannot mutate a work item or
  dependency graph directly.
- Keep the domain vocabulary connector-neutral. Linear GraphQL and OAuth scope checks remain an
  outer adapter that consumes a claimed comment item only after a user explicitly requests
  delivery.

## Consequences

- The board can show exactly what is pending, sent, uncertain, or conflicting after restart.
- A network timeout cannot be treated as proof that a comment failed, so the user is never exposed
  to automatic duplicate comment retries.
- A future webhook or explicit refresh can submit the same reconciliation observation safely; its
  immutable external revision is the de-duplication key.
- Comment and status write permissions remain opt-in. The existing read-only OAuth connection does
  not acquire mutation authority merely because a board has a durable outbox.
