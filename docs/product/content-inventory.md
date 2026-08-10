# Board content inventory

## Scope

This inventory covers persistent, authored board controls. It does not classify
user-authored task text, agent output, repository paths, account names, task
IDs in support disclosure, or data returned from Linear: those are content
rather than product copy. Each persistent string has one primary purpose:

- **Orient** — name the place or subject.
- **Act** — name the action a person can take.
- **Explain** — state status, consequence, or why it matters.
- **Support** — disclose a necessary technical detail only where it helps
  recovery or an advanced configuration task.

The inventory is maintained with the component that owns the copy. A string
with no listed purpose is removed rather than retained as decoration.

## Board entry and recovery

| Surface | Persistent controls and primary purpose |
| --- | --- |
| `BoardLibrary` | **Orient:** Your boards, No boards yet. **Act:** Create a board, Open board, Check again. **Explain:** local storage, repository availability, attention counts, missing-folder recovery, last opened time. |
| `BoardSetup` | **Orient:** Set up workspace, Where is your repository?, Project folder, Board name. **Act:** Use an existing local repository, Clone a GitHub repository, choose a folder/destination, Clone repository, Set up workspace, Back to your boards. **Explain:** cloning and local-repository consequences, credential boundary, selected project, safe task workspaces. **Support:** the optional starting point and its plain-language reason. |
| `BoardWorkspaceScreen` | **Orient:** Loading your local boards, Kanban could not load your boards. **Act:** Reload boards. **Explain:** saved boards have not changed and when to restart. |
| `WorkspaceErrorBoundary` | **Orient:** Kanban couldn't show this workspace. **Act:** Reload workspace. **Explain:** saved boards and work have not changed. |

## Workflow, planning, and supervision

| Surface | Persistent controls and primary purpose |
| --- | --- |
| `BoardView` and `BoardViewMenu` | **Orient:** board name, Your board, Workflow, Dependencies, Settings. **Act:** change view, Plan work with AI, Create task. **Explain:** board summary. |
| `WorkflowComposer` and `GoalPlanForm` | **Orient:** Prompt AI to orchestrate, Orchestrator, What do you want to achieve?. **Act:** Create plan preview and Create revised preview. **Explain:** a proposal is reviewed before it creates or starts work; no orchestrator state; preparation failure with the next step. |
| `PlanProposalPanel`, `PlanDraftEditor`, and `PlanPreview` | **Orient:** Plan work with AI, Review the proposed plan, proposed tasks, dependencies, assumptions, budget, and work order. **Act:** Edit proposed tasks, Save revised preview, Preview pasted plan, Confirm and create tasks, Cancel. **Explain:** confirmation is the only action that creates tasks; plan-update failure tells the person to check changes. **Support:** Paste an existing plan and JSON are deliberately advanced import tools. |
| `BoardHome` and `BoardCanvas` | **Orient:** What needs your attention, Needs your attention, Work in motion, Delivery picture, state columns. **Act:** Inspect, Review, Recover, Unblock, View work. **Explain:** state-specific reason, agent activity, task counts, and empty states. |
| `BoardAutomation` | **Orient:** How Kanban moves work. **Act:** You approve actions, Kanban coordinates, Pause automation, Ask Kanban what to do next. **Explain:** the current coordination authority, prerequisites, safeguards, and latest decision. |

## Tasks, review, and recovery

| Surface | Persistent controls and primary purpose |
| --- | --- |
| `BoardManagement` and `TaskForm` | **Orient:** Create task, task fields, Limits. **Act:** Create task. **Explain:** a manually created task and optional task limits. |
| `CompactWorkItemCard`, `TaskDetailView`, and `TaskDecisionSummary` | **Orient:** task title, status, evidence, worker, and task detail. **Act:** open task and return to the board. **Explain:** the next permitted action, prerequisite state, completion, and board rule enforcement. |
| `TaskActionPanel`, `AgentLaunchForm`, `ExecutionControl`, `TaskStateChangeForm`, and `RecoveryActions` | **Orient:** What you can do now and the active execution state. **Act:** start, stop, retry, return, or make a permitted state change. **Explain:** review and policy requirements are retained; recovery and failed/interrupted states identify the permitted route. |
| `ReviewCheckForm`, `ReviewDecisionForm`, and `CleanCodeReviewForm` | **Orient:** checks, review decision, and clean-code review. **Act:** record check, accept, return for correction, and record review. **Explain:** evidence required before the task can progress. |
| `TaskAiPrompt` | **Orient:** Ask task AI and Task AI decisions. **Act:** choose each supported task action, ask task AI, Apply, Reject, Dismiss. **Explain:** action-specific result, rationale, and whether a user decision is required; setup state tells the person to choose an orchestrator. |
| `ActivityStream` | **Orient:** Live agent activity. **Explain:** waiting, temporarily unavailable activity, and timestamped activity rows. |

