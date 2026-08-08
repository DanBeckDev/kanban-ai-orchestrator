import { useState } from "react";
import { BoardSetup, type CreateBoardInput } from "./BoardSetup";
import { BoardView } from "./BoardView";
import { tauriBoardGateway } from "./gateway";
import type {
  AddDependencyRequest,
  BoardGateway,
  BoardSnapshot,
  CreateWorkItemRequest,
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
      return gateway.createBoard({
        boardId: input.boardId,
        projectId: input.projectId,
        name: input.boardName,
      });
    });
  }

  async function openBoard(boardId: string) {
    await run(() => gateway.boardSnapshot(boardId));
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
          onAddDependency={addDependency}
          onCreateWorkItem={createWorkItem}
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
