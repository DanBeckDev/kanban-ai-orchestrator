# ADR 0027: Make task AI proposals durable, typed, and reviewable

- Status: Accepted
- Date: 2026-08-10

## Context

A task-level prompt is useful only if it can help a person act on the task. It
must not become an untracked chat box that changes task details, starts an
agent, retries work, or bypasses the established review and policy controls.
The product also needs a consistent meaning in manual and autonomous modes:
the user should see what task AI proposes, while autonomy remains a narrow
authority recorded for a particular board.

BookCtx — *AI Agents: The Definitive Guide*, Nicole Koenigstein, “Example
2-15. Approval gate for request” (chunk 4), illustrates the separation of a
structured proposal, explicit approval, and an effect. This decision applies
that separation to the project's own task lifecycle; it does not treat a
model's response as authorization.

## Decision

- A task prompt selects one closed, typed action: refine task details, add
  worker guidance, prepare start or restart, explain evidence, return for
  correction, or recover an interruption. The organiser response must return
  that same action and a small action-specific proposal; extra response fields
  are rejected.
- The daemon records a redacted, bounded prompt summary, safe recommendation
  and rationale, selected action, proposal, policy result, expected task
  version, authority revision, idempotency key, and outcome before it exposes
  the decision to the UI. Raw provider responses, credentials, commands, and
  unbounded task context are never stored.
- In manual mode, all state-changing actions become named `Apply`, `Reject`,
  or `Dismiss` decisions. Evidence explanation is read-only and may be shown
  immediately. Applying a proposal rechecks the task version and all existing
  state, review, policy, and launch controls immediately before an effect.
- In autonomous mode, the daemon compares the saved board-supervision revision
  and permitted-action set before acting. Autonomous task AI may only perform
  already-authorized guidance, explanation, start/retry/correction/recovery
  paths; it cannot refine a task specification, mark Done, weaken policy,
  perform protected Git, or write externally.
- A repeated request returns its durable decision rather than invoking or
  replaying the action again. On recovery after an uncertain interruption, a
  manual pending proposal becomes reviewable and an autonomous pending proposal
  is marked recovered without retrying its unknown side effect.

## Consequences

- The task detail view can be conversational without being a second authority:
  it shows only the safe, durable decision history and delegates all mutation
  to typed daemon commands.
- An approved guidance proposal is available to the next ticket-worker brief;
  it remains data, not an instruction that can escape the configured worker
  boundary.
- Adding a task-AI capability now requires a new typed action, validation,
  policy mapping, persistence behavior, UI language, and tests. This is
  deliberate: it prevents a provider prompt change from silently widening
  product authority.
