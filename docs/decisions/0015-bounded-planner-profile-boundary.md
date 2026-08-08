# ADR 0015: Generate plans through a bounded planner-profile boundary

- Status: Accepted
- Date: 2026-08-08

## Context

Natural-language planning is valuable only when an orchestrator's interpretation remains a reviewable suggestion. A provider response, malformed bridge, or mistaken goal must never create work, alter the dependency graph, or become a worker launch by itself. The product must also support multiple local model tools without placing any provider protocol in the domain model.

## Decision

- A planner profile stores a user-selected executable and structured arguments. The daemon starts that executable directly, never by evaluating a shell string, supplies one JSON input object through standard input, and discards standard error.
- Planner process input is limited to 8,000 bytes of goal text. Standard output is limited to 65,536 bytes and the direct child has a 45-second deadline. Launch, input, reader, output, timeout, and exit failures return an actionable error.
- The profile returns exactly one strict `PlanDraft` JSON object. The draft may contain only work-item facts, typed dependency facts, budgets, and unresolved assumptions. Unknown fields—including fields nested inside a budget—are rejected.
- A draft cannot nominate a board, project, plan identifier, task identifier, provenance, timestamp, lifecycle state, or confirmation. The daemon derives those values after it has resolved the selected board and stored planner profile.
- The daemon releases its service lock before external planner I/O. Once a draft has parsed and validated, it enters the existing durable plan-proposal use case. Only the existing named confirmation command can materialize tasks and dependencies.
- Raw goal text and raw provider output are not persisted. The durable proposal contains only the validated, derived plan facts needed for preview and confirmation.
- A generic direct executable is a local trust boundary, not a filesystem sandbox. Users must configure only bridges they trust and must keep credentials out of profile arguments because profile configuration is stored locally. A provider-specific bridge may impose its own read-only sandbox; such a capability must be documented and tested by that adapter before the app claims it.

## Consequences

- Codex, Claude Code, self-hosted models, and future providers can all be used via a small local bridge without changing the scheduler, board schema, or plan-confirmation semantics.
- A planner can fail safely: the selected board remains unchanged until a complete proposal passes the established preview validation and a person confirms it.
- The generic bridge is intentionally narrow. Native planning adapters, provider-specific filesystem guarantees, conversation continuations, and replanning are future extensions behind this boundary rather than special cases in the core.
