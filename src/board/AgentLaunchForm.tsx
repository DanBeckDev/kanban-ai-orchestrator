import { useEffect, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

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
  defaultProfileName?: string;
  onStart: (request: StartExecutionRequest) => Promise<void>;
}>;

export function AgentLaunchForm({
  busy,
  profiles,
  workItem,
  executionRole = "implementation",
  formLabel = `Start agent for ${workItem.title}`,
  buttonLabel = "Start agent",
  defaultProfileName,
  onStart,
}: AgentLaunchFormProps) {
  const [profileName, setProfileName] = useState(defaultProfileName ?? "");
  const [brief, setBrief] = useState(defaultBrief(workItem, executionRole));
  const selectedProfile = profiles.find(({ name }) => name === profileName);
  const selectedProfileLabel =
    selectedProfile === undefined
      ? undefined
      : `${selectedProfile.name} · ${agentProfilePresentation(selectedProfile.kind).label}`;

  useEffect(() => {
    const fallbackProfileName = defaultProfileName ?? profiles[0]?.name ?? "";
    setProfileName((currentProfileName) =>
      profiles.some(({ name }) => name === currentProfileName)
        ? currentProfileName
        : fallbackProfileName,
    );
  }, [defaultProfileName, profiles]);

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
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="agent-profile">Agent profile</FieldLabel>
          <Select value={profileName} onValueChange={setProfileName}>
            <SelectTrigger id="agent-profile">
              <SelectValue placeholder="Select a profile">
                {selectedProfileLabel}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {profiles.map((profile) => (
                  <SelectItem key={profile.name} value={profile.name}>
                    {profile.name} ·{" "}
                    {agentProfilePresentation(profile.kind).label}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          {selectedProfile !== undefined && (
            <FieldDescription aria-live="polite">
              {agentProfilePresentation(selectedProfile.kind).label}:{" "}
              {noninteractiveCapabilitySummary}
            </FieldDescription>
          )}
        </Field>
        <Field>
          <FieldLabel htmlFor="task-brief">Task brief</FieldLabel>
          <Textarea
            id="task-brief"
            required
            value={brief}
            onChange={(event) => setBrief(event.target.value)}
          />
        </Field>
        <Button disabled={busy || profileName.length === 0} type="submit">
          {buttonLabel}
        </Button>
      </FieldGroup>
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
