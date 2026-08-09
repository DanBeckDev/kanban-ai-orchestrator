import { useState, type FormEvent } from "react";

import {
  activityFor,
  blockersFor,
  budgetSummary,
  evidenceFor,
  externalLinksFor,
  executionsFor,
  manualTransitionStates,
  stateLabel,
  timestamp,
} from "./presentation";
import { AgentLaunchForm } from "./AgentLaunchForm";
import { ActivityStream } from "./ActivityStream";
import { CleanCodeReviewForm } from "./CleanCodeReviewForm";
import { ReviewCheckForm } from "./ReviewCheckForm";
import { ReviewDecisionForm } from "./ReviewDecisionForm";
import { ExecutionControl } from "./ExecutionControl";
import { ExternalLinks } from "./ExternalLinks";
import { RecoveryActions } from "./RecoveryActions";
import type {
  BoardSnapshot,
  AgentProfile,
  CompletionEvidence,
  Execution,
  ExecutionActivityPage,
  RecordCleanCodeReviewRequest,
  TransitionWorkItemRequest,
  StartExecutionRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  WorkItem,
  WorkItemState,
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

const emptyEvidence: CompletionEvidence = {
  qualityGatePassed: false,
  completionReportPresent: false,
  reviewAccepted: false,
};

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
  const [nextState, setNextState] = useState<WorkItemState | "">("");
  const [reason, setReason] = useState("");
  const [evidence, setEvidence] = useState(emptyEvidence);
  const dependencies = blockersFor(snapshot, workItem.id);
  const activity = activityFor(snapshot, workItem.id);
  const executions = executionsFor(snapshot, workItem.id);
  const evidenceRecords = evidenceFor(snapshot, workItem.id);
  const completedReviewExecutions = executions.filter(
    (execution) =>
      execution.role === "independent_review" &&
      execution.status === "completed",
  );
  const liveExecution = executions.find(isLiveExecution);
  const externalLinks = externalLinksFor(snapshot, workItem.id);
  const options = manualTransitionStates(workItem.state);

  async function submitTransition(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (nextState === "") return;
    const recordedAt = timestamp();
    await onTransition({
      eventId: `transition-${workItem.id}-${nextState}-${recordedAt}`,
      workItemId: workItem.id,
      nextState,
      evidence: nextState === "done" ? evidence : undefined,
      reason,
      recordedAt,
    });
    setNextState("");
    setReason("");
  }

  return (
    <article className="work-item-card">
      <div className="card-state">{stateLabel(workItem.state)}</div>
      <h4>{workItem.title}</h4>
      <p>{workItem.description}</p>
      <p className="budget-summary">{budgetSummary(workItem)}</p>
      {workItem.requiresHumanReview && (
        <p className="review-requirement">
          Independent Clean Code review and human review required before Done
        </p>
      )}
      <ul className="criteria-list">
        {workItem.acceptanceCriteria.map((criterion) => (
          <li key={criterion}>{criterion}</li>
        ))}
      </ul>
      <ExternalLinks links={externalLinks} />
      {executions.length > 0 && <ExecutionHistory executions={executions} />}
      {liveExecution !== undefined && (
        <ActivityStream
          execution={liveExecution}
          onLoad={onLoadExecutionActivity}
        />
      )}
      {evidenceRecords.length > 0 && (
        <EvidenceHistory evidence={evidenceRecords} />
      )}
      {activity.length > 0 && <DecisionHistory activity={activity} />}
      {dependencies.length > 0 && (
        <section className="dependency-notice">
          <strong>Dependency gate</strong>
          {dependencies.map((dependency) => (
            <p key={dependency.id}>
              {dependency.kind.replaceAll("_", " ")}: {dependency.reason}
              <span>
                Owner: {dependency.owner}. Next: {dependency.nextAction}
              </span>
            </p>
          ))}
        </section>
      )}
      {workItem.state === "ready" && (
        <AgentLaunchForm
          busy={busy}
          defaultProfileName={defaultAgentProfileName}
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
        <>
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
          />
        </>
      )}
      {isRecoveryState(workItem.state) && (
        <RecoveryActions
          busy={busy}
          executions={executions}
          workItem={workItem}
          onTransition={onTransition}
        />
      )}
      {options.length > 0 && (
        <form
          aria-label={`Transition ${workItem.title}`}
          className="transition-form"
          onSubmit={submitTransition}
        >
          <label>
            Move to
            <select
              required
              value={nextState}
              onChange={(event) =>
                setNextState(event.target.value as WorkItemState | "")
              }
            >
              <option value="">Select state</option>
              {options.map((state) => (
                <option key={state} value={state}>
                  {stateLabel(state)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Reason
            <input
              required
              value={reason}
              onChange={(event) => setReason(event.target.value)}
            />
          </label>
          {nextState === "done" && (
            <EvidenceFields
              evidence={evidence}
              requiresHumanReview={workItem.requiresHumanReview}
              onChange={setEvidence}
            />
          )}
          <button disabled={busy} type="submit">
            Request transition
          </button>
        </form>
      )}
    </article>
  );
}

function isRecoveryState(state: WorkItemState): boolean {
  return state === "blocked" || state === "failed" || state === "interrupted";
}

function isLiveExecution(execution: Execution): boolean {
  return (
    execution.status === "running" ||
    execution.status === "awaiting_input" ||
    execution.status === "awaiting_review"
  );
}

function ExecutionHistory({
  executions,
}: Readonly<{ executions: ReturnType<typeof executionsFor> }>) {
  return (
    <details className="execution-history">
      <summary>Recent agent attempts ({executions.length})</summary>
      <ol>
        {executions.map((execution) => (
          <li key={execution.id}>
            <p>
              {execution.adapterName} · {execution.status.replaceAll("_", " ")}
            </p>
            <span>Workspace: {execution.workspacePath}</span>
            {execution.sessionId && <span>Session: {execution.sessionId}</span>}
            <span>
              Usage: {execution.usage.inputTokens} input /{" "}
              {execution.usage.outputTokens} output tokens
            </span>
          </li>
        ))}
      </ol>
    </details>
  );
}

function EvidenceHistory({
  evidence,
}: Readonly<{ evidence: ReturnType<typeof evidenceFor> }>) {
  return (
    <details className="evidence-history">
      <summary>Recent review evidence ({evidence.length})</summary>
      <ol>
        {evidence.map((entry) => (
          <li key={entry.id}>
            <p>
              {entry.kind.replaceAll("_", " ")}: {entry.result}
            </p>
            <span>{entry.summary}</span>
            <time dateTime={entry.recordedAt}>{entry.recordedAt}</time>
          </li>
        ))}
      </ol>
    </details>
  );
}

function DecisionHistory({
  activity,
}: Readonly<{ activity: ReturnType<typeof activityFor> }>) {
  return (
    <details className="decision-history">
      <summary>Recent decision history ({activity.length})</summary>
      <ol>
        {activity.map((entry) => (
          <li key={entry.sequence}>
            <p>{entry.summary}</p>
            <time dateTime={entry.recordedAt}>{entry.recordedAt}</time>
            {entry.completionEvidence && (
              <span>
                Evidence: quality gate{" "}
                {entry.completionEvidence.qualityGatePassed
                  ? "passed"
                  : "not passed"}
                , report{" "}
                {entry.completionEvidence.completionReportPresent
                  ? "present"
                  : "missing"}
                , review{" "}
                {entry.completionEvidence.reviewAccepted
                  ? "accepted"
                  : "pending"}
                .
              </span>
            )}
          </li>
        ))}
      </ol>
    </details>
  );
}

function EvidenceFields({
  evidence,
  requiresHumanReview,
  onChange,
}: Readonly<{
  evidence: CompletionEvidence;
  requiresHumanReview: boolean;
  onChange: (evidence: CompletionEvidence) => void;
}>) {
  return (
    <fieldset>
      <legend>Completion evidence</legend>
      <EvidenceCheckbox
        checked={evidence.qualityGatePassed}
        label="Quality gate passed"
        onChange={(qualityGatePassed) =>
          onChange({ ...evidence, qualityGatePassed })
        }
      />
      <EvidenceCheckbox
        checked={evidence.completionReportPresent}
        label="Completion report present"
        onChange={(completionReportPresent) =>
          onChange({ ...evidence, completionReportPresent })
        }
      />
      {requiresHumanReview && (
        <EvidenceCheckbox
          checked={evidence.reviewAccepted}
          label="Independent and human reviews accepted"
          onChange={(reviewAccepted) =>
            onChange({ ...evidence, reviewAccepted })
          }
        />
      )}
    </fieldset>
  );
}

function EvidenceCheckbox({
  checked,
  label,
  onChange,
}: Readonly<{
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}>) {
  return (
    <label className="checkbox-label">
      <input
        checked={checked}
        type="checkbox"
        onChange={(event) => onChange(event.target.checked)}
      />
      {label}
    </label>
  );
}
