import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import {
  commentsAreAuthorized,
  connectedLinearDescription,
} from "./linearConnectionPresentation";
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
  const linkedExecutionAvailable = commentsAreAuthorized(connectionStatus);

  async function importIssue(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const importedAt = timestamp();
    await onImportIssue({
      externalLinkId: `linear-link-${issueId}-${importedAt}`,
      workItemId,
      issueId,
      displayIdentifier: identifier,
      url,
      connectionMode: linkedExecutionAvailable ? mode : "read_only",
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
    <section
      aria-labelledby="linear-import-title"
      className="linear-import-panel"
    >
      <div>
        <h3 id="linear-import-title">Linear import</h3>
        <p>
          Load assigned issues after connecting, or record a local read-only
          reference from an immutable Linear issue ID.
        </p>
      </div>
      <Button
        disabled={busy || connectionStatus.kind !== "connected"}
        onClick={onLoadIssues}
        type="button"
        variant="outline"
      >
        Load my assigned Linear issues
      </Button>
      {connectionStatus.kind !== "connected" && (
        <p>{connectedLinearDescription(connectionStatus)}</p>
      )}
      {issues.length > 0 && (
        <ul aria-label="Assigned Linear issues">
          {issues.map((issue) => (
            <li key={issue.id}>
              <Button
                onClick={() => selectIssue(issue)}
                size="sm"
                type="button"
                variant="outline"
              >
                Use {issue.identifier}: {issue.title}
              </Button>
            </li>
          ))}
        </ul>
      )}
      <form aria-label="Import Linear issue" onSubmit={importIssue}>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="linear-work-item">Local task</FieldLabel>
            <select
              id="linear-work-item"
              name="linear-work-item"
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
          </Field>
          <IssueFields
            identifier={identifier}
            issueId={issueId}
            url={url}
            onIdentifierChange={setIdentifier}
            onIssueIdChange={setIssueId}
            onUrlChange={setUrl}
          />
          <Field>
            <FieldTitle id="linear-connection-mode-label">
              How should this link work?
            </FieldTitle>
            <ToggleGroup
              aria-labelledby="linear-connection-mode-label"
              onValueChange={(value) => {
                if (value === "read_only" || value === "linked_execution") {
                  setMode(value);
                }
              }}
              type="single"
              value={mode}
            >
              <ToggleGroupItem value="read_only">Read only</ToggleGroupItem>
              <ToggleGroupItem
                disabled={!linkedExecutionAvailable}
                value="linked_execution"
              >
                Linked execution
              </ToggleGroupItem>
            </ToggleGroup>
            <FieldDescription>
              {linkedExecutionAvailable
                ? "Linked execution can prepare a public update. You still review and send every update yourself."
                : "Read-only links never send data. Enable manually sent comments above to choose linked execution."}
            </FieldDescription>
          </Field>
          <Button disabled={busy} type="submit">
            Import Linear issue
          </Button>
        </FieldGroup>
      </form>
      <form aria-label="Import Linear blocker" onSubmit={importBlocker}>
        <FieldGroup>
          <h4>Import blocker</h4>
          <BlockerFields
            downstreamIssueId={downstreamIssueId}
            nextAction={nextAction}
            owner={owner}
            reason={reason}
            upstreamIssueId={upstreamIssueId}
            onDownstreamIssueIdChange={setDownstreamIssueId}
            onNextActionChange={setNextAction}
            onOwnerChange={setOwner}
            onReasonChange={setReason}
            onUpstreamIssueIdChange={setUpstreamIssueId}
          />
          <Button disabled={busy} type="submit" variant="outline">
            Import blocker
          </Button>
        </FieldGroup>
      </form>
    </section>
  );
}

function IssueFields({
  identifier,
  issueId,
  url,
  onIdentifierChange,
  onIssueIdChange,
  onUrlChange,
}: Readonly<{
  identifier: string;
  issueId: string;
  url: string;
  onIdentifierChange: (value: string) => void;
  onIssueIdChange: (value: string) => void;
  onUrlChange: (value: string) => void;
}>) {
  return (
    <>
      <TextInput
        id="linear-issue-id"
        label="Linear issue UUID"
        value={issueId}
        onChange={onIssueIdChange}
      />
      <TextInput
        id="linear-issue-identifier"
        label="Linear issue identifier"
        value={identifier}
        onChange={onIdentifierChange}
      />
      <TextInput
        id="linear-issue-url"
        label="Linear issue URL"
        type="url"
        value={url}
        onChange={onUrlChange}
      />
    </>
  );
}

function BlockerFields({
  downstreamIssueId,
  nextAction,
  owner,
  reason,
  upstreamIssueId,
  onDownstreamIssueIdChange,
  onNextActionChange,
  onOwnerChange,
  onReasonChange,
  onUpstreamIssueIdChange,
}: Readonly<{
  downstreamIssueId: string;
  nextAction: string;
  owner: string;
  reason: string;
  upstreamIssueId: string;
  onDownstreamIssueIdChange: (value: string) => void;
  onNextActionChange: (value: string) => void;
  onOwnerChange: (value: string) => void;
  onReasonChange: (value: string) => void;
  onUpstreamIssueIdChange: (value: string) => void;
}>) {
  return (
    <>
      <TextInput
        id="linear-upstream-issue-id"
        label="Upstream Linear issue UUID"
        value={upstreamIssueId}
        onChange={onUpstreamIssueIdChange}
      />
      <TextInput
        id="linear-downstream-issue-id"
        label="Downstream Linear issue UUID"
        value={downstreamIssueId}
        onChange={onDownstreamIssueIdChange}
      />
      <TextInput
        id="linear-blocker-reason"
        label="Reason"
        value={reason}
        onChange={onReasonChange}
      />
      <TextInput
        id="linear-blocker-owner"
        label="Owner"
        value={owner}
        onChange={onOwnerChange}
      />
      <TextInput
        id="linear-blocker-next-action"
        label="Next action"
        value={nextAction}
        onChange={onNextActionChange}
      />
    </>
  );
}

function TextInput({
  id,
  label,
  type = "text",
  value,
  onChange,
}: Readonly<{
  id: string;
  label: string;
  type?: "text" | "url";
  value: string;
  onChange: (value: string) => void;
}>) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Input
        autoComplete="off"
        id={id}
        name={id}
        required
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </Field>
  );
}
