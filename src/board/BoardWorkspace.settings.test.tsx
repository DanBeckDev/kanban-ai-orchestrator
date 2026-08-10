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

  it("loads installed-agent models inside the selected provider and saves role-specific choices", async () => {
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
    await waitFor(() =>
      expect(boardGateway.providerModelCatalog).toHaveBeenCalledWith(
        "codex_cli",
      ),
    );
    expect(
      await within(codex).findByText(
        "Model options loaded from your installed AI runtime.",
      ),
    ).toBeVisible();
    expect(within(codex).queryByText("Connect provider API")).toBeNull();
    expect(within(codex).queryByLabelText("Codex API key")).toBeNull();
    await selectOption(codex, "Plan work", "Model", "GPT-5 Codex");
    await selectOption(codex, "Plan work", "Effort", "Thorough (high)");
    await selectOption(codex, "Work on tickets", "Model", "GPT-5 Codex");
    await selectOption(codex, "Work on tickets", "Effort", "Balanced (medium)");
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

  it("shows Claude Code's installed runtime capabilities without an API key", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");
    const claude = providerCard("Claude Code");
    fireEvent.click(within(claude).getByRole("button", { name: "Plan work" }));

    await waitFor(() =>
      expect(boardGateway.providerModelCatalog).toHaveBeenCalledWith(
        "claude_code",
      ),
    );
    expect(
      await within(claude).findByText(
        "Model options loaded from your installed AI runtime.",
      ),
    ).toBeVisible();
    await selectOption(claude, "Plan work", "Model", "Claude Fable");
    await selectOption(claude, "Plan work", "Effort", "Maximum (max)");
    expect(within(claude).queryByText("Connect provider API")).toBeNull();
    expect(within(claude).queryByLabelText("Claude Code API key")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Save AI setup" }));
    await waitFor(() =>
      expect(boardGateway.saveProjectAgentSettings).toHaveBeenCalledWith({
        boardId: "board-1",
        organiser: {
          plannerProfileName: "Default Claude Code orchestrator",
          model: { kind: "named", name: "fable" },
          effort: "maximum",
        },
        ticketWorker: undefined,
      }),
    );
  });

  it("keeps one clear provider-default state when no local catalogue exists", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    vi.mocked(boardGateway.providerModelCatalog).mockResolvedValueOnce({
      providerKind: "claude_code",
      status: "uses_provider_default",
      models: [],
    });

    await createBoard(boardGateway);
    openSettings("AI");
    const claude = providerCard("Claude Code");
    fireEvent.click(within(claude).getByRole("button", { name: "Plan work" }));

    expect(
      await within(claude).findByText(
        "Claude Code manages its models in its own app. Kanban will use that provider's default.",
      ),
    ).toBeVisible();
    expect(
      within(claude).getByText(
        "Plan work will use this AI's configured model and effort.",
      ),
    ).toBeVisible();
    expect(within(claude).queryByLabelText("Model")).toBeNull();
    expect(within(claude).queryByLabelText("Effort")).toBeNull();
  });

  it("offers a safe retry when an installed runtime is unavailable", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));
    vi.mocked(boardGateway.providerModelCatalog).mockResolvedValueOnce({
      providerKind: "codex_cli",
      status: "unavailable",
      models: [],
    });

    await createBoard(boardGateway);
    openSettings("AI");
    const codex = providerCard("Codex");
    fireEvent.click(within(codex).getByRole("button", { name: "Plan work" }));

    expect(
      await within(codex).findByText(
        "Kanban could not read models from Codex. Sign in or update it, then try again.",
      ),
    ).toBeVisible();
    expect(
      within(codex).getByRole("button", { name: "Refresh models" }),
    ).toBeVisible();
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
