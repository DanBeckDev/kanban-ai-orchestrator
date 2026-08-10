import { useCallback } from "react";

import type { BoardSnapshot } from "./types";

type BoardOperation = () => Promise<BoardSnapshot | undefined>;

type UseBoardOperationOptions = Readonly<{
  onError: (message: string | undefined) => void;
  onSnapshot: (snapshot: BoardSnapshot) => void;
  setBusy: (busy: boolean) => void;
}>;

export function useBoardOperation({
  onError,
  onSnapshot,
  setBusy,
}: UseBoardOperationOptions): (operation: BoardOperation) => Promise<void> {
  return useCallback(
    async (operation: BoardOperation) => {
      setBusy(true);
      onError(undefined);
      try {
        const snapshot = await operation();
        if (snapshot !== undefined) onSnapshot(snapshot);
      } catch (error) {
        onError(errorMessage(error));
      } finally {
        setBusy(false);
      }
    },
    [onError, onSnapshot, setBusy],
  );
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
