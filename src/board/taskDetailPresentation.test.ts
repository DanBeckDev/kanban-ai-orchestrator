import { describe, expect, it } from "vitest";

import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { taskDecision } from "./taskDetailPresentation";
import type { WorkItemState } from "./types";

describe("task detail presentation", () => {
  it.each([
    ["inbox", "Needs planning"],
    ["planned", "Waiting to become ready"],
    ["ready", "Ready to start"],
    ["running", "A worker is making progress"],
    ["awaiting_input", "A worker needs attention"],
    ["review", "Review this task"],
    ["done", "Completed"],
    ["blocked", "Blocked work needs a decision"],
    ["failed", "The last attempt failed"],
    ["cancelled", "Cancelled"],
    ["interrupted", "The last attempt was interrupted"],
  ] as const)("explains the %s state as %s", (state, title) => {
    const current = snapshot([workItem("task", state as WorkItemState)]);

    expect(taskDecision(current, current.workItems[0].workItem).title).toBe(
      title,
    );
  });

  it("names the worker, evidence, and unresolved prerequisite in task language", () => {
    const current = {
      ...snapshot(
        [workItem("api", "planned"), workItem("ui", "blocked")],
        [],
        [
          {
            id: "execution-1",
            workItemId: "ui",
            adapterName: "Codex",
            role: "implementation" as const,
            status: "failed" as const,
            workspacePath: "/workspaces/ui",
            usage: { inputTokens: 12, outputTokens: 8 },
            lastEventSequence: 2,
          },
        ],
        [
          {
            id: "evidence-1",
            workItemId: "ui",
            kind: "check" as const,
            result: "passed" as const,
            summary: "Type check passed.",
            recordedAt: "2026-08-09T00:00:00Z",
          },
        ],
      ),
      dependencies: [
        {
          id: "api-blocks-ui",
          upstreamWorkItemId: "api",
          downstreamWorkItemId: "ui",
          kind: "blocks" as const,
          reason: "The UI needs the API.",
          owner: "Platform",
          nextAction: "Finish the API.",
        },
      ],
    };

    const decision = taskDecision(current, current.workItems[1].workItem);

    expect(decision.worker).toBe("Task worker: Codex (failed).");
    expect(decision.evidenceStatus).toBe("1 review record available.");
    expect(decision.blockers).toEqual([
      {
        title: "Task api",
        type: "Must finish first",
        reason: "The UI needs the API.",
        owner: "Platform",
        nextAction: "Finish the API.",
        unresolved: true,
      },
    ]);
  });
});
