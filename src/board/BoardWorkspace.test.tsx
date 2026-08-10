import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "../App";
import {
  boardLibraryEntry,
  gateway,
  snapshot,
  workItem,
} from "./BoardWorkspace.test.fixtures";
import {
  createBoard,
  openDependencies,
  openNewTask,
  openTask,
  selectOption,
} from "./BoardWorkspace.test.helpers";

describe("board workspace", () => {
  it("creates a local board through the injected daemon gateway", async () => {
    const boardGateway = gateway();

    await createBoard(boardGateway);

    expect(boardGateway.inspectRepository).toHaveBeenCalledWith(
      "/projects/project",
    );
    expect(boardGateway.createLocalBoard).toHaveBeenCalledWith({
      name: "MVP",
      repositoryPath: "/projects/project",
      baseRef: "main",
      policySetId: "standard",
    });
  });

  it("adds a task and a typed dependency, then renders its reason and owner", async () => {
    const boardGateway = gateway(snapshot([workItem("api")]));
    await createBoard(boardGateway);
    openNewTask();
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
    fireEvent.click(screen.getByRole("button", { name: "Create task" }));
    await waitFor(() =>
      expect(boardGateway.createWorkItem).toHaveBeenCalledOnce(),
    );
    const createdWorkItemId = vi.mocked(boardGateway.createWorkItem).mock
      .calls[0][0].workItemId;

    openDependencies();
    fireEvent.click(
      screen.getByRole("button", { name: "Add a relationship manually" }),
    );
    await selectOption("Must happen first", "Task api");
    await selectOption("Depends on it", "UI");
    const dependencyForm = screen.getByRole("form", {
      name: "Add a relationship",
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Why"), {
      target: { value: "UI needs the API." },
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Owner"), {
      target: { value: "platform" },
    });
    fireEvent.change(within(dependencyForm).getByLabelText("Next action"), {
      target: { value: "Complete API." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add relationship" }));

    await waitFor(() =>
      expect(boardGateway.addDependency).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.addDependency).toHaveBeenCalledWith(
      expect.objectContaining({ downstreamWorkItemId: createdWorkItemId }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    expect(await screen.findByText("Waiting on Task api")).toBeVisible();
    openTask("UI");
    fireEvent.click(screen.getByRole("button", { name: "Dependencies" }));
    await screen.findByText(/UI needs the API/);
    expect(screen.getByText("Owner")).toBeInTheDocument();
    expect(screen.getByText("platform")).toBeInTheDocument();
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
              qualityGatePassed: true,
              completionReportPresent: true,
              reviewAccepted: true,
            },
          },
        ],
      ),
    );
    await createBoard(boardGateway);
    openTask("Task review-task");
    fireEvent.click(screen.getByRole("button", { name: "Decision history" }));
    expect(
      screen.getByText(
        "Evidence: quality gate passed, report present, review accepted.",
      ),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "More task options" }));
    const transitionForm = screen.getByRole("form", {
      name: "Change state for Task review-task",
    });
    fireEvent.pointerDown(screen.getByLabelText("Next state"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });
    expect(screen.queryByRole("option", { name: "ready" })).toBeNull();
    fireEvent.keyDown(document.activeElement ?? document.body, {
      key: "Escape",
    });
    await selectOption("Next state", "done");
    fireEvent.change(within(transitionForm).getByLabelText("Why"), {
      target: { value: "Review accepted." },
    });
    for (const label of [
      "Quality gate passed",
      "Completion report present",
      "Independent and human reviews accepted",
    ]) {
      fireEvent.click(within(transitionForm).getByLabelText(label));
    }
    fireEvent.click(
      screen.getByRole("button", { name: "Request state change" }),
    );
    await waitFor(() =>
      expect(boardGateway.transitionWorkItem).toHaveBeenCalledOnce(),
    );
    expect(boardGateway.transitionWorkItem).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "review-task",
        nextState: "done",
        evidence: {
          qualityGatePassed: true,
          completionReportPresent: true,
          reviewAccepted: true,
        },
      }),
    );

    const failingGateway = gateway(snapshot(), [boardLibraryEntry()]);
    failingGateway.openBoard = vi
      .fn()
      .mockRejectedValue(new Error("not found"));
    render(<App gateway={failingGateway} />);
    fireEvent.click(await screen.findByRole("button", { name: "Open MVP" }));
    expect(
      await screen.findByText(
        "Your saved work has not changed. Check your last action, then try again.",
      ),
    ).toBeVisible();
  });

  it("records a durable check while a task is awaiting review", async () => {
    const boardGateway = gateway(snapshot([workItem("review-task", "review")]));

    await createBoard(boardGateway);
    openTask("Task review-task");
    const form = screen.getByRole("form", {
      name: "Record quality gate for Task review-task",
    });
    fireEvent.change(within(form).getByLabelText("Result summary"), {
      target: { value: "Unit tests passed." },
    });
    fireEvent.click(
      within(form).getByRole("button", { name: "Record quality gate" }),
    );

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

  it("records a rejection and returns the task for correction", async () => {
    const boardGateway = gateway(snapshot([workItem("review-task", "review")]));

    await createBoard(boardGateway);
    openTask("Task review-task");
    const form = screen.getByRole("form", {
      name: "Record review decision for Task review-task",
    });
    fireEvent.change(within(form).getByLabelText("Reviewer"), {
      target: { value: "Daniel" },
    });
    fireEvent.change(within(form).getByLabelText("Decision summary"), {
      target: { value: "Acceptance criteria verified." },
    });
    fireEvent.click(within(form).getByLabelText("Accept this work"));
    fireEvent.click(
      within(form).getByRole("button", { name: "Return for correction" }),
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
    await waitFor(() =>
      expect(boardGateway.transitionWorkItem).toHaveBeenCalledWith(
        expect.objectContaining({
          workItemId: "review-task",
          nextState: "ready",
          reason: "Acceptance criteria verified.",
        }),
      ),
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
    openTask("Task interrupted-task");
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
    openTask("Task active-task");
    fireEvent.click(screen.getByRole("button", { name: "More task options" }));
    fireEvent.pointerDown(screen.getByLabelText("Next state"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });
    expect(screen.queryByRole("option", { name: "cancelled" })).toBeNull();
    fireEvent.keyDown(document.activeElement ?? document.body, {
      key: "Escape",
    });
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
    openTask("Task review-task");
    fireEvent.click(
      screen.getByRole("button", { name: "Activity and attempts" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Review evidence" }));

    expect(screen.getByText(/codex-cli · awaiting review/)).toBeVisible();
    expect(screen.getByText("Session: session-1")).toBeVisible();
    expect(screen.getByText("Unit tests passed.")).toBeVisible();
  });
});
