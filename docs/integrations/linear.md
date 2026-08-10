# Linear integration

## Product contract

Linear is a first-class planning-system integration. A linked board lets teams retain their Linear workflow while using this app to orchestrate agent execution, worktrees, reviews, and evidence.

The initial connector is not a background copy process. It provides user-triggered, provenance-preserving import; any future synchronization must add explicit permission boundaries, idempotency, and visible conflict resolution.

## Connection modes

| Mode | Permissions | Use case |
| --- | --- | --- |
| Read-only | Read issues, projects, labels, and relationships | Observe a Linear plan before adoption |
| Linked execution | Read plus an explicitly reauthorized, manually sent safe comment | Run existing Linear work with agent evidence while retaining the human as the remote-write authority |
| Plan publishing | Future: linked execution plus issue creation | Approve an orchestrator plan and publish its tickets to Linear |

OAuth is the release path. A personal API key may be offered only for private development/testing and is stored in the OS keychain.

## Desktop OAuth connection

The desktop connection uses Linear's authorization-code flow with PKCE. A
release that has a supported, product-managed Linear OAuth application supplies
its public client ID through `VITE_LINEAR_OAUTH_CLIENT_ID` at build time and
offers a one-action **Connect Linear** read-only path. When that release
configuration is absent, the normal UI says so plainly and leaves the board
unchanged; a board without an existing link remains local-only. It does not ask
an individual to create or configure an OAuth app.

Teams that already own a Linear OAuth application can deliberately open
**Use a self-managed Linear app** in Advanced setup. They supply only that
application's public client ID and configure this exact redirect URI:

`http://127.0.0.1:38471/linear/oauth/callback`

The app accepts only an explicit HTTP loopback IP address, port, and path. It
generates a fresh state value and S256 verifier for every attempt, opens the
system browser, and accepts one bounded callback. The callback state must match
before the daemon exchanges the code. A client secret is neither requested nor
embedded in the desktop app.

Access and refresh tokens, together with the public connection metadata needed to refresh them, are serialized only into the operating system credential store: Keychain Services on macOS, Credential Manager on Windows, and Secret Service on Linux. SQLite, diagnostics, activity history, and board snapshots never contain a token. A refresh happens only when an access token is within one minute of expiry; the existing credential is preserved until its replacement has been validated and saved. This produces no background polling and respects Linear's guidance to avoid it.

The desktop initially requests the least-privileged `read` scope. A connected user may explicitly reauthorize only manually sent comments with `read,comments:create`; it does not request Linear's broad `write` scope. A previously granted `write` token is recognized as sufficient, but the app never broadens its authorization automatically. Status mapping, app-actor installation, issue creation, webhooks, and automatic remote writes remain separate work. See Linear's [OAuth documentation](https://linear.app/developers/oauth-2-0-authentication) and [rate-limit guidance](https://linear.app/developers/rate-limiting).

## Current API verification

Verified against Linear's developer documentation on 2026-08-09: Linear completed its OAuth refresh-token migration on 2026-04-01. The connector accepts both the current string and legacy array scope shapes, requires a replacement refresh token, and replaces the credential-store value only after validating the complete response. The connector deliberately makes no timer-driven requests: it refreshes only for an explicit authenticated action when the access token is within one minute of expiry. This follows Linear's guidance to avoid polling, request only the fields needed, order by `updatedAt`, and use bounded pagination.

## Authenticated issue retrieval

After the connection status is explicitly `connected`, the user may press **Load my assigned Linear issues**. That one user action sends a read-only GraphQL `viewer.assignedIssues` query, ordered by `updatedAt` and bounded to 50 issue summaries (`id`, `identifier`, `title`, and `url`). It never creates, changes, comments on, or transitions a Linear issue, and it does not poll. Choosing a returned summary merely pre-fills the existing local import form; the daemon still validates the immutable ID and Linear HTTPS URL before it creates a durable link.

## Entry and access-mode guidance

Every new board starts **local-only**: it does not load from or send to Linear.
The board header and Linear settings name that state directly. A locally linked
issue is **read-only** unless the user has separately granted the narrow
`comments:create` scope. Read-only mode can load and link issues but cannot
send an update. After the narrow scope is confirmed, a person may deliberately
choose **linked execution** while linking an issue. That enables preparing a
bounded public update, but the outbox still requires an explicit **Send** for
each update.

This distinction is visible before importing an issue or queueing a comment;
the app never infers linked execution from an unconnected or read-only session.

## Mapping and provenance

Each external link stores the immutable Linear UUID, display identifier, URL, provenance, and connection mode. Each reconciliation record stores the observed remote revision, field, and both values. Never use a mutable title or identifier alone as the key.

| Field | Default source | Sync rule |
| --- | --- | --- |
| Team, project, cycle, assignee, estimate | Linear | Read into the app; app changes require explicit user action |
| Agent execution, workspace, logs, cost, evidence | App | Never mirror raw details to Linear automatically |
| Title, description | Local board and Linear | An explicit refresh records a comparison; a mismatch is visible and never overwrites either side |
| Workflow state | Local board and Linear | An explicit refresh records a comparison; automatic status writes are not implemented |
| Dependencies | Both, with provenance | Import Linear blockers and publish eligible app-created hard blockers when enabled |

## Dependency semantics

Linear relationships must be transformed into typed app dependencies with their source recorded. A Linear blocker remains a hard blocker until its mapped completion condition is satisfied. Local-only `contract` and `soft` edges never overwrite a Linear relationship unless a user deliberately publishes them.

The dependency graph view shows both systems' edges, identifies imported/external blockers, and warns when the external plan would produce a hard-dependency cycle.

## Sync design

1. A user queues a concise public Linear comment locally with a connector-local idempotency key; the outbox payload is immutable.
2. Sending is a deliberate user action. The daemon claims the item, releases the state lock for the GraphQL request, and records the confirmed result afterward.
3. A confirmed send is `delivered`. A timeout, rejected response, process stop, or restart while sending becomes `delivery_uncertain`; it is never retried automatically because the remote side may already have accepted it.
4. An explicit **Refresh shared fields** action reads the linked issue's title, description, and workflow state and records immutable reconciliation items keyed by Linear's `updatedAt` revision.
5. When local and remote values differ, the board shows both as `needs_resolution`; refresh cannot overwrite a work item, dependency, or Linear issue.

## Webhook and desktop constraint

Linear webhooks require a public HTTPS endpoint. The initial local-first connector therefore uses explicit/efficient refresh and incremental queries. A later optional, self-hostable or hosted relay can validate Linear webhook signatures and forward events to enrolled desktops. The relay must not require code, transcripts, or provider credentials.

## Privacy controls

- The user selects which Linear teams/projects may be connected.
- Manual comments require an explicit targeted OAuth reauthorization and an explicit **Send** action for each queued item.
- A comment is generated only from a user-entered, one-line public summary (maximum 512 bytes) and a typed local task state; agent transcripts, secrets, command output, and private diffs are excluded by the input model and validation.
- Pending, delivered, uncertain, matched, and conflicting records are visible in the board. The app never silently resolves a conflict or duplicates an uncertain comment.
