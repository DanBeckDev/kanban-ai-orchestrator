# ADR 0002: Typed dependencies form an authoritative DAG

- Status: Accepted
- Date: 2026-08-08

## Context

Task links are insufficient for an orchestrator. The scheduler must know which work is blocked, which can run in parallel, and which shared contracts require reconciliation.

## Decision

Represent dependencies as typed directed edges. `blocks` and `review_required` are hard prerequisites and must form an acyclic graph. A downstream task becomes eligible only when all of its incoming hard prerequisites satisfy their stated completion condition.

## Consequences

- The product can calculate critical paths and safe parallel work.
- Cycles are validation errors, not a scheduling edge case.
- Imported Linear relationships retain source/provenance and are mapped into the same graph model.
- Visual links, state transitions, and auto-start behavior all derive from this one model.
