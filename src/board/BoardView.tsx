import { useState } from "react";
import { ListPlusIcon, Settings2Icon, SparklesIcon } from "lucide-react";

import { Button } from "@/components/ui/button";

import { BoardCanvas } from "./BoardCanvas";
import { BoardManagement, SurfaceHeader } from "./BoardManagement";
import { BoardSettings } from "./BoardSettings";
import { PlanProposalPanel } from "./PlanProposalPanel";
import { WorkItemCard } from "./WorkItemCard";
import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  BoardPlan,
  BoardSnapshot,
  ConfirmPlanRequest,
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
  QueueLinearCommentRequest,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
  WorkItem,
} from "./types";

type BoardSurface =
  | "board"
  | "plan"
  | "new-task"
  | "dependencies"
  | "settings"
  | "task-detail";

type BoardViewProps = Readonly<{
  busy: boolean;
  agentProfiles: readonly AgentProfile[];
  defaultAgentProfileName?: string;
  providerAvailability: readonly AgentProviderAvailability[];
  plannerProfiles: readonly PlannerProfile[];
  boardPlan?: BoardPlan;
  snapshot: BoardSnapshot;
  onAddDependency: (request: AddDependencyRequest) => Promise<void>;
  onConfirmPlan: (request: ConfirmPlanRequest) => Promise<void>;
  onGeneratePlan: (request: GeneratePlanRequest) => Promise<void>;
  onCreateWorkItem: (request: CreateWorkItemRequest) => Promise<void>;
  onImportLinearBlocker: (request: ImportLinearBlockerRequest) => Promise<void>;
  onImportLinearIssue: (request: ImportLinearIssueRequest) => Promise<void>;
  linearConnectionStatus: LinearConnectionStatus;
  linearIssues: readonly LinearIssueSummary[];
  onConnectLinear: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onEnableLinearCommentAccess: () => Promise<void>;
  onQueueLinearComment: (request: QueueLinearCommentRequest) => Promise<void>;
  onDeliverLinearComment: (outboxItemId: string) => Promise<void>;
  onRefreshLinearSharedFields: (externalLinkId: string) => Promise<void>;
  onLoadLinearIssues: () => Promise<void>;
  onProposePlan: (request: ProposePlanRequest) => Promise<void>;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<boolean>;
  onSelectDefaultAgentProfile: (profileName: string) => void;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
  onStartExecution: (request: StartExecutionRequest) => Promise<void>;
  onStopExecution: (executionId: string) => Promise<void>;
  onLoadExecutionActivity: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
  onRecordReviewCheck: (request: RecordReviewCheckRequest) => Promise<void>;
  onRecordReviewDecision: (
    request: RecordReviewDecisionRequest,
  ) => Promise<void>;
  onRecordCleanCodeReview: (
    request: RecordCleanCodeReviewRequest,
  ) => Promise<void>;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>;

export function BoardView({
  busy,
  agentProfiles,
  defaultAgentProfileName,
  providerAvailability,
  plannerProfiles,
  boardPlan,
  snapshot,
  onAddDependency,
  onConfirmPlan,
  onGeneratePlan,
  onCreateWorkItem,
  onImportLinearBlocker,
  onImportLinearIssue,
  linearConnectionStatus,
  linearIssues,
  onConnectLinear,
  onEnableLinearCommentAccess,
  onQueueLinearComment,
  onDeliverLinearComment,
  onRefreshLinearSharedFields,
  onLoadLinearIssues,
  onProposePlan,
  onSaveAgentProfile,
  onSelectDefaultAgentProfile,
  onSavePlannerProfile,
  onStartExecution,
  onStopExecution,
  onLoadExecutionActivity,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onRecordCleanCodeReview,
  onTransition,
}: BoardViewProps) {
  const [surface, setSurface] = useState<BoardSurface>("board");
  const [selectedWorkItemId, setSelectedWorkItemId] = useState<string>();
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const selectedWorkItem = workItems.find(
    ({ id }) => id === selectedWorkItemId,
  );
  const returnToBoard = () => setSurface("board");
  const openTask = (workItemId: string) => {
    setSelectedWorkItemId(workItemId);
    setSurface("task-detail");
  };

  return (
    <section aria-labelledby="board-title" className="board-workspace">
      <BoardHeader
        boardName={snapshot.board.name}
        snapshot={snapshot}
        onCreateTask={() => setSurface("new-task")}
        onOpenSettings={() => setSurface("settings")}
        onPlanWork={() => setSurface("plan")}
      />
      {surface === "board" && (
        <BoardCanvas
          snapshot={snapshot}
          onCreateTask={() => setSurface("new-task")}
          onOpenTask={openTask}
          onPlanWork={() => setSurface("plan")}
        />
      )}
      {surface === "plan" && (
        <section aria-label="Plan work" className="workspace-surface">
          <SurfaceHeader
            description="Describe the outcome, inspect the proposal, then confirm the work you want to create."
            onBack={returnToBoard}
            title="Plan work"
          />
          <PlanProposalPanel
            boardId={snapshot.board.id}
            busy={busy}
            plan={boardPlan}
            onConfirm={onConfirmPlan}
            onGenerate={onGeneratePlan}
            onPropose={onProposePlan}
            plannerProfiles={plannerProfiles}
          />
        </section>
      )}
      {(surface === "new-task" || surface === "dependencies") && (
        <BoardManagement
          boardId={snapshot.board.id}
          busy={busy}
          defaultTab={surface === "new-task" ? "task" : "dependencies"}
          workItems={workItems}
          onAddDependency={onAddDependency}
          onBack={returnToBoard}
          onCreateWorkItem={onCreateWorkItem}
        />
      )}
      {surface === "settings" && (
        <BoardSettings
          agentProfiles={agentProfiles}
          busy={busy}
          defaultAgentProfileName={defaultAgentProfileName}
          linearConnectionStatus={linearConnectionStatus}
          linearIssues={linearIssues}
          plannerProfiles={plannerProfiles}
          providerAvailability={providerAvailability}
          snapshot={snapshot}
          onBack={returnToBoard}
          onConnectLinear={onConnectLinear}
          onDeliverLinearComment={onDeliverLinearComment}
          onEnableLinearCommentAccess={onEnableLinearCommentAccess}
          onImportLinearBlocker={onImportLinearBlocker}
          onImportLinearIssue={onImportLinearIssue}
          onLoadLinearIssues={onLoadLinearIssues}
          onQueueLinearComment={onQueueLinearComment}
          onRefreshLinearSharedFields={onRefreshLinearSharedFields}
          onSaveAgentProfile={onSaveAgentProfile}
          onSavePlannerProfile={onSavePlannerProfile}
          onSelectDefaultAgentProfile={onSelectDefaultAgentProfile}
        />
      )}
      {surface === "task-detail" && selectedWorkItem !== undefined && (
        <TaskDetail
          agentProfiles={agentProfiles}
          busy={busy}
          defaultAgentProfileName={defaultAgentProfileName}
          snapshot={snapshot}
          workItem={selectedWorkItem}
          onBack={returnToBoard}
          onLoadExecutionActivity={onLoadExecutionActivity}
          onRecordCleanCodeReview={onRecordCleanCodeReview}
          onRecordReviewCheck={onRecordReviewCheck}
          onRecordReviewDecision={onRecordReviewDecision}
          onStartExecution={onStartExecution}
          onStopExecution={onStopExecution}
          onTransition={onTransition}
        />
      )}
    </section>
  );
}

function BoardHeader({
  boardName,
  snapshot,
  onCreateTask,
  onOpenSettings,
  onPlanWork,
}: Readonly<{
  boardName: string;
  snapshot: BoardSnapshot;
  onCreateTask: () => void;
  onOpenSettings: () => void;
  onPlanWork: () => void;
}>) {
  const summary = boardSummary(snapshot);
  return (
    <header className="board-header">
      <div>
        <p className="eyebrow">Your board</p>
        <h2 id="board-title">{boardName}</h2>
        <p>{summary}</p>
      </div>
      <div className="board-toolbar">
        <Button onClick={onPlanWork} type="button">
          <SparklesIcon data-icon="inline-start" />
          Plan work
        </Button>
        <Button onClick={onCreateTask} type="button" variant="outline">
          <ListPlusIcon data-icon="inline-start" />
          New task
        </Button>
        <Button onClick={onOpenSettings} type="button" variant="ghost">
          <Settings2Icon data-icon="inline-start" />
          Settings
        </Button>
      </div>
    </header>
  );
}

function TaskDetail({
  agentProfiles,
  busy,
  defaultAgentProfileName,
  snapshot,
  workItem,
  onBack,
  onLoadExecutionActivity,
  onRecordCleanCodeReview,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onStartExecution,
  onStopExecution,
  onTransition,
}: Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  defaultAgentProfileName?: string;
  snapshot: BoardSnapshot;
  workItem: WorkItem;
  onBack: () => void;
  onLoadExecutionActivity: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
  onRecordCleanCodeReview: (
    request: RecordCleanCodeReviewRequest,
  ) => Promise<void>;
  onRecordReviewCheck: (request: RecordReviewCheckRequest) => Promise<void>;
  onRecordReviewDecision: (
    request: RecordReviewDecisionRequest,
  ) => Promise<void>;
  onStartExecution: (request: StartExecutionRequest) => Promise<void>;
  onStopExecution: (executionId: string) => Promise<void>;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>) {
  return (
    <section
      aria-label={`Task details for ${workItem.title}`}
      className="workspace-surface"
    >
      <SurfaceHeader
        description="Review the task's current decision, then act with the right context."
        onBack={onBack}
        title={workItem.title}
      />
      <WorkItemCard
        agentProfiles={agentProfiles}
        busy={busy}
        defaultAgentProfileName={defaultAgentProfileName}
        snapshot={snapshot}
        workItem={workItem}
        onLoadExecutionActivity={onLoadExecutionActivity}
        onRecordCleanCodeReview={onRecordCleanCodeReview}
        onRecordReviewCheck={onRecordReviewCheck}
        onRecordReviewDecision={onRecordReviewDecision}
        onStartExecution={onStartExecution}
        onStopExecution={onStopExecution}
        onTransition={onTransition}
      />
    </section>
  );
}

function boardSummary(snapshot: BoardSnapshot): string {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const activeCount = workItems.filter(
    ({ state }) => state === "running",
  ).length;
  const attentionCount = workItems.filter(({ state }) =>
    ["awaiting_input", "review", "blocked", "failed", "interrupted"].includes(
      state,
    ),
  ).length;
  if (attentionCount > 0) {
    return `${activeCount} active · ${attentionCount} need your attention`;
  }
  return activeCount > 0
    ? `${activeCount} active · everything else is moving normally`
    : `${workItems.length} tasks · nothing needs your attention`;
}
