import { ClipboardPlusIcon } from "lucide-react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Badge } from "@/components/ui/badge";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Button } from "@/components/ui/button";

import { CompactWorkItemCard } from "./CompactWorkItemCard";
import { boardColumns, workItemsForColumn } from "./presentation";
import type { BoardSnapshot } from "./types";

type BoardCanvasProps = Readonly<{
  snapshot: BoardSnapshot;
  onExplainDependencies: (workItemId: string) => void;
  onGoHome: () => void;
  onOpenTask: (workItemId: string) => void;
}>;

export function BoardCanvas({
  snapshot,
  onExplainDependencies,
  onGoHome,
  onOpenTask,
}: BoardCanvasProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);

  return (
    <section aria-labelledby="tickets-title" className="tickets-view">
      <header className="tickets-view-header">
        <div>
          <p className="eyebrow">Tickets</p>
          <h3 id="tickets-title">Keep work moving</h3>
          <p>
            Open a ticket for detail, or use Home to ask the orchestrator for a
            plan.
          </p>
        </div>
      </header>
      {workItems.length === 0 ? (
        <Empty className="board-empty-state">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ClipboardPlusIcon />
            </EmptyMedia>
            <EmptyTitle>No tasks yet</EmptyTitle>
            <EmptyDescription>
              Describe the outcome above, or create a task yourself.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={onGoHome} type="button" variant="outline">
              Go to Home
            </Button>
          </EmptyContent>
        </Empty>
      ) : (
        <TicketLanes
          snapshot={snapshot}
          onExplainDependencies={onExplainDependencies}
          onOpenTask={onOpenTask}
        />
      )}
    </section>
  );
}

function TicketLanes({
  snapshot,
  onExplainDependencies,
  onOpenTask,
}: Readonly<{
  snapshot: BoardSnapshot;
  onExplainDependencies: (workItemId: string) => void;
  onOpenTask: (workItemId: string) => void;
}>) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const workItemTitles = new Map(
    workItems.map((workItem) => [workItem.id, workItem.title]),
  );

  return (
    <section aria-label="Ticket lanes" className="ticket-lanes">
      <Accordion
        defaultValue={boardColumns.map(({ id }) => id)}
        type="multiple"
      >
        {boardColumns.map((column) => {
          const cards = workItemsForColumn(snapshot, column);
          return (
            <AccordionItem
              className="ticket-lane"
              key={column.id}
              value={column.id}
            >
              <AccordionTrigger className="ticket-lane-trigger">
                <span>{column.label}</span>
                <Badge variant="outline">
                  {cards.length} {cards.length === 1 ? "task" : "tasks"}
                </Badge>
              </AccordionTrigger>
              <AccordionContent className="ticket-lane-content">
                <div className="card-stack">
                  {cards.map((workItem) => (
                    <CompactWorkItemCard
                      key={workItem.id}
                      snapshot={snapshot}
                      workItem={workItem}
                      workItemTitles={workItemTitles}
                      onExplainDependencies={onExplainDependencies}
                      onOpen={onOpenTask}
                    />
                  ))}
                </div>
                {cards.length === 0 && (
                  <p className="empty-column-copy">No tasks here yet.</p>
                )}
              </AccordionContent>
            </AccordionItem>
          );
        })}
      </Accordion>
    </section>
  );
}
