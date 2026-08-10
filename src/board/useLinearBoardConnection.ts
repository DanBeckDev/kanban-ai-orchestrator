import { useCallback, useRef, useState } from "react";

import { errorMessage } from "./useBoardOperation";
import type {
  BoardGateway,
  BoardSnapshot,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  QueueLinearCommentRequest,
} from "./types";

type RunBoardOperation = (
  operation: () => Promise<BoardSnapshot | undefined>,
) => Promise<void>;

type UseLinearBoardConnectionOptions = Readonly<{
  clearError: () => void;
  gateway: BoardGateway;
  run: RunBoardOperation;
  setBusy: (busy: boolean) => void;
}>;

export function useLinearBoardConnection({
  clearError,
  gateway,
  run,
  setBusy,
}: UseLinearBoardConnectionOptions) {
  const [connectionStatus, setConnectionStatus] =
    useState<LinearConnectionStatus>({ kind: "disconnected" });
  const [issues, setIssues] = useState<readonly LinearIssueSummary[]>([]);
  const latestConnectionRequest = useRef(0);
  const latestIssueRequest = useRef(0);

  const startConnectionRequest = useCallback(() => {
    latestConnectionRequest.current += 1;
    return latestConnectionRequest.current;
  }, []);

  const applyConnectionStatus = useCallback(
    (requestId: number, nextStatus: LinearConnectionStatus) => {
      if (requestId === latestConnectionRequest.current) {
        setConnectionStatus(nextStatus);
      }
    },
    [],
  );

  const applyConnectionFailure = useCallback(
    (requestId: number, error: unknown) => {
      applyConnectionStatus(requestId, {
        kind: "failed",
        message: errorMessage(error),
      });
    },
    [applyConnectionStatus],
  );

  const refreshConnectionStatus = useCallback(async () => {
    const requestId = startConnectionRequest();
    try {
      applyConnectionStatus(requestId, await gateway.linearConnectionStatus());
    } catch (connectionError) {
      applyConnectionFailure(requestId, connectionError);
    }
  }, [
    applyConnectionFailure,
    applyConnectionStatus,
    gateway,
    startConnectionRequest,
  ]);

  async function beginOAuth(configuration: LinearOAuthConfiguration) {
    const requestId = startConnectionRequest();
    setBusy(true);
    clearError();
    try {
      applyConnectionStatus(
        requestId,
        await gateway.beginLinearOAuth(configuration),
      );
    } catch (connectionError) {
      applyConnectionFailure(requestId, connectionError);
    } finally {
      setBusy(false);
    }
  }

  async function beginCommentAccess() {
    const requestId = startConnectionRequest();
    setBusy(true);
    clearError();
    try {
      applyConnectionStatus(
        requestId,
        await gateway.beginLinearCommentAccess(),
      );
    } catch (connectionError) {
      applyConnectionFailure(requestId, connectionError);
    } finally {
      setBusy(false);
    }
  }

  async function loadAssignedIssues() {
    const requestId = startConnectionRequest();
    latestIssueRequest.current += 1;
    const issueRequestId = latestIssueRequest.current;
    setBusy(true);
    clearError();
    try {
      const nextIssues = await gateway.linearAssignedIssues();
      if (issueRequestId === latestIssueRequest.current) {
        setIssues(nextIssues);
      }
    } catch (linearError) {
      applyConnectionFailure(requestId, linearError);
    } finally {
      setBusy(false);
    }
  }

  function resetIssues() {
    latestIssueRequest.current += 1;
    setIssues([]);
  }

  async function queueComment(request: QueueLinearCommentRequest) {
    await run(() => gateway.queueLinearComment(request));
  }

  async function deliverComment(outboxItemId: string) {
    await run(() => gateway.deliverLinearComment(outboxItemId));
  }

  async function refreshSharedFields(externalLinkId: string) {
    await run(() => gateway.syncLinearSharedFields(externalLinkId));
  }

  return {
    beginCommentAccess,
    beginOAuth,
    connectionStatus,
    deliverComment,
    issues,
    loadAssignedIssues,
    queueComment,
    refreshConnectionStatus,
    refreshSharedFields,
    resetIssues,
  };
}
