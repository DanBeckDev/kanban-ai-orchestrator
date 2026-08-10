# User experience strategy

- Status: Approved product direction; implementation in progress
- Date: 2026-08-09
- Scope: local desktop experience on macOS first, with the same interaction model on Windows and Linux

## The experience we are designing for

This product coordinates consequential work. A person should arrive at a useful
board, understand what needs attention, and make a safe decision without first
learning the database model, agent-adapter vocabulary, or dependency notation.

The current first screen fails that test: it asks people to invent a project ID,
board ID, repository path, base ref, and policy-set ID, then asks them to remember
an existing board ID to return. Those are implementation details, not a person's
mental model of starting or resuming work.

The next phase makes the desktop app feel like a trustworthy work space:

- Returning people recognise and open a board by name and repository, with the
  most recently used board prominent.
- New people set up a workspace by cloning a GitHub repository
  or using an existing local repository, then begin planning. The app creates
  immutable identifiers and sensible safe defaults behind the scenes.
- A board opens to the next useful action: describe an outcome, resolve an
  attention item, review work, or observe active agents.
- Dependencies explain order and impact in plain language. People never have to
  infer whether a card is blocked from a line on a diagram.
- Advanced configuration stays available but does not obstruct the first useful
  action. It must not weaken the local daemon's policy, scheduling, or evidence
  boundaries.

## Design principles

1. **Recognition before recall.** Show named boards, repository context, and
   recent activity. Do not ask someone to remember or type a durable ID.
2. **Start with an outcome, not a schema.** The primary empty-board action is to
   describe the result wanted; creating an individual task manually remains an
   explicit secondary path.
3. **Progressive disclosure, not hidden consequences.** Keep required setup
   short, use safe defaults, and reveal advanced repository options only when
   needed. Before a protected action or plan confirmation, show the effect and
   require the existing approval.
4. **Explain the state in the place it matters.** A blocked card names the
   upstream task, dependency type, reason, owner, and next action. A waiting
   approval tells the person exactly what decision is needed.
5. **Calm, local, and honest.** Show local persistence, connection state, and
   agent activity without pretending that a disconnected connector, interrupted
   process, or unreviewed agent claim is complete.
6. **One interaction model on every desktop.** Use native folder selection and
   system conventions where they improve trust, while keeping the information
   architecture and keyboard model consistent across macOS, Windows, and Linux.
7. **Automation earns authority.** Start with a person approving the organiser's
   recommendations. Make autonomous coordination a deliberate, bounded choice
   with a visible explanation, activity trail, and immediate pause control.

## Information architecture

| Place | Primary question answered | Primary action |
| --- | --- | --- |
| **Your boards** | “What was I working on, and what needs attention?” | Open a board or set up a workspace |
| **Set up workspace** | “Which repository and outcome am I coordinating?” | Clone a GitHub repository or use a local repository |
| **Board home** | “What is the safest next move?” | Describe an outcome, resolve attention, or review |
| **Workflow** | “What work exists and what is its state?” | Prompt the organiser, create, or open work |
| **Dependencies** | “What is blocked, what can run in parallel, and why?” | Trace an impact or resolve a blocker |
| **Work detail** | “What evidence and decision are needed for this task?” | Act on the one current decision |
| **Settings and connections** | “How is this board configured?” | Adjust an advanced setting or connect Linear |

Workflow is the default workspace view. A top-left view menu switches between
Workflow, Dependencies, Settings, and focused task detail using the same
authoritative local data; these are not separate sources of truth.

## Journey backbone and release slices

The implementation backlog follows the user's narrative rather than the current
component layout. These activities form the journey backbone; a release slice
must let someone complete a coherent outcome across the row, rather than ship an
isolated screen or configuration field.

