import { AgentProfileForm } from "./AgentProfileForm";
import { boardColumns, workItemsForColumn } from "./presentation";
import { DependencyForm } from "./DependencyForm";
import { LinearConnectionPanel } from "./LinearConnectionPanel";
import { LinearImportForm } from "./LinearImportForm";
import { TaskForm } from "./TaskForm";
import { WorkItemCard } from "./WorkItemCard";
import type {
  AddDependencyRequest,
  AgentProfile,
  BoardSnapshot,
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

type BoardViewProps = Readonly<{
  busy: boolean;
  agentProfiles: readonly AgentProfile[];
  snapshot: BoardSnapshot;
  onAddDependency: (request: AddDependencyRequest) => Promise<void>;
  onCreateWorkItem: (request: CreateWorkItemRequest) => Promise<void>;
  onImportLinearBlocker: (request: ImportLinearBlockerRequest) => Promise<void>;
  onImportLinearIssue: (request: ImportLinearIssueRequest) => Promise<void>;
  linearConnectionStatus: LinearConnectionStatus;
  linearIssues: readonly LinearIssueSummary[];
  onConnectLinear: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onLoadLinearIssues: () => Promise<void>;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<void>;
  onStartExecution: (request: StartExecutionRequest) => Promise<void>;
  onStopExecution: (executionId: string) => Promise<void>;
  onRecordReviewCheck: (request: RecordReviewCheckRequest) => Promise<void>;
  onRecordReviewDecision: (
    request: RecordReviewDecisionRequest,
  ) => Promise<void>;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>;

export function BoardView({
  busy,
  agentProfiles,
  snapshot,
  onAddDependency,
  onCreateWorkItem,
  onImportLinearBlocker,
  onImportLinearIssue,
  linearConnectionStatus,
  linearIssues,
  onConnectLinear,
  onLoadLinearIssues,
  onSaveAgentProfile,
  onStartExecution,
  onStopExecution,
  onRecordReviewCheck,
  onRecordReviewDecision,
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
                    onRecordReviewCheck={onRecordReviewCheck}
                    onRecordReviewDecision={onRecordReviewDecision}
                  />
                ))}
              </div>
            </section>
          ))}
        </section>
        <aside className="board-actions">
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
          <LinearConnectionPanel
            busy={busy}
            status={linearConnectionStatus}
            onConnect={onConnectLinear}
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
        </aside>
      </div>
    </section>
  );
}
