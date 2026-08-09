import { useState } from "react";

const defaultAgentProfileStorageKey =
  "kanban-ai-orchestrator.default-agent-profile";

export function useDefaultAgentProfileName() {
  const [defaultAgentProfileName, setDefaultAgentProfileName] = useState(
    savedDefaultAgentProfileName,
  );

  function selectDefaultAgentProfile(profileName: string) {
    saveDefaultAgentProfileName(profileName);
    setDefaultAgentProfileName(profileName);
  }

  return { defaultAgentProfileName, selectDefaultAgentProfile };
}

export function savedDefaultAgentProfileName(): string | undefined {
  try {
    return (
      window.localStorage.getItem(defaultAgentProfileStorageKey) ?? undefined
    );
  } catch {
    return undefined;
  }
}

export function saveDefaultAgentProfileName(profileName: string): void {
  try {
    window.localStorage.setItem(defaultAgentProfileStorageKey, profileName);
  } catch {
    // Preference persistence is optional; profile safety remains daemon-owned.
  }
}
