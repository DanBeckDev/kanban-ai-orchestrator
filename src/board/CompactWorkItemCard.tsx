import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import { blockersFor, executionsFor, stateLabel } from "./presentation";
import type { BoardSnapshot, Dependency, WorkItem } from "./types";

type CompactWorkItemCardProps = Readonly<{
  snapshot: BoardSnapshot;
  workItemTitles: ReadonlyMap<string, string>;
  workItem: WorkItem;
  onExplainDependencies: (workItemId: string) => void;
  onOpen: (workItemId: string) => void;
}>;

export function CompactWorkItemCard({
  snapshot,
  workItemTitles,
  workItem,
  onExplainDependencies,
  onOpen,
}: CompactWorkItemCardProps) {
  const blockers = blockersFor(snapshot, workItem.id);
  const waitingOn = blockers.filter((dependency) =>
    snapshot.workItems.some(
      ({ workItem: upstream }) =>
        upstream.id === dependency.upstreamWorkItemId &&
        upstream.state !== "done",
    ),
  );
  const latestExecution = executionsFor(snapshot, workItem.id).at(-1);

  return (
    <article className="compact-work-item-card">
      <Card size="sm">
        <CardHeader>
          <Badge variant={badgeVariant(workItem.state)}>
            {stateLabel(workItem.state)}
          </Badge>
          <CardTitle as="h4">{workItem.title}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="compact-work-item-summary">{workItem.description}</p>
          <p className="compact-work-item-actor">
            {principalActor(latestExecution?.adapterName)}
          </p>
          <p className="compact-work-item-signal">
            {taskSignal(waitingOn, workItemTitles, workItem.state)}
          </p>
        </CardContent>
        <CardFooter>
          {blockers.length > 0 && (
            <Button
              aria-label={`Explore dependencies for ${workItem.title}`}
              onClick={() => onExplainDependencies(workItem.id)}
              type="button"
              variant="link"
            >
              Explore dependencies
            </Button>
          )}
          <Button
            aria-label={`Open task ${workItem.title}`}
            onClick={() => onOpen(workItem.id)}
            type="button"
            variant="outline"
          >
            Open task
          </Button>
        </CardFooter>
      </Card>
    </article>
  );
}

function taskSignal(
  blockers: readonly Dependency[],
  workItemTitles: ReadonlyMap<string, string>,
  state: WorkItem["state"],
): string {
  const blockedBy = blockers
    .map((dependency) => workItemTitles.get(dependency.upstreamWorkItemId))
    .filter((title): title is string => title !== undefined);
  if (blockedBy.length > 0) {
    return blockedBy.length === 1
      ? `Waiting on ${blockedBy[0]}`
      : `Waiting on ${blockedBy[0]} + ${blockedBy.length - 1} more`;
  }
  return stateSignal(state);
}

function principalActor(adapterName?: string): string {
  return adapterName === undefined
    ? "No task worker has run yet."
    : `Last worked by ${adapterName}`;
}

function stateSignal(state: WorkItem["state"]): string {
  switch (state) {
    case "running":
      return "A worker is active.";
    case "awaiting_input":
      return "A worker needs input.";
    case "review":
      return "A review decision is needed.";
    case "done":
      return "Completion evidence is available.";
    case "blocked":
    case "failed":
    case "interrupted":
      return "A recovery decision is needed.";
    case "cancelled":
      return "This task is cancelled.";
    default:
      return "Ready for the next decision.";
  }
}

function badgeVariant(
  state: WorkItem["state"],
): "default" | "secondary" | "destructive" | "outline" {
  if (state === "blocked" || state === "failed" || state === "interrupted") {
    return "destructive";
  }
  if (state === "done" || state === "cancelled") {
    return "secondary";
  }
  if (state === "review") {
    return "default";
  }
  return "outline";
}
