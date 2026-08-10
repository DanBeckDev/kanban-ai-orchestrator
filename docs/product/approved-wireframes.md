# Approved wireframe specification

- Status: Product-owner design direction accepted; interaction details confirmed
- Date: 2026-08-10
- Scope: setup, project agent configuration, planning, workflow, dependency, and task-detail views

This specification translates the approved four-screen wireframe set and the
product-owner clarifications into implementation requirements. It is product
direction, not usability research. Representative-build validation remains
UX-007 because the current product is not yet sufficiently complete to test the
intended end-to-end experience.

The product owner confirmed the interaction details on 2026-08-10: GitHub
selection means cloning locally; local selection means choosing an existing
repository folder; role defaults include provider, model, and effort; the
top-left menu switches board views; Manual mode permits direct task creation;
ticket AI supports all ticket actions within existing authority; and dependency
exploration is a dedicated view.

## Product interaction model

The app is an AI-managed local work space, not a collection of forms. A person
uses a concise prompt to ask an organiser to create and coordinate work, reviews
the proposed tickets, and can enter a ticket to direct its assigned worker. A
top-left view menu switches between views of the same board; it is not a global
settings drawer or a way to create separate data.

The initial named views are:

| View | User question | Primary action |
| --- | --- | --- |
| **Workflow** | What needs to happen next? | Prompt the organiser, create a task, open a ticket, or act on a card. |
| **Dependencies** | What is blocked, what is the impact, and what can run together? | Trace an upstream/downstream relationship. |
| **Task detail** | What should happen to this ticket now? | Prompt its AI or take the clearly permitted action. |
| **Settings** | Which approved local agents will coordinate and work on this project? | Enable a provider and set role defaults. |

Workflow is the default board view. Dependencies is a separate view, not a
diagram squeezed below the workflow lanes. The menu must make the current view
clear, work with keyboard navigation, and preserve the board context when a
person switches views.

## Setup workspace

The first setup screen is titled **Set up workspace**. It offers two equally
clear repository sources:

| Choice | Meaning | Required behaviour |
| --- | --- | --- |
| **Clone a GitHub repository** | Clone a repository so Kanban can work locally. | Let the person choose a repository and local destination, state the source and destination before cloning, then validate the cloned repository before creating the workspace. |
| **Use an existing local repository** | Use an existing local folder that already contains a repository. | Open the native directory picker, validate the chosen repository root, and preserve entered setup details on cancellation or validation failure. |

The normal path does not ask for an ID, raw path, base branch, policy name, Git
credential, OAuth client configuration, or provider command. GitHub linking
uses the person's already configured Git/SSH/credential-manager access; the app
does not store a GitHub credential in board metadata. The initial link flow must
support an explicit GitHub clone URL; where a trusted local GitHub account source
is available it may additionally offer repository selection without expanding
the credential boundary. A failed or cancelled clone is an actionable,
recoverable state and never leaves a partly created board.

After a repository source is valid, the screen asks which local AI providers to
enable for **this project**. The options include Codex, Claude Code, and Cline
Pass when detected. Detection only resolves known executable names and does not
launch a provider or inspect private provider data. A missing provider is clearly
unavailable and links to its official installation guide; installing or signing
in remains a deliberate external user action.

The project configuration then has two explicit defaults:

1. **Orchestrator** — default enabled provider, model, and effort for planning
   and supervision.
2. **Ticket workers** — default enabled provider, model, and effort for new
   ticket work.

The values are provider-neutral choices backed by safe adapter profiles. Raw
arguments, permission-bypass flags, worktree paths, credentials, and protocol
details remain outside the normal UI. Per-ticket worker changes remain possible
at review or ticket detail; they do not alter the organiser selection.

## Create and coordinate work

The Workflow view gives the orchestrator a prominent natural-language composer:
**Prompt AI to orchestrate**. It is the primary way to ask for an outcome,
decompose it into tickets, identify dependencies, revise a plan, or ask about
coordination. Settings owns the separate model and effort defaults without
turning the workspace into a configuration form.

The orchestrator first returns a typed, reviewable proposal: tickets, acceptance
criteria, dependency reasons, order, safe parallel work, assumptions, budgets,
and proposed workers. The person can revise the request, edit the proposal, or
confirm it. Confirmation creates tickets but does not itself start a worker.

