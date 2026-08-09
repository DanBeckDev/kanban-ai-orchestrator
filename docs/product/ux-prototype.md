# First-use and returning-user prototype

- Status: Low-fidelity study artifact for UX-008; not production UI
- Date: 2026-08-09
- Scope: **Find work → Set up safely → Describe an outcome**

This is deliberately a text prototype. It lets a participant react to the
information, language, sequence, and safety boundaries before visual styling or
desktop implementation makes changes expensive. It must be used with the
protocol in [UX validation](ux-validation.md), not presented as evidence that
the flow has already been validated.

## Journey slice

| Backbone activity | Participant outcome | Boundary that must remain visible |
| --- | --- | --- |
| Find work | Recognise and continue a local board, or start a new one | A missing repository is diagnosed rather than silently opening an empty board. |
| Set up safely | Create a board for a valid local Git repository without entering an ID or path | No project or board is persisted until explicit creation; advanced changes explain their effect. |
| Describe an outcome | Begin a plan in natural language and understand that workers have not started | A plan stays unconfirmed until the person reviews its tasks, dependencies, assumptions, and budgets. |

## Screen A — Your boards

### Returning state

```text
┌───────────────────────────────────────────────────────────────────────┐
│ Kanban AI Orchestrator                                 [Settings]      │
├───────────────────────────────────────────────────────────────────────┤
│ Your boards                                                           │
│ Pick up where you left off. Everything is stored on this device.      │
│                                                                       │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Website reliability                              Continue        │ │
│ │ kanban-ai-orchestrator · opened today                            │ │
│ │ Needs your attention: 2 decisions · 1 agent working              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Linear migration                                Continue          │ │
│ │ api-platform · opened 6 days ago                                │ │
│ │ Repository unavailable. [Locate repository] [Try again]          │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│ [Create a board]                                                     │
└───────────────────────────────────────────────────────────────────────┘
```

### Empty state

```text
┌───────────────────────────────────────────────────────────────────────┐
│ Your boards                                                           │
│ No local boards yet. Create one from a repository when you are ready. │
│                                                                       │
│ [Create a board]                                                      │
└───────────────────────────────────────────────────────────────────────┘
```

**Participant prompts**

- “You want to resume the Website reliability work. What would you do?”
- “What do you expect after choosing Continue on the unavailable board?”
- “Where would you create a board for a different repository?”

**Design decisions being tested**

- Board name, repository folder context, recency, and attention summary support
  recognition. IDs are absent from normal re-entry.
- The unavailable state preserves the board card and names the next recovery
  action. It must not load a misleading blank board.
- `Create a board` is the single primary empty-state action. Connecting Linear
  remains an optional route under creation/settings, not a competing first step.

## Screen B — Create a board

```text
┌───────────────────────────────────────────────────────────────────────┐
│ ‹ Your boards                                                        │
│ Create a board                                                       │
│ Choose a project folder and give your board a name.                  │
│                                                                       │
│ Project folder                                                       │
│ [Choose project folder…]  /Work/kanban-ai-orchestrator               │
│                                                                       │
│ Board name                                                           │
│ [Kanban AI Orchestrator                                        ]     │
│                                                                       │
│ Kanban will prepare a separate workspace for each task.              │
│ ▸ Use a different starting point                                     │
│                                                                       │
│ [Cancel]                                      [Create board]         │
└───────────────────────────────────────────────────────────────────────┘
```

### Validation and cancellation variants

```text
Project chooser cancelled
  No project folder selected. [Choose project folder…]
  No project or board has been created.

Selected folder is not ready to use as a project
  Choose the top-level folder for your project, not a folder inside it.
  [Choose another project folder]  [Back to your boards]
  No project or board has been created.

Use a different starting point expanded
  Start new work from [release              ]
  “Kanban normally uses your project's main line of work. Change this only
   if your team asked you to.”
```

**Participant prompts**

- “Create a board for the selected repository. Tell us what you believe will
  happen when you select Create board.”
