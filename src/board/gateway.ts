import { invoke } from "@tauri-apps/api/core";
import type {
  AddDependencyRequest,
  AgentProfile,
  BoardLibraryEntry,
  BoardGateway,
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
  ObserveLinearSharedFieldRequest,
  ProposePlanRequest,
  PlannerProfile,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  RepositorySetup,
  QueueLinearCommentRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
} from "./types";

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
  savePlannerProfile(profile: PlannerProfile): Promise<PlannerProfile> {
    return invoke("save_planner_profile", { profile });
  },
  plannerProfiles(): Promise<readonly PlannerProfile[]> {
    return invoke("planner_profiles");
  },
  generatePlan(request: GeneratePlanRequest): Promise<BoardPlan> {
    return invoke("generate_plan", { request });
  },
  startExecution(request: StartExecutionRequest): Promise<BoardSnapshot> {
    return invoke("start_execution", { request });
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
