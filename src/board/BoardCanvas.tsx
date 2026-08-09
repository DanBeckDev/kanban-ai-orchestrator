import { ClipboardPlusIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

import { CompactWorkItemCard } from "./CompactWorkItemCard";
import { boardColumns, workItemsForColumn } from "./presentation";
import type { BoardSnapshot } from "./types";

type BoardCanvasProps = Readonly<{
  snapshot: BoardSnapshot;
  onCreateTask: () => void;
  onOpenTask: (workItemId: string) => void;
  onPlanWork: () => void;
}>;

export function BoardCanvas({
  snapshot,
  onCreateTask,
  onOpenTask,
  onPlanWork,
}: BoardCanvasProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const workItemTitles = new Map(
    workItems.map((workItem) => [workItem.id, workItem.title]),
  );

  if (workItems.length === 0) {
    return (
      <Empty className="board-empty-state">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <ClipboardPlusIcon />
          </EmptyMedia>
          <EmptyTitle>No work is on this board yet</EmptyTitle>
          <EmptyDescription>
            Start with the outcome and review the proposed work, or add one task
            yourself.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button onClick={onPlanWork} type="button">
            Describe an outcome
          </Button>
          <Button onClick={onCreateTask} type="button" variant="outline">
            Create a task
          </Button>
        </EmptyContent>
      </Empty>
    );
  }

  return (
    <section aria-label="Kanban board" className="kanban-board">
      {boardColumns.map((column) => {
        const cards = workItemsForColumn(snapshot, column);
        return (
          <section
            aria-labelledby={`${column.id}-column`}
            className="board-column"
            key={column.id}
          >
            <div className="board-column-heading">
              <h3 id={`${column.id}-column`}>{column.label}</h3>
              <span>
                {cards.length} {cards.length === 1 ? "task" : "tasks"}
              </span>
            </div>
            <div className="card-stack">
              {cards.map((workItem) => (
                <CompactWorkItemCard
                  key={workItem.id}
                  snapshot={snapshot}
                  workItemTitles={workItemTitles}
                  workItem={workItem}
                  onOpen={onOpenTask}
                />
              ))}
            </div>
            {cards.length === 0 && (
              <p className="empty-column-copy">Nothing here</p>
            )}
          </section>
        );
      })}
    </section>
  );
}
