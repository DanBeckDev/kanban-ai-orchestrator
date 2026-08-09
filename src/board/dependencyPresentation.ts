import { isHardDependency, stateLabel } from "./presentation";
import type { BoardPlan, BoardSnapshot, Dependency, WorkItem } from "./types";

export type DependencyRelation = Readonly<{
  dependency: Dependency;
  workItem: WorkItem;
}>;

export type DependencyDetails = Readonly<{
  workItem: WorkItem;
  hardPrerequisites: readonly DependencyRelation[];
  guidance: readonly DependencyRelation[];
  downstreamImpact: readonly DependencyRelation[];
  criticalPath: readonly string[] | undefined;
  parallelNeighbours: readonly WorkItem[] | undefined;
}>;

export type DependencyViewData = Readonly<{
  workItems: readonly WorkItem[];
  dependencies: readonly Dependency[];
  currentPlan: BoardPlan | undefined;
}>;

export function dependencyViewData(
  snapshot: BoardSnapshot,
  boardPlan: BoardPlan | undefined,
): DependencyViewData {
  return {
    workItems: snapshot.workItems.map(({ workItem }) => workItem),
    dependencies: snapshot.dependencies,
    currentPlan: planForExactGraph(snapshot, boardPlan),
  };
}

export function dependencyDetails(
  data: DependencyViewData,
  workItemId: string,
): DependencyDetails | undefined {
  const workItem = workItemById(data.workItems, workItemId);
  if (workItem === undefined) return undefined;

  const upstream = relationsFor(
    data.dependencies,
    data.workItems,
    workItemId,
    "upstream",
  );
  const stage = data.currentPlan?.preview.parallelStages.find((itemIds) =>
    itemIds.includes(workItemId),
  );

  return {
    workItem,
    hardPrerequisites: upstream.filter(({ dependency }) =>
      isHardDependency(dependency),
    ),
    guidance: upstream.filter(
      ({ dependency }) => !isHardDependency(dependency),
    ),
    downstreamImpact: relationsFor(
      data.dependencies,
      data.workItems,
      workItemId,
      "downstream",
    ),
    criticalPath: data.currentPlan?.preview.criticalPath,
    parallelNeighbours:
      stage === undefined
        ? undefined
        : stage
            .filter((itemId) => itemId !== workItemId)
            .map((itemId) => workItemById(data.workItems, itemId))
            .filter((item): item is WorkItem => item !== undefined),
  };
}

export function dependencyKindLabel(dependency: Dependency): string {
  switch (dependency.kind) {
    case "blocks":
      return "Must finish first";
    case "review_required":
      return "Needs review first";
    case "contract":
      return "Shared contract";
    case "soft":
      return "Helpful order";
  }
}

export function dependencyReadiness(
  details: DependencyDetails,
): Readonly<{ title: string; description: string }> {
  const unfinished = details.hardPrerequisites.filter(
    ({ workItem }) => workItem.state !== "done",
  );
  if (unfinished.length > 0) {
    return {
      title: "Waiting on prerequisite work",
      description: `${details.workItem.title} is waiting on ${taskNames(unfinished)}.`,
    };
  }
  if (details.hardPrerequisites.length > 0) {
    return {
      title: "Prerequisite work is complete",
      description:
        "No unfinished hard prerequisite is recorded. The daemon still checks task state, evidence, policy, and capacity before it starts work.",
    };
  }
  return {
    title: "No dependency is holding this task",
    description:
      "No hard prerequisite is recorded. The daemon still decides whether this task may start.",
  };
}

export function taskDependencySummary(
  data: DependencyViewData,
  workItemId: string,
): string {
  const details = dependencyDetails(data, workItemId);
  if (details === undefined) return "No dependency information is available.";
  const unfinished = details.hardPrerequisites.filter(
    ({ workItem }) => workItem.state !== "done",
  );
  if (unfinished.length > 0) return `Waiting on ${taskNames(unfinished)}.`;
  if (details.hardPrerequisites.length > 0)
    return "Hard prerequisites are complete.";
  return "No hard prerequisites.";
}

export function relationDescription(
  dependency: Dependency,
  direction: "upstream" | "downstream",
): string {
  if (direction === "upstream") {
    return `${dependencyKindLabel(dependency)} for this task.`;
  }
  switch (dependency.kind) {
    case "blocks":
      return "This task must finish before the work below.";
    case "review_required":
      return "This task needs review before the work below can continue.";
    case "contract":
      return "This task shares a contract with the work below.";
    case "soft":
      return "This task is helpful context for the work below.";
  }
}

export function taskStateLabel(workItem: WorkItem): string {
  return stateLabel(workItem.state);
}

function planForExactGraph(
  snapshot: BoardSnapshot,
  boardPlan: BoardPlan | undefined,
): BoardPlan | undefined {
  if (boardPlan === undefined) return undefined;
  const snapshotTaskIds = snapshot.workItems.map(({ workItem }) => workItem.id);
  const previewTaskIds = boardPlan.preview.workItems.map(({ id }) => id);
  if (!sameValues(snapshotTaskIds, previewTaskIds)) return undefined;

  const dependenciesMatch = snapshot.dependencies.every((dependency) =>
    boardPlan.preview.dependencies.some(
      (preview) =>
        preview.id === dependency.id &&
        preview.upstreamWorkItemId === dependency.upstreamWorkItemId &&
        preview.downstreamWorkItemId === dependency.downstreamWorkItemId &&
        preview.kind === dependency.kind &&
        preview.reason === dependency.reason &&
        preview.owner === dependency.owner &&
        preview.nextAction === dependency.nextAction,
    ),
  );
  return dependenciesMatch &&
    snapshot.dependencies.length === boardPlan.preview.dependencies.length
    ? boardPlan
    : undefined;
}

function relationsFor(
  dependencies: readonly Dependency[],
  workItems: readonly WorkItem[],
  workItemId: string,
  direction: "upstream" | "downstream",
): readonly DependencyRelation[] {
  return dependencies.flatMap((dependency) => {
    const relatedId =
      direction === "upstream"
        ? dependency.downstreamWorkItemId === workItemId
          ? dependency.upstreamWorkItemId
          : undefined
        : dependency.upstreamWorkItemId === workItemId
          ? dependency.downstreamWorkItemId
          : undefined;
    const workItem =
      relatedId === undefined ? undefined : workItemById(workItems, relatedId);
    return workItem === undefined ? [] : [{ dependency, workItem }];
  });
}

function taskNames(relations: readonly DependencyRelation[]): string {
  const names = relations.map(({ workItem }) => workItem.title);
  return names.length === 1
    ? names[0]
    : `${names.slice(0, -1).join(", ")} and ${names.at(-1)}`;
}

function sameValues(
  left: readonly string[],
  right: readonly string[],
): boolean {
  return (
    left.length === right.length && left.every((value) => right.includes(value))
  );
}

function workItemById(
  workItems: readonly WorkItem[],
  workItemId: string,
): WorkItem | undefined {
  return workItems.find(({ id }) => id === workItemId);
}
