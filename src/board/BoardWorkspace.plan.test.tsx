import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { gateway } from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openPlan,
  openSettings,
} from "./BoardWorkspace.test.helpers";

describe("board plan workflow", () => {
  it("generates a proposal from a goal and still requires explicit confirmation", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    await configurePlanner();
    openPlan();

    const generationForm = await screen.findByRole("form", {
      name: "Plan with AI",
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
      screen.queryByRole("heading", { name: "Generated foundation" }),
    ).toBeNull();

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
    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    expect(await screen.findByText("Generated foundation")).toBeVisible();
  });

  it("keeps an unsaved planner profile editable when the daemon rejects it", async () => {
    const boardGateway = gateway();
    vi.mocked(boardGateway.savePlannerProfile).mockRejectedValueOnce(
      new Error("Planner profiles must use a declared program."),
    );
    await createBoard(boardGateway);
    openSettings("Organiser");
    const profileForm = screen.getByRole("form", {
      name: "Save organiser connection",
    });
    const nameInput = within(profileForm).getByLabelText("Connection name");
    fireEvent.change(nameInput, { target: { value: "local planner" } });
    fireEvent.click(
      within(profileForm).getByRole("button", {
        name: "Save organiser connection",
      }),
    );

    expect(await within(profileForm).findByRole("alert")).toHaveTextContent(
      "Planner profiles must use a declared program.",
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
      name: "Plan with AI",
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
      "Planner response exceeds the 65536-byte limit.",
    );
    expect(goalInput).toHaveValue("Build a safe plan.");
    expect(screen.queryByRole("list", { name: "Plan tasks" })).toBeNull();
  });

  it("previews a provider-neutral plan before an explicit confirmation materializes it", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Add an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview plan" }),
    );

    await waitFor(() =>
      expect(boardGateway.proposePlan).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.proposePlan).toHaveBeenCalledWith(
      expect.objectContaining({
        boardId: "board-1",
        proposedBy: "orchestrator",
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
    expect(screen.queryByRole("heading", { name: "Foundation" })).toBeNull();

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
    expect(screen.getByText(/Confirmed by Daniel at/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    expect(await screen.findByText("Foundation")).toBeVisible();
    expect(screen.getByText("Interface")).toBeVisible();
  });

  it("replaces an unconfirmed preview without creating its superseded tasks", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Add an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview plan" }),
    );
    await screen.findByRole("button", { name: "Revise plan" });

    fireEvent.click(screen.getByRole("button", { name: "Revise plan" }));
    const revisionForm = screen.getByRole("form", {
      name: "Revise an existing plan",
    });
    fireEvent.change(within(revisionForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(revisedPlanDraft()) },
    });
    fireEvent.click(
      within(revisionForm).getByRole("button", {
        name: "Preview revised plan",
      }),
    );

    await waitFor(() =>
      expect(boardGateway.proposePlan).toHaveBeenCalledTimes(2),
    );
    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Revised foundation",
    );
    expect(screen.queryByRole("heading", { name: "Foundation" })).toBeNull();
  });

  it("keeps the saved preview visible when a user cancels revision", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Add an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview plan" }),
    );
    await screen.findByRole("button", { name: "Revise plan" });

    fireEvent.click(screen.getByRole("button", { name: "Revise plan" }));
    const revisionForm = screen.getByRole("form", {
      name: "Revise an existing plan",
    });
    fireEvent.click(
      within(revisionForm).getByRole("button", { name: "Cancel revision" }),
    );

    expect(screen.getByRole("list", { name: "Plan tasks" })).toHaveTextContent(
      "Foundation",
    );
    expect(
      screen.queryByRole("form", { name: "Revise an existing plan" }),
    ).toBeNull();
  });

  it("explains malformed plan JSON before it reaches the daemon", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Add an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify({ dependencies: [] }) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview plan" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Plan JSON must contain a workItems array.",
    );
    expect(boardGateway.proposePlan).not.toHaveBeenCalled();
  });

  it("keeps a proposal editable when the daemon rejects its preview", async () => {
    const boardGateway = gateway();
    vi.mocked(boardGateway.proposePlan).mockRejectedValueOnce(
      new Error("The plan has an unresolved hard-dependency cycle."),
    );
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Add an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify(planDraft()) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", { name: "Preview plan" }),
    );

    expect(
      await screen.findAllByText(
        "The plan has an unresolved hard-dependency cycle.",
      ),
    ).toHaveLength(2);
    expect(
      screen.getByRole("form", { name: "Add an existing plan" }),
    ).toBeVisible();
    expect(screen.queryByRole("list", { name: "Plan tasks" })).toBeNull();
  });
});

async function configurePlanner() {
  openSettings("Organiser");
  const profileForm = screen.getByRole("form", {
    name: "Save organiser connection",
  });
  fireEvent.change(within(profileForm).getByLabelText("Connection name"), {
    target: { value: "local planner" },
  });
  fireEvent.click(
    within(profileForm).getByRole("button", {
      name: "Save organiser connection",
    }),
  );
  await waitFor(() => expect(screen.getByText("local planner")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
}

function planDraft() {
  return {
    workItems: [
      {
        id: "foundation",
        title: "Foundation",
        description: "Create the shared contract.",
        acceptanceCriteria: ["Contract is verified."],
      },
      {
        id: "interface",
        title: "Interface",
        description: "Use the shared contract.",
        acceptanceCriteria: ["Interface is verified."],
        requiresHumanReview: true,
      },
    ],
    dependencies: [
      {
        id: "foundation-interface",
        upstreamWorkItemId: "foundation",
        downstreamWorkItemId: "interface",
        kind: "blocks",
        reason: "The interface needs the shared contract.",
        owner: "orchestrator",
        nextAction: "Finish the foundation task.",
      },
    ],
    unresolvedAssumptions: ["The local base branch exists."],
  };
}

function revisedPlanDraft() {
  return {
    workItems: [
      {
        id: "revised-foundation",
        title: "Revised foundation",
        description: "Replace the initial shared contract.",
        acceptanceCriteria: ["The revised contract is verified."],
      },
    ],
    dependencies: [],
    unresolvedAssumptions: [],
  };
}
