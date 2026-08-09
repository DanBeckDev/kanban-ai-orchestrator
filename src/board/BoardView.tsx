import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

import { AgentProfileForm } from "./AgentProfileForm";
import { BoardSupportDetails } from "./BoardSupportDetails";
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
          <h2 id="board-title">{snapshot.board.name}</h2>
          <p>
            {workItems.length} tasks · {snapshot.dependencies.length}{" "}
            dependencies
          </p>
        </div>
        <div>
          <Badge variant="secondary">Board ready</Badge>
          <BoardSupportDetails board={snapshot.board} />
        </div>
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
        <aside className="board-actions" aria-label="Board controls">
          <Tabs defaultValue="plan">
            <TabsList aria-label="Board controls" variant="line">
              <TabsTrigger value="plan">Plan</TabsTrigger>
              <TabsTrigger value="organise">Organise</TabsTrigger>
              <TabsTrigger value="connections">Connections</TabsTrigger>
            </TabsList>
            <TabsContent value="plan">
              <PlanProposalPanel
                boardId={snapshot.board.id}
                busy={busy}
                plan={boardPlan}
                onConfirm={onConfirmPlan}
                onGenerate={onGeneratePlan}
                onPropose={onProposePlan}
                plannerProfiles={plannerProfiles}
              />
              <Separator />
              <PlannerProfileForm
                busy={busy}
                onSave={onSavePlannerProfile}
                profiles={plannerProfiles}
              />
            </TabsContent>
            <TabsContent value="organise">
              <TaskForm
                boardId={snapshot.board.id}
                busy={busy}
                onCreate={onCreateWorkItem}
              />
              <Separator />
              <DependencyForm
                busy={busy}
                onCreate={onAddDependency}
                workItems={workItems}
              />
              <Separator />
              <AgentProfileForm
                busy={busy}
                profiles={agentProfiles}
                onSave={onSaveAgentProfile}
              />
            </TabsContent>
            <TabsContent value="connections">
              <LinearConnectionPanel
                busy={busy}
                status={linearConnectionStatus}
                onConnect={onConnectLinear}
                onEnableCommentAccess={onEnableLinearCommentAccess}
              />
              <Separator />
              <LinearImportForm
                busy={busy}
                connectionStatus={linearConnectionStatus}
                issues={linearIssues}
                workItems={workItems}
                onImportBlocker={onImportLinearBlocker}
                onImportIssue={onImportLinearIssue}
                onLoadIssues={onLoadLinearIssues}
              />
              <Separator />
              <LinearSyncPanel
                busy={busy}
                snapshot={snapshot}
                onDeliver={onDeliverLinearComment}
                onQueue={onQueueLinearComment}
                onRefresh={onRefreshLinearSharedFields}
              />
            </TabsContent>
          </Tabs>
        </aside>
      </div>
    </section>
  );
}
