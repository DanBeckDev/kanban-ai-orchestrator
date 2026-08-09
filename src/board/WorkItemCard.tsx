import { executionsFor } from "./presentation";
import { TaskActionPanel } from "./TaskActionPanel";
import { TaskDecisionSummary } from "./TaskDecisionSummary";
import { TaskDetailSections } from "./TaskDetailSections";
import { taskDecision } from "./taskDetailPresentation";
import type {
  AgentProfile,
  BoardSnapshot,
  Execution,
  ExecutionActivityPage,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
  WorkItem,
} from "./types";

type WorkItemCardProps = Readonly<{
  busy: boolean;
  agentProfiles: readonly AgentProfile[];
  defaultAgentProfileName?: string;
  snapshot: BoardSnapshot;
  workItem: WorkItem;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
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
}>;

export function WorkItemCard({
  busy,
  agentProfiles,
  defaultAgentProfileName,
  snapshot,
  workItem,
  onTransition,
  onStartExecution,
  onStopExecution,
  onLoadExecutionActivity,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onRecordCleanCodeReview,
}: WorkItemCardProps) {
  const decision = taskDecision(snapshot, workItem);
  const liveExecution = executionsFor(snapshot, workItem.id).find(
    isLiveExecution,
  );

  return (
    <article className="work-item-card">
      <TaskDecisionSummary decision={decision} />
      <TaskActionPanel
        agentProfiles={agentProfiles}
        busy={busy}
        defaultAgentProfileName={defaultAgentProfileName}
        snapshot={snapshot}
        workItem={workItem}
        onRecordCleanCodeReview={onRecordCleanCodeReview}
        onRecordReviewCheck={onRecordReviewCheck}
        onRecordReviewDecision={onRecordReviewDecision}
        onStartExecution={onStartExecution}
        onStopExecution={onStopExecution}
        onTransition={onTransition}
      />
      <TaskDetailSections
        liveExecution={liveExecution}
        snapshot={snapshot}
        workItem={workItem}
        onLoadExecutionActivity={onLoadExecutionActivity}
      />
    </article>
  );
}

function isLiveExecution(execution: Execution): boolean {
  return (
    execution.status === "running" ||
    execution.status === "awaiting_input" ||
    execution.status === "awaiting_review"
  );
}
