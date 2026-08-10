import { WorkItemCard } from "./WorkItemCard";
import { SurfaceHeader } from "./BoardManagement";
import { TaskAiPrompt } from "./TaskAiPrompt";
import type { TicketEffectOperations } from "./ticketEffectOperations";
import type {
  AgentProfile,
  BoardSnapshot,
  ExecutionActivityPage,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
  WorkItem,
} from "./types";

type TaskDetailViewProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  defaultAgentProfileName?: string;
  hasOrganiser: boolean;
  snapshot: BoardSnapshot;
  ticketEffects: TicketEffectOperations;
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
}>;

export function TaskDetailView({
  agentProfiles,
  busy,
  defaultAgentProfileName,
  hasOrganiser,
  snapshot,
  ticketEffects,
  workItem,
  onBack,
  onLoadExecutionActivity,
  onRecordCleanCodeReview,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onStartExecution,
  onStopExecution,
  onTransition,
}: TaskDetailViewProps) {
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
      <TaskAiPrompt
        busy={busy}
        hasOrganiser={hasOrganiser}
        operations={ticketEffects}
        workItemId={workItem.id}
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
