# ADR 0011: Require zero source-structure exceptions

- Status: Accepted
- Date: 2026-08-08

## Context

ADR 0010 correctly introduced a deterministic 400-meaningful-line guard, but its temporary legacy-exception ledger let a passing quality check coexist with six oversized, mixed-responsibility modules. This failed the product requirement that Clean Code findings are fixed, including small and structural findings. It also meant an agent could report a green assessment without surfacing the repository-wide debt.

## Decision

- Keep the 400-meaningful-line limit for every production and test source file.
- Require the exception ledger to be empty; any entry is a quality failure.
- Run a whole-repository structural scan in `quality:verify`, including untracked source files, in addition to the fast changed-file scan.
- Treat an over-limit file as an actionable Clean Code finding that blocks handoff, review, merge, and task completion.

## Consequences

- No AI or human can normalize oversized files as temporary debt in a passing build.
- Existing large modules must be split before feature work can be considered complete.
- The repository-wide scan adds a small amount of quality-check time in exchange for reliable enforcement.
