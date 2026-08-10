import { useEffect, useState } from "react";
import { ListPlusIcon, SparklesIcon } from "lucide-react";

import { Button } from "@/components/ui/button";

import { BoardCanvas } from "./BoardCanvas";
import { BoardAutomation } from "./BoardAutomation";
import { BoardManagement, SurfaceHeader } from "./BoardManagement";
import { BoardSettings } from "./BoardSettings";
import { BoardViewMenu, type MainBoardView } from "./BoardViewMenu";
import { DependencyView } from "./DependencyView";
import { PlanProposalPanel } from "./PlanProposalPanel";
import { TaskDetailView } from "./TaskDetailView";
import type { TicketEffectOperations } from "./ticketEffectOperations";
import { boardSummary } from "./presentation";
import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  BoardPlan,
  BoardSnapshot,
  BoardSupervision,
  BoardSupervisionMode,
  ConfirmPlanRequest,
  CreateWorkItemRequest,
  GeneratePlanRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  PlannerProfile,
  ProjectAgentSettings,
  ProposePlanRequest,
  QueueLinearCommentRequest,
  SupervisionDecision,
  SaveProjectAgentSettingsRequest,
  TransitionWorkItemRequest,
} from "./types";

type BoardSurface = MainBoardView | "plan" | "new-task" | "task-detail";

