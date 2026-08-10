import type {
  AgentEffort,
  AgentModelPreference,
  NativeAgentProviderKind,
} from "./agentSettingsTypes";

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

export type BoardAttentionSummary = Readonly<{
  activeWorkItemCount: number;
  needsAttentionCount: number;
}>;

export type BoardLibraryEntry = Readonly<{
  boardId: string;
  name: string;
  repositoryName: string;
  repositoryAvailable: boolean;
  lastOpenedAt: string | null;
  attention: BoardAttentionSummary;
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
  assignedAgentProfileName?: string;
  assignedAgentModel: AgentModelPreference;
  assignedAgentEffort: AgentEffort;
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

export type RepositorySetup = Readonly<{
  repositoryPath: string;
  suggestedBoardName: string;
  baseRef: string;
}>;

export type CreateLocalBoardRequest = Readonly<{
  name: string;
  repositoryPath: string;
  baseRef?: string;
  policySetId?: string;
}>;

export type CloneGitHubRepositoryRequest = Readonly<{
  repositoryUrl: string;
  destinationParentPath: string;
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

export type AgentProviderAvailability = Readonly<{
  kind: NativeAgentProviderKind;
  label: string;
  program: string;
  installed: boolean;
}>;

export type PlannerProfile = Readonly<{
  name: string;
  kind: AgentProfileKind;
  program: string;
  arguments: readonly string[];
}>;

export type * from "./agentSettingsTypes";
export type * from "./planTypes";

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

export type BoardSupervisionMode = "manual" | "autonomous";

export type SupervisionAction =
  | "prepare_work"
  | "make_work_ready"
  | "start_work"
  | "retry_work"
  | "return_for_correction";

export type SupervisionPolicyResult = "not_required" | "allowed" | "denied";

export type SupervisionDecisionOutcome =
  | "pending"
  | "executed"
  | "recommended_for_approval"
  | "denied"
  | "stale"
  | "paused"
  | "recovered";

export type BoardSupervision = Readonly<{
  boardId: string;
  mode: BoardSupervisionMode;
  organiser: OrganiserDefaults;
  ticketWorker: TicketWorkerDefaults;
  limits: Readonly<{
    maxParallelWorkItems: number;
    maxRetriesPerWorkItem: number;
  }>;
  permittedActions: readonly SupervisionAction[];
  configuredBy: string;
  configuredAt: string;
  pausedBy?: string;
  pausedAt?: string;
  revision: number;
}>;

export type SupervisionDecision = Readonly<{
  id: string;
  boardId: string;
  workItemId?: string;
  organiserProfileName: string;
  action: SupervisionAction;
  recommendation: string;
  rationale: string;
  policyResult: SupervisionPolicyResult;
  outcome: SupervisionDecisionOutcome;
  recordedAt: string;
  resolvedAt?: string;
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
export type {
  ResolveTicketEffectRequest,
  TicketEffect,
  TicketEffectAction,
  TicketEffectOutcome,
  TicketEffectPromptRequest,
  TicketEffectProposal,
  TicketEffectResolution,
} from "./ticketEffectTypes";
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
