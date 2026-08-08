# ADR 0010: Enforce source-structure limits alongside Clean Code review

- Status: Accepted
- Date: 2026-08-08

## Context

Clean Code review requires judgment: a line-count limit alone cannot decide whether an algorithm is coherent. That judgment was applied too permissively when a new orchestration module combined plan validation, scheduling, data types, error rendering, and tests into files exceeding 500 lines. Existing modules also contained 600–900 lines of unrelated navigational detail. The review system therefore needed a deterministic backstop for vertical density and responsibility boundaries.

## Decision

- Enforce a 400-meaningful-line maximum for every changed Rust, TypeScript/TSX, JavaScript/MJS, and quality-script source file under the product source roots. Blank lines and whole-line comments do not count.
- Keep the check in `quality:changed`, so the local hook and every implementing agent encounter it before commit. CI compares every pull request with its base branch and applies the same rule.
- Treat the limit as a prompt to split independently understandable responsibilities, not as an instruction to fragment a single cohesive algorithm. The Clean Code skill must explicitly assess a changed file over the limit before calling it cohesive.
- Use only time-bounded exceptions for pre-existing legacy modules. Each exception has a work-item owner and expiry. New exceptions require product-owner approval and an ADR. `QUAL-004` removes every inherited exception rather than preserving them as normal.

## Consequences

- New source cannot silently grow into a navigation and review bottleneck; plan validation, scheduling, adapters, persistence, and focused tests remain independently findable.
- Review evidence is stronger because the subjective assessment and deterministic gate reinforce each other.
- Legacy oversized modules are visible, owned, and scheduled for removal. Their temporary presence does not weaken the rule for new work.
