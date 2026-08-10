# ADR 0034: Load Cline model choices through its installed SDK

- Status: Accepted
- Date: 2026-08-10

## Context

ADR 0032 establishes one account/configuration source: the agent already
installed and authenticated on the person's computer. Cline's non-interactive
configuration command opens its own terminal UI, so it cannot safely serve as a
model-list protocol. A static Cline list would equally be wrong: entitlement,
availability, and reasoning support vary by the signed-in Cline provider.

Cline's published Core SDK exposes local-provider model metadata. Its CLI also
documents a model identifier option and the native thinking values it accepts.
The board needs to expose that existing choice without reading provider settings
files, creating a Cline session, or introducing an API key connection.

## Decision

- Cline's adapter resolves the already-discovered Cline launcher to a verified
  installed package layout containing Cline's published Core SDK. It invokes a
  fixed Node module query that calls the SDK's local provider-model API and
  returns only public `id`, `name`, and `supportsReasoning` fields.
- The query is non-interactive, receives no standard input, discards standard
  error, and is bounded to ten seconds and 256 KiB of standard output. It never
  passes a prompt, authentication flag, or API key; it does not parse Cline's
  settings files, model files, or keychain data. The SDK remains responsible for
  its own authenticated local configuration.
- The provider-neutral catalogue maps a Cline reasoning-capable model only to
  Cline's native `low`, `medium`, `high`, and `xhigh` thinking levels. The
  native adapter sends an explicitly selected model as `--model` and an allowed
  level as `--thinking`; Provider default sends neither preference.
- If the launcher cannot be resolved to the expected SDK layout, Node cannot
  run the fixed query, or its bounded response is invalid, the adapter returns
  the existing **unavailable** state with a refresh action. It never guesses a
  Cline model list or asks the person to duplicate account setup.

## Consequences

- Cline becomes a first-class self-contained provider card, alongside Codex and
  Claude Code, while the shared model-settings contract remains vendor-neutral.
- The only Cline-specific dependency is a narrow outer adapter. Windows, Linux,
  and macOS use the same Rust boundary; the installed Cline package/launcher is
  the platform-specific concern.
- A direct native Cline binary or custom launcher that lacks the published Node
  package layout falls back truthfully. Supporting another documented local
  catalogue protocol is a future adapter change, not a shared settings rewrite.

## Evidence

- [Cline CLI README](https://github.com/cline/cline/blob/main/apps/cli/README.md)
  documents the CLI's local provider, model, and thinking controls.
- [Cline CLI development guide](https://github.com/cline/cline/blob/main/apps/cli/DEVELOPMENT.md)
  documents provider configuration through Cline's supported surfaces rather
  than manual settings-file edits.
- [Cline model reference](https://docs.cline.bot/api/models) documents model IDs
  and model-specific reasoning support.
- BookCtx — *The Product-Minded Engineer*, Gergely Orosz, “Chapter Summary”
  (chunk 16) supports the decision to expose real safe affordances and make
  recovery explicit. The Cline-specific architecture above is the project's
  inference from that principle and the official provider interfaces.
