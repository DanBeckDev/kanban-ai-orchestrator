export type WorkItemState =
  | "inbox"
  | "planned"
  | "ready"
  | "running"
  | "awaiting_input"
  | "review"
  | "done"
  | "blocked"
  | "failed"
  | "cancelled"
  | "interrupted";

export type DependencyKind = "blocks" | "review_required" | "contract" | "soft";

export type WorkItemBudget = Readonly<{
  maxAgentTurns?: number;
  maxDurationSeconds?: number;
  maxCostMicros?: number;
}>;

export type CompletionEvidence = Readonly<{
  qualityGatePassed: boolean;
  completionReportPresent: boolean;
  reviewAccepted: boolean;
}>;

export type Board = Readonly<{
  id: string;
  projectId: string;
  name: string;
}>;

export type WorkItem = Readonly<{
  id: string;
  boardId: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  state: WorkItemState;
  requiresHumanReview: boolean;
}>;

export type MaterializedWorkItem = Readonly<{
  workItem: WorkItem;
  lastEventSequence: number;
}>;

export type Dependency = Readonly<{
  id: string;
  upstreamWorkItemId: string;
  downstreamWorkItemId: string;
  kind: DependencyKind;
  reason: string;
  owner: string;
  nextAction: string;
}>;

export type BoardActivity = Readonly<{
  workItemId: string;
  sequence: number;
  recordedAt: string;
  summary: string;
  completionEvidence?: CompletionEvidence;
}>;

export type ExecutionStatus =
  | "pending"
  | "running"
  | "awaiting_input"
  | "awaiting_review"
  | "completed"
  | "failed"
  | "interrupted"
  | "cancelled";

export type ExecutionRole = "implementation" | "independent_review";

export type Execution = Readonly<{
  id: string;
  workItemId: string;
  role: ExecutionRole;
  adapterName: string;
  status: ExecutionStatus;
  sessionId?: string;
  workspacePath: string;
  usage: Readonly<{
    inputTokens: number;
    outputTokens: number;
    costMicros?: number;
  }>;
  lastEventSequence: number;
}>;

export type ExecutionActivityKind =
  | "activity"
  | "approval_requested"
  | "awaiting_input"
  | "awaiting_review"
  | "completed"
  | "failed"
  | "interrupted";

export type ExecutionActivityChunk = Readonly<{
  sequence: number;
  kind: ExecutionActivityKind;
  summary: string;
  recordedAt: string;
}>;

export type ExecutionActivityPage = Readonly<{
  chunks: readonly ExecutionActivityChunk[];
  hasMore: boolean;
}>;

export type EvidenceKind =
  | "agent_report"
  | "check"
  | "quality_gate"
  | "diff"
  | "commit"
  | "pull_request"
  | "completion_report"
  | "clean_code_review"
  | "review_decision";

export type EvidenceResult = "recorded" | "passed" | "failed";

export type ExternalConnectionMode = "read_only" | "linked_execution";

export type LinearOAuthConfiguration = Readonly<{
  clientId: string;
  redirectUri: string;
}>;

export type LinearConnectionStatus =
  | Readonly<{ kind: "disconnected" }>
  | Readonly<{ kind: "awaiting_authorization" }>
  | Readonly<{
      kind: "connected";
      expiresAt: string;
      scopes: readonly string[];
    }>
  | Readonly<{ kind: "failed"; message: string }>;

export type LinearIssueSummary = Readonly<{
  id: string;
  identifier: string;
  title: string;
  url: string;
}>;

export type ExternalLink = Readonly<{
  id: string;
  workItemId: string;
  connectorId: string;
  provenance: "imported" | "user_linked" | "synchronized";
  externalId: string;
  displayIdentifier: string;
  url: string;
  connectionMode: ExternalConnectionMode;
}>;

export type Evidence = Readonly<{
  id: string;
  workItemId: string;
  executionId?: string;
  kind: EvidenceKind;
  result: EvidenceResult;
  summary: string;
  recordedAt: string;
}>;

