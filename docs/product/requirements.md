# Product requirements

## Core actors

| Actor | Responsibility | Cannot do without approval |
| --- | --- | --- |
| User | Defines goals, policies, review decisions, and external authority | N/A |
| Orchestrator agent | Turns outcomes into reviewable plans and, within its declared mode and policy, coordinates workers, summaries, and escalation questions | Bypass plan confirmation, declared authority, policy, evidence, quality, or required human review |
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

- A person can give an outcome to a configured organiser in a focused prompt-first surface. It produces a preview of tasks, acceptance criteria, dependency types and reasons, critical path, parallel work, estimated budgets, proposed worker assignment, and unresolved assumptions.
- The person can request a revision, edit, remove, reject, or approve that preview. No initial plan may materialize tasks or launch workers before explicit approval.
- The scheduler observes hard dependencies, policy limits, repository concurrency, and agent/provider budgets.
- A plan may propose a compatible worker for each task; before a task starts, the
  person can select another installed compatible worker without changing the
  organiser, dependency graph, policy, or task evidence history.
- A blocker must include a concrete reason, owner, and proposed next action.
- The orchestrator asks the user when requirements conflict, the plan contains a cycle, an agent exceeds policy, a contract changes, or recovery needs a choice.

### Modes, authority, and worker supervision

- **Manual** is the default board mode. The organiser may suggest a launch, retry, cancellation, or return-for-correction, but every such action requires an explicit named human decision.
- **Autonomous** is a board-level opt-in, not a provider default. Before it is enabled, the person sees and confirms the approved organiser and worker choices, concurrency, time/cost limits, permitted action scope, and a one-action **Pause automation** control. The daemon remains the only authority that can authorize an action.
- In Autonomous mode, the organiser may start dependency-ready, policy-authorized work and make bounded retry or return-for-correction decisions. It may not create a plan without review, relax a policy, install or authenticate a provider, perform protected Git or external actions, or move work to `Done`.
- The organiser receives normalized task state, dependency facts, bounded activity summaries, and evidence results. It does not receive or persist raw provider transcripts, credentials, or secrets as orchestration context.
- Returning work for correction is an explainable daemon transition to `Ready` or `Blocked`, with the trigger, concise rationale, retained evidence, and next action. It never deletes a task, conceals a worker outcome, or substitutes for independent review or a required human decision.
- Settings configures the organiser separately from default task workers through provider-neutral profiles. A native adapter may expose an installed Codex, Claude Code, Cline, or future compatible option, but provider-specific commands, credentials, and permission flags remain adapter-owned.

## Board entry and setup requirements

- Launching the app presents a local board library. A returning user can recognise
  and open a board by its name and repository context, with recently used boards
  prominent; opening a board must never require typing or remembering a durable ID.
- A user sets up a local workspace by linking a GitHub repository, which the app
  clones into a selected local destination, or by selecting an existing local
  repository through a native directory picker. The command boundary validates
  the repository and generates project and board identifiers; IDs are
  implementation details, available only in advanced support details. GitHub
  linking uses the person's existing Git credential mechanism and must not store
  GitHub credentials in board metadata or leave a partly created board on failure.
- The normal setup path resolves the project's primary starting point and
  standard safety policy without Git or policy vocabulary. A deliberately opened
  plain-language starting-point override remains available for teams that need
  it. Invalid or unavailable repositories provide an actionable recovery choice
  and cannot silently create a different project/board.
- The first empty-board experience leads with an outcome prompt and the existing
  reviewed plan-preview/confirmation flow. Manual task creation remains available
  but is not the primary path; no UX shortcut may start a worker before plan and
  policy authorization.
- A saved board without an optional plan opens to that same outcome prompt. The
  desktop command adapter must normalise optional JSON values to the explicit
  frontend absence state, and any unexpected workspace rendering failure must
  show a clear retry state rather than a blank screen.
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

## Agent choice and focused-workspace requirements

- The default Workflow surface is a work surface, not a configuration form. A
  top-left view menu reaches Workflow, Dependencies, Settings, and focused task
  detail. Workflow shows vertically stacked, collapsible Backlog, In progress,
  Review, and Done lanes; concise task cards; dependency/attention signals; and
  clear paths to prompt the organiser, create a task, open a task, or open
  Settings.
  Provider, planner, dependency-editor, and Linear configuration controls must
  not be permanently rendered beside the board.
- The desktop detects the known local CLI executables for Codex, Claude Code,
  and Cline by resolving trusted program names on `PATH`; discovery must not
  launch a provider, submit credentials, or inspect a provider's private data.
  The settings UI clearly differentiates Installed from Not installed.
- Settings presents each detected provider as one self-contained configuration
  card. An installed card enables the provider independently for **Plan work**
  and **Work on tickets**, creates or reuses the corresponding safe native
  profile, and keeps each role's model and effort control on that card. The
  normal path never asks for raw command arguments, approval bypasses,
  worktree paths, or protocol flags.
- **Provider default** is the recommended model setting for each role. Kanban
  discovers models only through the installed agent's already-authenticated
  SDK, app server, or supported client protocol; it never asks for a second
  API key. The dropdown contains only models the local runtime returns. If a
  runtime cannot expose a supported catalogue, Provider default remains
  available with a clear retry or provider-owned configuration action. Native
  adapters translate the saved model and effort at invocation time. A generic
  bridge that cannot express either
  preference rejects it before a request starts rather than silently ignoring
  it.
- A native orchestrator performs bounded, read-only plan, supervision, and
  ticket-effect assessments through an adapter-owned protocol. Its output must
  still pass the same strict typed validation and cannot create work, start a
  worker, relax policy, or move work to Done without the established daemon
  gates.
- Installing a missing provider is always a deliberate external user action.
  The product may link to the provider's official installation guidance, but
  must not install software, change an account, or weaken permissions by itself.
- Task detail is opened deliberately from a card and leads with current state,
  the next permitted action, and blocker/evidence context. Criteria, activity,
  review evidence, worktree details, and recovery history use progressive
  disclosure. A selected task must remain fully operable by keyboard.
- Task detail includes a ticket-scoped AI prompt that can request any
  ticket-relevant action, including refinement, worker guidance, start/restart
  preparation, evidence explanation, correction, and recovery. The daemon
  evaluates every typed effect against the current manual/autonomous authority,
  policy, review, and protected-action rules; a prompt cannot bypass them.

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
