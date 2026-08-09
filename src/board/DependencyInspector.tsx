import { CircleCheckIcon, NetworkIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import {
  dependencyKindLabel,
  dependencyReadiness,
  relationDescription,
  taskStateLabel,
  type DependencyDetails,
  type DependencyRelation,
} from "./dependencyPresentation";

type DependencyInspectorProps = Readonly<{
  details: DependencyDetails;
  onOpenTask: (workItemId: string) => void;
}>;

export function DependencyInspector({
  details,
  onOpenTask,
}: DependencyInspectorProps) {
  const readiness = dependencyReadiness(details);
  const onCriticalPath = details.criticalPath?.includes(details.workItem.id);

  return (
    <Card aria-live="polite" className="dependency-inspector">
      <CardHeader>
        <CardTitle as="h3">{details.workItem.title}</CardTitle>
        <CardDescription>{taskStateLabel(details.workItem)}</CardDescription>
      </CardHeader>
      <CardContent className="dependency-inspector-content">
        <section aria-labelledby="dependency-readiness-title">
          <div className="dependency-section-heading">
            <NetworkIcon aria-hidden="true" />
            <h4 id="dependency-readiness-title">{readiness.title}</h4>
          </div>
          <p>{readiness.description}</p>
        </section>
        <RelationList
          emptyCopy="No hard prerequisite is recorded."
          relations={details.hardPrerequisites}
          title="Hard prerequisites"
          direction="upstream"
        />
        <RelationList
          emptyCopy="No contract or advisory relationship is recorded."
          relations={details.guidance}
          title="Other guidance"
          direction="upstream"
        />
        <RelationList
          emptyCopy="No downstream task is affected by this one."
          relations={details.downstreamImpact}
          title="Work affected next"
          direction="downstream"
        />
        <section aria-labelledby="dependency-plan-context-title">
          <div className="dependency-section-heading">
            <CircleCheckIcon aria-hidden="true" />
            <h4 id="dependency-plan-context-title">Plan context</h4>
          </div>
          <PlanContext
            criticalPath={details.criticalPath}
            onCriticalPath={onCriticalPath}
            parallelNeighbours={details.parallelNeighbours}
          />
        </section>
      </CardContent>
      <CardFooter>
        <Button
          onClick={() => onOpenTask(details.workItem.id)}
          type="button"
          variant="outline"
        >
          Open task
        </Button>
      </CardFooter>
    </Card>
  );
}

function RelationList({
  direction,
  emptyCopy,
  relations,
  title,
}: Readonly<{
  direction: "upstream" | "downstream";
  emptyCopy: string;
  relations: readonly DependencyRelation[];
  title: string;
}>) {
  const headingId = `${title.toLowerCase().replaceAll(" ", "-")}-title`;
  return (
    <section aria-labelledby={headingId}>
      <h4 id={headingId}>{title}</h4>
      {relations.length === 0 ? (
        <p>{emptyCopy}</p>
      ) : (
        <ul className="dependency-relation-list">
          {relations.map(({ dependency, workItem }) => (
            <li key={dependency.id}>
              <div className="dependency-relation-heading">
                <strong>{workItem.title}</strong>
                <Badge variant="outline">
                  {dependencyKindLabel(dependency)}
                </Badge>
              </div>
              <p>{relationDescription(dependency, direction)}</p>
              <dl>
                <div>
                  <dt>Why</dt>
                  <dd>{dependency.reason}</dd>
                </div>
                <div>
                  <dt>Owner</dt>
                  <dd>{dependency.owner}</dd>
                </div>
                <div>
                  <dt>Next action</dt>
                  <dd>{dependency.nextAction}</dd>
                </div>
              </dl>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function PlanContext({
  criticalPath,
  onCriticalPath,
  parallelNeighbours,
}: Readonly<{
  criticalPath: readonly string[] | undefined;
  onCriticalPath: boolean | undefined;
  parallelNeighbours: readonly { title: string }[] | undefined;
}>) {
  if (criticalPath === undefined || parallelNeighbours === undefined) {
    return (
      <p>
        No current plan has calculated an execution order for this exact graph.
        Ask the organiser to plan the work after you update relationships.
      </p>
    );
  }
  return (
    <dl className="dependency-plan-context">
      <div>
        <dt>Critical route</dt>
        <dd>
          {onCriticalPath ? "This task is on it" : "This task is not on it"}
        </dd>
      </div>
      <div>
        <dt>Can happen alongside</dt>
        <dd>
          {parallelNeighbours.length === 0
            ? "No other task in this plan stage"
            : parallelNeighbours.map(({ title }) => title).join(", ")}
        </dd>
      </div>
    </dl>
  );
}
