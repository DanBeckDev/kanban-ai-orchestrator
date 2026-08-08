import { useState, type FormEvent } from "react";

import {
  agentProfilePresentation,
  noninteractiveCapabilitySummary,
} from "./agentProfilePresentation";
import { timestamp } from "./presentation";
import type { AgentProfile, StartExecutionRequest, WorkItem } from "./types";

type AgentLaunchFormProps = Readonly<{
  busy: boolean;
  profiles: readonly AgentProfile[];
  workItem: WorkItem;
  onStart: (request: StartExecutionRequest) => Promise<void>;
}>;

export function AgentLaunchForm({
  busy,
  profiles,
  workItem,
  onStart,
}: AgentLaunchFormProps) {
  const [profileName, setProfileName] = useState("");
  const [brief, setBrief] = useState(defaultBrief(workItem));
  const selectedProfile = profiles.find(({ name }) => name === profileName);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onStart({
      executionId: `execution-${workItem.id}-${timestamp()}`,
      workItemId: workItem.id,
      agentProfileName: profileName,
      taskBrief: brief,
    });
  }

  if (profiles.length === 0) {
    return (
      <p className="launch-hint">
        Save an agent profile before starting this task.
      </p>
    );
  }

  return (
    <form
      aria-label={`Start agent for ${workItem.title}`}
      className="agent-launch-form"
      onSubmit={submit}
    >
      <label>
        Agent profile
        <select
          required
          value={profileName}
          onChange={(event) => setProfileName(event.target.value)}
        >
          <option value="">Select profile</option>
          {profiles.map((profile) => (
            <option key={profile.name} value={profile.name}>
              {profile.name} · {agentProfilePresentation(profile.kind).label}
            </option>
          ))}
        </select>
      </label>
      {selectedProfile !== undefined && (
        <p aria-live="polite" className="agent-capability-summary">
          {agentProfilePresentation(selectedProfile.kind).label}:{" "}
          {noninteractiveCapabilitySummary}
        </p>
      )}
      <label>
        Task brief
        <textarea
          required
          value={brief}
          onChange={(event) => setBrief(event.target.value)}
        />
      </label>
      <button disabled={busy} type="submit">
        Start agent
      </button>
    </form>
  );
}

function defaultBrief(workItem: WorkItem): string {
  const acceptanceCriteria = workItem.acceptanceCriteria
    .map((criterion) => `- ${criterion}`)
    .join("\n");
  return `Implement ${workItem.title}.\n\n${workItem.description}\n\nAcceptance criteria:\n${acceptanceCriteria}`;
}