- “Your team uses a release line for this work. Where would you change the
  starting point, and what do you expect this to affect?”
- “Cancel the repository chooser. What state do you expect the app to be in?”

**Design decisions being tested**

- The native picker replaces typed paths. The selected folder is validated as a
  project without making Git terminology a normal-user prerequisite.
- Board name is editable and defaults from the folder name. Project and board
  identifiers are generated locally and are only exposed in support details,
  never requested in this flow.
- Kanban automatically resolves the project's main line of work and applies its
  standard safety rules. Neither is a normal-user decision. The intentional
  starting-point override is closed because it is not required to create the
  ordinary local board.
- `Create board` is an explicit commit point. The confirmation is for durable
  local setup, not consent to start agents, sync Linear, or execute work.

## Screen C — Board home: describe an outcome

```text
┌───────────────────────────────────────────────────────────────────────┐
│ Website reliability                     Local board · Settings        │
├───────────────────────────────────────────────────────────────────────┤
│ Start with the outcome                                                │
│ What should be true when this work is complete?                       │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Reduce failed deployments by making the release check reliable.  │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│ Planner profile: [Local planner ▾]  [Manage planner profiles]         │
│                                                                       │
│ [Create plan proposal]                                                │
│ A proposal is only a preview. You will review tasks, dependencies,    │
│ assumptions, and budgets before any worker can start.                 │
│                                                                       │
│ Need a single item instead? [Create a task manually]                  │
└───────────────────────────────────────────────────────────────────────┘
```

### Proposal-review state

```text
┌───────────────────────────────────────────────────────────────────────┐
│ Plan proposal — not started                                           │
│ 4 tasks · 2 dependencies · 1 assumption to review                     │
│                                                                       │
│ Assumption: staging credentials remain available during the work.     │
│ [Inspect proposal] [Revise outcome] [Reject] [Confirm plan]           │
│                                                                       │
│ Confirming creates the shown local tasks and dependencies. It does    │
│ not bypass policy checks or automatically start a worker.             │
└───────────────────────────────────────────────────────────────────────┘
```

**Participant prompts**

- “You have just created this board. What is the next useful action?”
- “Create a plan proposal for the outcome shown. When, if ever, do you think an
  AI agent starts?”
- “What would you inspect before confirming this proposal?”

**Design decisions being tested**

- Outcome language is primary; manual task creation is a clearly available,
  secondary escape hatch.
- The planner profile is chosen intentionally because its capabilities can
  materially alter the proposal. If no profile is available, the action explains
  that a profile must be configured and links to that setup; it does not silently
  choose a provider.
- Proposal generation and plan confirmation are separate. Confirmation creates
  the reviewed local plan exactly once; existing policy and execution gates
  remain in force.

## Accessibility and interaction checks

- A keyboard user can tab through each page in visible order; Enter/Space invoke
  controls; focus moves to the new page heading after navigation and to the
  relevant field after validation failure.
- Board status and repository validity use words and icons, never colour alone.
- The native picker is invoked by a labelled button; its cancellation and error
  messages are announced without erasing entered board name text.
- The prototype uses plain language labels rather than icon-only controls or
  internal identifiers. Test participants may choose a screen reader, zoom, or
  reduced-motion setup that they normally use.

## What this prototype does not decide

- Visual styling, animation, exact responsive breakpoints, and graph rendering.
- The product-managed Linear OAuth owner/configuration; it remains a separate
  Linear decision and cannot be implied by local board creation.
- Changes to daemon authority, provider neutrality, plan semantics, scheduling,
  or policy enforcement.

## Research rationale

BookCtx — _User Story Mapping_, Jeff Patton, “Validate the Problem” (chunk 5)
treats sketches, prototypes, and scenario testing as inexpensive ways to test
assumptions with users before building. Its opening material (chunk 1) describes
the map as a way to preserve the end-to-end user narrative rather than optimise
isolated components. This prototype therefore tests a thin, coherent first-use
and returning-user slice instead of a polished replacement for the existing
form.
