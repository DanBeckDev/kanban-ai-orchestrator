import { useState, type FormEvent } from "react";

import {
  activityFor,
  blockersFor,
  budgetSummary,
  evidenceFor,
  executionsFor,
  nextTransitionStates,
  stateLabel,
  timestamp,
} from "./presentation";
import type {
  BoardSnapshot,
  CompletionEvidence,
  TransitionWorkItemRequest,
  WorkItem,
  WorkItemState,
} from "./types";

type WorkItemCardProps = Readonly<{
  busy: boolean;
  snapshot: BoardSnapshot;
  workItem: WorkItem;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>;

const emptyEvidence: CompletionEvidence = {
  checksPassed: false,
  completionReportPresent: false,
  reviewAccepted: false,
};

export function WorkItemCard({
  busy,
  snapshot,
  workItem,
  onTransition,
}: WorkItemCardProps) {
  const [nextState, setNextState] = useState<WorkItemState | "">("");
  const [reason, setReason] = useState("");
  const [evidence, setEvidence] = useState(emptyEvidence);
  const dependencies = blockersFor(snapshot, workItem.id);
  const activity = activityFor(snapshot, workItem.id);
  const executions = executionsFor(snapshot, workItem.id);
  const evidenceRecords = evidenceFor(snapshot, workItem.id);
  const options = nextTransitionStates(workItem.state);

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
        <p className="review-requirement">Human review required before Done</p>
      )}
      <ul className="criteria-list">
        {workItem.acceptanceCriteria.map((criterion) => (
          <li key={criterion}>{criterion}</li>
        ))}
      </ul>
      {executions.length > 0 && <ExecutionHistory executions={executions} />}
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
            <EvidenceFields evidence={evidence} onChange={setEvidence} />
          )}
          <button disabled={busy} type="submit">
            Request transition
          </button>
        </form>
      )}
    </article>
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
                Evidence: checks{" "}
                {entry.completionEvidence.checksPassed
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
  onChange,
}: Readonly<{
  evidence: CompletionEvidence;
  onChange: (evidence: CompletionEvidence) => void;
}>) {
  return (
    <fieldset>
      <legend>Completion evidence</legend>
      <EvidenceCheckbox
        checked={evidence.checksPassed}
        label="Checks passed"
        onChange={(checksPassed) => onChange({ ...evidence, checksPassed })}
      />
      <EvidenceCheckbox
        checked={evidence.completionReportPresent}
        label="Completion report present"
        onChange={(completionReportPresent) =>
          onChange({ ...evidence, completionReportPresent })
        }
      />
      <EvidenceCheckbox
        checked={evidence.reviewAccepted}
        label="Review accepted"
        onChange={(reviewAccepted) => onChange({ ...evidence, reviewAccepted })}
      />
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
