import { vi } from "vitest";

import type {
  AgentProfile,
  BoardGateway,
  BoardSnapshot,
  CreateWorkItemRequest,
  TransitionWorkItemRequest,
  WorkItemState,
} from "./types";

export function snapshot(
  workItems: BoardSnapshot["workItems"] = [],
  activity: BoardSnapshot["activity"] = [],
  executions: BoardSnapshot["executions"] = [],
  evidence: BoardSnapshot["evidence"] = [],
): BoardSnapshot {
  return {
    board: { id: "board-1", projectId: "project-1", name: "MVP" },
    workItems,
    dependencies: [],
    activity,
    executions,
    evidence,
    externalLinks: [],
  };
}

export function workItem(
  id: string,
  state: WorkItemState = "inbox",
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

export function gateway(initialSnapshot = snapshot()): BoardGateway {
  let current = initialSnapshot;
  let profiles: readonly AgentProfile[] = [];
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
    saveAgentProfile: vi
      .fn()
      .mockImplementation(async (profile: AgentProfile) => {
        profiles = [
          ...profiles.filter(({ name }) => name !== profile.name),
          profile,
        ];
        return profile;
      }),
    agentProfiles: vi.fn().mockImplementation(async () => profiles),
    startExecution: vi.fn().mockImplementation(async (request) => {
      current = {
        ...current,
        workItems: current.workItems.map((materializedWorkItem) =>
          materializedWorkItem.workItem.id === request.workItemId
            ? {
                ...materializedWorkItem,
                workItem: {
                  ...materializedWorkItem.workItem,
                  state: "running",
                },
              }
            : materializedWorkItem,
        ),
      };
      return current;
    }),
    stopExecution: vi.fn().mockImplementation(async () => current),
    recordReviewCheck: vi.fn().mockImplementation(async (request) => {
      current = {
        ...current,
        evidence: [
          ...current.evidence,
          {
            id: request.evidenceId,
            workItemId: request.workItemId,
            kind: "check",
            result: request.passed ? "passed" : "failed",
            summary: request.summary,
            recordedAt: request.recordedAt,
          },
        ],
      };
      return current;
    }),
    importLinearIssue: vi.fn().mockImplementation(async (request) => {
      current = {
        ...current,
        externalLinks: [
          ...current.externalLinks,
          {
            id: request.externalLinkId,
            workItemId: request.workItemId,
            connectorId: "linear",
            provenance: "imported",
            externalId: request.issueId,
            displayIdentifier: request.displayIdentifier,
            url: request.url,
            connectionMode: request.connectionMode,
          },
        ],
      };
      return current;
    }),
    importLinearBlocker: vi.fn().mockImplementation(async (request) => {
      const upstream = current.externalLinks.find(
        (link) => link.externalId === request.upstreamIssueId,
      );
      const downstream = current.externalLinks.find(
        (link) => link.externalId === request.downstreamIssueId,
      );
      current = {
        ...current,
        dependencies: [
          ...current.dependencies,
          {
            id: request.dependencyId,
            upstreamWorkItemId: upstream?.workItemId ?? request.upstreamIssueId,
            downstreamWorkItemId:
              downstream?.workItemId ?? request.downstreamIssueId,
            kind: "blocks",
            reason: request.reason,
            owner: request.owner,
            nextAction: request.nextAction,
          },
        ],
      };
      return current;
    }),
    boardSnapshot: vi.fn().mockImplementation(async () => current),
  };
}
