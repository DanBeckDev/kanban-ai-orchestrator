# ADR 0035: Separate outcome orchestration from ticket execution

- Status: Accepted
- Date: 2026-08-10

## Context

The board combined an outcome composer, automation controls, delivery summary,
and ticket lanes on one screen. That turned the first screen into a dashboard of
unrelated choices and did not match the approved wireframes. A person needs a
simple place to tell the orchestrator what outcome they want and observe its
work, then a separate place to work through the created tickets.

BookCtx — *AI Agents: The Definitive Guide*, Nicole Koenigstein, “Foundational
Evaluation and Operational Observation of Agentic Systems” (chunk 16) supports
the principle that agent behaviour needs observable, reviewable traces. The
project-specific inference is to make safe, bounded provider activity visible in
the product rather than hiding it behind task details or a generic status.

## Decision

- Open every board on **Home**. Home contains the outcome prompt, current
  orchestrator state, live safe activity from running ticket agents, and a short
  route to created tickets. It does not contain a delivery dashboard or provider
  configuration.
- Keep **Tickets** as the focused delivery view: Backlog, In progress, Review,
  and Done lanes, concise cards, and the explicit secondary action to create a
  ticket manually. Ticket detail remains a deliberate, focused view.
- Keep proposal editing and confirmation as a temporary Home sub-flow. It
  preserves the existing typed preview and confirmation gate, then returns the
  person to Tickets after materialisation.
- Move board automation configuration to Settings. Home may state the current
  user-facing mode, but does not duplicate controls or policy terminology.
- Surface only bounded, normalised provider activity: visible messages,
  tool/action updates, questions, failures, and outcomes. Never collect or show
  secrets, hidden reasoning, raw provider protocol frames, or unbounded
  transcripts.

## Consequences

- The primary journey is now outcome → observable planning/review → tickets,
  rather than a single crowded board page.
- The existing daemon remains authoritative for plans, task state, policy, and
  agent activity. The UI reorganises those facts; it does not create scheduling
  state or bypass confirmation.
- Providers with a richer supported event protocol can improve the observable
  activity detail behind the existing bounded activity contract without changing
  the Home information architecture.
