import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "../App";
import type {
  BoardGateway,
  BoardSnapshot,
  CreateWorkItemRequest,
  TransitionWorkItemRequest,
} from "./types";

function snapshot(workItems: BoardSnapshot["workItems"] = []): BoardSnapshot {
  return {
    board: { id: "board-1", projectId: "project-1", name: "MVP" },
    workItems,
    dependencies: [],
  };
}

function workItem(
  id: string,
  state: "inbox" | "review" = "inbox",
): BoardSnapshot["workItems"][number] {
  return {
    lastEventSequence: 1,
    workItem: {
      id,
      boardId: "board-1",
      title: `Task ${id}`,
      description: "A bounded task.",
      acceptanceCriteria: ["Tests pass."],
      budget: {},
      state,
      requiresHumanReview: state === "review",
    },
  };
}

function gateway(initialSnapshot = snapshot()): BoardGateway {
  let current = initialSnapshot;
  return {
    createProject: vi.fn().mockResolvedValue(undefined),
    createBoard: vi.fn().mockImplementation(async () => current),
    createWorkItem: vi
      .fn()
      .mockImplementation(async (request: CreateWorkItemRequest) => {
        current = snapshot([
          ...current.workItems,
          {
            lastEventSequence: current.workItems.length + 1,
            workItem: {
              id: request.workItemId,
              boardId: request.boardId,
              title: request.title,
              description: request.description,
              acceptanceCriteria: request.acceptanceCriteria,
              budget: request.budget,
              state: "inbox",
              requiresHumanReview: request.requiresHumanReview,
            },
          },
        ]);
        return current;
      }),
    addDependency: vi.fn().mockImplementation(async (request) => {
      current = {
        ...current,
        dependencies: [
          ...current.dependencies,
          {
            id: request.dependencyId,
            upstreamWorkItemId: request.upstreamWorkItemId,
            downstreamWorkItemId: request.downstreamWorkItemId,
            kind: request.kind,
            reason: request.reason,
            owner: request.owner,
            nextAction: request.nextAction,
          },
        ],
      };
      return current;
    }),
    transitionWorkItem: vi
      .fn()
      .mockImplementation(async (request: TransitionWorkItemRequest) => {
        current = {
          ...current,
          workItems: current.workItems.map((materializedWorkItem) =>
            materializedWorkItem.workItem.id === request.workItemId
              ? {
                  ...materializedWorkItem,
                  workItem: {
                    ...materializedWorkItem.workItem,
                    state: request.nextState,
                  },
                }
              : materializedWorkItem,
          ),
        };
        return current;
      }),
    boardSnapshot: vi.fn().mockImplementation(async () => current),
  };
}

async function createBoard(boardGateway: BoardGateway) {
  render(<App gateway={boardGateway} />);
  fireEvent.change(screen.getByLabelText("Project ID"), {
    target: { value: "project-1" },
  });
  fireEvent.change(screen.getByLabelText("Project name"), {
    target: { value: "Project" },
  });
  fireEvent.change(screen.getByLabelText("Repository path"), {
    target: { value: "/projects/project" },
  });
  fireEvent.change(screen.getByLabelText("New board ID"), {
    target: { value: "board-1" },
  });
  fireEvent.change(screen.getByLabelText("New board name"), {
    target: { value: "MVP" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create local board" }));
  await screen.findByRole("heading", { name: "MVP" });
}

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
    await screen.findByText("UI");
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
    const boardGateway = gateway(snapshot([workItem("review-task", "review")]));
    await createBoard(boardGateway);
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
      "Review accepted",
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
});
