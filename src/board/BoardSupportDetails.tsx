import type { BoardSnapshot } from "./types";

type BoardSupportDetailsProps = Readonly<{
  board: BoardSnapshot["board"];
}>;

export function BoardSupportDetails({ board }: BoardSupportDetailsProps) {
  return (
    <details className="board-support-details">
      <summary>Support details</summary>
      <dl>
        <div>
          <dt>Board ID</dt>
          <dd>{board.id}</dd>
        </div>
        <div>
          <dt>Project ID</dt>
          <dd>{board.projectId}</dd>
        </div>
      </dl>
    </details>
  );
}
