# Working agreement

This repository is being developed incrementally by people and AI agents. Preserve intent and make changes easy to review.

## Before starting a task

1. Read `README.md`, the relevant product or architecture document, and the task in `docs/planning/backlog.yaml`.
2. Check its `depends_on` list. Do not start a task whose hard prerequisites are incomplete without explicitly recording why.
3. Read every applicable accepted decision in `docs/decisions/`.
4. Confirm the task's acceptance criteria are specific enough to test. Refine the backlog before writing implementation code if they are not.

## While working

- Keep product state, scheduling state, agent-session state, and Git worktree state distinct.
- Treat agents, provider APIs, terminal hooks, and external connectors as unreliable I/O. The local durable state machine is authoritative.
- Do not put secrets, raw provider credentials, or complete agent transcripts in issue trackers or external integrations.
- Preserve the local-first and provider-neutral principles. A new vendor-specific feature belongs behind an adapter.
- Prefer small, independently reviewable changes. Add tests at the same time as behavior.

## Mandatory code-quality gate

For every change to executable code, tests, configuration that changes runtime behavior, or build/CI logic:

1. Use the installed `clean-code-review` skill to review the completed diff and the code it plausibly touches. Load the concurrency reference whenever the change contains async work, shared state, queues, workers, locking, process management, or request handling.
2. Fix every actionable finding before handoff: every `Must Fix`, `Should Fix`, and concrete finding outside the Clean Code rubric, including apparently small defects. Do not defer or suppress one merely to keep a task small.
3. Record every genuine `Consider` trade-off in a review receipt with a decision. A `Consider` is not a defect, but it must not disappear without an explicit resolution.
4. Run the repository's full quality command. It must pass linting, formatting, type checks, security/static analysis where configured, tests, and coverage.
5. Maintain at least 80% line, branch, function, and statement coverage in **each executable package**. New or changed executable behavior needs focused tests; never lower thresholds, add broad exclusions, or use a repository-wide aggregate to hide a weak package without an accepted ADR.

Never use `--no-verify`, skip a required check, or claim completion based only on an agent's self-assessment. See `docs/quality/code-requirements.md` for the authoritative policy and `docs/quality/review-receipt.template.yaml` for the required evidence format.

## Before handing work off

1. Run the task's relevant checks and record the result in the task/PR.
2. Attach a completed quality-review receipt for every code-bearing task, with zero unresolved actionable findings.
3. Update documentation when behavior, an interface, a risk, or a decision changed.
4. Update the matching backlog task's status, implementation notes, and any newly discovered dependencies.
5. Add an ADR in `docs/decisions/` for an enduring architectural decision; do not silently overwrite a prior accepted ADR.

## Documentation precedence

1. Accepted ADRs in `docs/decisions/`
2. Product requirements in `docs/product/requirements.md`
3. Architecture and integration documents
4. `docs/planning/backlog.yaml`
5. Implementation notes and code comments

If two documents disagree, stop and resolve the contradiction in the higher-precedence document before proceeding.
