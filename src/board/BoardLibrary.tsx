import type { BoardLibraryEntry } from "./types";

type BoardLibraryProps = Readonly<{
  boards: readonly BoardLibraryEntry[];
  busy: boolean;
  onCreateBoard: () => void;
  onOpenBoard: (boardId: string) => void;
}>;

export function BoardLibrary({
  boards,
  busy,
  onCreateBoard,
  onOpenBoard,
}: BoardLibraryProps) {
  return (
    <section aria-labelledby="board-library-title" className="board-library">
      <div className="board-library-heading">
        <div>
          <p className="eyebrow">Local-first board</p>
          <h2 id="board-library-title">Your boards</h2>
          <p>
            Pick up where you left off. Your board data stays on this device.
          </p>
        </div>
        <button disabled={busy} onClick={onCreateBoard} type="button">
          Create a board
        </button>
      </div>
      {boards.length === 0 ? (
        <div className="board-library-empty panel">
          <h3>No local boards yet</h3>
          <p>Create a board from a repository when you are ready.</p>
        </div>
      ) : (
        <ol className="board-library-list">
          {boards.map((board) => (
            <li className="board-library-entry" key={board.boardId}>
              <div>
                <h3>{board.name}</h3>
                <p>{board.repositoryName}</p>
                <p>{lastOpenedText(board.lastOpenedAt)}</p>
                <p>{attentionText(board.attention)}</p>
                {!board.repositoryAvailable && (
                  <p className="board-library-warning">
                    Repository unavailable. Restore the local folder, then try
                    again.
                  </p>
                )}
              </div>
              <button
                disabled={busy}
                onClick={() => onOpenBoard(board.boardId)}
                type="button"
              >
                {board.repositoryAvailable ? "Continue" : "Try again"}
              </button>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

function attentionText(attention: BoardLibraryEntry["attention"]): string {
  const decisions = countText(
    attention.needsAttentionCount,
    "decision needs",
    "decisions need",
  );
  const agents = countText(
    attention.activeWorkItemCount,
    "agent is",
    "agents are",
  );
  return `${decisions} your attention · ${agents} working`;
}

function countText(count: number, singular: string, plural: string): string {
  return count === 1 ? `1 ${singular}` : `${count} ${plural}`;
}

function lastOpenedText(lastOpenedAt: string | null): string {
  if (lastOpenedAt === null) return "Not opened yet";
  return `Last opened ${new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(lastOpenedAt))}`;
}
