import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openSettings,
  openTask,
} from "./BoardWorkspace.test.helpers";

describe("focused board and agent settings", () => {
  it("keeps setup off the board until a person asks for task details or settings", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);

    expect(screen.getByText("Task ready-task")).toBeVisible();
    expect(
      screen.queryByRole("form", { name: "Save agent profile" }),
    ).toBeNull();
    expect(screen.queryByRole("form", { name: "Add dependency" })).toBeNull();
    expect(
      screen.queryByRole("form", { name: "Start agent for Task ready-task" }),
    ).toBeNull();

    openTask("Task ready-task");

    expect(
      screen.getByRole("region", { name: "Task details for Task ready-task" }),
    ).toBeVisible();
    expect(
      screen.getByText("Save an agent profile before starting this task."),
    ).toBeVisible();
  });

  it("detects installed agents, creates a safe default profile, and keeps Cline install guidance separate", async () => {
    window.localStorage.clear();
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("Agent");

    expect(boardGateway.agentProviderAvailability).toHaveBeenCalledOnce();
    const providerList = screen.getByRole("list", {
      name: "Available task agents",
    });
    const codexItem = listItemFor(providerList, "Codex");
    const clineItem = listItemFor(providerList, "Cline");
    expect(within(codexItem).getByText("Installed")).toBeVisible();
    expect(within(clineItem).getByText("Not installed")).toBeVisible();
    expect(
      within(clineItem).getByRole("link", { name: "How to install" }),
    ).toHaveAttribute("href", "https://docs.cline.bot/cli");

    fireEvent.click(
      within(codexItem).getByRole("button", { name: "Use for tasks" }),
    );

    await waitFor(() =>
      expect(boardGateway.saveAgentProfile).toHaveBeenCalledWith({
        name: "Default Codex CLI",
        kind: "codex_cli",
        program: "codex",
        arguments: [],
      }),
    );
    expect(
      await screen.findByText(
        "New task runs use Default Codex CLI by default.",
      ),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    openTask("Task ready-task");
    const launchForm = await screen.findByRole("form", {
      name: "Start agent for Task ready-task",
    });
    expect(within(launchForm).getByLabelText("Agent profile")).toHaveValue(
      "Default Codex CLI",
    );
  });

  it("does not select an agent when its safe profile could not be saved", async () => {
    window.localStorage.clear();
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    vi.mocked(boardGateway.saveAgentProfile).mockRejectedValueOnce(
      new Error("Profile store is unavailable"),
    );

    await createBoard(boardGateway);
    openSettings("Agent");
    const providerList = screen.getByRole("list", {
      name: "Available task agents",
    });
    const codexItem = listItemFor(providerList, "Codex");

    fireEvent.click(
      within(codexItem).getByRole("button", { name: "Use for tasks" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Profile store is unavailable",
    );
    expect(
      within(codexItem).getByRole("button", { name: "Use for tasks" }),
    ).toHaveAttribute("aria-pressed", "false");
  });
});

function listItemFor(list: HTMLElement, name: string): HTMLElement {
  const item = within(list).getByText(name).closest("li");
  if (item === null) throw new Error(`Missing provider option: ${name}`);
  return item;
}
