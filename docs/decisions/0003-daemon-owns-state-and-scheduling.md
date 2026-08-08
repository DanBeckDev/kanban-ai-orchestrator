# ADR 0003: The local daemon owns state transitions and scheduling

- Status: Accepted
- Date: 2026-08-08

## Context

Agent hooks, terminals, browser windows, WebSockets, and external APIs can disconnect, duplicate events, or fail. UI-timer-driven automation becomes unreliable when a window is backgrounded or closed.

## Decision

Use a durable local daemon as the sole authority for task transitions, dependency evaluation, policy checks, and scheduling. The UI is a client; agent/connector events are validated input to the daemon's state machine.

## Consequences

- Automation continues correctly while the UI is closed or backgrounded.
- Restart recovery is possible using durable events and reconciliation.
- Adapters must be idempotent and expose enough information to recover uncertain runs.
- UI performance does not govern execution correctness.
