import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import { timestamp } from "./presentation";
import type { AddDependencyRequest, DependencyKind, WorkItem } from "./types";

type DependencyFormProps = Readonly<{
  busy: boolean;
  workItems: readonly WorkItem[];
  onCreate: (request: AddDependencyRequest) => Promise<void>;
}>;

type DependencyInput = Readonly<{
  upstream: string;
  downstream: string;
  kind: DependencyKind;
  reason: string;
  owner: string;
  nextAction: string;
}>;

const initialDependencyInput: DependencyInput = {
  upstream: "",
  downstream: "",
  kind: "blocks",
  reason: "",
  owner: "",
  nextAction: "",
};

export function DependencyForm({
  busy,
  workItems,
  onCreate,
}: DependencyFormProps) {
  const [input, setInput] = useState(initialDependencyInput);
  const [showValidation, setShowValidation] = useState(false);
  const missingTasks = input.upstream === "" || input.downstream === "";

  async function addDependency(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (missingTasks) {
      setShowValidation(true);
      return;
    }
    const createdAt = timestamp();
    await onCreate({
      dependencyId: `${input.upstream}-${input.kind}-${input.downstream}`,
      upstreamWorkItemId: input.upstream,
      downstreamWorkItemId: input.downstream,
      kind: input.kind,
      reason: input.reason,
      owner: input.owner,
      nextAction: input.nextAction,
      createdBy: "user",
      createdAt,
    });
    setInput(initialDependencyInput);
    setShowValidation(false);
  }

  return (
    <form aria-labelledby="add-dependency-title" onSubmit={addDependency}>
      <FieldGroup>
        <div>
          <h3 id="add-dependency-title">Add a relationship</h3>
          <p className="field-hint">
            Explain why the work is connected so the orchestrator and ticket
            agents can act on it safely.
          </p>
        </div>
        <TaskSelect
          id="dependency-upstream"
          invalid={showValidation && input.upstream === ""}
          label="Must happen first"
          value={input.upstream}
          workItems={workItems}
          onChange={(upstream) => setInput({ ...input, upstream })}
        />
        <TaskSelect
          id="dependency-downstream"
          invalid={showValidation && input.downstream === ""}
          label="Depends on it"
          value={input.downstream}
          workItems={workItems}
          onChange={(downstream) => setInput({ ...input, downstream })}
        />
        <Field>
          <FieldLabel htmlFor="dependency-kind">Relationship</FieldLabel>
          <Select
            onValueChange={(kind) =>
              setInput({ ...input, kind: kind as DependencyKind })
            }
            value={input.kind}
          >
            <SelectTrigger id="dependency-kind">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="blocks">Must finish first</SelectItem>
                <SelectItem value="review_required">
                  Needs review first
                </SelectItem>
                <SelectItem value="contract">Shared contract</SelectItem>
                <SelectItem value="soft">Helpful order</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
          <FieldDescription>
            Only the first two types hold up the scheduler.
          </FieldDescription>
        </Field>
        <Field>
          <FieldLabel htmlFor="dependency-reason">Why</FieldLabel>
          <Input
            id="dependency-reason"
            required
            value={input.reason}
            onChange={(event) =>
              setInput({ ...input, reason: event.target.value })
            }
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="dependency-owner">Owner</FieldLabel>
          <Input
            id="dependency-owner"
            required
            value={input.owner}
            onChange={(event) =>
              setInput({ ...input, owner: event.target.value })
            }
          />
        </Field>
        <Field>
          <FieldLabel htmlFor="dependency-next-action">Next action</FieldLabel>
          <Input
            id="dependency-next-action"
            required
            value={input.nextAction}
            onChange={(event) =>
              setInput({ ...input, nextAction: event.target.value })
            }
          />
        </Field>
        <Button disabled={busy || workItems.length < 2} type="submit">
          Add relationship
        </Button>
      </FieldGroup>
    </form>
  );
}

function TaskSelect({
  id,
  invalid,
  label,
  value,
  workItems,
  onChange,
}: Readonly<{
  id: string;
  invalid: boolean;
  label: string;
  value: string;
  workItems: readonly WorkItem[];
  onChange: (value: string) => void;
}>) {
  return (
    <Field data-invalid={invalid || undefined}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Select onValueChange={onChange} value={value}>
        <SelectTrigger aria-invalid={invalid} id={id}>
          <SelectValue placeholder="Choose a task" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {workItems.map((workItem) => (
              <SelectItem key={workItem.id} value={workItem.id}>
                {workItem.title}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      {invalid && <FieldError>Select a task to continue.</FieldError>}
    </Field>
  );
}
