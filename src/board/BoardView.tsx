import { AgentProfileForm } from "./AgentProfileForm";
import { boardColumns, workItemsForColumn } from "./presentation";
import { DependencyForm } from "./DependencyForm";
import { LinearConnectionPanel } from "./LinearConnectionPanel";
import { LinearImportForm } from "./LinearImportForm";
import { LinearSyncPanel } from "./LinearSyncPanel";
import { PlanProposalPanel } from "./PlanProposalPanel";
import { PlannerProfileForm } from "./PlannerProfileForm";
import { TaskForm } from "./TaskForm";
import { WorkItemCard } from "./WorkItemCard";
import type {
  AddDependencyRequest,
  AgentProfile,
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
  QueueLinearCommentRequest,
  ProposePlanRequest,
  PlannerProfile,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
} from "./types";

type BoardViewProps = Readonly<{
  busy: boolean;
  agentProfiles: readonly AgentProfile[];
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
  onSaveAgentProfile: (profile: AgentProfile) => Promise<void>;
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
  onSavePlannerProfile,
  onStartExecution,
  onStopExecution,
  onLoadExecutionActivity,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onRecordCleanCodeReview,
  onTransition,
}: BoardViewProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);

  return (
    <section aria-labelledby="board-title" className="board-workspace">
      <header className="board-header">
        <div>
          <p className="eyebrow">Project {snapshot.board.projectId}</p>
          <h2 id="board-title">{snapshot.board.name}</h2>
          <p>
            {workItems.length} tasks · {snapshot.dependencies.length}{" "}
            dependencies
          </p>
        </div>
        <span className="local-status">Local daemon connected</span>
      </header>
      <div className="board-layout">
        <section aria-label="Kanban board" className="kanban-board">
          {boardColumns.map((column) => (
            <section
              aria-labelledby={`${column.id}-column`}
              className="board-column"
              key={column.id}
            >
              <h3 id={`${column.id}-column`}>{column.label}</h3>
              <div className="card-stack">
                {workItemsForColumn(snapshot, column).map((workItem) => (
                  <WorkItemCard
                    busy={busy}
                    key={workItem.id}
                    snapshot={snapshot}
                    workItem={workItem}
                    onTransition={onTransition}
                    agentProfiles={agentProfiles}
                    onStartExecution={onStartExecution}
                    onStopExecution={onStopExecution}
                    onLoadExecutionActivity={onLoadExecutionActivity}
                    onRecordReviewCheck={onRecordReviewCheck}
                    onRecordReviewDecision={onRecordReviewDecision}
                    onRecordCleanCodeReview={onRecordCleanCodeReview}
                  />
                ))}
              </div>
            </section>
          ))}
        </section>
        <aside className="board-actions">
          <PlanProposalPanel
            boardId={snapshot.board.id}
            busy={busy}
            plan={boardPlan}
            onConfirm={onConfirmPlan}
            onGenerate={onGeneratePlan}
            onPropose={onProposePlan}
            plannerProfiles={plannerProfiles}
          />
          <TaskForm
            boardId={snapshot.board.id}
            busy={busy}
            onCreate={onCreateWorkItem}
          />
          <DependencyForm
            busy={busy}
            onCreate={onAddDependency}
            workItems={workItems}
          />
          <AgentProfileForm
            busy={busy}
            profiles={agentProfiles}
            onSave={onSaveAgentProfile}
          />
          <PlannerProfileForm
            busy={busy}
            onSave={onSavePlannerProfile}
            profiles={plannerProfiles}
          />
          <LinearConnectionPanel
            busy={busy}
            status={linearConnectionStatus}
            onConnect={onConnectLinear}
            onEnableCommentAccess={onEnableLinearCommentAccess}
          />
          <LinearImportForm
            busy={busy}
            connectionStatus={linearConnectionStatus}
            issues={linearIssues}
            workItems={workItems}
            onImportBlocker={onImportLinearBlocker}
            onImportIssue={onImportLinearIssue}
            onLoadIssues={onLoadLinearIssues}
          />
          <LinearSyncPanel
            busy={busy}
            snapshot={snapshot}
            onDeliver={onDeliverLinearComment}
            onQueue={onQueueLinearComment}
            onRefresh={onRefreshLinearSharedFields}
          />
        </aside>
      </div>
    </section>
  );
}
