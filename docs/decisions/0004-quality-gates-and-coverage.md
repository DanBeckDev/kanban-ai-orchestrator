# ADR 0004: Layered quality gates with per-package 80% coverage

- Status: Accepted
- Date: 2026-08-08

## Context

The product will be built by multiple AI agents as well as people. Written quality aspirations alone are not enforceable, and a local Git hook can be bypassed. The project also has a hard requirement for at least 80% test coverage.

## Decision

Apply a layered gate to every code-bearing change:

1. An agent follows the Clean Code Review skill, remediates all actionable findings, and records a review receipt.
2. A fast local pre-commit hook provides immediate feedback.
3. A required CI `quality:verify` status check is the merge/release authority.

Enforce line, statement, function, and branch coverage at or above 80% in each executable package/crate. Do not use a repository aggregate to mask weak coverage.

## Consequences

- A hook alone is insufficient; CI and branch protection are mandatory once the remote repository is configured.
- "Fix all issues" means all actionable findings in the changed scope, including small ones. Genuine design considerations need an explicit decision rather than an invented fix.
- The chosen technology stack must provide trustworthy TypeScript and Rust coverage tooling and a unified `quality:verify` command.
- Coverage does not replace behavior-focused testing or the Clean Code review; it is one required signal among several.
