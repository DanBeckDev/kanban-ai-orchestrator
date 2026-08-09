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
  onOpen: (workItemId: string) => void;
}>;

export function CompactWorkItemCard({
  snapshot,
  workItemTitles,
  workItem,
  onOpen,
}: CompactWorkItemCardProps) {
  const blockers = blockersFor(snapshot, workItem.id);
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
          <p className="compact-work-item-signal">
            {taskSignal(blockers, workItemTitles, latestExecution?.adapterName)}
          </p>
        </CardContent>
        <CardFooter>
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
  adapterName?: string,
): string {
  const blockedBy = blockers
    .map((dependency) => workItemTitles.get(dependency.upstreamWorkItemId))
    .filter((title): title is string => title !== undefined);
  if (blockedBy.length > 0) {
    return blockedBy.length === 1
      ? `Waiting on ${blockedBy[0]}`
      : `Waiting on ${blockedBy[0]} + ${blockedBy.length - 1} more`;
  }
  return adapterName === undefined
    ? "Ready for the next decision"
    : `Last worked by ${adapterName}`;
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
