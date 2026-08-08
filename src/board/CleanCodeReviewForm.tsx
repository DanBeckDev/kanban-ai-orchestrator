import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type {
  Execution,
  RecordCleanCodeReviewRequest,
  WorkItem,
} from "./types";

type CleanCodeReviewFormProps = Readonly<{
  busy: boolean;
  reviewExecutions: readonly Execution[];
  workItem: WorkItem;
  onRecord: (request: RecordCleanCodeReviewRequest) => Promise<void>;
}>;

export function CleanCodeReviewForm({
  busy,
  reviewExecutions,
  workItem,
  onRecord,
}: CleanCodeReviewFormProps) {
  const [reviewExecutionId, setReviewExecutionId] = useState("");
  const [findingCount, setFindingCount] = useState(0);
  const [summary, setSummary] = useState("");

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const recordedAt = timestamp();
    await onRecord({
      evidenceId: `clean-code-review-${workItem.id}-${recordedAt}`,
      workItemId: workItem.id,
      reviewExecutionId,
      actionableFindingCount: findingCount,
      summary,
      recordedAt,
    });
    setSummary("");
  }

  if (reviewExecutions.length === 0) {
    return (
      <p className="launch-hint">
        Start and complete an independent review agent before recording its
        decision.
      </p>
    );
  }

  return (
    <form
      aria-label={`Record Clean Code review for ${workItem.title}`}
      className="review-check-form"
      onSubmit={submit}
    >
      <h5>Independent Clean Code review</h5>
      <label>
        Completed reviewer run
        <select
          required
          value={reviewExecutionId}
          onChange={(event) => setReviewExecutionId(event.target.value)}
        >
          <option value="">Select reviewer run</option>
          {reviewExecutions.map((execution) => (
            <option key={execution.id} value={execution.id}>
              {execution.adapterName} · {execution.id}
            </option>
          ))}
        </select>
      </label>
      <label>
        Actionable finding count
        <input
          min="0"
          required
          type="number"
          value={findingCount}
          onChange={(event) => setFindingCount(Number(event.target.value))}
        />
      </label>
      <label>
        Decision summary
        <textarea
          maxLength={2000}
          required
          value={summary}
          onChange={(event) => setSummary(event.target.value)}
        />
      </label>
      <button disabled={busy} type="submit">
        Record Clean Code review
      </button>
    </form>
  );
}
