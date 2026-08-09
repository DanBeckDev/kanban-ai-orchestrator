import {
  ArrowRightIcon,
  FolderGit2Icon,
  PlusIcon,
  TriangleAlertIcon,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

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
          <h2 id="board-library-title">Your boards</h2>
          <p>
            Pick up where you left off. Your board data stays on this device.
          </p>
        </div>
        {boards.length > 0 && (
          <Button
            disabled={busy}
            onClick={onCreateBoard}
            size="lg"
            type="button"
          >
            <PlusIcon data-icon="inline-start" />
            Create a board
          </Button>
        )}
      </div>
      {boards.length === 0 ? (
        <Empty className="board-library-empty">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <FolderGit2Icon />
            </EmptyMedia>
            <EmptyTitle>No boards yet</EmptyTitle>
            <EmptyDescription>
              Create a board from a repository when you are ready.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button disabled={busy} onClick={onCreateBoard} type="button">
              <PlusIcon data-icon="inline-start" />
              Create a board
            </Button>
          </EmptyContent>
        </Empty>
      ) : (
        <ol className="board-library-list">
          {boards.map((board) => (
            <li key={board.boardId}>
              <Card>
                <CardHeader>
                  <CardTitle>{board.name}</CardTitle>
                  <CardDescription>{board.repositoryName}</CardDescription>
                  <CardAction>
                    <Badge
                      variant={
                        board.repositoryAvailable ? "outline" : "destructive"
                      }
                    >
                      {board.repositoryAvailable
                        ? "Available locally"
                        : "Needs attention"}
                    </Badge>
                  </CardAction>
                </CardHeader>
                <CardContent className="board-library-details">
                  <p>{lastOpenedText(board.lastOpenedAt)}</p>
                  <p>{attentionText(board.attention)}</p>
                  {!board.repositoryAvailable && (
                    <p className="board-library-warning">
                      <TriangleAlertIcon aria-hidden="true" />
                      Repository unavailable. Restore the local folder, then try
                      again.
                    </p>
                  )}
                </CardContent>
                <CardFooter className="board-library-footer">
                  <Button
                    aria-label={
                      board.repositoryAvailable
                        ? `Open ${board.name}`
                        : `Retry ${board.name}`
                    }
                    disabled={busy}
                    onClick={() => onOpenBoard(board.boardId)}
                    type="button"
                    variant={board.repositoryAvailable ? "default" : "outline"}
                  >
                    {board.repositoryAvailable ? "Open board" : "Try again"}
                    <ArrowRightIcon data-icon="inline-end" />
                  </Button>
                </CardFooter>
              </Card>
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
