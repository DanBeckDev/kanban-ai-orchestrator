import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  BoardLibraryEntry,
  BoardPlan,
  BoardSnapshot,
  BoardSupervision,
  BoardSupervisionMode,
  ConfirmPlanRequest,
  CreateBoardRequest,
  CloneGitHubRepositoryRequest,
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
  NativeAgentProviderKind,
  PlannerProfile,
  ProviderModelCatalog,
  ProjectAgentSettings,
  ProposePlanRequest,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  RepositorySetup,
  StartExecutionRequest,
  SupervisionDecision,
  SaveProjectAgentSettingsRequest,
  TransitionWorkItemRequest,
} from "./types";
import type {
  ResolveTicketEffectRequest,
  TicketEffect,
  TicketEffectPromptRequest,
} from "./ticketEffectTypes";
import type {
  ObserveLinearSharedFieldRequest,
  QueueLinearCommentRequest,
} from "./linearSyncTypes";

export interface BoardGateway {
  createProject(request: CreateProjectRequest): Promise<void>;
  createBoard(request: CreateBoardRequest): Promise<BoardSnapshot>;
  inspectRepository(repositoryPath: string): Promise<RepositorySetup>;
  cloneGitHubRepository(
    request: CloneGitHubRepositoryRequest,
  ): Promise<RepositorySetup>;
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
  agentProviderAvailability(): Promise<readonly AgentProviderAvailability[]>;
  providerModelCatalog(
    providerKind: NativeAgentProviderKind,
  ): Promise<ProviderModelCatalog>;
  savePlannerProfile(profile: PlannerProfile): Promise<PlannerProfile>;
  plannerProfiles(): Promise<readonly PlannerProfile[]>;
  saveProjectAgentSettings(
    request: SaveProjectAgentSettingsRequest,
  ): Promise<ProjectAgentSettings>;
  projectAgentSettings(
    boardId: string,
  ): Promise<ProjectAgentSettings | undefined>;
  generatePlan(request: GeneratePlanRequest): Promise<BoardPlan>;
  startExecution(request: StartExecutionRequest): Promise<BoardSnapshot>;
  configureBoardSupervision(
    boardId: string,
    mode: BoardSupervisionMode,
  ): Promise<BoardSupervision>;
  boardSupervision(boardId: string): Promise<BoardSupervision | undefined>;
  supervisionDecisions(
    boardId: string,
  ): Promise<readonly SupervisionDecision[]>;
  coordinateBoard(boardId: string): Promise<BoardSnapshot>;
  requestTicketEffect(
    request: TicketEffectPromptRequest,
  ): Promise<TicketEffect>;
  resolveTicketEffect(
    request: ResolveTicketEffectRequest,
  ): Promise<BoardSnapshot>;
  ticketEffects(workItemId: string): Promise<readonly TicketEffect[]>;
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
