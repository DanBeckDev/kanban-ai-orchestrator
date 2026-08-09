import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type { RecordReviewDecisionRequest, WorkItem } from "./types";

type ReviewDecisionFormProps = Readonly<{
  busy: boolean;
  workItem: WorkItem;
  onRecord: (request: RecordReviewDecisionRequest) => Promise<void>;
}>;

export function ReviewDecisionForm({
  busy,
  workItem,
  onRecord,
}: ReviewDecisionFormProps) {
  const [reviewer, setReviewer] = useState("");
  const [summary, setSummary] = useState("");
  const [accepted, setAccepted] = useState(true);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const recordedAt = timestamp();
    await onRecord({
      evidenceId: `review-decision-${workItem.id}-${recordedAt}`,
      workItemId: workItem.id,
      reviewer,
      summary,
      accepted,
      recordedAt,
    });
    setSummary("");
  }

  return (
    <form
      aria-label={`Record review decision for ${workItem.title}`}
      className="review-check-form"
      onSubmit={submit}
    >
      <h5>Review decision</h5>
      <label>
        Reviewer
        <input
          required
          value={reviewer}
          onChange={(event) => setReviewer(event.target.value)}
        />
      </label>
      <label>
        Decision summary
        <input
          required
          placeholder="e.g. Acceptance criteria verified"
          value={summary}
          onChange={(event) => setSummary(event.target.value)}
        />
      </label>
      <label className="checkbox-label">
        <input
          checked={accepted}
          type="checkbox"
          onChange={(event) => setAccepted(event.target.checked)}
        />
        Accept review
      </label>
      <button disabled={busy} type="submit">
        Record decision
      </button>
    </form>
  );
}
