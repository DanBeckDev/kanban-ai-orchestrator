import type {
  BoardActivity,
  BoardSnapshot,
  Dependency,
  Evidence,
  ExternalLink,
  Execution,
  WorkItem,
  WorkItemState,
} from "./types";

export type BoardColumn = Readonly<{
  id: string;
  label: string;
  states: readonly WorkItemState[];
}>;

export const boardColumns: readonly BoardColumn[] = [
  {
    id: "backlog",
    label: "Backlog",
    states: ["inbox", "planned", "ready"],
  },
  {
    id: "in-progress",
    label: "In progress",
    states: ["running", "awaiting_input"],
  },
  { id: "review", label: "Review", states: ["review"] },
  {
    id: "done",
    label: "Done",
    states: ["done", "cancelled"],
  },
];

export function workItemsForColumn(
  snapshot: BoardSnapshot,
  column: BoardColumn,
): readonly WorkItem[] {
  return snapshot.workItems
    .map(({ workItem }) => workItem)
    .filter((workItem) => column.states.includes(workItem.state));
}

export function blockersFor(
  snapshot: BoardSnapshot,
  workItemId: string,
): readonly Dependency[] {
  return snapshot.dependencies.filter(
    (dependency) =>
      dependency.downstreamWorkItemId === workItemId &&
      isHardDependency(dependency),
  );
}

export function activityFor(
  snapshot: BoardSnapshot,
  workItemId: string,
): readonly BoardActivity[] {
  return snapshot.activity.filter(
    (activity) => activity.workItemId === workItemId,
  );
}

export function executionsFor(
  snapshot: BoardSnapshot,
  workItemId: string,
): readonly Execution[] {
  return snapshot.executions.filter(
    (execution) => execution.workItemId === workItemId,
  );
}

export function evidenceFor(
  snapshot: BoardSnapshot,
  workItemId: string,
): readonly Evidence[] {
  return snapshot.evidence.filter(
    (evidence) => evidence.workItemId === workItemId,
  );
}

export function externalLinksFor(
  snapshot: BoardSnapshot,
  workItemId: string,
): readonly ExternalLink[] {
  return snapshot.externalLinks.filter(
    (link) => link.workItemId === workItemId,
  );
}

export function isHardDependency(dependency: Dependency): boolean {
  return dependency.kind === "blocks" || dependency.kind === "review_required";
}

export function stateLabel(state: WorkItemState): string {
  return state.replaceAll("_", " ");
}

export function budgetSummary(workItem: WorkItem): string {
  const limits = [
    workItem.budget.maxAgentTurns === undefined
      ? undefined
      : `Max turns: ${workItem.budget.maxAgentTurns}`,
    workItem.budget.maxDurationSeconds === undefined
      ? undefined
      : `Max duration: ${workItem.budget.maxDurationSeconds}s`,
    workItem.budget.maxCostMicros === undefined
      ? undefined
      : `Max cost: ${workItem.budget.maxCostMicros}µ`,
  ].filter((limit): limit is string => limit !== undefined);

  return limits.length === 0 ? "No agent budget set" : limits.join(" · ");
}

export function nextTransitionStates(
  state: WorkItemState,
): readonly WorkItemState[] {
  const transitions: Readonly<Record<WorkItemState, readonly WorkItemState[]>> =
    {
      inbox: ["planned", "cancelled"],
      planned: ["ready", "blocked", "cancelled"],
      ready: ["running", "blocked", "cancelled"],
      running: [
        "awaiting_input",
        "review",
        "blocked",
        "failed",
        "interrupted",
        "cancelled",
      ],
      awaiting_input: [
        "running",
        "blocked",
        "failed",
        "interrupted",
        "cancelled",
      ],
      review: [
        "done",
        "ready",
        "running",
        "blocked",
        "failed",
        "interrupted",
        "cancelled",
      ],
      done: [],
      blocked: ["ready", "cancelled"],
      failed: ["ready", "cancelled"],
      cancelled: [],
      interrupted: ["ready", "cancelled"],
    };
  return transitions[state];
}

/** States entered only by the execution runtime or a normalized agent event. */
export function manualTransitionStates(
  state: WorkItemState,
): readonly WorkItemState[] {
  return nextTransitionStates(state).filter(
    (nextState) =>
      nextState !== "running" &&
      nextState !== "awaiting_input" &&
      !(isActive(state) && nextState === "cancelled"),
  );
}

function isActive(state: WorkItemState): boolean {
  return state === "running" || state === "awaiting_input";
}

export function timestamp(): string {
  return new Date().toISOString();
}
