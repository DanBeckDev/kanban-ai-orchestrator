import { useState, type FormEvent } from "react";

import {
  agentProfilePresentation,
  agentProfilePresentations,
  noninteractiveCapabilitySummary,
} from "./agentProfilePresentation";
import type { AgentProfile, AgentProfileKind } from "./types";

type AgentProfileFormProps = Readonly<{
  busy: boolean;
  profiles: readonly AgentProfile[];
  onSave: (profile: AgentProfile) => Promise<boolean>;
}>;

export function AgentProfileForm({
  busy,
  profiles,
  onSave,
}: AgentProfileFormProps) {
  const [kind, setKind] = useState<AgentProfileKind>("structured_process");
  const [name, setName] = useState("");
  const [program, setProgram] = useState("agent-worker");
  const [argumentsText, setArgumentsText] = useState("");
  const selectedProfile = agentProfilePresentation(kind);

  function selectKind(nextKind: AgentProfileKind) {
    setKind(nextKind);
    setProgram(agentProfilePresentation(nextKind).defaultProgram);
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const saved = await onSave({
      name,
      kind,
      program,
      arguments: argumentsText
        .split("\n")
        .map((argument) => argument.trim())
        .filter(Boolean),
    });
    if (!saved) return;
    setKind("structured_process");
    setName("");
    setProgram("agent-worker");
    setArgumentsText("");
  }

  return (
    <section className="panel form-panel">
      <div>
        <h3>Agent profiles</h3>
        <p className="field-hint">
          Choose the adapter that owns the executable protocol. The daemon
          preserves only safe, normalized lifecycle summaries.
        </p>
      </div>
      {profiles.length > 0 && (
        <ul className="profile-list" aria-label="Saved agent profiles">
          {profiles.map((profile) => (
            <li key={profile.name}>
              <strong>{profile.name}</strong>
              <span>
                {agentProfilePresentation(profile.kind).label} ·{" "}
                {profile.program}
              </span>
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
          Adapter
          <select
            value={kind}
            onChange={(event) =>
              selectKind(event.target.value as AgentProfileKind)
            }
          >
            {agentProfilePresentations.map((presentation) => (
              <option key={presentation.kind} value={presentation.kind}>
                {presentation.label}
              </option>
            ))}
          </select>
        </label>
        <p className="field-hint">{selectedProfile.protocolSummary}</p>
        <p className="agent-capability-summary">
          {noninteractiveCapabilitySummary}
        </p>
        <label>
          Program
          <input
            required
            placeholder={selectedProfile.defaultProgram}
            value={program}
            onChange={(event) => setProgram(event.target.value)}
          />
        </label>
        <label>
          Arguments (one per line)
          <textarea
            placeholder={selectedProfile.argumentHint}
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
