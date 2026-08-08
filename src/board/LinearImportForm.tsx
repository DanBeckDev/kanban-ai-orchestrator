import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type {
  ExternalConnectionMode,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  WorkItem,
} from "./types";

type LinearImportFormProps = Readonly<{
  busy: boolean;
  connectionStatus: LinearConnectionStatus;
  issues: readonly LinearIssueSummary[];
  workItems: readonly WorkItem[];
  onImportBlocker: (request: ImportLinearBlockerRequest) => Promise<void>;
  onImportIssue: (request: ImportLinearIssueRequest) => Promise<void>;
  onLoadIssues: () => Promise<void>;
}>;

export function LinearImportForm({
  busy,
  connectionStatus,
  issues,
  workItems,
  onImportBlocker,
  onImportIssue,
  onLoadIssues,
}: LinearImportFormProps) {
  const [workItemId, setWorkItemId] = useState("");
  const [issueId, setIssueId] = useState("");
  const [identifier, setIdentifier] = useState("");
  const [url, setUrl] = useState("");
  const [mode, setMode] = useState<ExternalConnectionMode>("read_only");
  const [upstreamIssueId, setUpstreamIssueId] = useState("");
  const [downstreamIssueId, setDownstreamIssueId] = useState("");
  const [reason, setReason] = useState("");
  const [owner, setOwner] = useState("Linear");
  const [nextAction, setNextAction] = useState("");

  async function importIssue(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const importedAt = timestamp();
    await onImportIssue({
      externalLinkId: `linear-link-${issueId}-${importedAt}`,
      workItemId,
      issueId,
      displayIdentifier: identifier,
      url,
      connectionMode: mode,
    });
  }

  async function importBlocker(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const createdAt = timestamp();
    await onImportBlocker({
      dependencyId: `linear-blocker-${upstreamIssueId}-${downstreamIssueId}-${createdAt}`,
      upstreamIssueId,
      downstreamIssueId,
      reason,
      owner,
      nextAction,
      createdAt,
    });
  }

  function selectIssue(issue: LinearIssueSummary) {
    setIssueId(issue.id);
    setIdentifier(issue.identifier);
    setUrl(issue.url);
  }

  return (
    <section className="linear-import-panel">
      <h3>Linear import</h3>
      <p>
        Load your assigned Linear issues after connecting, or link an immutable
        issue UUID copied from Linear.
      </p>
      <button
        disabled={busy || connectionStatus.kind !== "connected"}
        type="button"
        onClick={onLoadIssues}
      >
        Load my assigned Linear issues
      </button>
      {issues.length > 0 && (
        <ul aria-label="Assigned Linear issues">
          {issues.map((issue) => (
            <li key={issue.id}>
              <button type="button" onClick={() => selectIssue(issue)}>
                Use {issue.identifier}: {issue.title}
              </button>
            </li>
          ))}
        </ul>
      )}
      <form aria-label="Import Linear issue" onSubmit={importIssue}>
        <label>
          Local task
          <select
            required
            value={workItemId}
            onChange={(event) => setWorkItemId(event.target.value)}
          >
            <option value="">Select task</option>
            {workItems.map((workItem) => (
              <option key={workItem.id} value={workItem.id}>
                {workItem.title}
              </option>
            ))}
          </select>
        </label>
        <TextInput
          label="Linear issue UUID"
          value={issueId}
          onChange={setIssueId}
        />
        <TextInput
          label="Linear issue identifier"
          value={identifier}
          onChange={setIdentifier}
        />
        <TextInput
          label="Linear issue URL"
          type="url"
          value={url}
          onChange={setUrl}
        />
        <label>
          Connection mode
          <select
            value={mode}
            onChange={(event) =>
              setMode(event.target.value as ExternalConnectionMode)
            }
          >
            <option value="read_only">Read only</option>
            <option value="linked_execution">Linked execution</option>
          </select>
        </label>
        <button disabled={busy} type="submit">
          Import Linear issue
        </button>
      </form>
      <form aria-label="Import Linear blocker" onSubmit={importBlocker}>
        <h4>Import blocker</h4>
        <TextInput
          label="Upstream Linear issue UUID"
          value={upstreamIssueId}
          onChange={setUpstreamIssueId}
        />
        <TextInput
          label="Downstream Linear issue UUID"
          value={downstreamIssueId}
          onChange={setDownstreamIssueId}
        />
        <TextInput label="Reason" value={reason} onChange={setReason} />
        <TextInput label="Owner" value={owner} onChange={setOwner} />
        <TextInput
          label="Next action"
          value={nextAction}
          onChange={setNextAction}
        />
        <button disabled={busy} type="submit">
          Import blocker
        </button>
      </form>
    </section>
  );
}

function TextInput({
  label,
  type = "text",
  value,
  onChange,
}: Readonly<{
  label: string;
  type?: "text" | "url";
  value: string;
  onChange: (value: string) => void;
}>) {
  return (
    <label>
      {label}
      <input
        required
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}
