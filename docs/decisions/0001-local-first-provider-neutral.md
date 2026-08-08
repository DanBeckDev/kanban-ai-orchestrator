# ADR 0001: Local-first, provider-neutral execution

- Status: Accepted
- Date: 2026-08-08

## Context

The product must coordinate AI coding work without forcing users onto one model provider or a hosted execution service. It also needs to run private repositories and local CLI agents safely.

## Decision

Run the authoritative board, scheduler, policy engine, worktree manager, and event store locally. Integrate coding agents through a provider-neutral adapter contract. Keep external services—including Linear—behind optional connectors.

## Consequences

- The product remains useful offline except for the chosen agent/provider/connector.
- Credential and process isolation are desktop concerns and must be tested across operating systems.
- Real-time external webhooks require an optional relay or self-hosted component; they cannot be assumed in the local core.
- A vendor-specific feature must not leak into core domain state.
