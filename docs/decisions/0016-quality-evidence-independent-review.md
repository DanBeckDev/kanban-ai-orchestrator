# ADR 0016: Make quality evidence and independent review durable workflow rules

- Status: Accepted
- Date: 2026-08-08

## Context

Repository hooks and CI already require a completed Clean Code receipt and a passing `quality:verify` command for changes to this product. The board, however, currently treats a passed generic check and a free-text reviewer decision as sufficient completion evidence. It cannot show that an independent reviewer actually ran, distinguish a quality gate from an arbitrary check, or return actionable findings to implementation. That makes the product unable to enforce the workflow it asks its own agents to follow.

## Decision

- A review-to-done transition must have a passed `quality_gate` evidence record and a completion report, both from the current implementation cycle. The daemon evaluates durable evidence order rather than trusting checkboxes submitted by the UI.
- For the MVP, a work item with `requires_human_review` is the explicit substantial/high-risk designation. When an implementation execution completes for such a work item, the daemon records a `clean_code_review` requirement with result `recorded`.
- Executions have one of two provider-neutral roles: `implementation` or `independent_review`. A review-role execution runs only while its work item is in `Review`; it does not itself change the work-item state. It must use an agent-profile name different from every implementation-role execution for that work item. Different profiles may use the same provider, but they must be separate sessions with no reused implementation context.
- A completed review-role execution may produce one concise Clean Code review decision. The durable evidence records the reviewer profile, review execution identity, a summary capped at 2,000 characters, and an actionable-finding count. Zero findings records passed `clean_code_review` evidence. Findings record failed evidence and move the work item from `Review` to `Ready` for a new implementation attempt.
- Completion eligibility is cycle-aware: after the latest implementation completion report, it requires a later passed quality gate; high-risk work additionally requires a later passed independent Clean Code review and then a human decision. Evidence from an earlier attempt cannot satisfy a later one.
- The product persists only structured outcome facts and concise summaries. It does not retain raw agent transcripts or provider credentials. The existing daemon, policy, worktree, and adapter boundaries continue to own launch authorization, workspace isolation, and lifecycle normalization.

## Consequences

- A task card explains why it cannot enter Done, including whether the current implementation still needs a quality gate, independent Clean Code review, or human decision.
- The board can start a distinct reviewer-agent session without putting a provider-specific protocol in the domain model. A reviewer result remains validated input; it is not authority to bypass the daemon state machine.
- A failed review cannot be hidden behind an accepted decision from a previous implementation cycle. The remediation path is explicit and durable.
- Older persisted execution JSON defaults to the `implementation` role. Existing review evidence remains readable but cannot satisfy the stricter independent-review requirement for work that now declares it.
