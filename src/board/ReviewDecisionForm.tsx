import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";

import { timestamp } from "./presentation";
import type { RecordReviewDecisionRequest, WorkItem } from "./types";

type ReviewDecisionFormProps = Readonly<{
  busy: boolean;
  workItem: WorkItem;
  onRecord: (request: RecordReviewDecisionRequest) => Promise<void>;
  onReturnForCorrection: (summary: string, recordedAt: string) => Promise<void>;
}>;

export function ReviewDecisionForm({
  busy,
  workItem,
  onRecord,
  onReturnForCorrection,
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
    if (!accepted) {
      await onReturnForCorrection(summary, recordedAt);
    }
    setSummary("");
  }

  return (
    <form
      aria-label={`Record review decision for ${workItem.title}`}
      className="review-check-form"
      onSubmit={submit}
    >
      <FieldGroup>
        <h5>Review decision</h5>
        <Field>
          <FieldLabel htmlFor="reviewer">Reviewer</FieldLabel>
          <Input
            id="reviewer"
            required
            value={reviewer}
            onChange={(event) => setReviewer(event.target.value)}
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="review-decision-summary">
            Decision summary
          </FieldLabel>
          <Input
            id="review-decision-summary"
            required
            placeholder="e.g. Acceptance criteria verified"
            value={summary}
            onChange={(event) => setSummary(event.target.value)}
          />
        </Field>
        <Field orientation="horizontal">
          <Input
            checked={accepted}
            id="review-decision-accepted"
            type="checkbox"
            onChange={(event) => setAccepted(event.target.checked)}
          />
          <FieldLabel htmlFor="review-decision-accepted">
            Accept this work
          </FieldLabel>
        </Field>
        {!accepted && (
          <FieldDescription>
            Kanban will keep the review record and return this task to Ready
            with this summary.
          </FieldDescription>
        )}
        <Button disabled={busy} type="submit">
          {accepted ? "Record decision" : "Return for correction"}
        </Button>
      </FieldGroup>
    </form>
  );
}
