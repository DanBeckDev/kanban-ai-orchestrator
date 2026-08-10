import { useEffect, useState } from "react";
import { ActivityIcon, ArrowRightIcon, BotIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ActivityStream } from "./ActivityStream";
import { WorkflowComposer } from "./WorkflowComposer";
import type {
  BoardSnapshot,
  BoardSupervision,
  Execution,
  ExecutionActivityPage,
  GeneratePlanRequest,
  PlannerProfile,
} from "./types";

type BoardHomeProps = Readonly<{
  busy: boolean;
  defaultPlannerProfileName?: string;
  plannerProfiles: readonly PlannerProfile[];
  snapshot: BoardSnapshot;
  supervision?: BoardSupervision;
  onGeneratePlan: (request: GeneratePlanRequest) => Promise<void>;
  onLoadExecutionActivity: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
  onLoadPlanningActivity: (
    boardId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
  onOpenTask: (workItemId: string) => void;
  onOpenPlanReview: () => void;
  onOpenTickets: () => void;
}>;

export function BoardHome({
  busy,
  defaultPlannerProfileName,
  plannerProfiles,
  snapshot,
  supervision,
  onGeneratePlan,
  onLoadExecutionActivity,
  onLoadPlanningActivity,
  onOpenTask,
  onOpenPlanReview,
  onOpenTickets,
}: BoardHomeProps) {
  const liveRuns = liveRunsFor(snapshot);
  const recentTickets = snapshot.workItems.slice(0, 4);
  const [plannerActivityVisible, setPlannerActivityVisible] = useState(busy);

  useEffect(() => {
    if (busy) setPlannerActivityVisible(true);
  }, [busy]);

  return (
    <section aria-labelledby="board-home-title" className="board-home">
      <header className="board-home-header">
        <div>
          <p className="eyebrow">Home</p>
          <h3 id="board-home-title">Start with the outcome</h3>
          <p>
            Tell the orchestrator what you want to achieve. You will review its
            tickets before any worker starts.
          </p>
        </div>
        <Badge variant="secondary">{modeCopy(supervision)}</Badge>
      </header>

      <WorkflowComposer
        boardId={snapshot.board.id}
        busy={busy}
        defaultPlannerProfileName={defaultPlannerProfileName}
        onGeneratePlan={onGeneratePlan}
        plannerProfiles={plannerProfiles}
      />
      <details className="advanced-disclosure">
        <summary>Use an existing plan</summary>
        <p className="field-hint">
          Paste or revise a plan that was created outside Kanban.
        </p>
        <Button onClick={onOpenPlanReview} type="button" variant="outline">
          Review an existing plan
        </Button>
      </details>

      <LiveFeedback
        busy={busy}
        isPlanning={busy || plannerActivityVisible}
        boardId={snapshot.board.id}
        runs={liveRuns}
        onLoadExecutionActivity={onLoadExecutionActivity}
        onLoadPlanningActivity={onLoadPlanningActivity}
      />

      <section aria-labelledby="home-tickets-title" className="home-tickets">
        <div className="home-section-header">
          <div>
            <p className="eyebrow">Tickets</p>
            <h4 id="home-tickets-title">
              {ticketHeading(recentTickets.length)}
            </h4>
          </div>
          <Button onClick={onOpenTickets} type="button" variant="outline">
            Open Tickets
            <ArrowRightIcon data-icon="inline-end" />
          </Button>
        </div>
        {recentTickets.length === 0 ? (
          <p className="home-empty-copy">
            Your confirmed tickets will appear here. Start by describing the
            outcome above.
          </p>
        ) : (
          <ul className="home-ticket-list">
            {recentTickets.map(({ workItem }) => (
              <li key={workItem.id}>
                <div>
                  <strong>{workItem.title}</strong>
                  <span>{workItem.state.replaceAll("_", " ")}</span>
                </div>
                <Button
                  aria-label={`Open ticket ${workItem.title}`}
                  onClick={() => onOpenTask(workItem.id)}
                  size="sm"
                  type="button"
                  variant="ghost"
                >
                  Open
                </Button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </section>
  );
}

function LiveFeedback({
  busy,
  isPlanning,
  boardId,
  runs,
  onLoadExecutionActivity,
  onLoadPlanningActivity,
}: Readonly<{
  busy: boolean;
  isPlanning: boolean;
  boardId: string;
  runs: readonly LiveRun[];
  onLoadExecutionActivity: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
  onLoadPlanningActivity: (
    boardId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
}>) {
  return (
    <section aria-labelledby="live-feedback-title" className="home-feedback">
      <div className="home-section-header">
        <div>
          <p className="eyebrow">Activity</p>
          <h4 id="live-feedback-title">Live AI feedback</h4>
        </div>
        <ActivityIcon aria-hidden="true" />
      </div>
      {busy && (
        <p aria-live="polite" className="home-planning-status" role="status">
          The orchestrator is preparing a reviewable plan.
        </p>
      )}
      {!isPlanning && runs.length === 0 ? (
        <p className="home-empty-copy">
          No AI is working right now. When a plan or ticket worker runs, its
          progress, questions, and outcome will appear here.
        </p>
      ) : (
        <div className="home-agent-feeds">
          {isPlanning && (
            <ActivityStream
              activityId={boardId}
              onLoad={onLoadPlanningActivity}
              title="Orchestrator"
            />
          )}
          {runs.map(({ execution, title }) => (
            <ActivityStream
              activityId={execution.id}
              key={execution.id}
              onLoad={onLoadExecutionActivity}
              title={`${execution.adapterName} · ${title}`}
            />
          ))}
        </div>
      )}
      <p className="home-feedback-boundary">
        <BotIcon aria-hidden="true" /> Updates are bounded and readable. Kanban
        does not collect private reasoning or credentials.
      </p>
    </section>
  );
}

type LiveRun = Readonly<{
  execution: Execution;
  title: string;
}>;

function liveRunsFor(snapshot: BoardSnapshot): readonly LiveRun[] {
  const titles = new Map(
    snapshot.workItems.map(({ workItem }) => [workItem.id, workItem.title]),
  );
  return snapshot.executions
    .filter(({ status }) => status === "running" || status === "awaiting_input")
    .map((execution) => ({
      execution,
      title: titles.get(execution.workItemId) ?? "Untitled ticket",
    }));
}

function modeCopy(supervision?: BoardSupervision): string {
  return supervision?.mode === "autonomous"
    ? "Kanban coordinates"
    : "You approve actions";
}

function ticketHeading(ticketCount: number): string {
  if (ticketCount === 0) return "Your tickets";
  return ticketCount === 1 ? "1 ticket" : `${ticketCount} tickets`;
}
