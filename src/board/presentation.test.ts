import { describe, expect, it } from "vitest";

import {
  blockersFor,
  activityFor,
  boardColumns,
  budgetSummary,
  nextTransitionStates,
  stateLabel,
  workItemsForColumn,
} from "./presentation";
import type { BoardSnapshot, WorkItemState } from "./types";

const snapshot: BoardSnapshot = {
  board: { id: "board-1", projectId: "project-1", name: "MVP" },
  workItems: [
    {
      lastEventSequence: 1,
      workItem: {
        id: "api",
        boardId: "board-1",
        title: "API",
        description: "Build the API.",
        acceptanceCriteria: ["Tests pass."],
        budget: {},
        state: "ready",
        requiresHumanReview: false,
      },
    },
    {
      lastEventSequence: 1,
      workItem: {
        id: "ui",
        boardId: "board-1",
        title: "UI",
        description: "Build the UI.",
        acceptanceCriteria: ["Tests pass."],
        budget: {},
        state: "planned",
        requiresHumanReview: true,
      },
    },
  ],
  dependencies: [
    {
      id: "api-blocks-ui",
      upstreamWorkItemId: "api",
      downstreamWorkItemId: "ui",
      kind: "blocks",
      reason: "UI needs the API.",
      owner: "platform",
      nextAction: "Complete API.",
    },
    {
      id: "advice",
      upstreamWorkItemId: "api",
      downstreamWorkItemId: "ui",
      kind: "soft",
      reason: "Coordinate release.",
      owner: "delivery",
      nextAction: "Monitor.",
    },
  ],
  activity: [
    {
      workItemId: "ui",
      sequence: 3,
      recordedAt: "2026-08-08T00:00:00Z",
      summary: "State changed from inbox to planned: Approved.",
    },
  ],
};

describe("board presentation", () => {
  it("places work in the right columns and shows hard dependency context", () => {
    const plan = boardColumns.find(({ id }) => id === "planned");

    if (plan === undefined) {
      throw new Error("The planned board column must be configured.");
    }
    expect(workItemsForColumn(snapshot, plan).map(({ id }) => id)).toEqual([
      "ui",
    ]);
    expect(blockersFor(snapshot, "ui").map(({ id }) => id)).toEqual([
      "api-blocks-ui",
    ]);
    expect(blockersFor(snapshot, "api")).toEqual([]);
    expect(activityFor(snapshot, "ui").map(({ sequence }) => sequence)).toEqual(
      [3],
    );
    expect(boardColumns.map(({ label }) => label)).toEqual([
      "Plan",
      "Ready",
      "Active",
      "Review",
      "Done",
      "Recovery",
    ]);
    expect(
      budgetSummary({
        ...snapshot.workItems[1].workItem,
        budget: { maxAgentTurns: 20, maxDurationSeconds: 3600 },
      }),
    ).toBe("Max turns: 20 · Max duration: 3600s");
    expect(budgetSummary(snapshot.workItems[0].workItem)).toBe(
      "No agent budget set",
    );
  });

  it("exposes only guarded transition choices for every lifecycle state", () => {
    const states: readonly WorkItemState[] = [
      "inbox",
      "planned",
      "ready",
      "running",
      "awaiting_input",
      "review",
      "done",
      "blocked",
      "failed",
      "cancelled",
      "interrupted",
    ];

    expect(nextTransitionStates("review")).toContain("done");
    expect(nextTransitionStates("done")).toEqual([]);
    expect(nextTransitionStates("failed")).toEqual(["ready", "cancelled"]);
    expect(nextTransitionStates("awaiting_input")).toContain("interrupted");
    expect(
      states.every((state) => Array.isArray(nextTransitionStates(state))),
    ).toBe(true);
    expect(stateLabel("awaiting_input")).toBe("awaiting input");
  });
});
