# ADR 0029: Offer managed Linear OAuth only when it is actually configured

- Status: Accepted
- Date: 2026-08-10

## Context

The previous Linear settings surface asked every person for an OAuth client ID
and redirect URI before they could understand whether Linear was optional,
available, read-only, or able to send a public update. That shifted
product-configuration ownership onto individual developers and made a
local-only board look incomplete.

The product requirements permit a normal product-managed OAuth path only when
the release owner has registered and supports that OAuth application. A public
client ID is not a secret, but inventing or silently sharing one would hide the
application owner, privacy notice, callback support, and credential-lifecycle
responsibilities that make the connection trustworthy.

BookCtx — *The Product-Minded Engineer*, Gergely Orosz, “Eliminate barriers to
entry” (chunk 10), recommends first unblocking the user and treating recurring
support friction as a product-design problem. Project inference: local-only
work must remain immediately usable; unavailable managed OAuth needs a direct
explanation and an advanced route for teams that already own the required
configuration.

## Decision

- A release can provide a product-managed, public Linear OAuth client ID at
  build time. When it does, the normal path offers a one-action read-only
  connection using the fixed, validated loopback callback.
- When no release-owned client is configured, normal UI explicitly says that a
  managed connection is unavailable and leaves any existing local links
  unchanged. Boards without a link remain local-only. It never asks a user for
  OAuth configuration on the normal path.
- Self-managed OAuth remains available under a collapsed **Use a self-managed
  Linear app** disclosure. It accepts only the public client ID and displays the
  fixed callback URL; it does not accept a client secret or token.
- Board creation and board home identify a board as local-only, Linear
  read-only, or linked execution. A link cannot be labelled linked execution
  until the existing narrow `comments:create` authorization is confirmed.
- Linked execution can prepare a bounded public update, but each update stays
  local until the person explicitly sends it. Existing outbox, idempotency, and
  conflict behavior remain unchanged.

## Consequences

- The app stays useful when a product-managed Linear application has not yet
  been registered, and it is honest about that limitation.
- The release owner must supply and support the product-managed client
  configuration before a one-click consumer connection is advertised.
- Teams that already operate their own Linear app retain a deliberate,
  documented advanced path without exposing secrets or credentials in board
  state.
