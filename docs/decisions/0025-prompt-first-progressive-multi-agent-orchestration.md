# ADR 0025: Use prompt-first, progressively authorized multi-agent orchestration

- Status: Accepted
- Date: 2026-08-09

## Context

The current planner safely turns one natural-language goal into a typed plan
preview, and the daemon can schedule dependency-safe work. That is necessary but
not the product vision: people should be able to ask an intelligent organiser to
plan an outcome, review the proposed tickets, give individual tickets to capable
workers, and let the organiser coordinate their progress at an authority level
they understand and choose.

Exposing the existing planner profile, task fields, commands, and scheduler as
the primary experience would put implementation vocabulary ahead of the user's
goal. Making a capable model autonomous without a visible boundary would be just
as harmful: it hides a consequential decision behind a status label and risks
letting worker activity masquerade as delivery.

BookCtx — *Site Reliability Engineering*, Betsy Beyer, Chris Jones, Christof
Leng, David Huska, Jennifer Petoff, and Niall Richard Murphy, “The Role of
Humans” (chunk 59) argues that AI systems earn autonomy through transparency,
contextual awareness, progressive authorization, least privilege, reversible
operations, and feedback loops. Applied here, the inference is not to import
production-operations mechanisms wholesale; it is to make coordination authority
small, explicit, observable, and reducible to human approval when confidence or
policy is insufficient.

## Decision

- Make **Plan with AI** the normal planning entry point. A configured
  provider-neutral organiser receives an outcome and produces a typed,
  unconfirmed plan draft. The person can revise, edit, remove, approve, or
  discard it before the daemon materializes any work.
- Keep three distinct authority layers: the UI presents intent and decisions;
  an organiser proposes and supervises; the local daemon validates policy,
  dependencies, state transitions, persistence, and execution capability. An
  organiser or provider event is never authoritative task state.
- Configure organiser and worker roles separately. The normal Settings path
  offers detected installed providers such as Codex, Claude Code, and Cline with
  safe native defaults. Provider-specific flags, credentials, protocols, and
  permissions remain inside the adapter boundary.
- Introduce two board modes. **You approve actions** is the default: each
  organiser-recommended launch, retry, cancellation, or return-for-correction
  needs a human decision. **Kanban coordinates within limits** is a deliberate
  board-level opt-in with stated worker choices, concurrency, time/cost, and
  allowed action scope. It may launch eligible work and perform bounded
  management actions only after the daemon authorizes each action.
- The first executable coordination slice is narrower: it may promote confirmed
  Inbox work through dependency-safe readiness and start one policy-authorized
  worker. It does not yet make model-led assessment, retry, or correction
  decisions; people retain those decisions until durable organiser records and
  their recovery tests are in place.
- Give the organiser only the normalized, privacy-bounded facts it needs:
  task/dependency state, policy outcome, bounded activity summaries, and
  structured evidence. Persist concise decision records, not raw transcripts,
  credentials, or chain-of-thought.
- A negative organiser assessment is a request to return a task to `Ready` or
  `Blocked` through the daemon. It retains the current evidence, explains the
  decision, and never deletes a task or marks it Done. Existing quality,
  independent-review, protected-action, and human-review gates remain in force.
- Keep automatic `Done`, arbitrary command execution, installation or account
  authentication, policy relaxation, protected Git/external actions, and
  unbounded autonomous work outside this decision.

## Consequences

- The product gets a real organiser/worker architecture without vendor lock-in
  or UI-owned scheduling. A stronger organiser can coordinate economical worker
  choices, but the domain remains portable across providers and platforms.
- The first delivery slice is deliberately small and must prove clear plan
  review, opt-in/pause, dependency-safe execution, recovery, and correction
  handling. Model-led autonomous supervision follows only after durable decision
  records and their tests exist; it is a bounded mode, not an implicit promise
  that the product can safely do everything unattended.
- The board adds an understandable mental model — plan, review, work, evidence,
  decision — instead of a permanently visible configuration console. Settings
  owns provider and automation configuration; task detail owns the explanation
  of one worker outcome.
