# User experience strategy

- Status: Proposed for the next delivery phase
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
- New people choose **Create a board**, select a local repository with a native
  folder picker, name the board, and begin planning. The app creates immutable
  identifiers and sensible safe defaults behind the scenes.
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

## Information architecture

| Place | Primary question answered | Primary action |
| --- | --- | --- |
| **Your boards** | “What was I working on, and what needs attention?” | Open a board or create one |
| **Create a board** | “Which repository and outcome am I coordinating?” | Select repository and create |
| **Board home** | “What is the safest next move?” | Describe an outcome, resolve attention, or review |
| **Board** | “What work exists and what is its state?” | Inspect, create, or transition work |
| **Dependencies** | “What is blocked, what can run in parallel, and why?” | Trace an impact or resolve a blocker |
| **Work detail** | “What evidence and decision are needed for this task?” | Act on the one current decision |
| **Settings and connections** | “How is this board configured?” | Adjust an advanced setting or connect Linear |

The board home is the default workspace view. The Kanban board, dependency map,
activity, and settings are named views of the same authoritative local data, not
separate sources of truth.

## Journey backbone and release slices

The implementation backlog follows the user's narrative rather than the current
component layout. These activities form the journey backbone; a release slice
must let someone complete a coherent outcome across the row, rather than ship an
isolated screen or configuration field.

| Activity | Current pain | Later experience | Minimum release evidence |
| --- | --- | --- | --- |
| **Find work** | Remember an opaque board ID | Recognise a recent local board or choose Create a board | Returning user opens a saved board by name and repository context |
| **Set up safely** | Enter internal IDs, paths, and policy codes | Name a board, choose a repository natively, accept clear safe defaults | New user creates a valid board without an ID or typed path |
| **Describe an outcome** | Translate an outcome into implementation fields | State the outcome in natural language and choose a planner profile | User can review an unconfirmed proposal and its assumptions |
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

1. The empty library has one clear primary button: **Create a board**.
2. The concise form asks for a board name and a repository through a native folder
   picker. The proposed board name defaults to the repository folder name and is
   editable.
3. The app validates that the selected folder is a Git repository, resolves the
   project's primary starting point, generates project/board IDs, selects the
   safe standard policy, and persists nothing until the person confirms creation.
   None of those implementation details obstruct ordinary setup.
4. A closed **Use a different starting point** control is available only for a
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

## Board workspace model

### Board home

The top of a board is an action-oriented summary, not a dashboard of internals.
It contains:

- **Needs your attention**: review decisions, blocked work needing an owner or
  next action, failed/interrupted executions, conflicts, and policy questions.
  Items are sorted by execution impact, then urgency, and link to the specific
  decision.
- **Continue planning**: an outcome prompt with the selected planner profile and
  a concise explanation that no worker starts until the displayed plan is
  confirmed.
- **Work in motion**: currently running workers, latest safe activity summary,
  budget/limit signal, and stop/recovery controls appropriate to policy.
- **Delivery picture**: ready work, hard-blocked work, review work, completed
  work, critical path, and safe parallel capacity. Counts always link to a view.

### Board and dependency views

The Board view uses visible state labels and colour as redundant reinforcement.
It is a focused delivery surface: columns, compact cards, a concise delivery
summary, and actions to plan work, add a task, open a task, or change Settings.
It never persistently displays provider, planner, raw command, dependency-editor,
or Linear configuration forms. Cards lead with task title, state, assigned agent
when one exists, and the one most important next fact; opaque task IDs move to
copyable details. Dense evidence, raw configuration, and long history are
contained in a deliberately opened work-detail view rather than repeated on every
card.

### Agent settings

Settings is where a person configures the product, not where they supervise a
board. Its first section answers one simple question: **Which installed agent
should work on new tasks?** It detects Codex CLI, Claude Code, and Cline CLI from
the local machine without starting them. Each installed option is selectable in
one action and shows the selected default. A missing option is marked **Not
installed** and links to that provider's official installation guidance; Kanban
does not install software or authenticate an account on the person's behalf.

The normal path contains no provider arguments, credentials, permission-bypass
flags, worktree paths, or event-protocol terminology. An intentionally separate
advanced section retains manual custom bridges and planner profiles for an
experienced team that needs them. Linear and board support/project details use
their own settings sections, so they cannot distract from choosing an agent or
moving work.

The Dependencies view is a navigable graph plus a companion list. Selecting a
task highlights upstream blockers, downstream impact, critical-path membership,
and parallel-safe neighbours. Every hard edge presents its type, reason, owner,
and next action in words. The list alternative is fully keyboard accessible and
provides the same explanation when a visual graph is unavailable or overwhelming.

### Work detail and recovery

Open a task to see a concise decision summary first: current state, why it is in
that state, assigned agent/workspace, current evidence, and the permitted next
action. Expandable sections hold acceptance criteria, dependency context, safe
activity, checks, review evidence, Git references, and history. Failure and
interruption states lead with recovery choices and their consequences; they never
look like successful completion.

## Interaction and accessibility requirements

- Full keyboard traversal, a visible focus indicator, semantic controls, and
  non-colour state labels are required. A graph always has an equivalent list.
- Use the platform's native folder picker rather than asking for a filesystem path
  in the primary flow. Tauri's dialog plugin returns native paths on macOS,
  Windows, and Linux.
- Loading is local and specific: show which board or action is loading without
  disabling unrelated safe navigation. Success and error messages identify the
  action, outcome, and recovery path.
- Never auto-start work, auto-send to Linear, or hide an unresolved plan
  assumption. The UX makes existing daemon authority and approval boundaries more
  legible; it does not replace them.

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
