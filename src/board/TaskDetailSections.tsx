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

import { ActivityStream } from "./ActivityStream";
import { dependencyKindLabel } from "./dependencyPresentation";
import { ExternalLinks } from "./ExternalLinks";
import {
  activityFor,
  budgetSummary,
  evidenceFor,
  executionsFor,
} from "./presentation";
import type {
  BoardSnapshot,
  Execution,
  ExecutionActivityPage,
  WorkItem,
} from "./types";

type TaskDetailSectionsProps = Readonly<{
  liveExecution: Execution | undefined;
  snapshot: BoardSnapshot;
  workItem: WorkItem;
  onLoadExecutionActivity: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
}>;

export function TaskDetailSections({
  liveExecution,
  snapshot,
  workItem,
  onLoadExecutionActivity,
}: TaskDetailSectionsProps) {
  const activity = activityFor(snapshot, workItem.id);
  const evidence = evidenceFor(snapshot, workItem.id);
  const executions = executionsFor(snapshot, workItem.id);
  const dependencies = dependencyContext(snapshot, workItem.id);
  const externalLinks = snapshot.externalLinks.filter(
    (link) => link.workItemId === workItem.id,
  );

  return (
    <Card className="task-detail-sections">
      <CardHeader>
        <CardTitle as="h3">Task information</CardTitle>
        <CardDescription>
          Open only the context you need to make the current decision.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Accordion className="task-detail-disclosures" type="multiple">
          <AccordionItem value="task-details">
            <AccordionTrigger>Task details and success checks</AccordionTrigger>
            <AccordionContent>
              <p>{workItem.description}</p>
              <p className="budget-summary">{budgetSummary(workItem)}</p>
              <h4>Acceptance criteria</h4>
              {workItem.acceptanceCriteria.length === 0 ? (
                <p>No acceptance criteria are recorded yet.</p>
              ) : (
                <ul className="criteria-list">
                  {workItem.acceptanceCriteria.map((criterion) => (
                    <li key={criterion}>{criterion}</li>
                  ))}
                </ul>
              )}
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="dependencies">
            <AccordionTrigger>Dependencies</AccordionTrigger>
            <AccordionContent>
              <DependencyContext dependencies={dependencies} />
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="activity">
            <AccordionTrigger>Activity and attempts</AccordionTrigger>
            <AccordionContent>
              {liveExecution === undefined ? (
                <p>No worker is currently reporting activity.</p>
              ) : (
                <ActivityStream
                  execution={liveExecution}
                  onLoad={onLoadExecutionActivity}
                />
              )}
              <ExecutionHistory executions={executions} />
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="evidence">
            <AccordionTrigger>Review evidence</AccordionTrigger>
            <AccordionContent>
              <EvidenceHistory evidence={evidence} />
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="history">
            <AccordionTrigger>Decision history</AccordionTrigger>
            <AccordionContent>
              <DecisionHistory activity={activity} />
            </AccordionContent>
          </AccordionItem>
          {externalLinks.length > 0 && (
            <AccordionItem value="linked-work">
              <AccordionTrigger>Connected work</AccordionTrigger>
              <AccordionContent>
                <ExternalLinks links={externalLinks} />
              </AccordionContent>
            </AccordionItem>
          )}
        </Accordion>
      </CardContent>
    </Card>
  );
}

function DependencyContext({
  dependencies,
}: Readonly<{
  dependencies: readonly Readonly<{
    id: string;
    title: string;
    type: string;
    reason: string;
    owner: string;
    nextAction: string;
  }>[];
}>) {
  if (dependencies.length === 0) {
    return <p>No upstream dependency is recorded for this task.</p>;
  }
  return (
    <ul className="task-dependency-list">
      {dependencies.map((dependency) => (
        <li key={dependency.id}>
          <strong>{dependency.title}</strong>
          <span>{dependency.type}</span>
          <p>{dependency.reason}</p>
          <dl>
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
  );
}

function ExecutionHistory({
  executions,
}: Readonly<{ executions: readonly Execution[] }>) {
  if (executions.length === 0) {
    return <p>No durable agent attempt has been recorded for this task.</p>;
  }
  return (
    <ol className="task-history-list">
      {executions.map((execution) => (
        <li key={execution.id}>
          <p>
            {execution.adapterName} · {execution.status.replaceAll("_", " ")}
          </p>
          <span>
            {execution.role === "independent_review"
              ? "Independent reviewer"
              : "Task worker"}
          </span>
          <span>Workspace: {execution.workspacePath}</span>
          {execution.sessionId && <span>Session: {execution.sessionId}</span>}
          <span>
            Usage: {execution.usage.inputTokens} input /{" "}
            {execution.usage.outputTokens} output tokens
          </span>
        </li>
      ))}
    </ol>
  );
}

function EvidenceHistory({
  evidence,
}: Readonly<{ evidence: ReturnType<typeof evidenceFor> }>) {
  if (evidence.length === 0) {
    return <p>No review evidence has been recorded for this task.</p>;
  }
  return (
    <ol className="task-history-list">
      {evidence.map((entry) => (
        <li key={entry.id}>
          <p>
            {entry.kind.replaceAll("_", " ")}: {entry.result}
          </p>
          <span>{entry.summary}</span>
          <time dateTime={entry.recordedAt}>{entry.recordedAt}</time>
        </li>
      ))}
    </ol>
  );
}

function DecisionHistory({
  activity,
}: Readonly<{ activity: ReturnType<typeof activityFor> }>) {
  if (activity.length === 0) {
    return <p>No durable decision has been recorded for this task.</p>;
  }
  return (
    <ol className="task-history-list">
      {activity.map((entry) => (
        <li key={entry.sequence}>
          <p>{entry.summary}</p>
          <time dateTime={entry.recordedAt}>{entry.recordedAt}</time>
          {entry.completionEvidence && (
            <span>
              Evidence: quality gate{" "}
              {entry.completionEvidence.qualityGatePassed
                ? "passed"
                : "not passed"}
              , report{" "}
              {entry.completionEvidence.completionReportPresent
                ? "present"
                : "missing"}
              , review{" "}
              {entry.completionEvidence.reviewAccepted ? "accepted" : "pending"}
              .
            </span>
          )}
        </li>
      ))}
    </ol>
  );
}

function dependencyContext(snapshot: BoardSnapshot, workItemId: string) {
  return snapshot.dependencies.flatMap((dependency) => {
    if (dependency.downstreamWorkItemId !== workItemId) return [];
    const title = snapshot.workItems.find(
      ({ workItem }) => workItem.id === dependency.upstreamWorkItemId,
    )?.workItem.title;
    if (title === undefined) return [];
    return [
      {
        id: dependency.id,
        title,
        type: dependencyKindLabel(dependency),
        reason: dependency.reason,
        owner: dependency.owner,
        nextAction: dependency.nextAction,
      },
    ];
  });
}
