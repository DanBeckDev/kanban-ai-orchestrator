# ADR 0032: Discover models through the installed agent runtime

- Status: Accepted
- Date: 2026-08-10

## Context

Provider cards need to offer meaningful model and effort choices without
asking a person to understand provider infrastructure. ADR 0031 added a
second, application-owned API-key connection so the board could query each
provider directly. That is the wrong boundary: a person who is already signed
in to Codex, Claude Code, or Cline reasonably expects the board to use that
existing session.

It also creates two competing sources of truth. A model available through a
newly entered key may not match the account, organisation, model policy, or
endpoint configured in the installed agent. Cline Kanban's reported
configuration-drift failures reinforce that the board must not bypass the
client runtime with a separate provider request.

## Decision

- The normal product path uses only an installed agent's supported local
  SDK, app-server, or client protocol. It never asks for an API key and never
  writes provider credentials to a keychain, database, worktree, log, or UI
  state.
- Model discovery stays behind a provider-neutral `ProviderModelCatalogClient`
  boundary. The first implementation invokes Codex's local app server,
  completes its JSON-RPC initialisation, and requests `model/list`. The
  adapter maps only the returned public model ID, display name, and supported
  reasoning effort into the board's neutral settings contract.
- An installed runtime that cannot expose a documented local catalogue returns
  **Use provider default**, not a guessed static list. The card makes that
  limitation clear, keeps the provider-default choice available, and can retry
  discovery after the person changes settings in the provider's own app.
- The board can persist an explicit project model and effort override only
  after it came from that runtime. `Provider default` delegates entirely to the
  installed client. Native execution continues to translate the neutral
  preference only at the adapter boundary.
- Provider-specific SDK integration is additive: Cline's SDK/Core and the
  Claude Code Agent SDK may implement the same narrow catalogue boundary when
  their supported local APIs provide it. No provider becomes the shared core.

## Consequences

- Settings is one self-contained card per provider and does not duplicate
  onboarding or account configuration.
- Cross-platform code remains shared: the Rust desktop backend speaks one
  typed catalogue contract while small adapter modules own each runtime
  protocol.
- The board degrades honestly when a client cannot list models, preserving the
  provider's own configuration rather than presenting a misleading chooser.
