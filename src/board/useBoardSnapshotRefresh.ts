import { useEffect } from "react";

import type { BoardGateway, BoardSnapshot } from "./types";

type UseBoardSnapshotRefreshOptions = Readonly<{
  boardId?: string;
  gateway: BoardGateway;
  isAwaitingLinearAuthorization: boolean;
  onLinearStatusRefresh: () => void;
  onSnapshot: (snapshot: BoardSnapshot) => void;
}>;

export function useBoardSnapshotRefresh({
  boardId,
  gateway,
  isAwaitingLinearAuthorization,
  onLinearStatusRefresh,
  onSnapshot,
}: UseBoardSnapshotRefreshOptions) {
  useEffect(() => {
    if (boardId === undefined) return undefined;
    const refresh = () => {
      void gateway.boardSnapshot(boardId).then(onSnapshot).catch(ignoreFailure);
      if (isAwaitingLinearAuthorization) onLinearStatusRefresh();
    };
    const intervalId = window.setInterval(refresh, 1_000);
    return () => window.clearInterval(intervalId);
  }, [
    boardId,
    gateway,
    isAwaitingLinearAuthorization,
    onLinearStatusRefresh,
    onSnapshot,
  ]);
}

function ignoreFailure() {
  // A later refresh can recover transient daemon availability.
}
