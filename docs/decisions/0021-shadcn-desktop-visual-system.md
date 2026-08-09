# ADR 0021: Use shadcn source-owned primitives for the desktop visual system

- Status: Accepted
- Date: 2026-08-09

## Context

The first desktop implementation is functionally capable but presents most
regions as adjacent dark, bordered rectangles. Repeated hand-written button,
input, notice, and card styles make visual hierarchy inconsistent and make
future UI work expensive. The product needs a coherent desktop interface that
can evolve without coupling the durable Rust domain, daemon authority, or
provider-neutral agent contract to a web component framework.

BookCtx — *Building Micro-Frontends*, Luca Mezzalira, "Implementing a Design
System" (chunk 6) supports a layered model of design tokens, generic primitives,
and product-specific compositions. It also cautions against premature
abstraction of domain components. The evidence therefore supports centralising
the visual language and basic controls, not a generic board-component framework.

## Decision

- Initialise the Vite client with shadcn/ui's Nova preset, Tailwind v4, and
  Radix accessibility primitives. Keep the CLI-generated component source in
  this repository rather than treating a vendor package as an opaque UI layer.
- Keep semantic CSS tokens in `src/styles.css`. Components consume roles such
  as `background`, `card`, `primary`, `muted`, `destructive`, `border`, and
  `ring`; product code must not encode one-off raw palette values in component
  classes.
- Use shadcn primitives for generic concerns: buttons, fields, inputs,
  textareas, badges, alerts, cards, empty states, separators, tabs, and
  tooltips. Compose board-specific views in `src/board` until a repeated,
  stable product pattern justifies extraction.
- Reserve cards for discrete, actionable units such as boards, work items, and
  focused forms. Page structure, grouping, and kanban columns use spacing,
  typography, and restrained separators instead of nested card shells.
- Preserve native HTML semantics and associated labels first. The Radix
  primitives supply focus and composite-widget behaviour; additional ARIA is
  used only when it communicates information native semantics cannot.
- Keep the shadcn agent skill in `.agents/skills` and `skills-lock.json` so
  Codex, Claude Code, and Cline receive the project-specific component rules.

## Consequences

- The desktop app gains a durable visual language while retaining full control
  of the generated component source and its theme.
- A new UI primitive can be added through the shadcn CLI and reviewed as local
  source. Updating a generated primitive requires a CLI diff rather than a
  blind overwrite.
- UI changes remain outside the Rust domain and command-boundary policy. Tests
  continue to exercise observable interactions through the existing injectable
  gateway.
- Theme tokens and generic primitives are shared deliberately; board-specific
  compositions are not prematurely generalised.
