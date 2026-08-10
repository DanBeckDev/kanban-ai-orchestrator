# ADR 0024: Keep the board focused and choose installed agents in Settings

- Status: Accepted
- Date: 2026-08-09

## Context

The initial desktop board placed plan proposal, manual task creation, dependency
editing, worker profiles, planner profiles, Linear connection, import, and
outbox controls in a sticky panel beside every state column. Each card then
repeated long execution, evidence, review, and recovery forms. The result made
the core question — “What work needs attention, and what should happen next?” —
harder to answer.

Direct product feedback compared this unfavourably with Cline Kanban's focused
board and Settings experience. Cline's public documentation describes its
useful interaction model: it detects an installed local CLI, keeps each task in
its own worktree, lets users link dependent cards, and opens a card to inspect
the detailed agent work. Its settings UI makes provider choice recognisable
instead of exposing CLI arguments on the board.

BookCtx — *The Staff Engineer's Path*, “Clarifying” — supports shared mental
models, familiar names, and visual structure that connect related information
instead of presenting unconnected facts. *The Site Reliability Workbook*,
“Simplicity” — identifies administrative diversity as a system-complexity cost
and recommends deliberate end-to-end simplification. The decision therefore
removes configuration from the delivery surface rather than merely restyling it.

## Decision

- The default board is a focused delivery surface. It renders compact task cards
  in explicit state columns, dependency/attention signals, and a small action
  bar. Planning, task creation, dependency editing, and settings are explicit
  separate surfaces.
- A card opens one task detail surface. Only that surface renders its execution,
  review, evidence, recovery, and full dependency controls. It remains the
  keyboard-accessible alternative to any future dependency visualisation.
- Settings has a first-class **Agent** section. It resolves only the fixed,
  trusted program names `codex`, `claude`, and `cline` on `PATH`; it does not run
  those programs. It labels each option Installed or Not installed.
- Choosing an installed option creates or reuses the product's safe native
  profile and makes that profile the default for task runs. Adapter-owned
  protocol, sandbox, worktree, provider, credential, and approval arguments
  remain uneditable on the normal path. An advanced custom-profile form stays
  available in Settings for deliberately configured bridges.
- A Not installed option can link to official installation guidance, but Kanban
  does not install a CLI, trigger account login, or bypass permissions.

## Consequences

- New and vibe-coding users can recognise the task board and choose a locally
  installed agent without learning adapter or command-line vocabulary.
- Existing profiles and all daemon policy, worktree, plan-confirmation, Linear,
  review, and evidence boundaries remain authoritative and backwards compatible.
- Provider discovery demonstrates executable presence only. It does not claim
  that an account is authenticated or that a provider is healthy; a task start
  still returns the actual actionable runtime result.
- Native organiser-provider parity is recorded in ADR 0030. An accessible
  dependency graph remains separate tracked work. This decision created the
  clear settings and task-detail boundaries those later slices need.