| Activity | Current pain | Later experience | Minimum release evidence |
| --- | --- | --- | --- |
| **Find work** | Remember an opaque board ID | Recognise a recent local board or set up a workspace | Returning user opens a saved board by name and repository context |
| **Set up safely** | Enter internal IDs, paths, and policy codes | Name a board, choose a repository natively, accept clear safe defaults | New user creates a valid board without an ID or typed path |
| **Describe an outcome** | Translate an outcome into implementation fields | State the outcome in natural language; the configured organiser drafts the work | User can review an unconfirmed proposal and its assumptions |
| **Decide the order** | Infer dependency consequences from cards and lines | See why work is blocked, what is safe in parallel, and the next decision | User explains one blocked task and traces its impact |
| **Supervise work** | Reconstruct agent state from dense detail | See activity, limits, and attention items without mistaking status for proof | User can find one agent that needs input or policy attention |
| **Review or recover** | Decode evidence/recovery mechanics | See the current decision first and evidence/history on demand | User makes a safe review or recovery choice without confusing it with Done |

Before a production slice begins, create a lightweight **now map** of how target
developers currently create and resume agent work. Add pains, questions, and
workarounds; then test a low-fidelity later-map prototype for the first three
activities. This avoids replacing one implementation-centred form with another
and keeps the first release focused on the minimal coherent user outcome.

## Entry and re-entry flows

### Returning person

1. Launch opens **Your boards**, not an ID field.
2. The most recently opened board is the first, clearly labelled **Open board**.
3. Each board row shows its name, repository folder name, last opened time, and a
   compact attention summary such as “2 need your decision” or “3 agents active”.
4. Choosing a board loads its durable snapshot. If its repository is unavailable,
   the row explains the problem and offers **Locate repository**; it never opens a
   misleading empty board.
5. A board without a saved plan opens to **Describe the outcome**. An unexpected
   workspace failure gives a clear retry state and never leaves a blank screen.

### First visit or creating a board

1. The empty library has one clear primary button: **Set up workspace**.
2. The concise form offers **Clone a GitHub repository** or **Use an existing
   local repository**. Cloning uses a user-selected local destination;
   local use opens a native directory picker. The proposed board name defaults
   to the repository folder name and is editable.
3. The app validates the selected or cloned repository, derives an editable board
   name from its folder, resolves the project's primary starting point, generates
   project/board IDs, selects the safe standard policy, and persists nothing
   until the person confirms creation. None of those implementation details
   obstruct ordinary setup.
4. The app asks which detected providers to enable for this project, then lets
   the person select model and effort defaults separately for the orchestrator
   and ticket workers. A closed **Use a different starting point** control is
   available only for a
   team that needs a different line of work. It uses plain language and explains
   its effect; policy and self-managed integrations belong in later settings,
   not the first-board form.
5. The new board opens to **Describe the outcome**. Manual task creation is
   available, but is deliberately secondary to a reviewed plan.

### Starting from Linear

Linear is an optional companion route, not a prerequisite for a local board. The
entry screen may offer **Connect Linear and import** beside the local-repository
route, but must accurately show connection state and the selected access mode.
Normal product onboarding must not make a person type an OAuth client ID or
redirect URI. Until a product-managed OAuth configuration exists, self-managed
configuration belongs under Advanced setup with an explanation of why it is
needed. Imported issues and blockers retain provenance and still pass the same
local cycle and dependency checks.

Every board starts local-only and says so in setup and on the board home. A
read-only Linear link can load and associate work but cannot send an update.
Linked execution is a separate, explicit choice after narrowly scoped comment
access has been granted; each public update still requires an explicit Send.

## Board workspace model

### Board home

The top of a board is an action-oriented summary, not a dashboard of internals.
It contains:

- **Needs your attention**: review decisions, blocked work needing an owner or
  next action, failed/interrupted executions, conflicts, and policy questions.
  Items are sorted by execution impact, then urgency, and link to the specific
  decision.
- **Plan with AI**: an outcome prompt and a concise explanation that Kanban will
  draft the work for review; no worker starts until the displayed tickets are
  approved.
- **Work in motion**: currently running workers, latest safe activity summary,
  budget/limit signal, and stop/recovery controls appropriate to policy. When
  automation is enabled, this includes a plain-language explanation of what the
  organiser may do and an always-visible **Pause automation** action.
