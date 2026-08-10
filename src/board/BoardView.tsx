import { useEffect, useState } from "react";
import { ListPlusIcon } from "lucide-react";

import { Button } from "@/components/ui/button";

import { BoardCanvas } from "./BoardCanvas";
import { BoardHome } from "./BoardHome";
import { BoardManagement, SurfaceHeader } from "./BoardManagement";
import { BoardSettings } from "./BoardSettings";
import { BoardViewMenu, type MainBoardView } from "./BoardViewMenu";
import { DependencyView } from "./DependencyView";
import { PlanProposalPanel } from "./PlanProposalPanel";
import { TaskDetailView } from "./TaskDetailView";
import type { TicketEffectOperations } from "./ticketEffectOperations";
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
  ExecutionActivityPage,
  GeneratePlanRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  PlannerProfile,
  ProjectAgentSettings,
  ProviderModelCatalog,
  ProposePlanRequest,
  QueueLinearCommentRequest,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  SaveProjectAgentSettingsRequest,
  StartExecutionRequest,
  SupervisionDecision,
  TransitionWorkItemRequest,
} from "./types";

type BoardSurface = MainBoardView | "plan-review" | "new-task" | "task-detail";

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
  onLoadProviderCatalog: (
    provider: AgentProviderAvailability,
  ) => Promise<ProviderModelCatalog>;
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
  onLoadPlanningActivity: (
    boardId: string,
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
  onLoadProviderCatalog,
  onSaveProjectAgentSettings,
  onSavePlannerProfile,
  ticketEffects,
  onCoordinateBoard,
  supervisionDecisions,
  onConfigureBoardSupervision,
  onStartExecution,
  onStopExecution,
  onLoadExecutionActivity,
  onLoadPlanningActivity,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onRecordCleanCodeReview,
  onTransition,
}: BoardViewProps) {
  const [surface, setSurface] = useState<BoardSurface>("home");
  const [selectedWorkItemId, setSelectedWorkItemId] = useState<string>();
  const [restoreMenuFocus, setRestoreMenuFocus] = useState(false);
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const selectedWorkItem = workItems.find(
    ({ id }) => id === selectedWorkItemId,
  );
  const activeView = activeViewFor(surface);
  const returnToHome = () => returnTo("home");
  const returnToTickets = () => returnTo("tickets");

  function returnTo(nextSurface: MainBoardView) {
    setRestoreMenuFocus(true);
    setSurface(nextSurface);
  }

  async function generatePlanFromHome(request: GeneratePlanRequest) {
    await onGeneratePlan(request);
    setSurface("plan-review");
  }

  async function confirmPlanAndOpenTickets(request: ConfirmPlanRequest) {
    await onConfirmPlan(request);
    setSurface("tickets");
  }

  function openTask(workItemId: string) {
    setSelectedWorkItemId(workItemId);
    setSurface("task-detail");
  }

  function openDependencies(workItemId?: string) {
    if (workItemId !== undefined) setSelectedWorkItemId(workItemId);
    setSurface("dependencies");
  }

  useEffect(() => {
    if (!restoreMenuFocus || surface === "task-detail") return;
    document.getElementById("board-view-menu")?.focus();
    setRestoreMenuFocus(false);
  }, [restoreMenuFocus, surface]);

  return (
    <section aria-labelledby="board-title" className="board-workspace">
      <BoardHeader
        activeView={activeView}
        boardName={snapshot.board.name}
        onCreateTask={() => setSurface("new-task")}
        onViewChange={setSurface}
        showCreateTask={surface === "tickets"}
      />
      {surface === "home" && (
        <BoardHome
          busy={busy}
          defaultPlannerProfileName={
            projectAgentSettings?.organiser?.plannerProfileName
          }
          plannerProfiles={plannerProfiles}
          snapshot={snapshot}
          supervision={boardSupervision}
          onGeneratePlan={generatePlanFromHome}
          onLoadExecutionActivity={onLoadExecutionActivity}
          onLoadPlanningActivity={onLoadPlanningActivity}
          onOpenTask={openTask}
          onOpenPlanReview={() => setSurface("plan-review")}
          onOpenTickets={() => setSurface("tickets")}
        />
      )}
      {surface === "tickets" && (
        <BoardCanvas
          snapshot={snapshot}
          onExplainDependencies={openDependencies}
          onGoHome={() => setSurface("home")}
          onOpenTask={openTask}
        />
      )}
      {surface === "plan-review" && (
        <section
          aria-label="Review proposed tickets"
          className="workspace-surface"
        >
          <SurfaceHeader
            backLabel="Back to Home"
            description="Review the proposed tickets and their order before creating anything."
            onBack={returnToHome}
            title="Review proposed tickets"
          />
          <PlanProposalPanel
            agentProfiles={agentProfiles}
            boardId={snapshot.board.id}
            busy={busy}
            defaultPlannerProfileName={
              projectAgentSettings?.organiser?.plannerProfileName
            }
            defaultTicketWorkerProfileName={
              projectAgentSettings?.ticketWorker?.agentProfileName
            }
            onConfirm={confirmPlanAndOpenTickets}
            onGenerate={onGeneratePlan}
            onPropose={onProposePlan}
            plan={boardPlan}
            plannerProfiles={plannerProfiles}
          />
        </section>
      )}
      {surface === "new-task" && (
        <BoardManagement
          boardId={snapshot.board.id}
          busy={busy}
          onBack={returnToTickets}
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
          onBack={returnToTickets}
          onOpenTask={openTask}
        />
      )}
      {surface === "settings" && (
        <BoardSettings
          agentProfiles={agentProfiles}
          boardSupervision={boardSupervision}
          busy={busy}
          projectAgentSettings={projectAgentSettings}
          linearConnectionStatus={linearConnectionStatus}
          linearIssues={linearIssues}
          plannerProfiles={plannerProfiles}
          providerAvailability={providerAvailability}
          snapshot={snapshot}
          supervisionDecisions={supervisionDecisions}
          onBack={returnToTickets}
          onConfigureBoardSupervision={onConfigureBoardSupervision}
          onConnectLinear={onConnectLinear}
          onCoordinateBoard={onCoordinateBoard}
          onDeliverLinearComment={onDeliverLinearComment}
          onEnableLinearCommentAccess={onEnableLinearCommentAccess}
          onImportLinearBlocker={onImportLinearBlocker}
          onImportLinearIssue={onImportLinearIssue}
          onLoadLinearIssues={onLoadLinearIssues}
          onQueueLinearComment={onQueueLinearComment}
          onRefreshLinearSharedFields={onRefreshLinearSharedFields}
          onSaveAgentProfile={onSaveAgentProfile}
          onLoadProviderCatalog={onLoadProviderCatalog}
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
          onBack={returnToTickets}
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
  showCreateTask,
  onCreateTask,
  onViewChange,
}: Readonly<{
  activeView: MainBoardView;
  boardName: string;
  showCreateTask: boolean;
  onCreateTask: () => void;
  onViewChange: (view: MainBoardView) => void;
}>) {
  return (
    <header className="board-header">
      <div className="board-heading">
        <BoardViewMenu activeView={activeView} onViewChange={onViewChange} />
        <div>
          <p className="eyebrow">Project</p>
          <h2 id="board-title">{boardName}</h2>
        </div>
      </div>
      {showCreateTask && (
        <div className="board-toolbar">
          <Button onClick={onCreateTask} type="button" variant="outline">
            <ListPlusIcon data-icon="inline-start" />
            Create ticket
          </Button>
        </div>
      )}
    </header>
  );
}

function activeViewFor(surface: BoardSurface): MainBoardView {
  if (surface === "plan-review") return "home";
  if (surface === "new-task" || surface === "task-detail") return "tickets";
  return surface;
}
