import { useEffect, useMemo, useState } from "react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

import { SurfaceHeader } from "./BoardManagement";
import { DependencyForm } from "./DependencyForm";
import { DependencyInspector } from "./DependencyInspector";
import { DependencyMap } from "./DependencyMap";
import {
  dependencyDetails,
  dependencyViewData,
  taskDependencySummary,
} from "./dependencyPresentation";
import type { AddDependencyRequest, BoardPlan, BoardSnapshot } from "./types";

type DependencyViewProps = Readonly<{
  busy: boolean;
  boardPlan: BoardPlan | undefined;
  snapshot: BoardSnapshot;
  selectedWorkItemId: string | undefined;
  onAddDependency: (request: AddDependencyRequest) => Promise<void>;
  onBack: () => void;
  onOpenTask: (workItemId: string) => void;
}>;

export function DependencyView({
  busy,
  boardPlan,
  snapshot,
  selectedWorkItemId,
  onAddDependency,
  onBack,
  onOpenTask,
}: DependencyViewProps) {
  const data = useMemo(
    () => dependencyViewData(snapshot, boardPlan),
    [boardPlan, snapshot],
  );
  const [selectedId, setSelectedId] = useState<string>();
  const selected = selectedId ?? selectedWorkItemId ?? data.workItems[0]?.id;
  const details =
    selected === undefined ? undefined : dependencyDetails(data, selected);

  useEffect(() => {
    if (
      selectedWorkItemId !== undefined &&
      data.workItems.some(({ id }) => id === selectedWorkItemId)
    ) {
      setSelectedId(selectedWorkItemId);
    }
  }, [data.workItems, selectedWorkItemId]);

  return (
    <section aria-labelledby="dependencies-title" className="workspace-surface">
      <SurfaceHeader
        description="See what is waiting, what it affects, and what can happen together — without having to read a diagram."
        headingId="dependencies-title"
        onBack={onBack}
        title="Dependencies"
      />
      {data.workItems.length === 0 ? (
        <Card>
          <CardHeader>
            <CardTitle as="h3">No tasks to map yet</CardTitle>
            <CardDescription>
              Create work first, then use this view to explain its
              relationships.
            </CardDescription>
          </CardHeader>
        </Card>
      ) : (
        <>
          <div className="dependency-view-grid">
            <DependencyMap
              data={data}
              selectedWorkItemId={selected}
              onSelect={setSelectedId}
            />
            {details !== undefined && (
              <DependencyInspector details={details} onOpenTask={onOpenTask} />
            )}
          </div>
          <Card aria-labelledby="dependency-list-title">
            <CardHeader>
              <CardTitle as="h3" id="dependency-list-title">
                Dependency list
              </CardTitle>
              <CardDescription>
                The same task relationships in a keyboard-friendly list.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ol className="dependency-task-list">
                {data.workItems.map((workItem) => (
                  <li key={workItem.id}>
                    <Button
                      aria-label={`Select ${workItem.title}. ${taskDependencySummary(data, workItem.id)}`}
                      aria-pressed={selected === workItem.id}
                      className="dependency-task-button"
                      onClick={() => setSelectedId(workItem.id)}
                      type="button"
                      variant="outline"
                    >
                      <strong>{workItem.title}</strong>
                      <span>{taskDependencySummary(data, workItem.id)}</span>
                    </Button>
                  </li>
                ))}
              </ol>
            </CardContent>
          </Card>
        </>
      )}
      {data.workItems.length >= 2 && (
        <Accordion collapsible type="single">
          <AccordionItem value="add-dependency">
            <AccordionTrigger>Add a relationship manually</AccordionTrigger>
            <AccordionContent>
              <DependencyForm
                busy={busy}
                workItems={data.workItems}
                onCreate={onAddDependency}
              />
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      )}
    </section>
  );
}
