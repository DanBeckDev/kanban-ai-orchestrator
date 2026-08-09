import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BoardHome } from "./BoardHome";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

describe("BoardHome", () => {
  it("explains attention in a stable order and opens the selected task", () => {
    const onOpenTask = vi.fn();

    render(
      <BoardHome
        onOpenTask={onOpenTask}
        snapshot={snapshot([
          workItem("blocked", "blocked"),
          workItem("review", "review"),
          workItem("failed", "failed"),
          workItem("awaiting-input", "awaiting_input"),
          workItem("interrupted", "interrupted"),
          workItem("planned", "planned"),
          workItem("ready", "ready"),
          workItem("done", "done"),
        ])}
      />,
    );

    const attention = screen.getByRole("region", {
      name: "Needs your attention",
    });
    expect(within(attention).getAllByRole("listitem")).toHaveLength(5);
    expect(
      within(attention)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual([
      expect.stringContaining("Task awaiting-input"),
      expect.stringContaining("Task review"),
      expect.stringContaining("Task failed"),
      expect.stringContaining("Task interrupted"),
      expect.stringContaining("Task blocked"),
    ]);
    const deliveryPicture = screen.getByRole("region", {
      name: "Delivery picture",
    });
    expect(deliveryCount(deliveryPicture, "Planned")).toBe(1);
    expect(deliveryCount(deliveryPicture, "Ready")).toBe(1);
    expect(deliveryCount(deliveryPicture, "In review")).toBe(1);
    expect(deliveryCount(deliveryPicture, "Completed")).toBe(1);
    expect(deliveryCount(deliveryPicture, "Recovery")).toBe(3);

    fireEvent.click(
      within(attention).getByRole("button", { name: "Review Task review" }),
    );
    expect(onOpenTask).toHaveBeenCalledWith("review");
  });

  it("identifies the selected worker for running work", () => {
    const onOpenTask = vi.fn();

    render(
      <BoardHome
        onOpenTask={onOpenTask}
        snapshot={snapshot(
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
        )}
      />,
    );

    const workInMotion = screen.getByRole("region", {
      name: "Work in motion",
    });
    expect(
      within(workInMotion).getByText("Codex is working on this task."),
    ).toBeVisible();
    expect(
      within(workInMotion).queryByText("/workspace/running"),
    ).not.toBeInTheDocument();

    fireEvent.click(
      within(workInMotion).getByRole("button", {
        name: "View work Task running",
      }),
    );
    expect(onOpenTask).toHaveBeenCalledWith("running");
  });

  it("uses an on-track summary when no work needs attention", () => {
    render(
      <BoardHome
        onOpenTask={vi.fn()}
        snapshot={snapshot([workItem("ready", "ready")])}
      />,
    );

    expect(screen.getByText("On track")).toBeVisible();
    expect(
      screen.getByText("1 task is ready for the next approved action."),
    ).toBeVisible();
    expect(
      screen.getByText("No task needs a decision right now."),
    ).toBeVisible();
    expect(screen.getByText("No agents are working right now.")).toBeVisible();
  });
});

function deliveryCount(deliveryPicture: HTMLElement, label: string): number {
  const entry = within(deliveryPicture).getByText(label).parentElement;
  return Number(entry?.querySelector("dd")?.textContent);
}
