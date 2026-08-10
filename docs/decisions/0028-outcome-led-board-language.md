# ADR 0028: Use outcome-led language and role-specific AI defaults

- Status: Accepted
- Date: 2026-08-10

## Context

The board had accumulated labels that mirrored implementation concepts rather
than the user's next decision: “Link a GitHub repository” hid the fact that it
clones; “Organiser” conflicted with the product's orchestration vocabulary;
“Bounded,” “daemon-checked,” and generic “Try again” asked people to infer a
technical model or an unspecified action. The project also persisted a model
preference for its two AI roles but exposed only an inert “Provider default”
control, so people could not set a model for either role.

BookCtx — *The Product-Minded Engineer*, Gergely Orosz, “Chapter Summary”
(chunk 16), supports making the safe, productive action easiest to discover,
using contextual defaults, and pairing an affordance with a clear signifier.
The Web Interface Guidelines review adds active voice, specific action labels,
and errors that include a next step.

## Decision

- Maintain a source-owned content inventory for all persistent board controls.
  Every string must orient, name an action, explain a consequence, or provide
  necessary support; otherwise it is removed.
- Use the user's workflow vocabulary in normal paths: clone or use a local
  repository; orchestrator; task worker; review; recover; and specific reload
  actions. Technical process terms remain only in advanced setup or support.
- Describe errors as a recoverable outcome and next action. Do not display raw
  local-service or provider messages in ordinary board flows.
- Keep separate project defaults for orchestrator and ticket worker. For each,
  a blank model name delegates to the provider default, while a named model is
  a deliberate preference passed through the existing validated provider-neutral
  model contract. Effort remains a separate preference.

## Consequences

- A first-time user can choose between a clone and an existing repository
  without having to interpret “link,” and a returning user sees specific
  recovery actions rather than a generic retry.
- Kanban does not invent an installed-provider model list. Providers differ in
  supported model names and local authentication; a named model is explicit and
  the adapter remains responsible for its native invocation.
- The advanced planner/profile configuration still exposes program and argument
  details for team-managed bridges. It is not placed in the primary board flow.
- Future UI work must update `docs/product/content-inventory.md` alongside any
  new persistent board control.
