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
  checksPassed: boolean;
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

export type BoardSnapshot = Readonly<{
  board: Board;
  workItems: readonly MaterializedWorkItem[];
  dependencies: readonly Dependency[];
  activity: readonly BoardActivity[];
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

export type TransitionWorkItemRequest = Readonly<{
  eventId: string;
  workItemId: string;
  nextState: WorkItemState;
  evidence?: CompletionEvidence;
  reason: string;
  recordedAt: string;
}>;

export interface BoardGateway {
  createProject(request: CreateProjectRequest): Promise<void>;
  createBoard(request: CreateBoardRequest): Promise<BoardSnapshot>;
  createWorkItem(request: CreateWorkItemRequest): Promise<BoardSnapshot>;
  addDependency(request: AddDependencyRequest): Promise<BoardSnapshot>;
  transitionWorkItem(
    request: TransitionWorkItemRequest,
  ): Promise<BoardSnapshot>;
  boardSnapshot(boardId: string): Promise<BoardSnapshot>;
}