type BoardViewProps = Readonly<{
  busy: boolean;
  agentProfiles: readonly AgentProfile[];
  projectAgentSettings?: ProjectAgentSettings;
  boardSupervision?: BoardSupervision;
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
  onSaveProjectAgentSettings: (
    request: SaveProjectAgentSettingsRequest,
  ) => Promise<void>;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
  ticketEffects: TicketEffectOperations;
  supervisionDecisions: readonly SupervisionDecision[];
  onConfigureBoardSupervision: (mode: BoardSupervisionMode) => Promise<void>;
  onCoordinateBoard: (boardId: string) => Promise<void>;
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
  projectAgentSettings,
  boardSupervision,
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
  onSaveProjectAgentSettings,
  onSavePlannerProfile,
  ticketEffects,
  onCoordinateBoard,
  supervisionDecisions,
  onConfigureBoardSupervision,
  onStartExecution,
  onStopExecution,
  onLoadExecutionActivity,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onRecordCleanCodeReview,
  onTransition,
}: BoardViewProps) {
  const [surface, setSurface] = useState<BoardSurface>("workflow");
  const [selectedWorkItemId, setSelectedWorkItemId] = useState<string>();
  const [restoreBoardFocus, setRestoreBoardFocus] = useState(false);
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const selectedWorkItem = workItems.find(
    ({ id }) => id === selectedWorkItemId,
  );
  const returnToWorkflow = () => {
    setRestoreBoardFocus(true);
    setSurface("workflow");
  };
  const activeView: MainBoardView =
    surface === "dependencies" || surface === "settings" ? surface : "workflow";
  const generatePlanFromWorkflow = async (request: GeneratePlanRequest) => {
    await onGeneratePlan(request);
    setSurface("plan");
  };
  const openTask = (workItemId: string) => {
    setSelectedWorkItemId(workItemId);
    setSurface("task-detail");
  };
  const openDependencies = (workItemId?: string) => {
    if (workItemId !== undefined) setSelectedWorkItemId(workItemId);
    setSurface("dependencies");
  };

  useEffect(() => {
    if (!restoreBoardFocus || surface !== "workflow") return;
    document.getElementById("board-view-menu")?.focus();
    setRestoreBoardFocus(false);
  }, [restoreBoardFocus, surface]);

  return (
    <section aria-labelledby="board-title" className="board-workspace">
      <BoardHeader
        boardName={snapshot.board.name}
        activeView={activeView}
        snapshot={snapshot}
        showQuickActions={surface === "workflow"}
        onCreateTask={() => setSurface("new-task")}
        onPlanWork={() => setSurface("plan")}
        onViewChange={(view) => setSurface(view)}
      />
      {surface === "workflow" && (
        <>
          <BoardAutomation
            snapshot={snapshot}
            supervision={boardSupervision}
            decisions={supervisionDecisions}
            hasConfiguredRoles={
              projectAgentSettings?.organiser !== undefined &&
              projectAgentSettings.ticketWorker !== undefined &&
              agentProfiles.some(
                ({ name }) =>
                  name === projectAgentSettings.ticketWorker?.agentProfileName,
              )
            }
            onConfigure={onConfigureBoardSupervision}
            onCoordinate={onCoordinateBoard}
          />
          <BoardCanvas
            busy={busy}
            defaultPlannerProfileName={
              projectAgentSettings?.organiser?.plannerProfileName
            }
            plannerProfiles={plannerProfiles}
            snapshot={snapshot}
            onGeneratePlan={generatePlanFromWorkflow}
            onExplainDependencies={openDependencies}
            onOpenTask={openTask}
          />
        </>
      )}
      {surface === "plan" && (
        <section aria-label="Plan with AI" className="workspace-surface">
          <SurfaceHeader
            description="Describe the outcome, review the proposed tasks, then decide what to create."
            onBack={returnToWorkflow}
            title="Plan with AI"
          />
          <PlanProposalPanel
            boardId={snapshot.board.id}
            busy={busy}
            plan={boardPlan}
            onConfirm={onConfirmPlan}
            onGenerate={onGeneratePlan}
            onPropose={onProposePlan}
            plannerProfiles={plannerProfiles}
            agentProfiles={agentProfiles}
            defaultPlannerProfileName={
              projectAgentSettings?.organiser?.plannerProfileName
            }
            defaultTicketWorkerProfileName={
              projectAgentSettings?.ticketWorker?.agentProfileName
            }
          />
        </section>
      )}
      {surface === "new-task" && (
        <BoardManagement
          boardId={snapshot.board.id}
          busy={busy}
          onBack={returnToWorkflow}
          onCreateWorkItem={onCreateWorkItem}
        />
      )}
      {surface === "dependencies" && (
        <DependencyView
          boardPlan={boardPlan}
          busy={busy}
          selectedWorkItemId={selectedWorkItemId}
          snapshot={snapshot}
          onAddDependency={onAddDependency}
          onBack={returnToWorkflow}
          onOpenTask={openTask}
        />
      )}
      {surface === "settings" && (
        <BoardSettings
          agentProfiles={agentProfiles}
          busy={busy}
          projectAgentSettings={projectAgentSettings}
          linearConnectionStatus={linearConnectionStatus}
          linearIssues={linearIssues}
          plannerProfiles={plannerProfiles}
          providerAvailability={providerAvailability}
          snapshot={snapshot}
          onBack={returnToWorkflow}
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
          onSaveProjectAgentSettings={onSaveProjectAgentSettings}
        />
      )}
      {surface === "task-detail" && selectedWorkItem !== undefined && (
        <TaskDetailView
          agentProfiles={agentProfiles}
          busy={busy}
          defaultAgentProfileName={
            selectedWorkItem.assignedAgentProfileName ??
            projectAgentSettings?.ticketWorker?.agentProfileName
          }
          hasOrganiser={projectAgentSettings?.organiser !== undefined}
          snapshot={snapshot}
          ticketEffects={ticketEffects}
          workItem={selectedWorkItem}
          onBack={returnToWorkflow}
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
  activeView,
  boardName,
  showQuickActions,
  snapshot,
  onCreateTask,
  onPlanWork,
  onViewChange,
}: Readonly<{
  activeView: MainBoardView;
  boardName: string;
  showQuickActions: boolean;
  snapshot: BoardSnapshot;
  onCreateTask: () => void;
  onPlanWork: () => void;
  onViewChange: (view: MainBoardView) => void;
}>) {
  const summary = boardSummary(snapshot);
  return (
    <header className="board-header">
      <div className="board-heading">
        <BoardViewMenu activeView={activeView} onViewChange={onViewChange} />
        <div>
          <p className="eyebrow">Your board</p>
          <h2 id="board-title">{boardName}</h2>
          <p>{summary}</p>
        </div>
      </div>
      {showQuickActions && (
        <div className="board-toolbar">
          <Button onClick={onPlanWork} type="button">
            <SparklesIcon data-icon="inline-start" />
            Plan with AI
          </Button>
          <Button onClick={onCreateTask} type="button" variant="outline">
            <ListPlusIcon data-icon="inline-start" />
            Create task
          </Button>
        </div>
      )}
    </header>
  );
}
