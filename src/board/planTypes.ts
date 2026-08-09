import type { AgentEffort, AgentModelPreference } from "./agentSettingsTypes";
import type { Dependency, WorkItemBudget } from "./types";

export type PlanWorkItemPreview = Readonly<{
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  requiresHumanReview: boolean;
  assignedAgentProfileName?: string;
  assignedAgentModel: AgentModelPreference;
  assignedAgentEffort: AgentEffort;
}>;

export type PlanBudgetSummary = Readonly<{
  maxAgentTurns?: number;
  maxDurationSeconds?: number;
  maxCostMicros?: number;
  workItemsMissingAgentTurnBudget: readonly string[];
  workItemsMissingDurationBudget: readonly string[];
  workItemsMissingCostBudget: readonly string[];
}>;

export type PlanPreview = Readonly<{
  id: string;
  projectId: string;
  workItems: readonly PlanWorkItemPreview[];
  dependencies: readonly Dependency[];
  criticalPath: readonly string[];
  parallelStages: readonly (readonly string[])[];
  budget: PlanBudgetSummary;
  unresolvedAssumptions: readonly string[];
}>;

export type PlanConfirmation = Readonly<{
  planId: string;
  confirmedBy: string;
  confirmedAt: string;
}>;

export type BoardPlan = Readonly<{
  preview: PlanPreview;
  confirmation?: PlanConfirmation;
}>;

export type ProposedPlanWorkItemRequest = Readonly<{
  workItemId: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  requiresHumanReview: boolean;
  assignedAgentProfileName?: string;
  assignedAgentModel?: AgentModelPreference;
  assignedAgentEffort?: AgentEffort;
}>;

export type ProposedPlanDependencyRequest = Readonly<{
  dependencyId: string;
  upstreamWorkItemId: string;
  downstreamWorkItemId: string;
  kind: Dependency["kind"];
  reason: string;
  owner: string;
  nextAction: string;
}>;

export type ProposePlanRequest = Readonly<{
  planId: string;
  boardId: string;
  proposedBy: string;
  proposedAt: string;
  workItems: readonly ProposedPlanWorkItemRequest[];
  dependencies: readonly ProposedPlanDependencyRequest[];
  unresolvedAssumptions: readonly string[];
}>;

export type ConfirmPlanRequest = Readonly<{
  boardId: string;
  planId: string;
  confirmedBy: string;
  confirmedAt: string;
}>;
