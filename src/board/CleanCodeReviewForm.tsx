import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

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
      <FieldGroup>
        <h5>Independent Clean Code review</h5>
        <Field>
          <FieldLabel htmlFor="completed-reviewer-run">
            Completed reviewer run
          </FieldLabel>
          <Select
            value={reviewExecutionId}
            onValueChange={setReviewExecutionId}
          >
            <SelectTrigger id="completed-reviewer-run">
              <SelectValue placeholder="Select a reviewer run" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {reviewExecutions.map((execution) => (
                  <SelectItem key={execution.id} value={execution.id}>
                    {execution.adapterName} · {execution.id}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
        <Field>
          <FieldLabel htmlFor="finding-count">
            Actionable finding count
          </FieldLabel>
          <Input
            id="finding-count"
            min="0"
            required
            type="number"
            value={findingCount}
            onChange={(event) => setFindingCount(Number(event.target.value))}
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="clean-code-summary">Decision summary</FieldLabel>
          <Textarea
            id="clean-code-summary"
            maxLength={2000}
            required
            value={summary}
            onChange={(event) => setSummary(event.target.value)}
          />
        </Field>
        <Button disabled={busy || reviewExecutionId.length === 0} type="submit">
          Record Clean Code review
        </Button>
      </FieldGroup>
    </form>
  );
}
