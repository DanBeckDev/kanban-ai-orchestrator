import { useCallback, useEffect, useState } from "react";

export type CoordinationMode = "manual" | "autonomous";

const coordinationModeStoragePrefix =
  "kanban-ai-orchestrator.coordination-mode.";

export function useCoordinationMode(boardId: string) {
  const [mode, setMode] = useState<CoordinationMode>(() => savedMode(boardId));

  useEffect(() => {
    setMode(savedMode(boardId));
  }, [boardId]);

  const selectMode = useCallback(
    (nextMode: CoordinationMode) => {
      saveMode(boardId, nextMode);
      setMode(nextMode);
    },
    [boardId],
  );

  return { mode, selectMode };
}

function savedMode(boardId: string): CoordinationMode {
  try {
    return window.localStorage.getItem(storageKey(boardId)) === "autonomous"
      ? "autonomous"
      : "manual";
  } catch {
    return "manual";
  }
}

function saveMode(boardId: string, mode: CoordinationMode): void {
  try {
    window.localStorage.setItem(storageKey(boardId), mode);
  } catch {
    // This is a convenience preference. Coordination authority stays in the daemon.
  }
}

function storageKey(boardId: string): string {
  return `${coordinationModeStoragePrefix}${boardId}`;
}
