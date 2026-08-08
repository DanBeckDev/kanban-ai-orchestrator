import { useState, type FormEvent } from "react";

import type { AgentProfile } from "./types";

type AgentProfileFormProps = Readonly<{
  busy: boolean;
  profiles: readonly AgentProfile[];
  onSave: (profile: AgentProfile) => Promise<void>;
}>;

export function AgentProfileForm({
  busy,
  profiles,
  onSave,
}: AgentProfileFormProps) {
  const [name, setName] = useState("");
  const [program, setProgram] = useState("");
  const [argumentsText, setArgumentsText] = useState("");

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSave({
      name,
      program,
      arguments: argumentsText
        .split("\n")
        .map((argument) => argument.trim())
        .filter(Boolean),
    });
    setName("");
    setProgram("");
    setArgumentsText("");
  }

  return (
    <section className="panel form-panel">
      <div>
        <h3>Agent profiles</h3>
        <p className="field-hint">
          Run one approved executable directly. It receives the task brief on
          stdin and emits normalized JSONL events on stdout.
        </p>
      </div>
      {profiles.length > 0 && (
        <ul className="profile-list" aria-label="Saved agent profiles">
          {profiles.map((profile) => (
            <li key={profile.name}>
              <strong>{profile.name}</strong>
              <span>{profile.program}</span>
            </li>
          ))}
        </ul>
      )}
      <form aria-label="Save agent profile" onSubmit={submit}>
        <label>
          Profile name
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
            placeholder="agent-worker"
            value={program}
            onChange={(event) => setProgram(event.target.value)}
          />
        </label>
        <label>
          Arguments (one per line)
          <textarea
            placeholder="--jsonl"
            value={argumentsText}
            onChange={(event) => setArgumentsText(event.target.value)}
          />
        </label>
        <button disabled={busy} type="submit">
          Save profile
        </button>
      </form>
    </section>
  );
}
