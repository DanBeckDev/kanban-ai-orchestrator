# Reliability and safety requirements

## Lessons from the reference project's issue history

The reference project's public issue history is a useful failure-mode catalogue. The product must explicitly defend against the following classes of problems:

| Failure mode | Required protection |
| --- | --- |
| Long terminal sessions freeze the board | Keep only safe normalized activity summaries: 1 KiB per entry, 128 entries per execution, 32 entries per request, and a virtualized card renderer. Never restore an unbounded terminal snapshot on the UI thread. |
| Restart loses task/session state | Persist domain events and execution checkpoints; reconcile rather than discard on restart. |
| UI state and runtime state diverge | Only the daemon transitions task state; the UI observes it. |
| Failed work appears complete | Keep `Failed`, `Interrupted`, `Cancelled`, and `Done` distinct; require evidence for `Done`. |
| Multiple dependencies launch too early | Require all incoming hard blockers; reject cycles; test high-cardinality graphs. |
| Worktree setup races or modifies the base repository | Treat provisioning as a recoverable transaction; validate project boundary and worktree identity before agent launch. |
| Shared ignored-file symlinks break projects | Offer isolated install, managed shared cache, or explicit project-approved linking; never force a single mode. |
| Provider differences fail silently | Adapter conformance tests, capability checks, visible context/budget exhaustion, and structured failure reasons. |
| Background UI stops automation | Daemon-owned scheduling with no browser timer dependency. |
| Platform-specific command/PTY failures | Native OS matrix tests; structured task specs rather than oversized command lines; explicit process-tree policy. |
| Agent escapes project scope | Filesystem and command policies enforced outside prompt text, with auditable exception flow. |

## Release gates

### Functional correctness

- A downstream task with two hard blockers does not start after only one completes.
- A dependency cycle is rejected before any agent starts.
- Cancellation requests stop the owned process tree or report an actionable inability to do so.
- A failed provider run cannot satisfy completion criteria.
- External sync conflicts enter a reconciliation queue and preserve both versions.
- A Linear comment whose remote result is ambiguous is marked `delivery_uncertain` and is never retried automatically.

### Recovery

- Kill the daemon while a task is running, restart, and recover its task history, workspace identity, evidence, and next actions.
- Interrupt workspace provisioning and retry without duplicate worktrees or base-repository mutations.
- Disconnect and reconnect the UI while work continues; scheduling and state remain correct.

### Performance

- A board with 100 active/recent cards remains interactive while terminal output streams.
- Switching between a large and small task requests no more than 32 safe activity entries at once and renders a fixed-height virtual window.
- An execution retains at most 128 activity entries; at most 32 completed feeds remain in memory, and daemon restart discards them.
- Sustained agent hooks/events use the existing execution monitor and do not create one heavyweight runtime process or worker per event.

### Security and privacy

- Tests prove an agent cannot access undeclared directories without an approved exception.
- Secrets do not appear in SQLite board fields, logs, screenshots, Linear comments, or diagnostics by default.
- Every protected action has an audit entry with policy result.

### Compatibility

- Adapter conformance tests run against each supported agent.
- Core workspace, state-machine, and connector tests run on macOS, Windows, and Linux in CI from the first implementation milestone.
- Packaging, credential, and provider process capability evidence is tracked in [platform release evidence](platform-release.md); the product never claims an unverified native capability.
