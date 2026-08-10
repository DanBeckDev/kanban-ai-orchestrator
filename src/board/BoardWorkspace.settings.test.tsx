import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openSettings,
  openTask,
} from "./BoardWorkspace.test.helpers";

describe("focused board and provider-owned AI settings", () => {
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

  it("keeps each provider's roles and configuration in one card", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");

    expect(boardGateway.agentProviderAvailability).toHaveBeenCalledOnce();
    const codex = providerCard("Codex");
    const cline = providerCard("Cline");
    expect(within(codex).getByText("Installed")).toBeVisible();
    expect(within(cline).getByText("Not installed")).toBeVisible();
    expect(
      within(cline).getByRole("link", { name: "How to install" }),
    ).toHaveAttribute("href", "https://docs.cline.bot/cli");
    expect(screen.queryByLabelText("AI connection")).toBeNull();
    expect(screen.queryByLabelText("Specific model name")).toBeNull();

    fireEvent.click(
      within(codex).getByRole("button", { name: "Work on tickets" }),
    );

    await waitFor(() =>
      expect(boardGateway.providerModelCatalog).toHaveBeenCalledWith(
        "codex_cli",
      ),
    );
    await waitFor(() =>
      expect(boardGateway.saveAgentProfile).toHaveBeenCalledWith({
        name: "Default Codex CLI",
        kind: "codex_cli",
        program: "codex",
        arguments: [],
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save AI setup" }));
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

  it("loads account models inside the selected provider and saves role-specific choices", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");
    const codex = providerCard("Codex");
    fireEvent.click(within(codex).getByRole("button", { name: "Plan work" }));
    await waitFor(() =>
      expect(boardGateway.savePlannerProfile).toHaveBeenCalledWith({
        name: "Default Codex CLI orchestrator",
        kind: "codex_cli",
        program: "codex",
        arguments: [],
      }),
    );
    fireEvent.click(
      within(codex).getByRole("button", { name: "Work on tickets" }),
    );
    await waitFor(() =>
      expect(boardGateway.saveAgentProfile).toHaveBeenCalledOnce(),
    );

    fireEvent.click(within(codex).getByText("Connect provider API"));
    fireEvent.change(within(codex).getByLabelText("Codex API key"), {
      target: { value: "test-key" },
    });
    fireEvent.click(
      within(codex).getByRole("button", { name: "Connect and load models" }),
    );

    await waitFor(() =>
      expect(boardGateway.saveProviderCatalogCredential).toHaveBeenCalledWith({
        providerKind: "codex_cli",
        apiKey: "test-key",
      }),
    );
    expect(
      await within(codex).findByText(
        "Model list loaded from this provider account.",
      ),
    ).toBeVisible();
    await selectOption(codex, "Plan work", "Model", "GPT-5 Codex");
    await selectOption(codex, "Plan work", "Effort", "Thorough");
    await selectOption(codex, "Work on tickets", "Model", "GPT-5 Codex");
    await selectOption(codex, "Work on tickets", "Effort", "Balanced");
    fireEvent.click(screen.getByRole("button", { name: "Save AI setup" }));

    await waitFor(() =>
      expect(boardGateway.saveProjectAgentSettings).toHaveBeenCalledWith({
        boardId: "board-1",
        organiser: {
          plannerProfileName: "Default Codex CLI orchestrator",
          model: { kind: "named", name: "gpt-5-codex" },
          effort: "thorough",
        },
        ticketWorker: {
          agentProfileName: "Default Codex CLI",
          model: { kind: "named", name: "gpt-5-codex" },
          effort: "balanced",
        },
      }),
    );
  });

  it("does not select an agent when its safe profile could not be saved", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    vi.mocked(boardGateway.saveAgentProfile).mockRejectedValueOnce(
      new Error("Profile store is unavailable"),
    );

    await createBoard(boardGateway);
    openSettings("AI");
    const codex = providerCard("Codex");
    fireEvent.click(
      within(codex).getByRole("button", { name: "Work on tickets" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your saved work has not changed. Check your last action, then try again.",
    );
    expect(
      within(codex).getByRole("button", { name: "Work on tickets" }),
    ).toHaveAttribute("aria-pressed", "false");
  });
});

function providerCard(name: string): HTMLElement {
  const card = screen
    .getByRole("heading", { level: 4, name })
    .closest('[data-slot="card"]');
  if (card === null) throw new Error(`Missing ${name} provider card`);
  return card;
}

async function selectOption(
  card: HTMLElement,
  groupName: string,
  label: string,
  optionName: string,
) {
  const group = within(card).getByRole("group", { name: groupName });
  fireEvent.pointerDown(within(group).getByLabelText(label), {
    button: 0,
    ctrlKey: false,
    pointerType: "mouse",
  });
  fireEvent.click(await screen.findByRole("option", { name: optionName }));
}
