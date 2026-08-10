# ADR 0030: Make native role defaults effective at the adapter boundary

- Status: Accepted
- Date: 2026-08-10

## Context

The project could persist separate organiser and ticket-worker model and effort
preferences, but the installed-provider flow created only ticket workers. The
organiser still required a raw executable bridge, and native worker launches
did not consistently receive stored preferences. That made the normal settings
surface promise choices which its process boundary could silently ignore.

BookCtx — *The Site Reliability Workbook*, “Configuration Design and Best
Practices” — supports the project-specific decision to minimise mandatory
configuration, make safe defaults obvious, and validate semantics before work
starts. Provider CLIs do not expose a single reliable model catalogue, so a
static list would be misleading.

## Decision

- Settings creates or reuses a safe native profile independently for each role:
  orchestrator and ticket worker. The normal path uses recognised installed
  programs and never asks for command arguments, credentials, permission
  switches, or provider-private state.
- Each role defaults to the provider's own model. A named model is an explicit
  optional override; effort remains provider-neutral (`Focused`, `Balanced`,
  `Thorough`) until the adapter translates it. There is no static model list.
- Codex, Claude Code, and Cline/ClinePass translation belongs only in native
  adapter code. Worker invocations use the saved task preference. Native
  organiser invocations use the saved organiser preference for plan,
  supervision, and ticket-effect assessments.
- Native organiser output is bounded, extracted from the provider's
  non-interactive response protocol, then parsed against the existing strict
  plan/supervision/ticket-effect contracts. The daemon retains confirmation,
  policy, review, and Done gates.
- A structured generic bridge is still supported, but it may use only provider
  defaults. Saving an explicit model or effort against it fails with a clear
  role-specific explanation.

## Consequences

- The promise made by Settings matches real CLI invocation without coupling the
  domain model to a vendor protocol.
- Existing persisted bridge profiles remain readable because an omitted profile
  kind defaults to the structured bridge protocol.
- Provider capability changes remain contained to small adapter modules and
  their fixtures. The UI does not guess account-specific model availability.
