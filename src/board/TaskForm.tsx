import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type { CreateWorkItemRequest } from "./types";

type TaskFormProps = Readonly<{
  boardId: string;
  busy: boolean;
  onCreate: (request: CreateWorkItemRequest) => Promise<void>;
}>;

type TaskInput = Readonly<{
  id: string;
  title: string;
  description: string;
  criteria: string;
  requiresHumanReview: boolean;
  maxAgentTurns: string;
  maxDurationSeconds: string;
  maxCostMicros: string;
}>;

const initialTaskInput: TaskInput = {
  id: "",
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
    await onCreate({
      eventId: `create-${input.id}-${recordedAt}`,
      workItemId: input.id,
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
      <h3 id="add-task-title">Add task</h3>
      <label>
        Task ID
        <input
          required
          value={input.id}
          onChange={(event) => setInput({ ...input, id: event.target.value })}
        />
      </label>
      <label>
        Title
        <input
          required
          value={input.title}
          onChange={(event) =>
            setInput({ ...input, title: event.target.value })
          }
        />
      </label>
      <label>
        Description
        <textarea
          required
          value={input.description}
          onChange={(event) =>
            setInput({ ...input, description: event.target.value })
          }
        />
      </label>
      <label>
        Acceptance criteria <span className="field-hint">one per line</span>
        <textarea
          required
          value={input.criteria}
          onChange={(event) =>
            setInput({ ...input, criteria: event.target.value })
          }
        />
      </label>
      <label className="checkbox-label">
        <input
          checked={input.requiresHumanReview}
          type="checkbox"
          onChange={(event) =>
            setInput({ ...input, requiresHumanReview: event.target.checked })
          }
        />
        Require human review before Done
      </label>
      <fieldset>
        <legend>Agent budget (optional)</legend>
        <BudgetInput
          label="Max agent turns"
          value={input.maxAgentTurns}
          onChange={(maxAgentTurns) => setInput({ ...input, maxAgentTurns })}
        />
        <BudgetInput
          label="Max duration seconds"
          value={input.maxDurationSeconds}
          onChange={(maxDurationSeconds) =>
            setInput({ ...input, maxDurationSeconds })
          }
        />
        <BudgetInput
          label="Max cost micros"
          value={input.maxCostMicros}
          onChange={(maxCostMicros) => setInput({ ...input, maxCostMicros })}
        />
      </fieldset>
      <button disabled={busy} type="submit">
        Add task
      </button>
    </form>
  );
}

function BudgetInput({
  label,
  value,
  onChange,
}: Readonly<{
  label: string;
  value: string;
  onChange: (value: string) => void;
}>) {
  return (
    <label>
      {label}
      <input
        min="1"
        step="1"
        type="number"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}

function optionalPositiveInteger(value: string): number | undefined {
  return value === "" ? undefined : Number(value);
}
