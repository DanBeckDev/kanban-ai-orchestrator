# Linear integration

## Product contract

Linear is a first-class planning-system integration. A linked board lets teams retain their Linear workflow while using this app to orchestrate agent execution, worktrees, reviews, and evidence.

The connector is not a background copy process. It is a synchronized mapping with provenance, permission boundaries, idempotency, and visible conflict resolution.

## Connection modes

| Mode | Permissions | Use case |
| --- | --- | --- |
| Read-only | Read issues, projects, labels, and relationships | Observe a Linear plan before adoption |
| Linked execution | Read plus selected issue/comment/status writes | Run existing Linear work with agent evidence |
| Plan publishing | Linked execution plus issue creation | Approve an orchestrator plan and publish its tickets to Linear |

OAuth is the release path. A personal API key may be offered only for private development/testing and is stored in the OS keychain.

## Mapping and provenance

Each link stores the immutable Linear UUID, display identifier, URL, last observed revision, field provenance, and sync state. Never use a mutable title or identifier alone as the key.

| Field | Default source | Sync rule |
| --- | --- | --- |
| Team, project, cycle, assignee, estimate | Linear | Read into the app; app changes require explicit user action |
| Agent execution, workspace, logs, cost, evidence | App | Never mirror raw details to Linear automatically |
| Title, description, priority | Configured per board | Present conflicts for review; no last-write-wins |
| Workflow state | Configured mapping | Mirror only approved state transitions |
| Dependencies | Both, with provenance | Import Linear blockers and publish eligible app-created hard blockers when enabled |

## Dependency semantics

Linear relationships must be transformed into typed app dependencies with their source recorded. A Linear blocker remains a hard blocker until its mapped completion condition is satisfied. Local-only `contract` and `soft` edges never overwrite a Linear relationship unless a user deliberately publishes them.

The dependency graph view shows both systems' edges, identifies imported/external blockers, and warns when the external plan would produce a hard-dependency cycle.

## Sync design

1. Write local changes to a durable outbox with an idempotency key.
2. Apply remote events to an inbox, validate them against the graph and state machine, then record a reconciliation outcome.
3. When both sides changed a shared field, create a visible reconciliation item; do not overwrite either value.
4. Batch and deduplicate outbound status/comments to avoid noisy activity logs.
5. A deleted or inaccessible Linear issue unlinks into a recoverable local state; it never deletes local execution history.

## Webhook and desktop constraint

Linear webhooks require a public HTTPS endpoint. The initial local-first connector therefore uses explicit/efficient refresh and incremental queries. A later optional, self-hostable or hosted relay can validate Linear webhook signatures and forward events to enrolled desktops. The relay must not require code, transcripts, or provider credentials.

## Privacy controls

- The user selects which Linear teams/projects may be connected.
- Status mapping and automatic comments are opt-in per board.
- Agent transcript, secrets, command output, and private diffs are excluded from Linear by default.
- Generated comments are previewable when a board is configured for manual external updates.
