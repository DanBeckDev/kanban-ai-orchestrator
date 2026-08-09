import type {
  AddDependencyRequest,
  AgentProfile,
  BoardLibraryEntry,
  BoardPlan,
  BoardSnapshot,
  ConfirmPlanRequest,
  CreateBoardRequest,
  CreateLocalBoardRequest,
  CreateProjectRequest,
  CreateWorkItemRequest,
  ExecutionActivityPage,
  GeneratePlanRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  PlannerProfile,
  ProposePlanRequest,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  RepositorySetup,
  StartExecutionRequest,
  TransitionWorkItemRequest,
} from "./types";
import type {
  ObserveLinearSharedFieldRequest,
  QueueLinearCommentRequest,
} from "./linearSyncTypes";

export interface BoardGateway {
  createProject(request: CreateProjectRequest): Promise<void>;
  createBoard(request: CreateBoardRequest): Promise<BoardSnapshot>;
  inspectRepository(repositoryPath: string): Promise<RepositorySetup>;
  createLocalBoard(request: CreateLocalBoardRequest): Promise<BoardSnapshot>;
  boardLibrary(): Promise<readonly BoardLibraryEntry[]>;
  openBoard(boardId: string): Promise<BoardSnapshot>;
  createWorkItem(request: CreateWorkItemRequest): Promise<BoardSnapshot>;
  addDependency(request: AddDependencyRequest): Promise<BoardSnapshot>;
  proposePlan(request: ProposePlanRequest): Promise<BoardPlan>;
  boardPlan(boardId: string): Promise<BoardPlan | undefined>;
  confirmPlan(request: ConfirmPlanRequest): Promise<BoardSnapshot>;
  transitionWorkItem(
    request: TransitionWorkItemRequest,
  ): Promise<BoardSnapshot>;
  saveAgentProfile(profile: AgentProfile): Promise<AgentProfile>;
  agentProfiles(): Promise<readonly AgentProfile[]>;
  savePlannerProfile(profile: PlannerProfile): Promise<PlannerProfile>;
  plannerProfiles(): Promise<readonly PlannerProfile[]>;
  generatePlan(request: GeneratePlanRequest): Promise<BoardPlan>;
  startExecution(request: StartExecutionRequest): Promise<BoardSnapshot>;
  stopExecution(executionId: string): Promise<BoardSnapshot>;
  executionActivity(
    executionId: string,
    afterSequence?: number,
  ): Promise<ExecutionActivityPage>;
  recordReviewCheck(request: RecordReviewCheckRequest): Promise<BoardSnapshot>;
  recordReviewDecision(
    request: RecordReviewDecisionRequest,
  ): Promise<BoardSnapshot>;
  recordCleanCodeReview(
    request: RecordCleanCodeReviewRequest,
  ): Promise<BoardSnapshot>;
  beginLinearOAuth(
    configuration: LinearOAuthConfiguration,
  ): Promise<LinearConnectionStatus>;
  beginLinearCommentAccess(): Promise<LinearConnectionStatus>;
  linearConnectionStatus(): Promise<LinearConnectionStatus>;
  linearAssignedIssues(): Promise<readonly LinearIssueSummary[]>;
  importLinearIssue(request: ImportLinearIssueRequest): Promise<BoardSnapshot>;
  importLinearBlocker(
    request: ImportLinearBlockerRequest,
  ): Promise<BoardSnapshot>;
  queueLinearComment(
    request: QueueLinearCommentRequest,
  ): Promise<BoardSnapshot>;
  observeLinearSharedField(
    request: ObserveLinearSharedFieldRequest,
  ): Promise<BoardSnapshot>;
  syncLinearSharedFields(externalLinkId: string): Promise<BoardSnapshot>;
  deliverLinearComment(outboxItemId: string): Promise<BoardSnapshot>;
  boardSnapshot(boardId: string): Promise<BoardSnapshot>;
}
