import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "../App";
import {
  boardLibraryEntry,
  gateway,
  snapshot,
  workItem,
} from "./BoardWorkspace.test.fixtures";

describe("board library", () => {
  it("opens a recognised local board without asking for an ID", async () => {
    const boardGateway = gateway(snapshot([workItem("task-1")]), [
      boardLibraryEntry({
        name: "Website reliability",
        repositoryName: "kanban-ai-orchestrator",
        attention: { activeWorkItemCount: 1, needsAttentionCount: 2 },
      }),
    ]);
    render(<App gateway={boardGateway} />);

    expect(await screen.findByText("Website reliability")).toBeVisible();
    expect(screen.getByText("kanban-ai-orchestrator")).toBeVisible();
    expect(
      screen.getByText("2 decisions need your attention · 1 agent is working"),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByRole("heading", { name: "MVP" })).toBeVisible();
    expect(boardGateway.openBoard).toHaveBeenCalledWith("board-1");
    expect(
      screen.queryByLabelText("Existing board ID"),
    ).not.toBeInTheDocument();
  });

  it("explains an unavailable repository and lets the user retry", async () => {
    const boardGateway = gateway(snapshot(), [
      boardLibraryEntry({
        name: "Moved repository",
        repositoryAvailable: false,
      }),
    ]);
    render(<App gateway={boardGateway} />);

    expect(await screen.findByText("Moved repository")).toBeVisible();
    expect(
      screen.getByText(
        "Repository unavailable. Restore the local folder, then try again.",
      ),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    await waitFor(() =>
      expect(boardGateway.openBoard).toHaveBeenCalledWith("board-1"),
    );
  });

  it("recovers from a failed local-library load", async () => {
    const boardGateway = gateway();
    boardGateway.boardLibrary = vi
      .fn()
      .mockRejectedValueOnce(new Error("local daemon unavailable"))
      .mockResolvedValueOnce([boardLibraryEntry()]);
    render(<App gateway={boardGateway} />);

    expect(
      await screen.findByRole("heading", {
        name: "Your boards could not be loaded",
      }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(await screen.findByText("MVP")).toBeVisible();
    expect(boardGateway.boardLibrary).toHaveBeenCalledTimes(2);
  });

  it("offers one clear create action from an empty library", async () => {
    const boardGateway = gateway();
    render(<App gateway={boardGateway} />);

    expect(await screen.findByText("No local boards yet")).toBeVisible();
    expect(
      screen.getAllByRole("button", { name: "Create a board" }),
    ).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Create a board" }));

    expect(
      await screen.findByRole("heading", { name: "Create a board" }),
    ).toBeVisible();
    fireEvent.click(
      screen.getByRole("button", { name: "Back to your boards" }),
    );
    expect(await screen.findByText("No local boards yet")).toBeVisible();
  });

  it("does not invent an opening time for a board never opened locally", async () => {
    const boardGateway = gateway(snapshot(), [
      boardLibraryEntry({ lastOpenedAt: null }),
    ]);
    render(<App gateway={boardGateway} />);

    expect(await screen.findByText("Not opened yet")).toBeVisible();
    expect(screen.queryByText(/1970/)).not.toBeInTheDocument();
  });
});
