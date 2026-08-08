# Product vision

## One-line promise

Help a person safely turn an outcome into a transparent, dependency-aware plan, then coordinate isolated AI workers until each task has reviewable evidence of completion.

## Problem

Coding agents are increasingly capable, but coordinating many of them safely is difficult. Existing task boards often show cards and terminals, yet leave people to infer the actual state of work, dependencies, failures, repository impact, and cost.

The product should make multi-agent delivery understandable and governable:

- an orchestrator decomposes work and explains its proposed order;
- worker agents execute bounded tasks in isolated workspaces;
- a dependency graph reveals blockers and safe parallelism;
- the board records evidence, approvals, failures, and recovery options;
- Linear can remain the team's planning system while the app provides the execution layer.

## Target users

1. Individual developers who want several coding agents to work safely in one or more repositories.
2. Technical leads who manage a Linear backlog and need clear agent progress, review checkpoints, and dependency visibility.
3. Small engineering teams that value local code execution and provider choice over a hosted agent platform.

## Product principles

### Local-first

The app works without an account or hosted execution service. Board data, task evidence, worktree metadata, and audit history persist locally. Cloud services are optional connectors, not a requirement for core work.

### Human authority, useful autonomy

The orchestrator may propose, schedule, and ask for clarification. Policies decide which actions need approval. It must never disguise uncertainty or report a task complete without evidence.

### Provider-neutral by design

CLI agents, direct model APIs, and self-hosted agents are integrations behind a common adapter contract. The domain model must not contain a provider-specific state machine.

### Dependencies are real data

Dependencies are typed graph edges with explicit semantics. They determine eligibility to run, not merely the direction of a drawn line.

### Evidence before completion

An agent's final message is not completion evidence by itself. A task's outcome includes changed files, Git status/commit or PR, requested checks, results, reviewer decision, and an explicit final state.

### Safe isolation

Each worker receives the smallest useful task brief and an isolated Git worktree. Access outside the declared project boundary requires an explicit policy-approved exception.

## Non-goals for the first release

- A hosted multi-user collaboration suite.
- An agent marketplace or proprietary model platform.
- Fully autonomous merge/push behavior without explicit policy configuration.
- Replacing Linear's team planning features.
- Remote execution clusters or browser-only operation.

## Product success measures

- Users can understand why every Ready or Blocked task has its current status.
- Independent tasks start in parallel; hard-blocked tasks never start early.
- An interrupted app process can recover active work without losing task history or confusing a failure with completion.
- Users can switch agent providers without moving their board, task evidence, or policy model.
- A Linear-linked board remains trustworthy after concurrent edits in both systems.
