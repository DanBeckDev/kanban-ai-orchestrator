# ADR 0014: Make changed-source structure review evidence explicit

- Status: Accepted
- Date: 2026-08-08

## Context

ADR 0011 made oversized source objectively unmergeable and removed every legacy exception. That prevents a source file from exceeding 400 meaningful lines, but a Clean Code receipt could still record “no findings” without showing that the reviewer inspected the changed files’ vertical density or responsibility boundaries. The original assessment had already treated 600+ line files as cohesive, so an unverified assertion is insufficient evidence.

## Decision

- For every code-bearing change that modifies source under `src/`, `src-tauri/src/`, or `scripts/`, the changed review receipt must inventory each source path.
- Each inventory item records the source-structure gate’s exact meaningful-line count, the file’s cohesive responsibility, and the reviewer’s decision.
- The receipt validator recalculates the count and rejects a changed source file that is absent, stale, or missing its responsibility/decision.
- Existing historical receipts remain readable for repository-wide validation; the stricter inventory applies to current staged and pull-request changes.

## Consequences

- The deterministic 400-line ceiling remains the enforcement authority, while reviewers must now account for files that are below—but approach—the ceiling.
- Review evidence makes an SRP/vertical-density judgment auditable without imposing a second arbitrary “warning” limit or mechanically fragmenting coherent algorithms.
- A reviewer cannot hand-wave a changed source file as clean by omitting it from the receipt.
