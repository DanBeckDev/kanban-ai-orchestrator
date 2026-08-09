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
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

import { BoardHome } from "./BoardHome";
import { CompactWorkItemCard } from "./CompactWorkItemCard";
import { WorkflowComposer } from "./WorkflowComposer";
import { boardColumns, workItemsForColumn } from "./presentation";
import type {
  BoardSnapshot,
  GeneratePlanRequest,
  PlannerProfile,
} from "./types";

type BoardCanvasProps = Readonly<{
  snapshot: BoardSnapshot;
  busy: boolean;
  defaultPlannerProfileName?: string;
  plannerProfiles: readonly PlannerProfile[];
  onGeneratePlan: (request: GeneratePlanRequest) => Promise<void>;
  onExplainDependencies: (workItemId: string) => void;
  onOpenTask: (workItemId: string) => void;
}>;

export function BoardCanvas({
  snapshot,
  busy,
  defaultPlannerProfileName,
  plannerProfiles,
  onGeneratePlan,
  onExplainDependencies,
  onOpenTask,
}: BoardCanvasProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);

  return (
    <>
      <WorkflowComposer
        boardId={snapshot.board.id}
        busy={busy}
        defaultPlannerProfileName={defaultPlannerProfileName}
        onGeneratePlan={onGeneratePlan}
        plannerProfiles={plannerProfiles}
      />
      <BoardHome snapshot={snapshot} onOpenTask={onOpenTask} />
      {workItems.length === 0 ? (
        <Empty className="board-empty-state">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ClipboardPlusIcon />
            </EmptyMedia>
            <EmptyTitle>No tasks yet</EmptyTitle>
            <EmptyDescription>
              Describe the outcome above, or create one task yourself in manual
              mode.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <WorkflowLanes
          snapshot={snapshot}
          onExplainDependencies={onExplainDependencies}
          onOpenTask={onOpenTask}
        />
      )}
    </>
  );
}

function WorkflowLanes({
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
    <section aria-label="Workflow lanes" className="workflow-lanes">
      <Accordion
        defaultValue={boardColumns.map(({ id }) => id)}
        type="multiple"
      >
        {boardColumns.map((column) => {
          const cards = workItemsForColumn(snapshot, column);
          return (
            <AccordionItem
              className="workflow-lane"
              key={column.id}
              value={column.id}
            >
              <AccordionTrigger className="workflow-lane-trigger">
                <span>{column.label}</span>
                <Badge variant="outline">
                  {cards.length} {cards.length === 1 ? "task" : "tasks"}
                </Badge>
              </AccordionTrigger>
              <AccordionContent className="workflow-lane-content">
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
                  <p className="empty-column-copy">No work here yet</p>
                )}
              </AccordionContent>
            </AccordionItem>
          );
        })}
      </Accordion>
    </section>
  );
}