- **Delivery picture**: ready work, hard-blocked work, review work, completed
  work, critical path, and safe parallel capacity. Counts always link to a view.

### Workflow and dependency views

Workflow uses visible state labels and colour as redundant reinforcement. It is
a focused delivery surface: vertically stacked, collapsible Backlog, In
progress, Review, and Done lanes; compact cards; a concise delivery summary; and
actions to prompt the organiser, add a task, open a task, or change Settings.
It never persistently displays provider, planner, raw command, dependency-editor,
or Linear configuration forms. Cards lead with task title, state, assigned agent
when one exists, and the one most important next fact; opaque task IDs move to
copyable details. Dense evidence, raw configuration, and long history are
contained in a deliberately opened work-detail view rather than repeated on every
card.

### Plan with AI

**Prompt AI to orchestrate** is a focused conversation about an outcome, not a form for
creating database records. It begins with one question: **What do you want to
achieve?** The supporting copy makes the consequence explicit: Kanban will draft
tasks and their order, and the person decides whether to create them.

The draft-review surface shows concise task cards, the proposed order and safe
parallel work, assumptions requiring an answer, proposed worker, and expected
scope. It supports **Ask to revise**, direct task editing/removal, **Create
tasks**, and **Discard draft**. A proposed worker can be changed per task without
opening command configuration. Creating tickets is distinct from starting them.
The normal view does not expose planner profile names, model flags, or raw agent
messages.

After tickets exist, the board explains its working mode in user language:

- **You approve actions** (the default) means the organiser can recommend what
  should start, retry, stop, or return for correction, but the person confirms
  each action.
- **Kanban coordinates within limits** is the autonomous opt-in. Before enabling
  it, the person reviews the coordinator, task-worker defaults, approved limits,
  and actions Kanban will never take. The board keeps its activity and **Pause
  automation** control immediately accessible.

When the organiser returns a task for correction, the task detail says what it
found, what evidence it considered, and what happens next. It never makes a
rejected result disappear or look like a completed task.

### Project agents and automation settings

Settings is where a person configures the project, not where they supervise a
board. It first asks which local providers to enable for this project, then
which enabled provider, model, and effort should default to **Orchestrator** and
**Ticket workers**. It detects Codex CLI, Claude Code, and Cline CLI from the
local machine without starting them. Each installed option is selectable in one
action and shows the chosen role default. A missing option is marked **Not
installed** and links to that provider's official installation guidance; Kanban
does not install software or authenticate an account on the person's behalf.

The normal path contains no provider arguments, credentials, permission-bypass
flags, worktree paths, or event-protocol terminology. An intentionally separate
advanced section retains manual custom bridges and planner profiles for an
experienced team that needs them. Linear and board support/project details use
their own settings sections, so they cannot distract from choosing an agent or
moving work.

The separate **Automation** section translates policy into consequences. It
defaults to **You approve actions**. Choosing **Kanban coordinates within
limits** presents the allowed workers, parallel-task limit, time/cost boundary,
and actions that still always require the person. It does not use unexplained
terms such as policy IDs or execution authority, and it never makes unsafe
permission bypasses a convenience setting.

The Dependencies view is a navigable graph plus a companion list. Selecting a
task highlights upstream blockers, downstream impact, critical-path membership,
and parallel-safe neighbours. Every hard edge presents its type, reason, owner,
and next action in words. The list alternative is fully keyboard accessible and
provides the same explanation when a visual graph is unavailable or overwhelming.

### Work detail and recovery

Open a task to see a concise decision summary first: current state, why it is in
that state, assigned agent/workspace, current evidence, and the permitted next
action. A ticket-scoped **Prompt AI** composer can refine the task, request
worker guidance, prepare start/restart/recovery, explain evidence, or request a
return for correction. Expandable sections hold acceptance criteria, dependency
context, safe activity, checks, review evidence, Git references, and history.
Failure and interruption states lead with recovery choices and their
consequences; they never look like successful completion.

## Interaction and accessibility requirements

