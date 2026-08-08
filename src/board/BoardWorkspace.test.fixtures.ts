import { vi } from "vitest";

import type {
  AgentProfile,
  BoardGateway,
  BoardPlan,
  BoardSnapshot,
  ConfirmPlanRequest,
  CreateWorkItemRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  ProposePlanRequest,
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
  let linearConnectionStatus: LinearConnectionStatus = { kind: "disconnected" };
  const linearIssues: readonly LinearIssueSummary[] = [];
  let profiles: readonly AgentProfile[] = [];
  let savedPlan: BoardPlan | undefined;
  let proposedPlan: ProposePlanRequest | undefined;
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
    proposePlan: vi
      .fn()
      .mockImplementation(async (request: ProposePlanRequest) => {
        proposedPlan = request;
        savedPlan = {
          preview: {
            id: request.planId,
            projectId: current.board.projectId,
            workItems: request.workItems.map((workItem) => ({
              id: workItem.workItemId,
              title: workItem.title,
              acceptanceCriteria: workItem.acceptanceCriteria,
              budget: workItem.budget,
            })),
            dependencies: request.dependencies.map((dependency) => ({
              id: dependency.dependencyId,
              upstreamWorkItemId: dependency.upstreamWorkItemId,
              downstreamWorkItemId: dependency.downstreamWorkItemId,
              kind: dependency.kind,
              reason: dependency.reason,
              owner: dependency.owner,
              nextAction: dependency.nextAction,
            })),
            criticalPath: request.workItems.map(
              (workItem) => workItem.workItemId,
            ),
            parallelStages: [
              request.workItems.map((workItem) => workItem.workItemId),
            ],
            budget: {
              workItemsMissingAgentTurnBudget: [],
              workItemsMissingDurationBudget: [],
              workItemsMissingCostBudget: [],
            },
            unresolvedAssumptions: request.unresolvedAssumptions,
          },
        };
        return savedPlan;
      }),
    boardPlan: vi.fn().mockImplementation(async () => savedPlan),
    confirmPlan: vi
      .fn()
      .mockImplementation(async (request: ConfirmPlanRequest) => {
        if (savedPlan === undefined || proposedPlan === undefined) {
          throw new Error("plan not found");
        }
        if (savedPlan.preview.id !== request.planId) {
          throw new Error("plan confirmation does not match the saved plan");
        }
        savedPlan = {
          ...savedPlan,
          confirmation: {
            planId: request.planId,
            confirmedBy: request.confirmedBy,
            confirmedAt: request.confirmedAt,
          },
        };
        current = {
          ...current,
          workItems: [
            ...current.workItems,
            ...proposedPlan.workItems.map((workItem, index) => ({
              lastEventSequence: current.workItems.length + index + 1,
              workItem: {
                id: workItem.workItemId,
                boardId: current.board.id,
                title: workItem.title,
                description: workItem.description,
                acceptanceCriteria: workItem.acceptanceCriteria,
                budget: workItem.budget,
                state: "inbox" as const,
                requiresHumanReview: workItem.requiresHumanReview,
              },
            })),
          ],
          dependencies: savedPlan.preview.dependencies,
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
    recordReviewDecision: vi.fn().mockImplementation(async (request) => {
      current = {
        ...current,
        evidence: [
          ...current.evidence,
          {
            id: request.evidenceId,
            workItemId: request.workItemId,
            kind: "review_decision",
            result: request.accepted ? "passed" : "failed",
            summary: `${request.reviewer}: ${request.summary}`,
            recordedAt: request.recordedAt,
          },
        ],
      };
      return current;
    }),
    beginLinearOAuth: vi
      .fn()
      .mockImplementation(async (_configuration: LinearOAuthConfiguration) => {
        linearConnectionStatus = { kind: "awaiting_authorization" };
        return linearConnectionStatus;
      }),
    linearConnectionStatus: vi
      .fn()
      .mockImplementation(async () => linearConnectionStatus),
    linearAssignedIssues: vi.fn().mockImplementation(async () => linearIssues),
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
