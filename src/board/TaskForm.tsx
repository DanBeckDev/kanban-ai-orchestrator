import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

import { timestamp } from "./presentation";
import type { CreateWorkItemRequest } from "./types";

type TaskFormProps = Readonly<{
  boardId: string;
  busy: boolean;
  onCreate: (request: CreateWorkItemRequest) => Promise<void>;
}>;

type TaskInput = Readonly<{
  title: string;
  description: string;
  criteria: string;
  requiresHumanReview: boolean;
  maxAgentTurns: string;
  maxDurationSeconds: string;
  maxCostMicros: string;
}>;

const initialTaskInput: TaskInput = {
  title: "",
  description: "",
  criteria: "",
  requiresHumanReview: false,
  maxAgentTurns: "",
  maxDurationSeconds: "",
  maxCostMicros: "",
};

export function TaskForm({ boardId, busy, onCreate }: TaskFormProps) {
  const [input, setInput] = useState(initialTaskInput);

  async function createTask(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const recordedAt = timestamp();
    const workItemId = generatedWorkItemId(input.title, recordedAt);
    await onCreate({
      eventId: `create-${workItemId}-${recordedAt}`,
      workItemId,
      boardId,
      title: input.title,
      description: input.description,
      acceptanceCriteria: input.criteria
        .split("\n")
        .map((criterion) => criterion.trim())
        .filter(Boolean),
      budget: {
        maxAgentTurns: optionalPositiveInteger(input.maxAgentTurns),
        maxDurationSeconds: optionalPositiveInteger(input.maxDurationSeconds),
        maxCostMicros: optionalPositiveInteger(input.maxCostMicros),
      },
      requiresHumanReview: input.requiresHumanReview,
      recordedAt,
    });
    setInput(initialTaskInput);
  }

  return (
    <form
      aria-labelledby="add-task-title"
      className="panel form-panel"
      onSubmit={createTask}
    >
      <h3 id="add-task-title">Create a task</h3>
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="task-title">Title</FieldLabel>
          <Input
            autoComplete="off"
            id="task-title"
            name="title"
            required
            value={input.title}
            onChange={(event) =>
              setInput({ ...input, title: event.target.value })
            }
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="task-description">Description</FieldLabel>
          <Textarea
            id="task-description"
            name="description"
            required
            value={input.description}
            onChange={(event) =>
              setInput({ ...input, description: event.target.value })
            }
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="task-criteria">Acceptance criteria</FieldLabel>
          <FieldDescription>Write one outcome per line.</FieldDescription>
          <Textarea
            id="task-criteria"
            name="acceptance-criteria"
            required
            value={input.criteria}
            onChange={(event) =>
              setInput({ ...input, criteria: event.target.value })
            }
          />
        </Field>
        <Field orientation="horizontal">
          <input
            checked={input.requiresHumanReview}
            id="task-requires-review"
            name="requires-human-review"
            type="checkbox"
            onChange={(event) =>
              setInput({ ...input, requiresHumanReview: event.target.checked })
            }
          />
          <FieldLabel htmlFor="task-requires-review">
            Require human review before Done
          </FieldLabel>
        </Field>
        <FieldSet>
          <FieldLegend>Limits (optional)</FieldLegend>
          <FieldDescription>
            Set a ceiling only when this task needs one.
          </FieldDescription>
          <BudgetInput
            id="task-max-agent-turns"
            label="Max agent turns"
            value={input.maxAgentTurns}
            onChange={(maxAgentTurns) => setInput({ ...input, maxAgentTurns })}
          />
          <BudgetInput
            id="task-max-duration-seconds"
            label="Max duration seconds"
            value={input.maxDurationSeconds}
            onChange={(maxDurationSeconds) =>
              setInput({ ...input, maxDurationSeconds })
            }
          />
          <BudgetInput
            id="task-max-cost-micros"
            label="Max cost micros"
            value={input.maxCostMicros}
            onChange={(maxCostMicros) => setInput({ ...input, maxCostMicros })}
          />
        </FieldSet>
      </FieldGroup>
      <Button disabled={busy} type="submit">
        Create task
      </Button>
    </form>
  );
}

function generatedWorkItemId(title: string, recordedAt: string): string {
  const titleFragment = title
    .trim()
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, "-")
    .replaceAll(/^-|-$/g, "")
    .slice(0, 36);
  return `${titleFragment || "task"}-${recordedAt.replaceAll(/\D/g, "")}`;
}

function BudgetInput({
  id,
  label,
  value,
  onChange,
}: Readonly<{
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
}>) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Input
        id={id}
        min="1"
        name={id}
        step="1"
        type="number"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </Field>
  );
}

function optionalPositiveInteger(value: string): number | undefined {
  return value === "" ? undefined : Number(value);
}
