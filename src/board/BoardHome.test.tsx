import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BoardHome } from "./BoardHome";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import type { BoardSnapshot } from "./types";

describe("BoardHome", () => {
  it("keeps the outcome prompt and ticket route separate from delivery dashboard copy", () => {
    const onOpenTickets = vi.fn();
    const onOpenTask = vi.fn();
    const boardSnapshot = snapshot([workItem("planned", "planned")]);

    renderHome({ boardSnapshot, onOpenTask, onOpenTickets });

    expect(
      screen.getByRole("heading", { name: "Start with the outcome" }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Live AI feedback" }),
    ).toBeVisible();
    expect(screen.getByText("You approve actions")).toBeVisible();
    expect(screen.queryByText("Delivery picture")).not.toBeInTheDocument();

    screen.getByRole("button", { name: "Open Tickets" }).click();
    expect(onOpenTickets).toHaveBeenCalledOnce();

    const tickets = screen.getByRole("region", { name: "1 ticket" });
    within(tickets)
      .getByRole("button", { name: "Open ticket Task planned" })
      .click();
    expect(onOpenTask).toHaveBeenCalledWith("planned");
  });

  it("shows live bounded feedback for each running ticket", async () => {
    const boardSnapshot = snapshot(
      [workItem("running", "running")],
      [],
      [
        {
          id: "execution-1",
          workItemId: "running",
          role: "implementation",
          adapterName: "Codex",
          status: "running",
          workspacePath: "/workspace/running",
          usage: { inputTokens: 0, outputTokens: 0 },
          lastEventSequence: 1,
        },
      ],
    );
    const onLoadExecutionActivity = vi.fn().mockResolvedValue({
      chunks: [
        {
          sequence: 1,
          kind: "activity",
          summary: "Updated the ticket plan.",
          recordedAt: "2026-08-10T12:00:00Z",
        },
      ],
      hasMore: false,
    });

    renderHome({ boardSnapshot, onLoadExecutionActivity });

    expect(await screen.findByText("Updated the ticket plan.")).toBeVisible();
    expect(
      screen.getByRole("region", { name: "Codex · Task running" }),
    ).toBeVisible();
    expect(screen.queryByText("/workspace/running")).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Updates are bounded and readable. Kanban does not collect private reasoning or credentials.",
      ),
    ).toBeVisible();
  });

  it("reports the orchestrator's current planning state", async () => {
    const onLoadPlanningActivity = vi.fn().mockResolvedValue({
      chunks: [
        {
          sequence: 1,
          kind: "activity",
          summary: "The planner process started.",
          recordedAt: "2026-08-10T12:00:00Z",
        },
      ],
      hasMore: false,
    });

    renderHome({ busy: true, onLoadPlanningActivity });

    expect(
      screen.getByText("The orchestrator is preparing a reviewable plan."),
    ).toHaveAttribute("role", "status");
    expect(
      await screen.findByText("The planner process started."),
    ).toBeVisible();
    expect(onLoadPlanningActivity).toHaveBeenCalledWith("board-1", undefined);
  });

  it("keeps a completed planning error visible after the request stops", async () => {
    const onLoadPlanningActivity = vi.fn().mockResolvedValue({
      chunks: [
        {
          sequence: 1,
          kind: "failed",
          summary: "The planner did not produce a reviewable proposal.",
          recordedAt: "2026-08-10T12:00:00Z",
        },
      ],
      hasMore: false,
    });
    const view = renderHome({ busy: true, onLoadPlanningActivity });

    expect(
      await screen.findByText(
        "The planner did not produce a reviewable proposal.",
      ),
    ).toBeVisible();
    view.rerender(
      <BoardHome
        busy={false}
        plannerProfiles={[]}
        snapshot={snapshot()}
        onGeneratePlan={vi.fn()}
        onLoadExecutionActivity={vi.fn().mockResolvedValue({
          chunks: [],
          hasMore: false,
        })}
        onLoadPlanningActivity={onLoadPlanningActivity}
        onOpenPlanReview={vi.fn()}
        onOpenTask={vi.fn()}
        onOpenTickets={vi.fn()}
      />,
    );

    expect(
      screen.getByText("The planner did not produce a reviewable proposal."),
    ).toBeVisible();
    expect(screen.queryByRole("status")).toBeNull();
  });
});

function renderHome({
  boardSnapshot = snapshot(),
  busy = false,
  onLoadExecutionActivity = vi.fn().mockResolvedValue({
    chunks: [],
    hasMore: false,
  }),
  onLoadPlanningActivity = vi.fn().mockResolvedValue({
    chunks: [],
    hasMore: false,
  }),
  onOpenTask = vi.fn(),
  onOpenTickets = vi.fn(),
}: Partial<{
  boardSnapshot: BoardSnapshot;
  busy: boolean;
  onLoadExecutionActivity: ReturnType<typeof vi.fn>;
  onLoadPlanningActivity: ReturnType<typeof vi.fn>;
  onOpenTask: ReturnType<typeof vi.fn>;
  onOpenTickets: ReturnType<typeof vi.fn>;
}> = {}) {
  return render(
    <BoardHome
      busy={busy}
      plannerProfiles={[]}
      snapshot={boardSnapshot}
      onGeneratePlan={vi.fn()}
      onLoadExecutionActivity={onLoadExecutionActivity}
      onLoadPlanningActivity={onLoadPlanningActivity}
      onOpenPlanReview={vi.fn()}
      onOpenTask={onOpenTask}
      onOpenTickets={onOpenTickets}
    />,
  );
}