- Full keyboard traversal, a visible focus indicator, semantic controls, and
  non-colour state labels are required. A graph always has an equivalent list.
- Use the platform's native folder picker rather than asking for a filesystem path
  in the primary flow. Tauri's dialog plugin returns native paths on macOS,
  Windows, and Linux.
- Loading is local and specific: show which board or action is loading without
  disabling unrelated safe navigation. Success and error messages identify the
  action, outcome, and recovery path.
- In **You approve actions** mode, never auto-start work, auto-send to Linear,
  or hide an unresolved plan assumption. In autonomous mode, only the explicitly
  approved, daemon-authorized coordination actions may run automatically; the UI
  always exposes their scope, evidence, and immediate pause control. The UX makes
  existing daemon authority and approval boundaries more legible; it does not
  replace them.

## What is deliberately out of scope for this phase

- A cloud account, remote board directory, shared real-time collaboration, or
  telemetry that exports local project metadata.
- A tutorial that blocks the person from creating or returning to a board.
- Visual graph effects that obscure dependency semantics or exclude keyboard users.
- Treating a Linear-linked board as an authoritative replacement for either the
  local board or Linear.

## Acceptance scenarios and measures

The implementation backlog uses these observable scenarios:

| Scenario | Success condition |
| --- | --- |
| First local board | A developer selects a valid repository, gives the board a name, creates it, and reaches the outcome prompt without entering an ID or filesystem path. |
| Returning board | A developer opens a previously used board by recognition from the library; no ID is required. A missing repository is explained before opening. |
| First plan | A developer enters an outcome, inspects the proposed tasks/dependencies/assumptions, and can confirm or reject it without a worker starting early. |
| Organised delivery | A developer can tell whether they or Kanban will decide the next worker action, see why an organiser made a recommendation, pause automation, and return a worker result for correction without losing evidence. |
| Blocker explanation | A developer can answer “why cannot this task start?” from one task view, including upstream work, reason, owner, and next action. |
| Recovery | A developer can distinguish failed, interrupted, awaiting-input, review, and done work and choose the permitted recovery action. |
| Accessible navigation | Keyboard-only use reaches the same board, task, dependency, and recovery decisions as pointer use. |

Before release, test these scenarios with at least five representative developers
across new and returning-user flows. Record observed completion, confusion, and
critical failures locally in the repository without collecting repository names,
agent transcripts, or credentials. A critical failure is any case where a person
cannot create/reopen a board, cannot tell whether work is safe to start, or cannot
find the action needed to recover/review work.

## Research basis

- Apple recommends concise, optional onboarding that teaches through interaction,
  postpones nonessential setup, and gives context-specific help: [Apple Human
  Interface Guidelines — Onboarding](https://developer.apple.com/design/human-interface-guidelines/onboarding).
- Tauri supports native file and directory selectors on macOS, Windows, and Linux:
  [Tauri dialog plugin](https://v2.tauri.app/plugin/dialog/).
- Linear creates projects from a project view with only a name required and makes
  projects browsable as a list, board, or timeline: [Linear Projects
  documentation](https://linear.app/docs/projects).
- **BookCtx — _User Story Mapping_, Jeff Patton, chapter 4.** Organise work as a
  left-to-right user-activity backbone, slice releases around a specific outcome,
  and use a now map to expose pains and assumptions before committing to a later
  experience. This is the basis for the journey backbone and pre-build prototype
  checkpoint above.
- **BookCtx — _The Product-Minded Engineer_, Drew Hoskins, chapter 8 summary.**
  Make the green, safe path the most discoverable path; use defaults for routine
  choices but deliberately require a decision where a default could be unsafe.
  This supports generated IDs and standard policy defaults while retaining
  explicit plan confirmation and protected-action approval.
- **BookCtx — _Inclusive Design for Accessibility_, Dale Cruse and Denis
  Boudreau, chapters 11 and 13.** Treat keyboard navigation, visible focus,
  clear labels, zoom/reflow, and a nonvisual equivalent as part of the design
  itself. The suggested Needs Walkthrough informs the pre-build and release
  validation tasks.
