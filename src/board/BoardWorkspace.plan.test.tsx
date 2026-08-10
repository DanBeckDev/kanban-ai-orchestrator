import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { planDraft } from "./BoardWorkspace.plan.fixtures";
import { gateway } from "./BoardWorkspace.test.fixtures";
import {
  configurePlanner,
  createBoard,
  openPlan,
  openSettings,
} from "./BoardWorkspace.test.helpers";

describe("board plan workflow", () => {
  it("keeps the primary organiser prompt reviewable before it creates work", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    await configurePlanner();

    const composer = screen.getByRole("form", {
      name: "Prompt AI to orchestrate",
    });
    fireEvent.change(
      within(composer).getByLabelText("What do you want to achieve?"),
      { target: { value: "Make planning easier for the whole team." } },
    );
    fireEvent.click(
      within(composer).getByRole("button", { name: "Create plan preview" }),
    );

    await waitFor(() =>
      expect(boardGateway.generatePlan).toHaveBeenCalledWith({
        boardId: "board-1",
        goal: "Make planning easier for the whole team.",
        plannerProfileName: "local planner",
      }),
    );
    expect(
      await screen.findByRole("list", { name: "Plan tasks" }),
    ).toHaveTextContent("Generated foundation");
    expect(
      screen.getByText("The workspace policy is still being confirmed."),
    ).toBeVisible();
    expect(boardGateway.createWorkItem).not.toHaveBeenCalled();
    expect(boardGateway.startExecution).not.toHaveBeenCalled();
    expect(boardGateway.coordinateBoard).not.toHaveBeenCalled();
    expect(
      screen.getByRole("form", { name: "Confirm board plan" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Your name")).toHaveAttribute(
      "autocomplete",
      "name",
    );
    expect(screen.getByLabelText("Your name")).toHaveAttribute(
      "name",
      "confirmed-by",
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Edit proposed tasks" }),
    );
    expect(
      screen.getByRole("form", { name: "Edit plan proposal" }),
    ).toBeVisible();
    expect(
      screen.getByRole("checkbox", {
        name: "Require a person to approve this task before it is done",
      }),
    ).toHaveAttribute("name", "plan-task-1-requires-human-review");
  });

  it("generates a proposal from a goal and still requires explicit confirmation", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    await configurePlanner();
    openPlan();

    const generationForm = await screen.findByRole("form", {
      name: "Plan work with AI",
    });
    fireEvent.change(
      within(generationForm).getByLabelText("What do you want to achieve?"),
      {
        target: { value: "Build a dependable planning workflow." },
      },
    );
    fireEvent.click(
      within(generationForm).getByRole("button", {
        name: "Create plan preview",
      }),
    );

    await waitFor(() =>
      expect(boardGateway.generatePlan).toHaveBeenCalledWith({
        boardId: "board-1",
        plannerProfileName: "local planner",
        goal: "Build a dependable planning workflow.",
      }),
    );
    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Generated foundation",
    );
    expect(
      screen.getByRole("heading", { name: "Task 1: Generated foundation" }),
    ).toBeVisible();

    const confirmationForm = screen.getByRole("form", {
      name: "Confirm board plan",
    });
    fireEvent.change(within(confirmationForm).getByLabelText("Your name"), {
      target: { value: "Daniel" },
    });
    fireEvent.click(
      within(confirmationForm).getByRole("button", {
        name: "Confirm and create tasks",
      }),
    );

    await waitFor(() =>
      expect(boardGateway.confirmPlan).toHaveBeenCalledOnce(),
    );
    expect(await screen.findByText("Generated foundation")).toBeVisible();
  });

  it("keeps an unsaved planner profile editable when the daemon rejects it", async () => {
    const boardGateway = gateway();
    vi.mocked(boardGateway.savePlannerProfile).mockRejectedValueOnce(
      new Error("Planner profiles must use a declared program."),
    );
    await createBoard(boardGateway);
    openSettings("AI");
    const profileForm = screen.getByRole("form", {
      name: "Save orchestrator connection",
    });
    const nameInput = within(profileForm).getByLabelText("Connection name");
    fireEvent.change(nameInput, { target: { value: "local planner" } });
    fireEvent.click(
      within(profileForm).getByRole("button", {
        name: "Save orchestrator connection",
      }),
    );

    expect(await within(profileForm).findByRole("alert")).toHaveTextContent(
      "Check the connection name, program, and arguments, then try again.",
    );
    expect(nameInput).toHaveValue("local planner");
  });

  it("shows a planner generation error without creating or clearing a proposal", async () => {
    const boardGateway = gateway();
    vi.mocked(boardGateway.generatePlan).mockRejectedValueOnce(
      new Error("Planner response exceeds the 65536-byte limit."),
    );
    await createBoard(boardGateway);
    await configurePlanner();
    openPlan();
    const generationForm = await screen.findByRole("form", {
      name: "Plan work with AI",
    });
    const goalInput = within(generationForm).getByLabelText(
      "What do you want to achieve?",
    );
    fireEvent.change(goalInput, { target: { value: "Build a safe plan." } });
    fireEvent.click(
      within(generationForm).getByRole("button", {
        name: "Create plan preview",
      }),
    );

    expect(await within(generationForm).findByRole("alert")).toHaveTextContent(
      "Check the outcome and selected orchestrator, then try again.",
    );
    expect(goalInput).toHaveValue("Build a safe plan.");
    expect(screen.queryByRole("list", { name: "Plan tasks" })).toBeNull();
  });

  it("previews a provider-neutral plan before an explicit confirmation materializes it", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview pasted plan" }),
    );

    await waitFor(() =>
      expect(boardGateway.proposePlan).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.proposePlan).toHaveBeenCalledWith(
      expect.objectContaining({
        boardId: "board-1",
        proposedBy: "user",
        workItems: [
          expect.objectContaining({ workItemId: "foundation" }),
          expect.objectContaining({
            workItemId: "interface",
            requiresHumanReview: true,
          }),
        ],
      }),
    );
    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Foundation",
    );
    expect(screen.getByText("Contract is verified.")).toBeVisible();
    expect(screen.getByText("The local base branch exists.")).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Task 1: Foundation" }),
    ).toBeVisible();

    const confirmationForm = screen.getByRole("form", {
      name: "Confirm board plan",
    });
    fireEvent.change(within(confirmationForm).getByLabelText("Your name"), {
      target: { value: "Daniel" },
    });
    fireEvent.click(
      within(confirmationForm).getByRole("button", {
        name: "Confirm and create tasks",
      }),
    );

    await waitFor(() =>
      expect(boardGateway.confirmPlan).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.confirmPlan).toHaveBeenCalledWith(
      expect.objectContaining({ boardId: "board-1", confirmedBy: "Daniel" }),
    );
    expect(await screen.findByText("Foundation")).toBeVisible();
    expect(screen.getByText("Interface")).toBeVisible();
  });

  it("replaces an unconfirmed preview without creating its superseded tasks", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview pasted plan" }),
    );
    await screen.findByRole("button", { name: "Edit proposed tasks" });

    fireEvent.click(
      screen.getByRole("button", { name: "Edit proposed tasks" }),
    );
    const revisionForm = screen.getByRole("form", {
      name: "Edit plan proposal",
    });
    fireEvent.change(within(revisionForm).getAllByLabelText("Task name")[0], {
      target: { value: "Revised foundation" },
    });
    fireEvent.click(
      within(revisionForm).getByRole("button", {
        name: "Save revised preview",
      }),
    );

    await waitFor(() =>
      expect(boardGateway.proposePlan).toHaveBeenCalledTimes(2),
    );
    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Revised foundation",
    );
    expect(
      screen.getByRole("heading", { name: "Task 1: Revised foundation" }),
    ).toBeVisible();
  });

  it("keeps the saved preview visible when a user cancels revision", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview pasted plan" }),
    );
    await screen.findByRole("button", { name: "Edit proposed tasks" });

    fireEvent.click(
      screen.getByRole("button", { name: "Edit proposed tasks" }),
    );
    const revisionForm = screen.getByRole("form", {
      name: "Edit plan proposal",
    });
    fireEvent.click(
      within(revisionForm).getByRole("button", { name: "Cancel" }),
    );

    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Foundation",
    );
    expect(
      screen.queryByRole("form", { name: "Edit plan proposal" }),
    ).toBeNull();
  });

  it("keeps a proposal editable when the daemon rejects its preview", async () => {
    const boardGateway = gateway();
    vi.mocked(boardGateway.proposePlan).mockRejectedValueOnce(
      new Error("The plan has an unresolved hard-dependency cycle."),
    );
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview pasted plan" }),
    );

    expect(
      await screen.findAllByText("Check your changes, then try again."),
    ).toHaveLength(1);
    expect(
      screen.getByRole("form", { name: "Paste an existing plan" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("list", { name: "Plan tasks" })).toBeNull();
  });

  it("lets a person reorder, remove, and reassign proposed tasks before confirmation", async () => {
    const boardGateway = gateway();
    await boardGateway.saveAgentProfile({
      name: "focused worker",
      kind: "codex_cli",
      program: "codex",
      arguments: [],
    });
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview pasted plan" }),
    );
    await screen.findByRole("button", { name: "Edit proposed tasks" });

    fireEvent.click(
      screen.getByRole("button", { name: "Edit proposed tasks" }),
    );
    const editor = screen.getByRole("form", { name: "Edit plan proposal" });
    fireEvent.pointerDown(
      within(editor).getAllByLabelText("Ticket worker")[1],
      {
        button: 0,
        ctrlKey: false,
        pointerType: "mouse",
      },
    );
    fireEvent.click(
      await screen.findByRole("option", { name: "focused worker" }),
    );
    fireEvent.click(
      within(editor).getByRole("button", { name: "Move task 2 up" }),
    );
    fireEvent.click(
      within(editor).getByRole("button", { name: "Remove task 2" }),
    );
    fireEvent.click(
      within(editor).getByRole("button", { name: "Save revised preview" }),
    );

    await waitFor(() =>
      expect(boardGateway.proposePlan).toHaveBeenLastCalledWith(
        expect.objectContaining({
          workItems: [
            expect.objectContaining({
              workItemId: "interface",
              assignedAgentProfileName: "focused worker",
            }),
          ],
          dependencies: [],
        }),
      ),
    );
    expect(
      screen.getByRole("heading", { name: "Task 1: Interface" }),
    ).toBeVisible();
  });
});
