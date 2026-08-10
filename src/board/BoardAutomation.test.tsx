import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BoardAutomation } from "./BoardAutomation";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

describe("board automation", () => {
  it("defaults to explicit approval and does not start a task", () => {
    const onCoordinate = vi.fn().mockResolvedValue(undefined);

    render(
      <BoardAutomation
        decisions={[]}
        hasConfiguredRoles
        onConfigure={vi.fn().mockResolvedValue(undefined)}
        onCoordinate={onCoordinate}
        snapshot={snapshot([workItem("foundation", "inbox")])}
      />,
    );

    expect(screen.getByText("You approve actions")).toBeVisible();
    expect(screen.getByText(/You decide when each task starts/)).toBeVisible();
    expect(onCoordinate).not.toHaveBeenCalled();
  });

  it("persists opt-in before asking the daemon to coordinate", async () => {
    const onConfigure = vi.fn().mockResolvedValue(undefined);
    const onCoordinate = vi.fn().mockResolvedValue(undefined);

    render(
      <BoardAutomation
        decisions={[]}
        hasConfiguredRoles
        onConfigure={onConfigure}
        onCoordinate={onCoordinate}
        snapshot={snapshot([workItem("foundation", "planned")])}
      />,
    );

    fireEvent.click(screen.getByText("Kanban coordinates"));

    await waitFor(() => expect(onConfigure).toHaveBeenCalledWith("autonomous"));
    expect(onCoordinate).toHaveBeenCalledWith("board-1");
  });

  it("requires both configured roles before automation can be enabled", () => {
    render(
      <BoardAutomation
        decisions={[]}
        hasConfiguredRoles={false}
        onConfigure={vi.fn().mockResolvedValue(undefined)}
        onCoordinate={vi.fn().mockResolvedValue(undefined)}
        snapshot={snapshot([workItem("foundation", "ready")])}
      />,
    );

    expect(screen.getByText("Choose a task agent first")).toBeVisible();
    expect(
      screen.getByRole("radio", { name: "Kanban coordinates" }),
    ).toBeDisabled();
  });
});
