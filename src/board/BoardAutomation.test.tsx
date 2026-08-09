import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { BoardAutomation } from "./BoardAutomation";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

describe("board automation", () => {
  afterEach(() => {
    window.localStorage.clear();
  });

  it("defaults to explicit approval and does not start a task", () => {
    const onCoordinate = vi.fn().mockResolvedValue(undefined);

    render(
      <BoardAutomation
        defaultAgentProfileName="Codex"
        hasDefaultAgent
        snapshot={snapshot([workItem("foundation", "inbox")])}
        onCoordinate={onCoordinate}
      />,
    );

    expect(screen.getByText("You approve actions")).toBeVisible();
    expect(screen.getByText(/You decide when each task starts/)).toBeVisible();
    expect(onCoordinate).not.toHaveBeenCalled();
  });

  it("coordinates dependency-ready work only after the user turns it on", async () => {
    const onCoordinate = vi.fn().mockResolvedValue(undefined);

    render(
      <BoardAutomation
        defaultAgentProfileName="Claude Code"
        hasDefaultAgent
        snapshot={snapshot([workItem("foundation", "planned")])}
        onCoordinate={onCoordinate}
      />,
    );

    fireEvent.click(screen.getByText("Kanban coordinates"));

    await waitFor(() =>
      expect(onCoordinate).toHaveBeenCalledWith("board-1", "Claude Code"),
    );
    expect(screen.getByText("Coordination is on")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Pause automation" }));

    expect(screen.getByText(/You decide when each task starts/)).toBeVisible();
  });

  it("requires a selected task agent before coordination can be enabled", async () => {
    const onCoordinate = vi.fn().mockResolvedValue(undefined);
    window.localStorage.setItem(
      "kanban-ai-orchestrator.coordination-mode.board-1",
      "autonomous",
    );

    render(
      <BoardAutomation
        defaultAgentProfileName="stale-agent"
        hasDefaultAgent={false}
        snapshot={snapshot([workItem("foundation", "ready")])}
        onCoordinate={onCoordinate}
      />,
    );

    expect(screen.getByText("Choose a task agent first")).toBeVisible();
    expect(
      screen.getByRole("radio", { name: "Kanban coordinates" }),
    ).toBeDisabled();
    await waitFor(() => expect(onCoordinate).not.toHaveBeenCalled());
    expect(
      window.localStorage.getItem(
        "kanban-ai-orchestrator.coordination-mode.board-1",
      ),
    ).toBe("manual");
  });
});
