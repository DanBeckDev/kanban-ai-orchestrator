import { useEffect, useState } from "react";
import { BoardSetup, type CreateBoardInput } from "./BoardSetup";
import { BoardView } from "./BoardView";
import { tauriBoardGateway } from "./gateway";
import type {
  AddDependencyRequest,
  AgentProfile,
  BoardGateway,
  BoardSnapshot,
  CreateWorkItemRequest,
  RecordReviewCheckRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
} from "./types";

type BoardWorkspaceProps = Readonly<{
  gateway?: BoardGateway;
}>;

export function BoardWorkspace({
  gateway = tauriBoardGateway,
}: BoardWorkspaceProps) {
  const [snapshot, setSnapshot] = useState<BoardSnapshot>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [agentProfiles, setAgentProfiles] = useState<readonly AgentProfile[]>(
    [],
  );

  async function run(operation: () => Promise<BoardSnapshot | undefined>) {
    setBusy(true);
    setError(undefined);
    try {
      const updatedSnapshot = await operation();
      if (updatedSnapshot !== undefined) {
        setSnapshot(updatedSnapshot);
      }
    } catch (operationError) {
      setError(errorMessage(operationError));
    } finally {
      setBusy(false);
    }
  }

  async function createBoard(input: CreateBoardInput) {
    await run(async () => {
      await gateway.createProject({
        projectId: input.projectId,
        name: input.projectName,
        repositoryPath: input.repositoryPath,
        baseRef: input.baseRef,
        policySetId: input.policySetId,
      });
      const boardSnapshot = await gateway.createBoard({
        boardId: input.boardId,
        projectId: input.projectId,
        name: input.boardName,
      });
      setAgentProfiles(await gateway.agentProfiles());
      return boardSnapshot;
    });
  }

  async function openBoard(boardId: string) {
    await run(async () => {
      const boardSnapshot = await gateway.boardSnapshot(boardId);
      setAgentProfiles(await gateway.agentProfiles());
      return boardSnapshot;
    });
  }

  async function createWorkItem(request: CreateWorkItemRequest) {
    await run(() => gateway.createWorkItem(request));
  }

  async function addDependency(request: AddDependencyRequest) {
    await run(() => gateway.addDependency(request));
  }

  async function transitionWorkItem(request: TransitionWorkItemRequest) {
    await run(() => gateway.transitionWorkItem(request));
  }

  async function saveAgentProfile(profile: AgentProfile) {
    setBusy(true);
    setError(undefined);
    try {
      await gateway.saveAgentProfile(profile);
      setAgentProfiles(await gateway.agentProfiles());
    } catch (operationError) {
      setError(errorMessage(operationError));
    } finally {
      setBusy(false);
    }
  }

  async function startExecution(request: StartExecutionRequest) {
    await run(() => gateway.startExecution(request));
  }

  async function stopExecution(executionId: string) {
    await run(() => gateway.stopExecution(executionId));
  }

  async function recordReviewCheck(request: RecordReviewCheckRequest) {
    await run(() => gateway.recordReviewCheck(request));
  }

  const boardId = snapshot?.board.id;
  useEffect(() => {
    if (boardId === undefined) return undefined;
    const refresh = () => {
      void gateway
        .boardSnapshot(boardId)
        .then(setSnapshot)
        .catch(() => undefined);
    };
    const intervalId = window.setInterval(refresh, 1_000);
    return () => window.clearInterval(intervalId);
  }, [boardId, gateway]);

  return (
    <section className="board-shell">
      {error !== undefined && (
        <div aria-live="polite" className="error-notice" role="alert">
          <strong>The local daemon rejected that request.</strong> {error}
        </div>
      )}
      {snapshot === undefined ? (
        <BoardSetup busy={busy} onCreate={createBoard} onOpen={openBoard} />
      ) : (
        <BoardView
          busy={busy}
          agentProfiles={agentProfiles}
          onAddDependency={addDependency}
          onCreateWorkItem={createWorkItem}
          onSaveAgentProfile={saveAgentProfile}
          onStartExecution={startExecution}
          onStopExecution={stopExecution}
          onRecordReviewCheck={recordReviewCheck}
          onTransition={transitionWorkItem}
          snapshot={snapshot}
        />
      )}
    </section>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
