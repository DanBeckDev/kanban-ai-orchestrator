import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import { AgentLaunchForm } from "./AgentLaunchForm";
import { CleanCodeReviewForm } from "./CleanCodeReviewForm";
import { correctionTransition } from "./correctionTransition";
import { ExecutionControl } from "./ExecutionControl";
import { manualTransitionStates } from "./presentation";
import { RecoveryActions } from "./RecoveryActions";
import { ReviewCheckForm } from "./ReviewCheckForm";
import { ReviewDecisionForm } from "./ReviewDecisionForm";
import { TaskStateChangeForm } from "./TaskStateChangeForm";
import type {
  AgentProfile,
  BoardSnapshot,
  Execution,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
  WorkItem,
} from "./types";

type TaskActionPanelProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  defaultAgentProfileName?: string;
  snapshot: BoardSnapshot;
  workItem: WorkItem;
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

export function TaskActionPanel({
  agentProfiles,
  busy,
  defaultAgentProfileName,
  snapshot,
  workItem,
  onRecordCleanCodeReview,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onStartExecution,
  onStopExecution,
  onTransition,
}: TaskActionPanelProps) {
  const executions = snapshot.executions.filter(
    (execution) => execution.workItemId === workItem.id,
  );
  const completedReviewExecutions = executions.filter(
    (execution) =>
      execution.role === "independent_review" &&
      execution.status === "completed",
  );
  const options = permittedManualStateChanges(workItem.state);
  const hasActions =
    workItem.state === "ready" ||
    workItem.state === "review" ||
    isRecoveryState(workItem.state) ||
    executions.some(isControllableExecution) ||
    options.length > 0;

  if (!hasActions) return null;

  return (
    <Card className="task-action-panel">
      <CardHeader>
        <CardTitle as="h3">What you can do now</CardTitle>
        <CardDescription>
          These actions request a daemon-checked change. They do not bypass the
          task's review or policy requirements.
        </CardDescription>
      </CardHeader>
      <CardContent className="task-action-panel-content">
        {workItem.state === "ready" && (
          <AgentLaunchForm
            busy={busy}
            buttonLabel="Start task worker"
            defaultProfileName={defaultAgentProfileName}
            formLabel={`Prompt AI for ${workItem.title}`}
            profiles={agentProfiles}
            workItem={workItem}
            onStart={onStartExecution}
          />
        )}
        <ExecutionControl
          busy={busy}
          executions={executions}
          onStop={onStopExecution}
        />
        {workItem.state === "review" && (
          <ReviewActions
            agentProfiles={agentProfiles}
            busy={busy}
            completedReviewExecutions={completedReviewExecutions}
            defaultAgentProfileName={defaultAgentProfileName}
            workItem={workItem}
            onRecordCleanCodeReview={onRecordCleanCodeReview}
            onRecordReviewCheck={onRecordReviewCheck}
            onRecordReviewDecision={onRecordReviewDecision}
            onTransition={onTransition}
            onStartExecution={onStartExecution}
          />
        )}
        {isRecoveryState(workItem.state) && (
          <RecoveryActions
            busy={busy}
            executions={executions}
            workItem={workItem}
            onTransition={onTransition}
          />
        )}
        <TaskStateChangeForm
          busy={busy}
          options={options}
          workItem={workItem}
          onTransition={onTransition}
        />
      </CardContent>
    </Card>
  );
}

function ReviewActions({
  agentProfiles,
  busy,
  completedReviewExecutions,
  defaultAgentProfileName,
  workItem,
  onRecordCleanCodeReview,
  onRecordReviewCheck,
  onRecordReviewDecision,
  onTransition,
  onStartExecution,
}: Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  completedReviewExecutions: readonly Execution[];
  defaultAgentProfileName?: string;
  workItem: WorkItem;
  onRecordCleanCodeReview: (
    request: RecordCleanCodeReviewRequest,
  ) => Promise<void>;
  onRecordReviewCheck: (request: RecordReviewCheckRequest) => Promise<void>;
  onRecordReviewDecision: (
    request: RecordReviewDecisionRequest,
  ) => Promise<void>;
  onStartExecution: (request: StartExecutionRequest) => Promise<void>;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>) {
  const returnForCorrection = (summary: string, recordedAt: string) =>
    onTransition(correctionTransition(workItem, summary, recordedAt));

  return (
    <section aria-label={`Review actions for ${workItem.title}`}>
      <h4>Review actions</h4>
      <ReviewCheckForm
        busy={busy}
        workItem={workItem}
        onRecord={onRecordReviewCheck}
      />
      {workItem.requiresHumanReview && (
        <>
          <AgentLaunchForm
            busy={busy}
            buttonLabel="Start independent reviewer"
            defaultProfileName={defaultAgentProfileName}
            executionRole="independent_review"
            formLabel={`Start independent reviewer for ${workItem.title}`}
            profiles={agentProfiles}
            workItem={workItem}
            onStart={onStartExecution}
          />
          <CleanCodeReviewForm
            busy={busy}
            reviewExecutions={completedReviewExecutions}
            workItem={workItem}
            onRecord={onRecordCleanCodeReview}
          />
        </>
      )}
      <ReviewDecisionForm
        busy={busy}
        workItem={workItem}
        onRecord={onRecordReviewDecision}
        onReturnForCorrection={returnForCorrection}
      />
    </section>
  );
}

function isRecoveryState(state: WorkItem["state"]): boolean {
  return state === "blocked" || state === "failed" || state === "interrupted";
}

function isControllableExecution({
  status,
}: BoardSnapshot["executions"][number]) {
  return status === "running" || status === "awaiting_input";
}

function permittedManualStateChanges(state: WorkItem["state"]) {
  return manualTransitionStates(state).filter(
    (nextState) => !(state === "review" && nextState === "ready"),
  );
}
