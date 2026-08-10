import { vi } from "vitest";

import { linearGatewayMethods } from "./BoardWorkspace.test.linear.fixtures";
import { executionGatewayMethods } from "./BoardWorkspace.test.execution.fixtures";

import type {
  AgentProfile,
  BoardGateway,
  BoardLibraryEntry,
  BoardPlan,
  BoardSnapshot,
  BoardSupervision,
  ConfirmPlanRequest,
  CreateWorkItemRequest,
  GeneratePlanRequest,
  ProposePlanRequest,
  PlannerProfile,
  ProjectAgentSettings,
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
    connectorOutboxItems: [],
    connectorReconciliationItems: [],
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
      assignedAgentModel: { kind: "provider_default" },
      assignedAgentEffort: "provider_default",
    },
  };
}

export function boardLibraryEntry(
  overrides: Partial<BoardLibraryEntry> = {},
): BoardLibraryEntry {
  return {
    boardId: "board-1",
    name: "MVP",
    repositoryName: "project",
    repositoryAvailable: true,
    lastOpenedAt: "2026-08-09T08:00:00Z",
    attention: {
      activeWorkItemCount: 0,
      needsAttentionCount: 0,
    },
    ...overrides,
  };
}

export function gateway(
  initialSnapshot = snapshot(),
  initialLibrary: readonly BoardLibraryEntry[] = [],
): BoardGateway {
  let current = initialSnapshot;
  let profiles: readonly AgentProfile[] = [];
  let plannerProfiles: readonly PlannerProfile[] = [];
  let agentSettings: ProjectAgentSettings | undefined;
  let boardSupervision: BoardSupervision | undefined;
  const supervisionDecisions: readonly [] = [];
  let savedPlan: BoardPlan | undefined;
  let proposedPlan: ProposePlanRequest | undefined;
  return {
    createProject: vi.fn().mockResolvedValue(undefined),
    createBoard: vi.fn().mockImplementation(async () => current),
    inspectRepository: vi.fn().mockResolvedValue({
      repositoryPath: "/projects/project",
      suggestedBoardName: "Project",
      baseRef: "main",
    }),
    cloneGitHubRepository: vi.fn().mockResolvedValue({
      repositoryPath: "/projects/project",
      suggestedBoardName: "Project",
      baseRef: "main",
    }),
    createLocalBoard: vi.fn().mockImplementation(async () => current),
    boardLibrary: vi.fn().mockResolvedValue(initialLibrary),
    openBoard: vi.fn().mockImplementation(async () => current),
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
              assignedAgentProfileName:
                agentSettings?.ticketWorker?.agentProfileName,
              assignedAgentModel: { kind: "provider_default" },
              assignedAgentEffort: "provider_default",
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
              description: workItem.description,
              acceptanceCriteria: workItem.acceptanceCriteria,
              budget: workItem.budget,
              requiresHumanReview: workItem.requiresHumanReview,
              assignedAgentProfileName: workItem.assignedAgentProfileName,
              assignedAgentModel: workItem.assignedAgentModel ?? {
                kind: "provider_default",
              },
              assignedAgentEffort:
                workItem.assignedAgentEffort ?? "provider_default",
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
                assignedAgentProfileName: workItem.assignedAgentProfileName,
                assignedAgentModel: workItem.assignedAgentModel ?? {
                  kind: "provider_default",
                },
                assignedAgentEffort:
                  workItem.assignedAgentEffort ?? "provider_default",
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
    agentProviderAvailability: vi.fn().mockResolvedValue([
      {
        kind: "codex_cli",
        label: "Codex",
        program: "codex",
        installed: true,
      },
      {
        kind: "claude_code",
        label: "Claude Code",
        program: "claude",
        installed: true,
      },
      {
        kind: "cline_pass_cli",
        label: "Cline",
        program: "cline",
        installed: false,
      },
    ]),
    savePlannerProfile: vi
      .fn()
      .mockImplementation(async (profile: PlannerProfile) => {
        plannerProfiles = [
          ...plannerProfiles.filter(({ name }) => name !== profile.name),
          profile,
        ];
        return profile;
      }),
    plannerProfiles: vi.fn().mockImplementation(async () => plannerProfiles),
    saveProjectAgentSettings: vi.fn().mockImplementation(async (request) => {
      agentSettings = {
        projectId: current.board.projectId,
        organiser: request.organiser,
        ticketWorker: request.ticketWorker,
      };
      return agentSettings;
    }),
    projectAgentSettings: vi.fn().mockImplementation(async () => agentSettings),
    configureBoardSupervision: vi.fn().mockImplementation(async (_, mode) => {
      if (!agentSettings?.organiser || !agentSettings.ticketWorker) {
        throw new Error("choose roles first");
      }
      boardSupervision = {
        boardId: current.board.id,
        mode,
        organiser: agentSettings.organiser,
        ticketWorker: agentSettings.ticketWorker,
        limits: { maxParallelWorkItems: 1, maxRetriesPerWorkItem: 1 },
        permittedActions: ["prepare_work", "make_work_ready", "start_work"],
        configuredBy: "local-user",
        configuredAt: "2026-08-10T00:00:00Z",
        revision: (boardSupervision?.revision ?? 0) + 1,
      };
      return boardSupervision;
    }),
    boardSupervision: vi.fn().mockImplementation(async () => boardSupervision),
    supervisionDecisions: vi
      .fn()
      .mockImplementation(async () => supervisionDecisions),
    generatePlan: vi
      .fn()
      .mockImplementation(async (request: GeneratePlanRequest) => {
        proposedPlan = {
          planId: "generated-plan",
          boardId: request.boardId,
          proposedBy: `planner:${request.plannerProfileName}`,
          proposedAt: "2026-08-08T00:00:00Z",
          workItems: [
            {
              workItemId: "generated-foundation",
              title: "Generated foundation",
              description: "A planner-generated starting task.",
              acceptanceCriteria: ["The generated task is reviewed."],
              budget: {},
              requiresHumanReview: true,
            },
          ],
          dependencies: [],
          unresolvedAssumptions: [],
        };
        savedPlan = {
          preview: {
            id: proposedPlan.planId,
            projectId: current.board.projectId,
            workItems: proposedPlan.workItems.map((workItem) => ({
              id: workItem.workItemId,
              title: workItem.title,
              description: workItem.description,
              acceptanceCriteria: workItem.acceptanceCriteria,
              budget: workItem.budget,
              requiresHumanReview: workItem.requiresHumanReview,
              assignedAgentProfileName: workItem.assignedAgentProfileName,
              assignedAgentModel: workItem.assignedAgentModel ?? {
                kind: "provider_default",
              },
              assignedAgentEffort:
                workItem.assignedAgentEffort ?? "provider_default",
            })),
            dependencies: [],
            criticalPath: ["generated-foundation"],
            parallelStages: [["generated-foundation"]],
            budget: {
              workItemsMissingAgentTurnBudget: ["generated-foundation"],
              workItemsMissingDurationBudget: ["generated-foundation"],
              workItemsMissingCostBudget: ["generated-foundation"],
            },
            unresolvedAssumptions: [
              "The workspace policy is still being confirmed.",
            ],
          },
        };
        return savedPlan;
      }),
    ...executionGatewayMethods({
      current: () => current,
      replace: (snapshot) => {
        current = snapshot;
      },
    }),
    ...linearGatewayMethods({
      current: () => current,
      replace: (snapshot) => {
        current = snapshot;
      },
    }),
    boardSnapshot: vi.fn().mockImplementation(async () => current),
  };
}
