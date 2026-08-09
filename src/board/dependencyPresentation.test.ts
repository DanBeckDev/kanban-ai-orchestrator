import { describe, expect, it } from "vitest";

import {
  dependencyDetails,
  dependencyKindLabel,
  dependencyReadiness,
  dependencyViewData,
  relationDescription,
  taskDependencySummary,
} from "./dependencyPresentation";
import type { BoardPlan, BoardSnapshot } from "./types";

const snapshot: BoardSnapshot = {
  board: { id: "board-1", projectId: "project-1", name: "MVP" },
  workItems: [
    materialized("api", "Build API", "done"),
    materialized("ui", "Build UI", "planned"),
    materialized("docs", "Write docs", "planned"),
  ],
  dependencies: [
    dependency("api-blocks-ui", "api", "ui", "blocks"),
    dependency("api-contract-docs", "api", "docs", "contract"),
  ],
  activity: [],
  executions: [],
  evidence: [],
  externalLinks: [],
  connectorOutboxItems: [],
  connectorReconciliationItems: [],
};

const plan: BoardPlan = {
  preview: {
    id: "plan-1",
    projectId: "project-1",
    workItems: snapshot.workItems.map(({ workItem }) => ({
      id: workItem.id,
      title: workItem.title,
      acceptanceCriteria: workItem.acceptanceCriteria,
      budget: workItem.budget,
    })),
    dependencies: snapshot.dependencies,
    criticalPath: ["api", "ui"],
    parallelStages: [["api"], ["ui", "docs"]],
    budget: {
      workItemsMissingAgentTurnBudget: [],
      workItemsMissingDurationBudget: [],
      workItemsMissingCostBudget: [],
    },
    unresolvedAssumptions: [],
  },
};

describe("dependency presentation", () => {
  it("explains blockers, impact, critical work, and safe plan peers", () => {
    const details = dependencyDetails(dependencyViewData(snapshot, plan), "ui");
    if (details === undefined) {
      throw new Error("Expected dependency details for the UI task.");
    }

    expect(
      details.hardPrerequisites.map(({ workItem }) => workItem.title),
    ).toEqual(["Build API"]);
    expect(details.downstreamImpact).toEqual([]);
    expect(details.criticalPath).toEqual(["api", "ui"]);
    expect(details.parallelNeighbours?.map(({ title }) => title)).toEqual([
      "Write docs",
    ]);
    expect(dependencyReadiness(details).title).toBe(
      "Prerequisite work is complete",
    );
  });

  it("keeps the plan-derived route unavailable after the graph changes", () => {
    const changedSnapshot = {
      ...snapshot,
      dependencies: [snapshot.dependencies[0]],
    };
    const data = dependencyViewData(changedSnapshot, plan);
    const details = dependencyDetails(data, "ui");

    expect(data.currentPlan).toBeUndefined();
    expect(details?.criticalPath).toBeUndefined();
    expect(details?.parallelNeighbours).toBeUndefined();
    expect(taskDependencySummary(data, "ui")).toBe(
      "Hard prerequisites are complete.",
    );
  });

  it.each([
    [
      "blocks",
      "Must finish first",
      "This task must finish before the work below.",
    ],
    [
      "review_required",
      "Needs review first",
      "This task needs review before the work below can continue.",
    ],
    [
      "contract",
      "Shared contract",
      "This task shares a contract with the work below.",
    ],
    [
      "soft",
      "Helpful order",
      "This task is helpful context for the work below.",
    ],
  ] as const)(
    "keeps %s distinct in the dependency explanation",
    (kind, label, downstreamCopy) => {
      const relationship = {
        ...snapshot.dependencies[0],
        kind,
      };

      expect(dependencyKindLabel(relationship)).toBe(label);
      expect(relationDescription(relationship, "upstream")).toBe(
        `${label} for this task.`,
      );
      expect(relationDescription(relationship, "downstream")).toBe(
        downstreamCopy,
      );
    },
  );
});

function materialized(
  id: string,
  title: string,
  state: "done" | "planned",
): BoardSnapshot["workItems"][number] {
  return {
    lastEventSequence: 1,
    workItem: {
      id,
      boardId: "board-1",
      title,
      description: "A bounded task.",
      acceptanceCriteria: [],
      budget: {},
      state,
      requiresHumanReview: false,
    },
  };
}

function dependency(
  id: string,
  upstreamWorkItemId: string,
  downstreamWorkItemId: string,
  kind: "blocks" | "contract",
): BoardSnapshot["dependencies"][number] {
  return {
    id,
    upstreamWorkItemId,
    downstreamWorkItemId,
    kind,
    reason: "The downstream work needs this context.",
    owner: "Delivery",
    nextAction: "Finish the upstream task.",
  };
}
