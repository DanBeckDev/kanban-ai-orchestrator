# Product requirements

## Core actors

| Actor | Responsibility | Cannot do without approval |
| --- | --- | --- |
| User | Defines goals, policies, review decisions, and external authority | N/A |
| Orchestrator agent | Proposes plans, dependencies, schedules, summaries, and escalation questions | Create or start work beyond the active policy |
| Worker agent | Executes one bounded task in its assigned workspace | Access outside scope, perform protected Git actions, or exceed budgets |
| Local daemon | Authoritative state, policy enforcement, scheduling, recovery, external sync | Invent user intent or bypass policy |
| Connector | Synchronizes a bounded external system such as Linear | Overwrite a conflicting external change silently |

## Task state model

`Inbox → Planned → Ready → Running → Awaiting input → Review → Done`

Alternative terminal or recovery states are `Blocked`, `Failed`, `Cancelled`, and `Interrupted`.

Rules:

- `Done` requires a completion report and a passed quality gate from the current implementation cycle. A task declared as substantial or high-risk (`requires_human_review` in the MVP) also requires a completed independent Clean Code reviewer-agent run, a zero-actionable-finding decision, and then human review from that same cycle.
- `Failed` and `Interrupted` are never aliases for `Done`, `Trash`, or `Cancelled`.
- Moving a task to `Cancelled` must request cancellation from its agent and retain its audit history.
- The daemon owns transitions; the UI renders them and sends commands, but does not run scheduling logic.

## Dependency model

Every edge has a type, source, reason, creator, and timestamp.

| Type | Scheduler behavior |
| --- | --- |
| `blocks` | Downstream task starts only when **all** incoming hard blockers are done with accepted evidence. |
| `review_required` | Downstream task waits for upstream review acceptance, even if code exists. |
| `contract` | Tasks can run in parallel but share an interface/schema decision that must be reconciled before integration. |
| `soft` | Scheduler may run the task but highlights the risk and required monitoring. |

The graph must be acyclic for hard dependencies. An attempted cycle is rejected with an explanation and suggested edge reversal/removal.

## Orchestration requirements

- Natural-language planning produces a preview of tasks, acceptance criteria, dependency types, critical path, parallel work, estimated budgets, and unresolved assumptions.
- The user approves or edits that preview before workers launch.
- The scheduler observes hard dependencies, policy limits, repository concurrency, and agent/provider budgets.
- A blocker must include a concrete reason, owner, and proposed next action.
- The orchestrator asks the user when requirements conflict, the plan contains a cycle, an agent exceeds policy, a contract changes, or recovery needs a choice.

## Board entry and setup requirements

- Launching the app presents a local board library. A returning user can recognise
  and open a board by its name and repository context, with recently used boards
  prominent; opening a board must never require typing or remembering a durable ID.
- A user creates a local board by naming it and selecting a repository through a
  native directory picker. The command boundary validates the repository and
  generates project and board identifiers; IDs are implementation details,
  available only in advanced support details.
- Base ref and policy configuration have safe defaults and are disclosed as
  advanced settings with their consequences. Invalid or unavailable repositories
  provide an actionable recovery choice and cannot silently create a different
  project/board.
- The first empty-board experience leads with an outcome prompt and the existing
  reviewed plan-preview/confirmation flow. Manual task creation remains available
  but is not the primary path; no UX shortcut may start a worker before plan and
  policy authorization.
- The board home derives a concise, actionable attention list from authoritative
  task, execution, review, policy, and connector state. It must not invent status
  or scheduling decisions in the UI.

## Interaction and dependency requirements

- A task view explains every non-ready state in plain language. For a hard
  dependency it names upstream work, edge type, reason, owner, and next action;
  it also exposes downstream impact and safe parallel work where relevant.
- The board provides both a visual dependency exploration view and an equivalent
  keyboard-accessible list. State is communicated with text and colour, never
  colour alone.
- Task cards lead with a title, authoritative state, and next relevant fact.
  Evidence, workspace metadata, identifiers, and lengthy history use progressive
  disclosure in task detail rather than obscuring the board.
- Recovery, review, approval, Linear conflict, and disconnected-connector states
  make the required human decision and consequence clear. None may resemble a
  successful completion or silently send external data.

## Linear entry requirements

- A user can choose a local-only board or an optional Linear-connected route
  without treating Linear as a prerequisite for core use.
- The normal Linear connection path must not require a user to enter OAuth client
  configuration. If self-managed OAuth remains necessary, it is an advanced,
  clearly explained option; connection/access mode and data boundaries remain
  visible before import or send actions.

## Review and evidence requirements

Each task retains a structured completion record:

- task specification and accepted criteria;
- agent/session identity and significant events;
- worktree/branch identity and Git diff or commit/PR;
- requested checks and their result;
- quality-gate result, independent review execution, concise finding outcome, and reviewer decision;
- known limitations and follow-up work.

An actionable independent-review finding records a failed review outcome and returns the task to `Ready`; a later implementation completion must collect fresh quality and review evidence. Raw provider transcripts are not retained as review evidence.

## Privacy and safety requirements

- Credentials are stored in the operating system keychain, never board metadata.
- Agent tool access is policy-scoped to the project/worktree.
- External connectors receive the minimum selected data; raw transcripts and secrets are excluded by default.
- Every protected action is auditable with actor, policy decision, input summary, outcome, and time.
