import { useState, type FormEvent } from "react";

import type { PlannerProfile } from "./types";

type PlannerProfileFormProps = Readonly<{
  busy: boolean;
  profiles: readonly PlannerProfile[];
  onSave: (profile: PlannerProfile) => Promise<void>;
}>;

export function PlannerProfileForm({
  busy,
  profiles,
  onSave,
}: PlannerProfileFormProps) {
  const [name, setName] = useState("");
  const [program, setProgram] = useState("planner-bridge");
  const [argumentsText, setArgumentsText] = useState("");
  const [saveError, setSaveError] = useState<string>();

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await onSave({
        name,
        program,
        arguments: argumentsText
          .split("\n")
          .map((argument) => argument.trim())
          .filter(Boolean),
      });
      setSaveError(undefined);
      setName("");
      setProgram("planner-bridge");
      setArgumentsText("");
    } catch (error) {
      setSaveError(errorMessage(error));
    }
  }

  return (
    <section className="panel form-panel">
      <div>
        <h3>Organiser connections</h3>
        <p className="field-hint">
          Use an existing provider bridge to generate plans. Kanban sends it one
          outcome and keeps only the proposed tasks and dependencies, not the
          conversation.
        </p>
      </div>
      {profiles.length > 0 && (
        <ul aria-label="Saved organiser connections" className="profile-list">
          {profiles.map((profile) => (
            <li key={profile.name}>
              <strong>{profile.name}</strong>
              <span>{profile.program}</span>
            </li>
          ))}
        </ul>
      )}
      <form aria-label="Save organiser connection" onSubmit={submit}>
        <label>
          Connection name
          <input
            required
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        </label>
        <label>
          Program
          <input
            required
            placeholder="planner-bridge"
            value={program}
            onChange={(event) => setProgram(event.target.value)}
          />
        </label>
        <label>
          Arguments (one per line)
          <textarea
            placeholder="--model&#10;your-provider-model"
            value={argumentsText}
            onChange={(event) => setArgumentsText(event.target.value)}
          />
        </label>
        {saveError !== undefined && (
          <p className="inline-error" role="alert">
            {saveError}
          </p>
        )}
        <button disabled={busy} type="submit">
          Save organiser connection
        </button>
      </form>
    </section>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
