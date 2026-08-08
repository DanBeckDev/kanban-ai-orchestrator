import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "../App";
import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard } from "./BoardWorkspace.test.helpers";

describe("board workspace", () => {
  it("creates a local project and board through the injected daemon gateway", async () => {
    const boardGateway = gateway();

    await createBoard(boardGateway);

    expect(boardGateway.createProject).toHaveBeenCalledWith({
      projectId: "project-1",
      name: "Project",
      repositoryPath: "/projects/project",
      baseRef: "main",
      policySetId: "standard",
    });
    expect(boardGateway.createBoard).toHaveBeenCalledWith({
      boardId: "board-1",
      projectId: "project-1",
      name: "MVP",
    });
  });

  it("opens an existing local board by ID", async () => {
    const boardGateway = gateway(snapshot([workItem("task-1")]));
    render(<App gateway={boardGateway} />);

    fireEvent.change(screen.getByLabelText("Existing board ID"), {
      target: { value: "board-1" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Open board" }));

    expect(await screen.findByRole("heading", { name: "MVP" })).toBeVisible();
    expect(boardGateway.boardSnapshot).toHaveBeenCalledWith("board-1");
  });

  it("adds a task and a typed dependency, then renders its reason and owner", async () => {
    const boardGateway = gateway(snapshot([workItem("api")]));
    await createBoard(boardGateway);
    fireEvent.change(screen.getByLabelText("Task ID"), {
      target: { value: "ui" },
    });
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "UI" },
    });
    fireEvent.change(screen.getByLabelText("Description"), {
      target: { value: "Build the board." },
    });
    fireEvent.change(screen.getByLabelText(/Acceptance criteria/), {
      target: { value: "Task is visible." },
    });
    fireEvent.change(screen.getByLabelText("Max agent turns"), {
      target: { value: "24" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add task" }));
    await screen.findByRole("heading", { name: "UI" });
    expect(screen.getByText("Max turns: 24")).toBeVisible();
    fireEvent.change(screen.getByLabelText("Upstream task"), {
      target: { value: "api" },
    });
    fireEvent.change(screen.getByLabelText("Downstream task"), {
      target: { value: "ui" },
    });
    const dependencyForm = screen.getByRole("form", {
      name: "Add dependency",
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Reason"), {
      target: { value: "UI needs the API." },
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Owner"), {
      target: { value: "platform" },
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Next action"), {
      target: { value: "Complete API." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add dependency" }));

    await screen.findByText(/UI needs the API/);
    expect(boardGateway.addDependency).toHaveBeenCalledOnce();
    expect(screen.getByText(/Owner: platform/)).toBeInTheDocument();
  });

  it("submits lifecycle transition evidence and presents command errors", async () => {
    const boardGateway = gateway(
      snapshot(
        [workItem("review-task", "review")],
        [
          {
            workItemId: "review-task",
            sequence: 1,
            recordedAt: "2026-08-08T00:00:00Z",
            summary: "State changed from running to review: Ready for review.",
            completionEvidence: {
              checksPassed: true,
              completionReportPresent: true,
              reviewAccepted: true,
            },
          },
        ],
      ),
    );
    await createBoard(boardGateway);
    fireEvent.click(screen.getByText("Recent decision history (1)"));
    expect(
      screen.getByText(
        "Evidence: checks passed, report present, review accepted.",
      ),
    ).toBeVisible();
    const transitionForm = screen.getByRole("form", {
      name: "Transition Task review-task",
    });
    fireEvent.change(within(transitionForm).getByLabelText("Move to"), {
      target: { value: "done" },
    });
    fireEvent.change(within(transitionForm).getByLabelText("Reason"), {
      target: { value: "Review accepted." },
    });
    for (const label of [
      "Checks passed",
      "Completion report present",
      "Recorded review accepted",
    ]) {
      fireEvent.click(within(transitionForm).getByLabelText(label));
    }
    fireEvent.click(screen.getByRole("button", { name: "Request transition" }));
    await waitFor(() =>
      expect(boardGateway.transitionWorkItem).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.transitionWorkItem).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "review-task",
        nextState: "done",
        evidence: {
          checksPassed: true,
          completionReportPresent: true,
          reviewAccepted: true,
        },
      }),
    );

    const failingGateway = gateway();
    failingGateway.boardSnapshot = vi
      .fn()
      .mockRejectedValue(new Error("not found"));
    render(<App gateway={failingGateway} />);
    fireEvent.change(screen.getByLabelText("Existing board ID"), {
      target: { value: "missing" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Open board" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("not found");
  });

  it("records a durable check while a task is awaiting review", async () => {
    const boardGateway = gateway(snapshot([workItem("review-task", "review")]));

    await createBoard(boardGateway);
    const form = screen.getByRole("form", {
      name: "Record review check for Task review-task",
    });
    fireEvent.change(within(form).getByLabelText("Result summary"), {
      target: { value: "Unit tests passed." },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Record check" }));

    await waitFor(() =>
      expect(boardGateway.recordReviewCheck).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.recordReviewCheck).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "review-task",
        passed: true,
        summary: "Unit tests passed.",
      }),
    );
  });

  it("records the human reviewer decision before a task can be marked done", async () => {
    const boardGateway = gateway(snapshot([workItem("review-task", "review")]));

    await createBoard(boardGateway);
    const form = screen.getByRole("form", {
      name: "Record review decision for Task review-task",
    });
    fireEvent.change(within(form).getByLabelText("Reviewer"), {
      target: { value: "Daniel" },
    });
    fireEvent.change(within(form).getByLabelText("Decision summary"), {
      target: { value: "Acceptance criteria verified." },
    });
    fireEvent.click(within(form).getByLabelText("Accept review"));
    fireEvent.click(
      within(form).getByRole("button", { name: "Record decision" }),
    );

    await waitFor(() =>
      expect(boardGateway.recordReviewDecision).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.recordReviewDecision).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "review-task",
        reviewer: "Daniel",
        summary: "Acceptance criteria verified.",
        accepted: false,
      }),
    );
  });

  it("offers inspect, recovery, and cancellation actions after an interrupted attempt", async () => {
    const boardGateway = gateway(
      snapshot(
        [workItem("interrupted-task", "interrupted")],
        [],
        [
          {
            id: "execution-1",
            workItemId: "interrupted-task",
            adapterName: "codex-cli",
            status: "interrupted",
            workspacePath: "/workspaces/interrupted-task",
            usage: { inputTokens: 42, outputTokens: 24 },
            lastEventSequence: 3,
          },
        ],
      ),
    );

    await createBoard(boardGateway);
    const actions = screen.getByRole("region", {
      name: "Recovery actions for Task interrupted-task",
    });
    fireEvent.click(
      within(actions).getByRole("button", { name: "Inspect last attempt" }),
    );
    expect(
      within(actions).getByText("/workspaces/interrupted-task"),
    ).toBeVisible();
    fireEvent.click(
      within(actions).getByRole("button", { name: "Recover to Ready" }),
    );

    await waitFor(() =>
      expect(boardGateway.transitionWorkItem).toHaveBeenCalledWith(
        expect.objectContaining({
          workItemId: "interrupted-task",
          nextState: "ready",
        }),
      ),
    );
  });

  it("uses direct process control instead of a lifecycle cancellation for active work", async () => {
    const boardGateway = gateway(
      snapshot(
        [workItem("active-task", "running")],
        [],
        [
          {
            id: "execution-1",
            workItemId: "active-task",
            adapterName: "codex-cli",
            status: "running",
            workspacePath: "/workspaces/active-task",
            usage: { inputTokens: 42, outputTokens: 24 },
            lastEventSequence: 3,
          },
        ],
      ),
    );

    await createBoard(boardGateway);
    const transition = screen.getByRole("form", {
      name: "Transition Task active-task",
    });
    expect(
      within(transition).queryByRole("option", { name: "cancelled" }),
    ).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Stop agent" }));

    await waitFor(() =>
      expect(boardGateway.stopExecution).toHaveBeenCalledWith("execution-1"),
    );
  });

  it("shows durable agent attempts and review evidence for the selected task", async () => {
    const boardGateway = gateway(
      snapshot(
        [workItem("review-task", "review")],
        [],
        [
          {
            id: "execution-1",
            workItemId: "review-task",
            adapterName: "codex-cli",
            status: "awaiting_review",
            sessionId: "session-1",
            workspacePath: "/workspaces/review-task",
            usage: { inputTokens: 42, outputTokens: 24 },
            lastEventSequence: 3,
          },
        ],
        [
          {
            id: "check-1",
            workItemId: "review-task",
            kind: "check",
            result: "passed",
            summary: "Unit tests passed.",
            recordedAt: "2026-08-08T00:02:00Z",
          },
        ],
      ),
    );

    await createBoard(boardGateway);
    fireEvent.click(screen.getByText("Recent agent attempts (1)"));
    fireEvent.click(screen.getByText("Recent review evidence (1)"));

    expect(screen.getByText(/codex-cli · awaiting review/)).toBeVisible();
    expect(screen.getByText("Session: session-1")).toBeVisible();
    expect(screen.getByText("Unit tests passed.")).toBeVisible();
  });

  it("saves a direct agent profile and starts it from a ready task", async () => {
    const boardGateway = gateway(snapshot([workItem("ready-task", "ready")]));

    await createBoard(boardGateway);
    const profileForm = screen.getByRole("form", {
      name: "Save agent profile",
    });
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

    const launchForm = screen.getByRole("form", {
      name: "Start agent for Task ready-task",
    });
    fireEvent.change(within(launchForm).getByLabelText("Agent profile"), {
      target: { value: "structured-worker" },
    });
    fireEvent.click(
      within(launchForm).getByRole("button", { name: "Start agent" }),
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
});
