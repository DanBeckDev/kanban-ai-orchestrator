# Linear integration

## Product contract

Linear is a first-class planning-system integration. A linked board lets teams retain their Linear workflow while using this app to orchestrate agent execution, worktrees, reviews, and evidence.

The initial connector is not a background copy process. It provides user-triggered, provenance-preserving import; any future synchronization must add explicit permission boundaries, idempotency, and visible conflict resolution.

## Connection modes

| Mode | Permissions | Use case |
| --- | --- | --- |
| Read-only | Read issues, projects, labels, and relationships | Observe a Linear plan before adoption |
| Linked execution | Read plus selected issue/comment/status writes | Run existing Linear work with agent evidence |
| Plan publishing | Linked execution plus issue creation | Approve an orchestrator plan and publish its tickets to Linear |

OAuth is the release path. A personal API key may be offered only for private development/testing and is stored in the OS keychain.

## Desktop OAuth connection

The desktop connection uses Linear's authorization-code flow with PKCE. The user creates a Linear OAuth application and configures this exact redirect URI:

`http://127.0.0.1:38471/linear/oauth/callback`

The app accepts only an explicit HTTP loopback IP address, port, and path. It generates a fresh state value and S256 verifier for every attempt, opens the system browser, and accepts one bounded callback. The callback state must match before the daemon exchanges the code. The public client ID is supplied in the UI; a client secret is neither requested nor embedded in the desktop app.

Access and refresh tokens, together with the public connection metadata needed to refresh them, are serialized only into the operating system credential store: Keychain Services on macOS, Credential Manager on Windows, and Secret Service on Linux. SQLite, diagnostics, activity history, and board snapshots never contain a token. A refresh happens only when an access token is within one minute of expiry; the existing credential is preserved until its replacement has been validated and saved. This produces no background polling and respects Linear's guidance to avoid it.

The desktop currently requests the least-privileged `read` scope. Linked execution may be selected for an imported task, but remote writes, app-actor installation, comments, status mapping, outbox processing, and webhooks are still separate, opt-in work; choosing the mode must not grant those powers early. See Linear's [OAuth documentation](https://linear.app/developers/oauth-2-0-authentication) and [rate-limit guidance](https://linear.app/developers/rate-limiting).

## Current API verification

Verified against Linear's developer documentation on 2026-08-09: Linear completed its OAuth refresh-token migration on 2026-04-01. The connector accepts both the current string and legacy array scope shapes, requires a replacement refresh token, and replaces the credential-store value only after validating the complete response. The connector deliberately makes no timer-driven requests: it refreshes only for an explicit authenticated action when the access token is within one minute of expiry. This follows Linear's guidance to avoid polling, request only the fields needed, order by `updatedAt`, and use bounded pagination.

## Authenticated issue retrieval

After the connection status is explicitly `connected`, the user may press **Load my assigned Linear issues**. That one user action sends a read-only GraphQL `viewer.assignedIssues` query, ordered by `updatedAt` and bounded to 50 issue summaries (`id`, `identifier`, `title`, and `url`). It never creates, changes, comments on, or transitions a Linear issue, and it does not poll. Choosing a returned summary merely pre-fills the existing local import form; the daemon still validates the immutable ID and Linear HTTPS URL before it creates a durable link.

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
