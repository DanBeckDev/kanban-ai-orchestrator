# ADR 0008: Daemon-enforced policy capabilities and durable audit

- Status: Accepted
- Date: 2026-08-08

## Context

An agent prompt is an instruction, not an authorization boundary. A worker could be asked to use an undeclared tool, exceed a task budget, or push a branch. Those actions must remain denied even when a provider emits conflicting text or a UI is unavailable. The daemon also needs a reliable record of why an action ran, was rejected, or stopped for approval.

## Decision

- The Rust policy gate evaluates typed actions only: execution start, a named tool scope, or a named protected Git action. It receives usage counters and the work-item budget, never agent instruction text. The active policy is therefore the authority; prompt text cannot expand tool access.
- Policy limits use the strictest configured project or work-item cap for agent turns, duration, and cost. Concurrency is checked before a new execution starts. A zero concurrency limit safely pauses all new starts.
- An `Allow` produces an opaque `AuthorizedAction` capability. Future worker and scheduler boundaries must require that capability immediately before a side effect; an absent capability is not executable authority.
- The gate records its `Allow`, `Deny`, or `ApprovalRequired` decision before returning it. A failure to write the durable audit record prevents the caller from receiving a decision result.
- A protected Git action needs an exact, durable `ProtectedGitApproval` that is scoped to its project, work item, and action. Recording an approval requires a prior durable `Allow` decision for the same project, work item, human actor, and typed Git action. The gate verifies the stored approval before it can issue an execution capability.
- SQLite owns the append-only policy-decision audit and the protected-Git approval records. Both records are idempotent by decision identifier and reject conflicting reuse. Existing event stores migrate from schema version 1 through policy-audit version 3 transactionally.

## Consequences

- Policy enforcement can be tested without a provider, UI, shell, or network. Provider adapters remain outside the policy core.
- The future scheduler and agent command boundary must classify every privileged tool or Git operation into these typed actions before execution; they cannot invoke an arbitrary shell command on the basis of agent text alone.
- The initial local desktop app treats the authenticated local user as the authority that creates approval records. If collaboration or remote approval is added, its identity verification must remain outside the policy core and preserve this durable approval contract.
- Policy audit history survives restart and is available for board review and recovery views.
