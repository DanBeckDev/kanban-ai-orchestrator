# ADR 0022: Use a dark-first, concise, accessible visual language

- Status: Accepted
- Date: 2026-08-09

## Context

The desktop currently uses its most prominent space for technical labels such
as "Execution authority" and "Current milestone." Those facts do not help a
person choose a board or begin useful work. Its long product name also behaves
as a description rather than a recognisable label. The user needs an expressive
interface, but saturated decoration, colour-only status, or a forced theme
would make the product harder—not easier—to use.

BookCtx's *The Product-Minded Engineer* frames a usable flow around discovery,
understanding, and use: names and signifiers must help people navigate the
journey rather than make them learn machine vocabulary. *Practical Web
Accessibility* adds that a compliant default cannot suit every person, so an
explicit, remembered appearance choice is necessary. W3C contrast guidance and
Radix's documented colour-scale composition provide the technical boundaries.

## Decision

- Present the product as **Kanban** in the desktop window, document title, and
  application chrome. Keep technical package names, application identifiers,
  repositories, and persisted data stable.
- Make Dark the first-visit default. Offer an explicit Dark/Light appearance
  control in the application chrome, apply the selected theme before React
  renders, and store the preference only in local browser storage.
- Define Dark tokens at `:root` and a Light override at `.light`; retain the
  `.dark` marker for shadcn's dark variants. Use neutral slate surfaces with an
  iris primary action/focus colour, while semantic success, warning, and
  destructive roles are used sparingly.
- Require every prominent persistent string to orient the person, name an
  available action, explain a current state/consequence, or provide necessary
  support detail. Move implementation facts out of first-use chrome and into
  contextual support/recovery disclosure.
- Meet the WCAG 2.1 AA 4.5:1 contrast requirement for normal text. Colour may
  reinforce state, but labels, icons, structure, and focus must carry the
  meaning independently. Keep visible focus treatment in both authored themes
  and support forced-colours overrides.

## Consequences

- The first screen leads with a short product identity and an understandable
  next step rather than architecture claims.
- Theme selection is local UI preference only; it cannot affect scheduling,
  policy, persistence, provider choice, or connector data.
- Components continue to consume semantic tokens. Product components do not
  introduce raw palette values or one-off dark-mode overrides.
- A concise copy style needs a later audit of detailed board and connector
  controls after the outcome-first board-home work is ready.
