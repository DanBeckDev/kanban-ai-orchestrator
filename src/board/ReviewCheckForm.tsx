import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

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
      <FieldGroup>
        <h5>Quality gate</h5>
        <Field>
          <FieldLabel htmlFor="quality-gate-summary">Result summary</FieldLabel>
          <Input
            id="quality-gate-summary"
            required
            placeholder="e.g. npm test passed"
            value={summary}
            onChange={(event) => setSummary(event.target.value)}
          />
        </Field>
        <Field orientation="horizontal">
          <Input
            checked={passed}
            id="quality-gate-passed"
            type="checkbox"
            onChange={(event) => setPassed(event.target.checked)}
          />
          <FieldLabel htmlFor="quality-gate-passed">Check passed</FieldLabel>
        </Field>
        <Button disabled={busy} type="submit">
          Record quality gate
        </Button>
      </FieldGroup>
    </form>
  );
}