## Dependencies

| Surface | Persistent controls and primary purpose |
| --- | --- |
| `DependencyView` and `DependencyMap` | **Orient:** Dependencies, No tasks to map yet, Dependency list. **Act:** select task, open task, Add a relationship manually. **Explain:** what is waiting, affected, and parallel-safe; the list is the keyboard equivalent of the graph. |
| `DependencyInspector` and `dependencyPresentation` | **Orient:** selected task, readiness, prerequisites, guidance, downstream impact, plan context. **Act:** Open task. **Explain:** why work is not ready, owner, next action, critical route, and parallel work. |
| `DependencyForm` | **Orient:** Add a relationship, Must happen first, Depends on it, Relationship, Why, Owner, Next action. **Act:** Add relationship. **Explain:** which relationship types delay work and how to fix missing choices. |

## Settings, agents, Linear, and support

| Surface | Persistent controls and primary purpose |
| --- | --- |
| `BoardSettings` | **Orient:** Settings, AI, Linear, Project. **Act:** choose a settings view. **Explain:** AI roles and connected tools apply to this project. |
| `ProjectAgentDefaultsForm` | **Orient:** AI roles, Orchestrator, Ticket workers, Available on this computer. **Act:** choose an AI connection or worker, use installed worker, save AI defaults, open install guidance. **Explain:** separate project defaults, provider availability, model-name fallback, effort choice, and ticket-level reassignment. |
| `PlannerProfileForm` and `AgentProfileForm` | **Orient:** Orchestrator connections and Agent profiles. **Act:** add and save advanced connections/profiles. **Support:** program and argument fields are hidden behind advanced disclosure because they are only needed for a team-managed bridge. |
| `LinearConnectionPanel` | **Orient:** Connect Linear. **Act:** Connect Linear and enable manually sent comments. **Explain:** authorization state, credential-store boundary, and scope. **Support:** OAuth client ID and loopback redirect are administrator setup details until product-managed Linear OAuth replaces them. |
| `LinearImportForm` | **Orient:** Linear import and the selected local task. **Act:** load assigned issues, use an issue, import issue, import blocker. **Explain:** an issue can be linked after connection. **Support:** immutable issue UUID and connection mode remain for precise import and are not shown in board chrome. |
| `LinearSyncPanel` | **Orient:** Linear synchronization, comment outbox, shared-field reconciliation. **Act:** queue/send a public update and refresh shared fields. **Explain:** only the entered update is sent, delivery uncertainty, and conflicting values needing resolution. |
| `BoardSupportDetails` | **Support:** board and project identifiers are available only in Project support details for recovery. |

## Copy decisions

- The product uses **orchestrator** in normal UI copy. The persisted domain name
  remains `organiser` for compatibility.
- “Clone a GitHub repository” describes the actual Git action; “use an existing
  local repository” makes the alternative equally explicit.
- A named model is optional for each project role. An empty model name means
  provider default; Kanban never claims to know a provider's available models.
- Error copy does not display raw local-service or provider errors in normal
  workflow. It states what did not complete, confirms saved-work safety where
  true, and gives the next permitted action.

## Automated developer walkthroughs

These are executable developer walkthroughs, not a substitute for UX-007's
representative-person research:

| Journey | Evidence |
| --- | --- |
| Discover a board, recover a missing repository, and enter setup | `BoardWorkspace.library.test.tsx` and `BoardSetup.test.tsx` assert named board actions, the explicit clone/local choice, and safe cancellation. |
| Understand a normal plan and a planning failure | `BoardWorkspace.plan.test.tsx` and `BoardWorkspace.plan.validation.test.tsx` assert the outcome prompt, confirmation gate, no raw planner error, and a specific JSON correction action. |
| Choose project AI defaults safely | `BoardWorkspace.settings.test.tsx` asserts installed-provider selection and distinct orchestrator/worker model and effort preferences. |
| Recover from a screen, activity, or Linear connection failure | `WorkspaceErrorBoundary.test.tsx`, `ActivityStream.test.tsx`, `LinearConnectionPanel.test.tsx`, and `BoardWorkspace.linear.test.tsx` assert saved-work reassurance plus the actionable next step. |
