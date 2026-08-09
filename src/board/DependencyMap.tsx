import { ArrowRightIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

import {
  dependencyKindLabel,
  taskStateLabel,
  type DependencyViewData,
} from "./dependencyPresentation";

type DependencyMapProps = Readonly<{
  data: DependencyViewData;
  selectedWorkItemId: string | undefined;
  onSelect: (workItemId: string) => void;
}>;

export function DependencyMap({
  data,
  selectedWorkItemId,
  onSelect,
}: DependencyMapProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3">Visual dependency map</CardTitle>
        <CardDescription>
          Each line shows what must happen before the next task. Select any task
          to read the full explanation.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {data.dependencies.length === 0 ? (
          <TaskMap
            selectedWorkItemId={selectedWorkItemId}
            workItemIds={data.workItems.map(({ id }) => id)}
            data={data}
            onSelect={onSelect}
          />
        ) : (
          <ol aria-label="Visual dependency map" className="dependency-map">
            {data.dependencies.map((dependency) => (
              <li key={dependency.id}>
                <TaskNode
                  selected={
                    selectedWorkItemId === dependency.upstreamWorkItemId
                  }
                  workItemId={dependency.upstreamWorkItemId}
                  data={data}
                  onSelect={onSelect}
                />
                <div aria-hidden="true" className="dependency-map-edge">
                  <Badge variant="outline">
                    {dependencyKindLabel(dependency)}
                  </Badge>
                  <ArrowRightIcon />
                </div>
                <TaskNode
                  selected={
                    selectedWorkItemId === dependency.downstreamWorkItemId
                  }
                  workItemId={dependency.downstreamWorkItemId}
                  data={data}
                  onSelect={onSelect}
                />
              </li>
            ))}
          </ol>
        )}
      </CardContent>
    </Card>
  );
}

function TaskMap({
  data,
  selectedWorkItemId,
  workItemIds,
  onSelect,
}: Readonly<{
  data: DependencyViewData;
  selectedWorkItemId: string | undefined;
  workItemIds: readonly string[];
  onSelect: (workItemId: string) => void;
}>) {
  return (
    <ol
      aria-label="Tasks without relationships"
      className="dependency-map-free"
    >
      {workItemIds.map((workItemId) => (
        <li key={workItemId}>
          <TaskNode
            selected={selectedWorkItemId === workItemId}
            workItemId={workItemId}
            data={data}
            onSelect={onSelect}
          />
        </li>
      ))}
    </ol>
  );
}

function TaskNode({
  data,
  selected,
  workItemId,
  onSelect,
}: Readonly<{
  data: DependencyViewData;
  selected: boolean;
  workItemId: string;
  onSelect: (workItemId: string) => void;
}>) {
  const workItem = data.workItems.find(({ id }) => id === workItemId);
  if (workItem === undefined) return null;

  return (
    <Button
      aria-label={`Select ${workItem.title}, ${taskStateLabel(workItem)}`}
      aria-pressed={selected}
      className="dependency-map-node"
      data-selected={selected || undefined}
      onClick={() => onSelect(workItem.id)}
      type="button"
      variant="outline"
    >
      <span>{workItem.title}</span>
      <small>{taskStateLabel(workItem)}</small>
    </Button>
  );
}
