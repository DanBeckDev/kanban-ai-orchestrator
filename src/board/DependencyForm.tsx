import { useState, type FormEvent } from "react";

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

  async function addDependency(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
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
  }

  return (
    <form
      aria-labelledby="add-dependency-title"
      className="panel form-panel"
      onSubmit={addDependency}
    >
      <h3 id="add-dependency-title">Add dependency</h3>
      <WorkItemSelect
        label="Upstream task"
        value={input.upstream}
        workItems={workItems}
        onChange={(upstream) => setInput({ ...input, upstream })}
      />
      <WorkItemSelect
        label="Downstream task"
        value={input.downstream}
        workItems={workItems}
        onChange={(downstream) => setInput({ ...input, downstream })}
      />
      <label>
        Type
        <select
          value={input.kind}
          onChange={(event) =>
            setInput({ ...input, kind: event.target.value as DependencyKind })
          }
        >
          <option value="blocks">Blocks</option>
          <option value="review_required">Review required</option>
          <option value="contract">Contract</option>
          <option value="soft">Soft</option>
        </select>
      </label>
      <label>
        Reason
        <input
          required
          value={input.reason}
          onChange={(event) =>
            setInput({ ...input, reason: event.target.value })
          }
        />
      </label>
      <label>
        Owner
        <input
          required
          value={input.owner}
          onChange={(event) =>
            setInput({ ...input, owner: event.target.value })
          }
        />
      </label>
      <label>
        Next action
        <input
          required
          value={input.nextAction}
          onChange={(event) =>
            setInput({ ...input, nextAction: event.target.value })
          }
        />
      </label>
      <button disabled={busy || workItems.length < 2} type="submit">
        Add dependency
      </button>
    </form>
  );
}

function WorkItemSelect({
  label,
  value,
  workItems,
  onChange,
}: Readonly<{
  label: string;
  value: string;
  workItems: readonly WorkItem[];
  onChange: (value: string) => void;
}>) {
  return (
    <label>
      {label}
      <select
        required
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        <option value="">Select task</option>
        {workItems.map((workItem) => (
          <option key={workItem.id} value={workItem.id}>
            {workItem.title} ({workItem.id})
          </option>
        ))}
      </select>
    </label>
  );
}
