import { useState, type FormEvent } from "react";

import {
  agentProfilePresentation,
  noninteractiveCapabilitySummary,
} from "./agentProfilePresentation";
import { timestamp } from "./presentation";
import type {
  AgentProfile,
  ExecutionRole,
  StartExecutionRequest,
  WorkItem,
} from "./types";

type AgentLaunchFormProps = Readonly<{
  busy: boolean;
  profiles: readonly AgentProfile[];
  workItem: WorkItem;
  executionRole?: ExecutionRole;
  formLabel?: string;
  buttonLabel?: string;
  onStart: (request: StartExecutionRequest) => Promise<void>;
}>;

export function AgentLaunchForm({
  busy,
  profiles,
  workItem,
  executionRole = "implementation",
  formLabel = `Start agent for ${workItem.title}`,
  buttonLabel = "Start agent",
  onStart,
}: AgentLaunchFormProps) {
  const [profileName, setProfileName] = useState("");
  const [brief, setBrief] = useState(defaultBrief(workItem, executionRole));
  const selectedProfile = profiles.find(({ name }) => name === profileName);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onStart({
      executionId: `execution-${workItem.id}-${timestamp()}`,
      workItemId: workItem.id,
      agentProfileName: profileName,
      taskBrief: brief,
      executionRole,
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
      aria-label={formLabel}
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
        {buttonLabel}
      </button>
    </form>
  );
}

function defaultBrief(workItem: WorkItem, role: ExecutionRole): string {
  const acceptanceCriteria = workItem.acceptanceCriteria
    .map((criterion) => `- ${criterion}`)
    .join("\n");
  if (role === "independent_review") {
    return `Independently review ${workItem.title}. Do not edit files.\n\n${workItem.description}\n\nAcceptance criteria:\n${acceptanceCriteria}\n\nReview the current task worktree against the repository Clean Code requirements and the acceptance criteria. Report every actionable finding, including small correctness or maintainability defects. Finish only after the review is complete; a person records the structured decision on the board.`;
  }
  return `Implement ${workItem.title}.\n\n${workItem.description}\n\nAcceptance criteria:\n${acceptanceCriteria}`;
}
