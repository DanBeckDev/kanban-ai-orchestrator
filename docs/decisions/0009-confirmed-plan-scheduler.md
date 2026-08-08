# ADR 0009: Confirmed plans drive daemon scheduling

- Status: Accepted
- Date: 2026-08-08

## Context

An orchestrator can suggest useful work, but it must not turn an unreviewed interpretation of a goal into concurrent agent execution. Once a plan is accepted, the scheduler needs to respect typed hard dependencies, a repository's available execution capacity, and the policy gate. The desktop UI can be hidden, disconnected, or unable to render without changing that outcome.

## Decision

- The planning boundary produces a typed `PlanPreview`: work items and their acceptance criteria, typed dependencies, the hard-dependency critical path, deterministic parallel stages, aggregate budgets with explicit missing limits, and unresolved assumptions.
- A daemon accepts a `PlanConfirmation` only when it names the exact non-blank plan, user identity, and confirmation time. It must not produce a worker launch before that confirmation exists.
- A daemon scheduler takes a `SchedulerTick` assembled from authoritative task progress, measured usage, and repository capacity; it takes no UI state. It first filters for dependency-safe ready work, then defers work that cannot fit in the repository capacity, and finally asks the policy gate to authorize each possible start.
- Repository capacity and policy concurrency are distinct controls. Capacity protects a repository and its worktrees from contention; policy concurrency governs what a user allows an agent system to start. Both must permit a launch.
- The scheduler is a provider-neutral use case. It never launches a process itself; it returns only the opaque policy authorization that the worker-adapter boundary must require immediately before starting an execution.
- The application-level daemon must persist the accepted plan and confirmation as part of its durable command/event history before it supports restart-resume of planned work. The pure scheduling use case remains free of SQLite, UI, and agent-provider dependencies so that this persistence boundary is explicit and independently testable.

## Consequences

- Users can inspect and edit the entire proposed execution shape before giving consent, including unknown budget values and assumptions that require attention.
- A hidden or disconnected UI cannot accidentally pause, launch, or reorder work. The running daemon supplies the same scheduler tick regardless of client state.
- A capacity or policy deferral is explainable and does not silently become an execution. A policy denial is also written to the existing durable policy audit before the scheduler returns it.
- Adding a provider, UI, or workspace implementation cannot change scheduling semantics without passing through the typed scheduler and policy boundaries.
