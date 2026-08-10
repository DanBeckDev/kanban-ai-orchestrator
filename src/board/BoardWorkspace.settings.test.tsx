import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openSettings,
  openTask,
  selectOption,
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
      screen.queryByRole("form", { name: "Prompt AI for Task ready-task" }),
    ).toBeNull();

    openTask("Task ready-task");

    expect(
      screen.getByRole("region", { name: "Task details for Task ready-task" }),
    ).toBeVisible();
    expect(
      screen.getByText("Save an agent profile before starting this task."),
    ).toBeVisible();
  });

  it("detects installed agents and saves a project ticket-worker default", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");

    expect(boardGateway.agentProviderAvailability).toHaveBeenCalledOnce();
    const providerList = screen.getByRole("list", {
      name: "Available ticket workers",
    });
    const codexItem = listItemFor(providerList, "Codex");
    const clineItem = listItemFor(providerList, "Cline");
    expect(within(codexItem).getByText("Installed")).toBeVisible();
    expect(within(clineItem).getByText("Not installed")).toBeVisible();
    expect(
      within(clineItem).getByRole("link", { name: "How to install" }),
    ).toHaveAttribute("href", "https://docs.cline.bot/cli");

    fireEvent.click(
      within(codexItem).getByRole("button", { name: "Use as worker" }),
    );

    await waitFor(() =>
      expect(boardGateway.saveAgentProfile).toHaveBeenCalledWith({
        name: "Default Codex CLI",
        kind: "codex_cli",
        program: "codex",
        arguments: [],
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save AI defaults" }));
    await waitFor(() =>
      expect(boardGateway.saveProjectAgentSettings).toHaveBeenCalledWith({
        boardId: "board-1",
        organiser: undefined,
        ticketWorker: {
          agentProfileName: "Default Codex CLI",
          model: { kind: "provider_default" },
          effort: "provider_default",
        },
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    openTask("Task ready-task");
    const launchForm = await screen.findByRole("form", {
      name: "Prompt AI for Task ready-task",
    });
    expect(
      within(launchForm).getByLabelText("Agent profile"),
    ).toHaveTextContent("Default Codex CLI");
  });

  it("does not select an agent when its safe profile could not be saved", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    vi.mocked(boardGateway.saveAgentProfile).mockRejectedValueOnce(
      new Error("Profile store is unavailable"),
    );

    await createBoard(boardGateway);
    openSettings("AI");
    const providerList = screen.getByRole("list", {
      name: "Available ticket workers",
    });
    const codexItem = listItemFor(providerList, "Codex");

    fireEvent.click(
      within(codexItem).getByRole("button", { name: "Use as worker" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your saved work has not changed. Check your last action, then try again.",
    );
    expect(
      within(codexItem).getByRole("button", { name: "Use as worker" }),
    ).toBeEnabled();
  });

  it("saves separate model and effort defaults for the orchestrator and workers", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    await boardGateway.savePlannerProfile({
      name: "Planning agent",
      program: "planner",
      arguments: [],
    });

    await createBoard(boardGateway);
    openSettings("AI");
    await selectOption("AI connection", "Planning agent");

    await selectPreference("Orchestrator", "Thorough");
    await selectPreference("Ticket workers", "Balanced");
    const models = screen.getAllByLabelText("Model name (optional)");
    fireEvent.change(models[0], { target: { value: "gpt-5" } });

    const providerList = screen.getByRole("list", {
      name: "Available ticket workers",
    });
    fireEvent.click(
      within(listItemFor(providerList, "Codex")).getByRole("button", {
        name: "Use as worker",
      }),
    );
    await screen.findByRole("button", { name: "Chosen" });
    fireEvent.change(screen.getAllByLabelText("Model name (optional)")[1], {
      target: { value: "gpt-5-mini" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save AI defaults" }));

    await waitFor(() =>
      expect(boardGateway.saveProjectAgentSettings).toHaveBeenCalledWith({
        boardId: "board-1",
        organiser: {
          plannerProfileName: "Planning agent",
          model: { kind: "named", name: "gpt-5" },
          effort: "thorough",
        },
        ticketWorker: {
          agentProfileName: "Default Codex CLI",
          model: { kind: "named", name: "gpt-5-mini" },
          effort: "balanced",
        },
      }),
    );
  });
});

function listItemFor(list: HTMLElement, name: string): HTMLElement {
  const item = within(list).getByText(name).closest("li");
  if (item === null) throw new Error(`Missing provider option: ${name}`);
  return item;
}

async function selectPreference(groupName: string, optionName: string) {
  const group = screen.getByRole("group", { name: groupName });
  fireEvent.pointerDown(within(group).getByLabelText("Effort"), {
    button: 0,
    ctrlKey: false,
    pointerType: "mouse",
  });
  fireEvent.click(await screen.findByRole("option", { name: optionName }));
}
