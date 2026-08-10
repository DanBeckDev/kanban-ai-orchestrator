# ADR 0031: Keep role configuration and model discovery with each provider

- Status: Accepted
- Date: 2026-08-10

## Context

The first native-provider settings implementation split one decision across two
surfaces: people chose a provider from an availability card, then separately
chose an orchestrator or worker profile and entered model names in generic role
forms. This made the relationship between a provider, its two possible roles,
and its configuration needlessly hard to understand. It also asked people to
know account-specific model identifiers that the product can obtain from the
provider.

The Product-Minded Engineer, Gergely Orosz, “A Simulation” (chunk 3), supports
the project-specific inference that a configuration journey must make the
relationship between a choice and its consequence understandable at the moment
of choice. Official OpenAI, Anthropic, and Cline API documentation confirms
that a connected account can enumerate model information through each
provider's API boundary. The APIs expose different capability detail, so the
catalogue must remain adapter-owned rather than becoming domain state.

## Decision

- Settings renders one self-contained card per detected provider. A card owns
  its installed state, role selection, model list, effort choices, connection
  action, and refresh action. There is no duplicated generic provider picker or
  free-text model field on the normal path.
- An installed provider can be enabled independently for **Plan work** and
  **Work on tickets**. Selecting a role creates or reuses its safe native
  profile; model and effort remain separately persisted per role.
- A person explicitly connects a provider API before its account-specific
  model catalogue is loaded. The supplied key is stored only in the operating
  system keychain. The board database, board metadata, worktrees, logs,
  front-end state, and error messages never retain or echo the key.
- Catalogue refresh is explicit and is adapter I/O, not discovery. It uses the
  provider's documented API, returns provider-supplied model identifiers and
  labels, and presents them in a dropdown alongside **Provider default**.
  If an account is not connected or a refresh fails, the user can still choose
  Provider default and receives a specific next action instead of an invented
  model list.
- Effort is configured within the same provider card. Adapters expose only
  effort values their native invocation can express; where a provider's model
  API reports model-level effort capability, the card narrows the choices to
  that model. Provider-default remains the safe fallback.
- Native execution remains the source of truth for whether an account and
  selected model can actually run work. A model catalogue request never starts
  an agent, changes provider CLI authentication, or relaxes sandbox,
  confirmation, policy, review, or Done gates.

## Consequences

- Provider selection, model selection, and effort now form one comprehensible
  configuration path without tying project settings to any vendor schema.
- A user deliberately grants separate API-catalogue access when they want a
  live model list. Existing CLI authentication can continue to run work with
  Provider default even when it cannot enumerate models through an API.
- Future providers implement a small catalogue adapter and keychain credential
  namespace rather than adding their own Settings form or leaking credentials
  into durable board data.
