import { useState, type FormEvent } from "react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
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

import { stateLabel, timestamp } from "./presentation";
import type {
  CompletionEvidence,
  TransitionWorkItemRequest,
  WorkItem,
  WorkItemState,
} from "./types";

type TaskStateChangeFormProps = Readonly<{
  busy: boolean;
  options: readonly WorkItemState[];
  workItem: WorkItem;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>;

const emptyEvidence: CompletionEvidence = {
  qualityGatePassed: false,
  completionReportPresent: false,
  reviewAccepted: false,
};

export function TaskStateChangeForm({
  busy,
  options,
  workItem,
  onTransition,
}: TaskStateChangeFormProps) {
  const [nextState, setNextState] = useState<WorkItemState>();
  const [reason, setReason] = useState("");
  const [evidence, setEvidence] = useState(emptyEvidence);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (nextState === undefined) return;
    const recordedAt = timestamp();
    await onTransition({
      eventId: `transition-${workItem.id}-${nextState}-${recordedAt}`,
      workItemId: workItem.id,
      nextState,
      evidence: nextState === "done" ? evidence : undefined,
      reason,
      recordedAt,
    });
    setNextState(undefined);
    setReason("");
    setEvidence(emptyEvidence);
  }

  if (options.length === 0) return null;

  return (
    <Accordion collapsible type="single">
      <AccordionItem value="state-change">
        <AccordionTrigger>More task options</AccordionTrigger>
        <AccordionContent>
          <form
            aria-label={`Change state for ${workItem.title}`}
            className="task-state-change-form"
            onSubmit={submit}
          >
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="next-task-state">Next state</FieldLabel>
                <Select
                  onValueChange={(state) =>
                    setNextState(state as WorkItemState)
                  }
                  value={nextState ?? ""}
                >
                  <SelectTrigger id="next-task-state">
                    <SelectValue placeholder="Choose a permitted state" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {options.map((state) => (
                        <SelectItem key={state} value={state}>
                          {stateLabel(state)}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Field>
              <Field>
                <FieldLabel htmlFor="state-change-reason">Why</FieldLabel>
                <Input
                  id="state-change-reason"
                  required
                  value={reason}
                  onChange={(event) => setReason(event.target.value)}
                />
              </Field>
              {nextState === "done" && (
                <CompletionEvidenceFields
                  evidence={evidence}
                  requiresHumanReview={workItem.requiresHumanReview}
                  onChange={setEvidence}
                />
              )}
              <Button disabled={busy || nextState === undefined} type="submit">
                Request state change
              </Button>
            </FieldGroup>
          </form>
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
}

function CompletionEvidenceFields({
  evidence,
  requiresHumanReview,
  onChange,
}: Readonly<{
  evidence: CompletionEvidence;
  requiresHumanReview: boolean;
  onChange: (evidence: CompletionEvidence) => void;
}>) {
  return (
    <fieldset className="completion-evidence-fields">
      <legend>Completion evidence</legend>
      <EvidenceCheckbox
        checked={evidence.qualityGatePassed}
        id="state-change-quality-gate"
        label="Quality gate passed"
        onChange={(qualityGatePassed) =>
          onChange({ ...evidence, qualityGatePassed })
        }
      />
      <EvidenceCheckbox
        checked={evidence.completionReportPresent}
        id="state-change-completion-report"
        label="Completion report present"
        onChange={(completionReportPresent) =>
          onChange({ ...evidence, completionReportPresent })
        }
      />
      {requiresHumanReview && (
        <EvidenceCheckbox
          checked={evidence.reviewAccepted}
          id="state-change-review-accepted"
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
  id,
  label,
  onChange,
}: Readonly<{
  checked: boolean;
  id: string;
  label: string;
  onChange: (checked: boolean) => void;
}>) {
  return (
    <Field orientation="horizontal">
      <Input
        checked={checked}
        id={id}
        type="checkbox"
        onChange={(event) => onChange(event.target.checked)}
      />
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
    </Field>
  );
}
