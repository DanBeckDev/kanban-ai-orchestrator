import type {
  BoardActivity,
  BoardSnapshot,
  Dependency,
  Evidence,
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
  { id: "planned", label: "Plan", states: ["inbox", "planned"] },
  { id: "ready", label: "Ready", states: ["ready"] },
  { id: "active", label: "Active", states: ["running", "awaiting_input"] },
  { id: "review", label: "Review", states: ["review"] },
  {
    id: "done",
    label: "Done",
    states: ["done", "cancelled"],
  },
  {
    id: "recovery",
    label: "Recovery",
    states: ["blocked", "failed", "interrupted"],
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

export function timestamp(): string {
  return new Date().toISOString();
}
