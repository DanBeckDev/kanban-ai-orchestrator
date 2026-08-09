import { describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { tauriBoardGateway } from "./gateway";

const boardSnapshot = {
  board: { id: "board-1", projectId: "project-1", name: "MVP" },
  dependencies: [],
  workItems: [],
};

describe("tauri board gateway", () => {
  it("maps each board operation to its typed local daemon command", async () => {
    invoke.mockResolvedValue(boardSnapshot);
    const project = {
      projectId: "project-1",
      name: "Project",
      repositoryPath: "/projects/project",
      baseRef: "main",
      policySetId: "standard",
    };
    const board = { boardId: "board-1", projectId: "project-1", name: "MVP" };
    const workItem = {
      eventId: "create-1",
      workItemId: "task-1",
      boardId: "board-1",
      title: "Task",
      description: "Description",
      acceptanceCriteria: ["Criterion"],
      budget: {},
      requiresHumanReview: false,
      recordedAt: "2026-08-08T00:00:00.000Z",
    };
    const dependency = {
      dependencyId: "task-1-blocks-task-2",
      upstreamWorkItemId: "task-1",
      downstreamWorkItemId: "task-2",
      kind: "blocks" as const,
      reason: "Required first",
      owner: "platform",
      nextAction: "Finish task one",
      createdBy: "user",
      createdAt: "2026-08-08T00:00:00.000Z",
    };
    const transition = {
      eventId: "transition-1",
      workItemId: "task-1",
      nextState: "planned" as const,
      reason: "Ready to plan",
      recordedAt: "2026-08-08T00:00:00.000Z",
    };

    await tauriBoardGateway.createProject(project);
    await tauriBoardGateway.createBoard(board);
    await tauriBoardGateway.createWorkItem(workItem);
    await tauriBoardGateway.addDependency(dependency);
    await tauriBoardGateway.transitionWorkItem(transition);
    await tauriBoardGateway.executionActivity("execution-1", 2);
    await tauriBoardGateway.boardSnapshot("board-1");

    expect(invoke.mock.calls).toEqual([
      ["create_project", { request: project }],
      ["create_board", { request: board }],
      ["create_work_item", { request: workItem }],
      ["add_dependency", { request: dependency }],
      ["transition_work_item", { request: transition }],
      ["execution_activity", { executionId: "execution-1", afterSequence: 2 }],
      ["board_snapshot", { boardId: "board-1" }],
    ]);
  });
});
