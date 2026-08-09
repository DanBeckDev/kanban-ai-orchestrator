import {
  blockersFor,
  evidenceFor,
  executionsFor,
  stateLabel,
} from "./presentation";
import { dependencyKindLabel } from "./dependencyPresentation";
import type { BoardSnapshot, WorkItem, WorkItemState } from "./types";

export type TaskBlocker = Readonly<{
  title: string;
  type: string;
  reason: string;
  owner: string;
  nextAction: string;
  unresolved: boolean;
}>;

export type TaskDecision = Readonly<{
  state: WorkItemState;
  stateLabel: string;
  title: string;
  description: string;
  nextAction: string;
  worker: string;
  evidenceStatus: string;
  blockers: readonly TaskBlocker[];
}>;

export function taskDecision(
  snapshot: BoardSnapshot,
  workItem: WorkItem,
): TaskDecision {
  const blockers = blockersFor(snapshot, workItem.id).map((dependency) => {
    const upstream = snapshot.workItems.find(
      ({ workItem: candidate }) =>
        candidate.id === dependency.upstreamWorkItemId,
    )?.workItem;
    return {
      title: upstream?.title ?? "Unavailable prerequisite",
      type: dependencyKindLabel(dependency),
      reason: dependency.reason,
      owner: dependency.owner,
      nextAction: dependency.nextAction,
      unresolved: upstream?.state !== "done",
    };
  });

  return {
    state: workItem.state,
    stateLabel: stateLabel(workItem.state),
    ...stateDecision(workItem.state),
    worker: workerSummary(snapshot, workItem),
    evidenceStatus: evidenceSummary(snapshot, workItem),
    blockers,
  };
}

function stateDecision(
  state: WorkItemState,
): Pick<TaskDecision, "title" | "description" | "nextAction"> {
  switch (state) {
    case "inbox":
      return {
        title: "Needs planning",
        description: "This task has not been prepared for work yet.",
        nextAction: "Plan the task or choose a different state below.",
      };
    case "planned":
      return {
        title: "Waiting to become ready",
        description:
          "The task is planned. The daemon decides when its dependencies and policy allow work to begin.",
        nextAction:
          "Review its dependencies or update its state when appropriate.",
      };
    case "ready":
      return {
        title: "Ready to start",
        description:
          "No current hard dependency is holding this task. The daemon will still check its policy before starting a worker.",
        nextAction: "Prompt a task worker with the outcome you need.",
      };
    case "running":
      return {
        title: "A worker is making progress",
        description:
          "The task has an active worker. Its safe activity is available below.",
        nextAction:
          "Review the live activity or stop the worker if recovery is needed.",
      };
    case "awaiting_input":
      return {
        title: "A worker needs attention",
        description:
          "The worker has paused for input. Its bounded activity explains what was requested.",
        nextAction:
          "Review the request and recover or stop the worker if it cannot continue.",
      };
    case "review":
      return {
        title: "Review this task",
        description:
          "The worker has finished its attempt. Review the evidence before accepting it or returning it for correction.",
        nextAction: "Record checks and make a review decision.",
      };
    case "done":
      return {
        title: "Completed",
        description:
          "The task has passed its required completion gate. Its evidence remains available below.",
        nextAction: "No further task action is available.",
      };
    case "blocked":
      return {
        title: "Blocked work needs a decision",
        description: "This task cannot continue in its current state.",
        nextAction:
          "Resolve the blocker, then recover the task when it is safe to retry.",
      };
    case "failed":
      return {
        title: "The last attempt failed",
        description:
          "The task is not complete. Inspect the attempt before deciding whether to retry or cancel it.",
        nextAction: "Inspect the attempt and choose a recovery action.",
      };
    case "cancelled":
      return {
        title: "Cancelled",
        description:
          "This task was cancelled and will not continue automatically.",
        nextAction: "No further task action is available.",
      };
    case "interrupted":
      return {
        title: "The last attempt was interrupted",
        description:
          "The task stopped before it could finish. Its recorded attempt is available for review.",
        nextAction:
          "Inspect the attempt and choose whether to recover or cancel it.",
      };
  }
}

function workerSummary(snapshot: BoardSnapshot, workItem: WorkItem): string {
  const execution = executionsFor(snapshot, workItem.id).at(-1);
  if (execution === undefined) return "No worker run is recorded yet.";
  const role =
    execution.role === "independent_review"
      ? "Independent reviewer"
      : "Task worker";
  return `${role}: ${execution.adapterName} (${execution.status.replaceAll("_", " ")}).`;
}

function evidenceSummary(snapshot: BoardSnapshot, workItem: WorkItem): string {
  const evidenceCount = evidenceFor(snapshot, workItem.id).length;
  if (workItem.requiresHumanReview) {
    return `${evidenceCount} review record${pluralSuffix(evidenceCount)} available. Independent Clean Code review and your decision are required before this task can finish.`;
  }
  return evidenceCount === 0
    ? "No review evidence is recorded yet."
    : `${evidenceCount} review record${pluralSuffix(evidenceCount)} available.`;
}

function pluralSuffix(count: number): string {
  return count === 1 ? "" : "s";
}
