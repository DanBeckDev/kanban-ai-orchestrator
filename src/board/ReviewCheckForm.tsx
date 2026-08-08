import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type { RecordReviewCheckRequest, WorkItem } from "./types";

type ReviewCheckFormProps = Readonly<{
  busy: boolean;
  workItem: WorkItem;
  onRecord: (request: RecordReviewCheckRequest) => Promise<void>;
}>;

export function ReviewCheckForm({
  busy,
  workItem,
  onRecord,
}: ReviewCheckFormProps) {
  const [summary, setSummary] = useState("");
  const [passed, setPassed] = useState(true);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const recordedAt = timestamp();
    await onRecord({
      evidenceId: `review-check-${workItem.id}-${recordedAt}`,
      workItemId: workItem.id,
      summary,
      passed,
      recordedAt,
    });
    setSummary("");
    setPassed(true);
  }

  return (
    <form
      aria-label={`Record quality gate for ${workItem.title}`}
      className="review-check-form"
      onSubmit={submit}
    >
      <h5>Quality gate</h5>
      <label>
        Result summary
        <input
          required
          placeholder="e.g. npm test passed"
          value={summary}
          onChange={(event) => setSummary(event.target.value)}
        />
      </label>
      <label className="checkbox-label">
        <input
          checked={passed}
          type="checkbox"
          onChange={(event) => setPassed(event.target.checked)}
        />
        Check passed
      </label>
      <button disabled={busy} type="submit">
        Record quality gate
      </button>
    </form>
  );
}
