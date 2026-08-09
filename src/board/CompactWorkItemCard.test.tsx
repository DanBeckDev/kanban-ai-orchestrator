import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { CompactWorkItemCard } from "./CompactWorkItemCard";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

describe("CompactWorkItemCard", () => {
  it("keeps the actor and next fact separate from dense task detail", () => {
    const current = {
      ...snapshot(
        [workItem("api", "planned"), workItem("ui", "blocked")],
        [],
        [
          {
            id: "execution-1",
            workItemId: "ui",
            adapterName: "Codex",
            status: "failed" as const,
            workspacePath: "/workspaces/ui",
            usage: { inputTokens: 12, outputTokens: 8 },
            lastEventSequence: 2,
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

    render(
      <CompactWorkItemCard
        snapshot={current}
        workItem={current.workItems[1].workItem}
        workItemTitles={
          new Map(
            current.workItems.map(({ workItem: item }) => [
              item.id,
              item.title,
            ]),
          )
        }
        onExplainDependencies={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("Last worked by Codex")).toBeVisible();
    expect(screen.getByText("Waiting on Task api")).toBeVisible();
    expect(screen.queryByText("/workspaces/ui")).toBeNull();
  });
});
