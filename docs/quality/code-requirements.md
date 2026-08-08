# Code requirements and quality gates

These are release requirements, not suggestions. They apply to people and every AI agent working in this repository.

## Definition of done for a code-bearing task

A task that changes executable code, runtime/build configuration, tests, or CI is complete only when all of the following are true:

1. The intended behavior and acceptance criteria are implemented.
2. The completed diff has been reviewed with the `clean-code-review` skill, including its concurrency reference whenever applicable.
3. Every actionable finding has been fixed in the same task: all `Must Fix`, `Should Fix`, and concrete correctness, security, reliability, data-integrity, or build findings outside the book rubric.
4. Each `Consider` is explicitly resolved as `accepted`, `implemented`, or `not-applicable` in the quality-review receipt. An accepted trade-off needs a reason and, when enduring, an ADR.
5. Lint, format, source-structure, static/security analysis, type checks, tests, and coverage commands all pass with no ignored errors or warnings.
6. A quality-review receipt records the scope, reviewer, findings, changes made, verification commands, and coverage result. For every changed source file, it must also record the exact meaningful-line count, its cohesive responsibility, and the review decision. The receipt validator compares each recorded count with the current source; a source file cannot be omitted from the review evidence.

The Clean Code skill remains deliberately non-dogmatic: it distinguishes defects from genuine trade-offs. This policy makes **actionable** defects blocking; it does not pretend every design consideration has one objectively correct answer.

## Coverage policy

Every executable application, library, daemon, and package must maintain at least **80%** coverage for all of:

| Metric | Minimum |
| --- | ---: |
| Lines | 80% |
| Statements | 80% |
| Functions | 80% |
| Branches | 80% |

Rules:

- Enforce thresholds per executable package/crate, not only as a repository-wide aggregate.
- Test every new or changed behavioral branch. The overall percentage is a floor, never permission to leave new behavior untested.
- Exclude only generated code, vendored code, type-only declarations, and deliberately thin platform/framework wrappers. Every exclusion is narrow, named, and documented; broad source-directory exclusions are prohibited.
- A low-coverage or untestable area is a design signal. Prefer refactoring toward clear, cohesive code with injectable boundaries rather than padding coverage with low-value tests.
- For critical scheduling, policy, worktree, persistence, and connector-sync logic, target 90% branch coverage. The hard release floor is still 80%.
- Use mutation testing selectively for high-risk code when line/branch coverage is high but confidence remains low.

## Source structure policy

Every production and test source file must stay within the repository's 400-meaningful-line limit. The gate applies to Rust, TypeScript/TSX, JavaScript/MJS, and quality scripts under the product source roots. It is intentionally stricter than a subjective review because a long file hides unrelated responsibilities and makes review ineffective. Source-structure exceptions are prohibited: a non-empty `docs/quality/code-structure-exceptions.json` fails the quality gate. See [source-structure gate](code-structure.md).

## Required verification layers

### 1. Agent workflow

The implementing agent performs a Clean Code review and remediation pass before handoff. For substantial or high-risk changes, a separate reviewer agent repeats the review without seeing the implementer's conclusions. The orchestrator must refuse `Review → Done` while an actionable finding is unresolved.

### 2. Local Git hook

The repository provides an installable pre-commit hook. It runs the local quality command and rejects code changes when required checks or a review receipt are missing. Its source-structure check scans the whole repository, including untracked files, so legacy or newly added oversized files cannot hide outside the staged diff. It is early feedback, not the authority: Git hooks can be bypassed.

### 3. Required CI status check

CI is the non-bypassable merge/release authority. The `quality:verify` job must run the full quality suite and publish coverage. Branch protection must require its successful status before merge. A green build is required even when a local hook was bypassed.

## Receipt and exceptions

Copy `docs/quality/review-receipt.template.yaml` to `docs/quality/reviews/<work-item>.yaml`. A valid receipt has no unresolved actionable findings. For code-bearing changes, the pre-commit hook and pull-request CI require a changed receipt that identifies the Clean Code skill, reviewer, passing `npm run quality:verify` command, met coverage threshold, and zero unresolved actionable findings.

The only permitted exception is an explicit product-owner decision for a real trade-off or platform limitation. It must name the omitted check/finding, reason, risk, owner, expiry/review date, and related ADR or work item. "The agent did not have time" and "the issue is small" are never valid exceptions.

## Tooling implementation requirement

The first technology-stack task must add these commands and wire them into the hook and CI:

- `quality:changed` — local commit checks, including a whole-repository source-structure scan;
- `quality:verify` — full format, lint, static/security analysis, type checks, tests, and coverage thresholds;
- `structure:check` — source-structure validation for the current working-tree change;
- `test:coverage` — machine-readable per-package coverage report.

The exact tools may differ between TypeScript and Rust, but the policy may not. The delivery backlog tracks this work as `QUAL-002`.