**Manual mode** means a person may create an individual ticket directly, then
choose when to ask or authorize a worker to act on it. It does not mean manual
data-entry is the default planning experience, nor does it weaken task state,
policy, quality, or review gates. **Autonomous mode** remains a deliberate
board-level opt-in in which the daemon may authorize only the stated, bounded
coordination actions. The UI must always expose the scope and **Pause
automation** control.

## Workflow view

Workflow is a scannable delivery surface, not a dense dashboard. It uses four
full-width, vertically stacked, collapsible lanes in this order:

1. **Backlog** — unstarted work and the clear **Create task** secondary action.
2. **In progress** — active worker work and meaningful latest activity.
3. **Review** — work awaiting evidence or a person’s decision.
4. **Done** — completed work, visually quieter but still discoverable.

Each card presents only its title, authoritative state, chosen agent when
relevant, and the single most useful next fact, such as a hard blocker, review
request, or active attempt. IDs, raw configuration, lengthy logs, and evidence
remain in task detail. State uses text and colour; colour never carries meaning
by itself. A compact dependency signal names a blocker on the card and links to
the Dependencies view for explanation.

## Dependency view

Dependencies receives its own view with a visual graph and an equivalent
keyboard-accessible list. Selecting a ticket reveals its upstream blockers,
downstream impact, edge type, reason, owner, next action, critical-path status,
and parallel-safe work. The graph cannot be the sole explanation and must not
be rendered underneath workflow lanes where it competes with work cards.

## Ticket detail and ticket AI

Opening a ticket shows its description and decision context first: state,
assigned worker, blocker/evidence status, and next permitted action. It contains
a large **Prompt AI** composer that is deliberately scoped to that ticket. It
supports all ticket-relevant requests, including refining specification or
criteria, requesting worker guidance, preparing a start or restart, explaining
evidence, returning work for correction, and recovering from interruption.

The prompt is not a bypass. In Manual mode a state-changing effect is presented
as an explicit, named user decision; protected actions retain their required
confirmation. In Autonomous mode the same request is evaluated against the
board’s recorded authority and policy. The daemon owns the typed effect,
persists its rationale and outcome, and never allows a prompt to mark a task
Done, bypass independent review, relax a policy, or perform an external action
without its existing authorization.

## Implementation and validation implications

- The existing repository-first setup must grow to include a safe GitHub clone
  path. This is APP-004. ORCH-006 implements the durable project-scoped agent
  role defaults after its typed organiser/worker contracts exist; setup points
  people to that next configuration rather than persisting inert UI choices.
- UX-004 implements the Workflow view, organiser composer, manual task route,
  and view-menu structure. It supersedes the earlier card-column assumption.
- UX-005 implements the standalone Dependencies view and accessible graph/list
  equivalence. UX-006 implements the focused ticket entry and progressive
  disclosure; ORCH-008 supplies the typed daemon contract for the complete
  ticket-prompt action set.
- ORCH-006 completed draft editing plus distinct durable organiser/worker
  selection on 2026-08-10. UI-003 adds an optional named model and separate
  effort preference for both roles without exposing raw command flags; verified
  model discovery and provider-native argument mapping remain adapter work.
  ORCH-007 implements durable autonomous supervision. These product surfaces
  must use their daemon-authoritative contracts rather than UI timers.
- UX-007 validates a current build with representative developers. Its findings
  are separate evidence from this product-owner handoff.

## Evidence and design rationale

**BookCtx — _AI Agents: The Definitive Guide_, Nicole Koenigstein, “From LLMs
to Agents: The Foundational Blueprint” (chunk 1).** The book describes agents
as bounded systems of state, guards, actions, permissions, safeguards, and
termination. The requirement to keep prompt-driven effects typed, policy-gated,
and attributable is an implementation inference from that model, not a claim
that the wireframes themselves were user-tested.

**BookCtx — _The Product-Minded Engineer_, Gergely Orosz, “Turn Unknown
Unknowns into Known Unknowns” (chunk 4).** The chapter recommends clear names
and redundant explanations that describe how a feature helps its audience,
rather than its implementation. The dependency view therefore pairs a visual
map with an equivalent plain-language list and selected-task explanation; that
is a project-specific implementation inference, not usability-test evidence.
