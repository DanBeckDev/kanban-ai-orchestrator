import { invoke } from "@tauri-apps/api/core";
import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  NativeAgentProviderKind,
  BoardLibraryEntry,
  BoardGateway,
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
  ObserveLinearSharedFieldRequest,
  ProposePlanRequest,
  PlannerProfile,
  ProviderModelCatalog,
  ProjectAgentSettings,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  RepositorySetup,
  QueueLinearCommentRequest,
  SaveProjectAgentSettingsRequest,
  StartExecutionRequest,
  SupervisionDecision,
  TransitionWorkItemRequest,
} from "./types";
import type {
  ResolveTicketEffectRequest,
  TicketEffect,
  TicketEffectPromptRequest,
} from "./ticketEffectTypes";

export const tauriBoardGateway: BoardGateway = {
  async createProject(request: CreateProjectRequest): Promise<void> {
    await invoke("create_project", { request });
  },
  createBoard(request: CreateBoardRequest): Promise<BoardSnapshot> {
    return invoke("create_board", { request });
  },
  inspectRepository(repositoryPath: string): Promise<RepositorySetup> {
    return invoke("inspect_repository", { repositoryPath });
  },
  cloneGitHubRepository(
    request: CloneGitHubRepositoryRequest,
  ): Promise<RepositorySetup> {
    return invoke("clone_github_repository", { request });
  },
  createLocalBoard(request: CreateLocalBoardRequest): Promise<BoardSnapshot> {
    return invoke("create_local_board", { request });
  },
  boardLibrary(): Promise<readonly BoardLibraryEntry[]> {
    return invoke("board_library");
  },
  openBoard(boardId: string): Promise<BoardSnapshot> {
    return invoke("open_board", { boardId });
  },
  createWorkItem(request: CreateWorkItemRequest): Promise<BoardSnapshot> {
    return invoke("create_work_item", { request });
  },
  addDependency(request: AddDependencyRequest): Promise<BoardSnapshot> {
    return invoke("add_dependency", { request });
  },
  proposePlan(request: ProposePlanRequest): Promise<BoardPlan> {
    return invoke("propose_plan", { request });
  },
  async boardPlan(boardId: string): Promise<BoardPlan | undefined> {
    const plan = await invoke<BoardPlan | null>("board_plan", { boardId });
    return plan ?? undefined;
  },
  confirmPlan(request: ConfirmPlanRequest): Promise<BoardSnapshot> {
    return invoke("confirm_plan", { request });
  },
  transitionWorkItem(
    request: TransitionWorkItemRequest,
  ): Promise<BoardSnapshot> {
    return invoke("transition_work_item", { request });
  },
  saveAgentProfile(profile: AgentProfile): Promise<AgentProfile> {
    return invoke("save_agent_profile", { profile });
  },
  agentProfiles(): Promise<readonly AgentProfile[]> {
    return invoke("agent_profiles");
  },
  agentProviderAvailability(): Promise<readonly AgentProviderAvailability[]> {
    return invoke("agent_provider_availability");
  },
  providerModelCatalog(
    providerKind: NativeAgentProviderKind,
  ): Promise<ProviderModelCatalog> {
    return invoke("provider_model_catalog", { providerKind });
  },
  savePlannerProfile(profile: PlannerProfile): Promise<PlannerProfile> {
    return invoke("save_planner_profile", { profile });
  },
  plannerProfiles(): Promise<readonly PlannerProfile[]> {
    return invoke("planner_profiles");
  },
  saveProjectAgentSettings(
    request: SaveProjectAgentSettingsRequest,
  ): Promise<ProjectAgentSettings> {
    return invoke("save_project_agent_settings", { request });
  },
  async projectAgentSettings(
    boardId: string,
  ): Promise<ProjectAgentSettings | undefined> {
    const settings = await invoke<ProjectAgentSettings | null>(
      "project_agent_settings",
      { boardId },
    );
    return settings ?? undefined;
  },
  generatePlan(request: GeneratePlanRequest): Promise<BoardPlan> {
    return invoke("generate_plan", { request });
  },
  planningActivity(
    boardId: string,
    afterSequence?: number,
  ): Promise<ExecutionActivityPage> {
    return invoke("planning_activity", { boardId, afterSequence });
  },
  startExecution(request: StartExecutionRequest): Promise<BoardSnapshot> {
    return invoke("start_execution", { request });
  },
  configureBoardSupervision(
    boardId: string,
    mode: BoardSupervisionMode,
  ): Promise<BoardSupervision> {
    return invoke("configure_board_supervision", { boardId, mode });
  },
  async boardSupervision(
    boardId: string,
  ): Promise<BoardSupervision | undefined> {
    const supervision = await invoke<BoardSupervision | null>(
      "board_supervision",
      {
        boardId,
      },
    );
    return supervision ?? undefined;
  },
  supervisionDecisions(
    boardId: string,
  ): Promise<readonly SupervisionDecision[]> {
    return invoke("supervision_decisions", { boardId });
  },
  coordinateBoard(boardId: string): Promise<BoardSnapshot> {
    return invoke("coordinate_board", { boardId });
  },
  requestTicketEffect(
    request: TicketEffectPromptRequest,
  ): Promise<TicketEffect> {
    return invoke("request_ticket_effect", { request });
  },
  resolveTicketEffect(
    request: ResolveTicketEffectRequest,
  ): Promise<BoardSnapshot> {
    return invoke("resolve_ticket_effect", { request });
  },
  ticketEffects(workItemId: string): Promise<readonly TicketEffect[]> {
    return invoke("ticket_effects", { workItemId });
  },
  stopExecution(executionId: string): Promise<BoardSnapshot> {
    return invoke("stop_execution", { executionId });
  },
  executionActivity(
    executionId: string,
    afterSequence?: number,
  ): Promise<ExecutionActivityPage> {
    return invoke("execution_activity", { executionId, afterSequence });
  },
  recordReviewCheck(request: RecordReviewCheckRequest): Promise<BoardSnapshot> {
    return invoke("record_review_check", { request });
  },
  recordReviewDecision(
    request: RecordReviewDecisionRequest,
  ): Promise<BoardSnapshot> {
    return invoke("record_review_decision", { request });
  },
  recordCleanCodeReview(
    request: RecordCleanCodeReviewRequest,
  ): Promise<BoardSnapshot> {
    return invoke("record_clean_code_review", { request });
  },
  beginLinearOAuth(
    configuration: LinearOAuthConfiguration,
  ): Promise<LinearConnectionStatus> {
    return invoke("begin_linear_oauth", { configuration });
  },
  beginLinearCommentAccess(): Promise<LinearConnectionStatus> {
    return invoke("begin_linear_comment_access");
  },
  linearConnectionStatus(): Promise<LinearConnectionStatus> {
    return invoke("linear_connection_status");
  },
  linearAssignedIssues(): Promise<readonly LinearIssueSummary[]> {
    return invoke("linear_assigned_issues");
  },
  importLinearIssue(request: ImportLinearIssueRequest): Promise<BoardSnapshot> {
    return invoke("import_linear_issue", { request });
  },
  importLinearBlocker(
    request: ImportLinearBlockerRequest,
  ): Promise<BoardSnapshot> {
    return invoke("import_linear_blocker", { request });
  },
  queueLinearComment(
    request: QueueLinearCommentRequest,
  ): Promise<BoardSnapshot> {
    return invoke("queue_linear_comment", { request });
  },
  observeLinearSharedField(
    request: ObserveLinearSharedFieldRequest,
  ): Promise<BoardSnapshot> {
    return invoke("observe_linear_shared_field", { request });
  },
  syncLinearSharedFields(externalLinkId: string): Promise<BoardSnapshot> {
    return invoke("sync_linear_shared_fields", { externalLinkId });
  },
  deliverLinearComment(outboxItemId: string): Promise<BoardSnapshot> {
    return invoke("deliver_linear_comment", { outboxItemId });
  },
  boardSnapshot(boardId: string): Promise<BoardSnapshot> {
    return invoke("board_snapshot", { boardId });
  },
};