export type BoardSnapshot = Readonly<{
  board: Board;
  workItems: readonly MaterializedWorkItem[];
  dependencies: readonly Dependency[];
  activity: readonly BoardActivity[];
  executions: readonly Execution[];
  evidence: readonly Evidence[];
  externalLinks: readonly ExternalLink[];
  connectorOutboxItems: readonly ConnectorOutboxItem[];
  connectorReconciliationItems: readonly ConnectorReconciliationItem[];
}>;

export type PlanWorkItemPreview = Readonly<{
  id: string;
  title: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
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

export type CreateProjectRequest = Readonly<{
  projectId: string;
  name: string;
  repositoryPath: string;
  baseRef: string;
  policySetId: string;
}>;

export type CreateBoardRequest = Readonly<{
  boardId: string;
  projectId: string;
  name: string;
}>;

export type CreateWorkItemRequest = Readonly<{
  eventId: string;
  workItemId: string;
  boardId: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  requiresHumanReview: boolean;
  recordedAt: string;
}>;

export type AddDependencyRequest = Readonly<{
  dependencyId: string;
  upstreamWorkItemId: string;
  downstreamWorkItemId: string;
  kind: DependencyKind;
  reason: string;
  owner: string;
  nextAction: string;
  createdBy: string;
  createdAt: string;
}>;

export type ProposedPlanWorkItemRequest = Readonly<{
  workItemId: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  requiresHumanReview: boolean;
}>;

export type ProposedPlanDependencyRequest = Readonly<{
  dependencyId: string;
  upstreamWorkItemId: string;
  downstreamWorkItemId: string;
  kind: DependencyKind;
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

export type TransitionWorkItemRequest = Readonly<{
  eventId: string;
  workItemId: string;
  nextState: WorkItemState;
  evidence?: CompletionEvidence;
  reason: string;
  recordedAt: string;
}>;

export type AgentProfileKind =
  | "structured_process"
  | "codex_cli"
  | "claude_code"
  | "cline_pass_cli";

export type AgentProfile = Readonly<{
  name: string;
  kind: AgentProfileKind;
  program: string;
  arguments: readonly string[];
}>;

export type PlannerProfile = Readonly<{
  name: string;
  program: string;
  arguments: readonly string[];
}>;

export type GeneratePlanRequest = Readonly<{
  boardId: string;
  plannerProfileName: string;
  goal: string;
}>;

export type StartExecutionRequest = Readonly<{
  executionId: string;
  workItemId: string;
  agentProfileName: string;
  taskBrief: string;
  executionRole: ExecutionRole;
}>;

export type RecordReviewCheckRequest = Readonly<{
  evidenceId: string;
  workItemId: string;
  summary: string;
  passed: boolean;
  recordedAt: string;
}>;

export type RecordReviewDecisionRequest = Readonly<{
  evidenceId: string;
  workItemId: string;
  reviewer: string;
  summary: string;
  accepted: boolean;
  recordedAt: string;
}>;

export type RecordCleanCodeReviewRequest = Readonly<{
  evidenceId: string;
  workItemId: string;
  reviewExecutionId: string;
  actionableFindingCount: number;
  summary: string;
  recordedAt: string;
}>;

export type ImportLinearIssueRequest = Readonly<{
  externalLinkId: string;
  workItemId: string;
  issueId: string;
  displayIdentifier: string;
  url: string;
  connectionMode: ExternalConnectionMode;
}>;

export type ImportLinearBlockerRequest = Readonly<{
  dependencyId: string;
  upstreamIssueId: string;
  downstreamIssueId: string;
  reason: string;
  owner: string;
  nextAction: string;
  createdAt: string;
}>;

export type { BoardGateway } from "./boardGateway";
import type {
  ConnectorOutboxItem,
  ConnectorReconciliationItem,
} from "./linearSyncTypes";

export type {
  ConnectorOutboxItem,
  ConnectorOutboxOperation,
  ConnectorOutboxState,
  ConnectorReconciliationItem,
  ConnectorReconciliationState,
  ConnectorSharedField,
  ObserveLinearSharedFieldRequest,
  QueueLinearCommentRequest,
} from "./linearSyncTypes";
