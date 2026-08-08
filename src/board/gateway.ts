import { invoke } from "@tauri-apps/api/core";
import type {
  AddDependencyRequest,
  AgentProfile,
  BoardGateway,
  BoardSnapshot,
  CreateBoardRequest,
  CreateProjectRequest,
  CreateWorkItemRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
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
  createWorkItem(request: CreateWorkItemRequest): Promise<BoardSnapshot> {
    return invoke("create_work_item", { request });
  },
  addDependency(request: AddDependencyRequest): Promise<BoardSnapshot> {
    return invoke("add_dependency", { request });
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
  startExecution(request: StartExecutionRequest): Promise<BoardSnapshot> {
    return invoke("start_execution", { request });
  },
  stopExecution(executionId: string): Promise<BoardSnapshot> {
    return invoke("stop_execution", { executionId });
  },
  recordReviewCheck(request: RecordReviewCheckRequest): Promise<BoardSnapshot> {
    return invoke("record_review_check", { request });
  },
  recordReviewDecision(
    request: RecordReviewDecisionRequest,
  ): Promise<BoardSnapshot> {
    return invoke("record_review_decision", { request });
  },
  beginLinearOAuth(
    configuration: LinearOAuthConfiguration,
  ): Promise<LinearConnectionStatus> {
    return invoke("begin_linear_oauth", { configuration });
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
  boardSnapshot(boardId: string): Promise<BoardSnapshot> {
    return invoke("board_snapshot", { boardId });
  },
};
