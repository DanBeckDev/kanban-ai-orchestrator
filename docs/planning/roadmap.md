# Delivery roadmap

## Phase 0 — Product foundation

**Outcome:** a stable, testable product contract before implementation starts.

**Exit gate:** requirements, ADRs, typed dependency semantics, reliability scenarios, and the initial delivery backlog are accepted.

## Phase 1 — Local execution core

**Outcome:** a desktop shell backed by a durable local daemon, a Git project/workspace manager, and the task/dependency state machine.

**Exit gate:** the app can create projects and cards, represent dependency graphs, recover after restart, and enforce a project boundary without launching real agents.

## Phase 2 — Agent and workspace alpha

**Outcome:** provider-neutral agent adapter contract, initial CLI agents, isolated worker execution, evidence capture, and review handoff.

**Exit gate:** three adapters pass the same lifecycle contract and a multi-task local feature can be run/reviewed in isolated worktrees.

## Phase 3 — Orchestration and dependency scheduling

**Outcome:** plan preview, typed dependency creation, scheduler, budget/policy controls, replanning, and escalation UX.

**Exit gate:** an approved graph executes safe work in parallel, explains every block, and requires human decisions at policy boundaries.

## Phase 4 — Linear first-class alpha

**Outcome:** OAuth connection, board/project mapping, dependency import, explicit shared-field reconciliation, and safe manually sent comments/evidence links. Configured status writes, issue publishing, webhooks, and automatic synchronization remain later opt-in work.

**Exit gate:** a real Linear project can be linked without data loss, silent overwrites, or an unexplainable difference between systems.

## Phase 5 — Cross-platform beta and hardening

**Outcome:** signed macOS release, continuously tested Windows/Linux builds, performance and crash recovery validation, optional webhook-relay design, onboarding, and import/migration tooling.

**Exit gate:** the release gates in `docs/quality/reliability.md` are automated where feasible and manually verified otherwise.
