import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openSettings,
  openTask,
  selectOption,
} from "./BoardWorkspace.test.helpers";

describe("board task agents", () => {
  it("saves a direct agent profile and starts it from a ready task", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");
    fireEvent.click(screen.getByText("Set up a custom ticket worker"));
    const profileForm = screen.getByRole("form", { name: /save agent/i });
    fireEvent.change(within(profileForm).getByLabelText("Profile name"), {
      target: { value: "structured-worker" },
    });
    fireEvent.change(within(profileForm).getByLabelText("Program"), {
      target: { value: "agent-worker" },
    });
    fireEvent.change(
      within(profileForm).getByLabelText("Arguments (one per line)"),
      {
        target: { value: "--jsonl" },
      },
    );
    fireEvent.click(
      within(profileForm).getByRole("button", { name: "Save profile" }),
    );
    await waitFor(() =>
      expect(boardGateway.saveAgentProfile).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.saveAgentProfile).toHaveBeenCalledWith({
      name: "structured-worker",
      kind: "structured_process",
      program: "agent-worker",
      arguments: ["--jsonl"],
    });

    fireEvent.click(screen.getByRole("button", { name: "Back to Tickets" }));
    openTask("Task ready-task");

    const launchForm = screen.getByRole("form", {
      name: "Prompt AI for Task ready-task",
    });
    await selectOption("Agent profile", /structured-worker/);
    fireEvent.click(
      within(launchForm).getByRole("button", { name: "Start task worker" }),
    );

    await waitFor(() =>
      expect(boardGateway.startExecution).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.startExecution).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "ready-task",
        agentProfileName: "structured-worker",
      }),
    );
  });

  it("shows the selected provider's unsupported capabilities before task start", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    openSettings("AI");
    fireEvent.click(screen.getByText("Set up a custom ticket worker"));
    const profileForm = screen.getByRole("form", { name: /save agent/i });
    fireEvent.change(within(profileForm).getByLabelText("Profile name"), {
      target: { value: "cline-pass-worker" },
    });
    fireEvent.change(within(profileForm).getByLabelText("Adapter"), {
      target: { value: "cline_pass_cli" },
    });
    expect(within(profileForm).getByLabelText("Program")).toHaveValue("cline");
    expect(
      within(profileForm).getByText(/locally configured clinepass account/i),
    ).toBeVisible();
    fireEvent.click(
      within(profileForm).getByRole("button", { name: "Save profile" }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Back to Tickets" }));
    openTask("Task ready-task");

    const launchForm = await screen.findByRole("form", {
      name: "Prompt AI for Task ready-task",
    });
    await selectOption("Agent profile", /cline-pass-worker/);

    expect(
      within(launchForm).getByText(
        /feedback, session resume, and safe process-tree cancellation are not available yet/i,
      ),
    ).toBeVisible();
  });
});
