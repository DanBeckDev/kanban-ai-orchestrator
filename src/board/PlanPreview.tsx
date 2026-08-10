import type { FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import type { AgentProfile, BoardPlan, WorkItemBudget } from "./types";

type PlanPreviewProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  confirmedBy: string;
  plan: BoardPlan;
  onConfirmedByChange: (value: string) => void;
  onConfirm: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onEdit: () => void;
}>;

export function PlanPreview({
  agentProfiles,
  busy,
  confirmedBy,
  plan,
  onConfirmedByChange,
  onConfirm,
  onEdit,
}: PlanPreviewProps) {
  const { preview } = plan;
  const taskNames = new Map(
    preview.workItems.map((workItem) => [workItem.id, workItem.title]),
  );
  const profileNames = new Set(agentProfiles.map(({ name }) => name));

  return (
    <div className="plan-preview">
      <div>
        <h3>Review the proposed plan</h3>
        <p className="field-hint">
          Review the scope, ordering, workers, and assumptions. Confirmation is
          the only action that adds these tasks to your board.
        </p>
      </div>
      <p>
        Work that must happen in order:{" "}
        {formatTaskSequence(preview.criticalPath, taskNames) || "None"}
      </p>
      <ol aria-label="Plan tasks" className="plan-task-list">
        {preview.workItems.map((workItem, index) => (
          <li key={workItem.id}>
            <Card>
              <CardHeader>
                <CardTitle as="h4">
                  Task {index + 1}: {workItem.title}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p>{workItem.description}</p>
                <p className="budget-summary">
                  {formatBudget(workItem.budget)}
                </p>
                <p>
                  Worker:{" "}
                  {workerName(workItem.assignedAgentProfileName, profileNames)}
                  {` · ${effortLabel(workItem.assignedAgentEffort)} effort`}
                  {workItem.requiresHumanReview
                    ? " · Human approval required"
                    : ""}
                </p>
                <ul className="criteria-list">
                  {workItem.acceptanceCriteria.map((criterion) => (
                    <li key={criterion}>{criterion}</li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          </li>
        ))}
      </ol>
      <p>
        Work that can happen together:{" "}
        {formatStages(preview.parallelStages, taskNames)}
      </p>
      <p className="budget-summary">
        {formatPlanBudget(preview.budget, taskNames)}
      </p>
      {preview.dependencies.length > 0 && (
        <section aria-labelledby="proposal-dependencies-title">
          <h4 id="proposal-dependencies-title">Dependencies</h4>
          <ul aria-label="Plan dependencies" className="criteria-list">
            {preview.dependencies.map((dependency) => (
              <li key={dependency.id}>
                {taskName(dependency.upstreamWorkItemId, taskNames)} must happen
                before {taskName(dependency.downstreamWorkItemId, taskNames)}
              </li>
            ))}
          </ul>
        </section>
      )}
      {preview.unresolvedAssumptions.length > 0 && (
        <section aria-labelledby="proposal-assumptions-title">
          <h4 id="proposal-assumptions-title">Assumptions to review</h4>
          <ul
            aria-label="Unresolved plan assumptions"
            className="criteria-list"
          >
            {preview.unresolvedAssumptions.map((assumption) => (
              <li key={assumption}>{assumption}</li>
            ))}
          </ul>
        </section>
      )}
      {plan.confirmation === undefined ? (
        <>
          <Button
            disabled={busy}
            onClick={onEdit}
            type="button"
            variant="outline"
          >
            Edit proposed tasks
          </Button>
          <form
            aria-label="Confirm board plan"
            className="confirm-plan-form"
            onSubmit={onConfirm}
          >
            <label>
              Your name
              <input
                autoComplete="name"
                name="confirmed-by"
                required
                value={confirmedBy}
                onChange={(event) => onConfirmedByChange(event.target.value)}
              />
            </label>
            <Button disabled={busy} type="submit">
              Confirm and create tasks
            </Button>
          </form>
        </>
      ) : (
        <p className="local-status">
          Confirmed by {plan.confirmation.confirmedBy} at{" "}
          {plan.confirmation.confirmedAt}
        </p>
      )}
    </div>
  );
}

function effortLabel(
  effort: "provider_default" | "focused" | "balanced" | "thorough",
): string {
  return effort === "provider_default"
    ? "Provider default"
    : `${effort[0].toUpperCase()}${effort.slice(1)}`;
}

function workerName(
  profileName: string | undefined,
  availableProfiles: ReadonlySet<string>,
): string {
  if (profileName === undefined) return "Project default";
  return availableProfiles.has(profileName)
    ? profileName
    : `${profileName} (unavailable)`;
}

function formatTaskSequence(
  taskIds: readonly string[],
  taskNames: ReadonlyMap<string, string>,
): string {
  return taskIds.map((taskId) => taskName(taskId, taskNames)).join(" → ");
}

function formatStages(
  stages: readonly (readonly string[])[],
  taskNames: ReadonlyMap<string, string>,
): string {
  return stages
    .map(
      (stage, index) => `${index + 1}: ${formatTaskSequence(stage, taskNames)}`,
    )
    .join(" · ");
}

function taskName(
  taskId: string,
  taskNames: ReadonlyMap<string, string>,
): string {
  return taskNames.get(taskId) ?? "Removed task";
}

function formatBudget(budget: WorkItemBudget): string {
  const limits = [
    budget.maxAgentTurns === undefined
      ? undefined
      : `turns ${budget.maxAgentTurns}`,
    budget.maxDurationSeconds === undefined
      ? undefined
      : `seconds ${budget.maxDurationSeconds}`,
    budget.maxCostMicros === undefined
      ? undefined
      : `cost µ${budget.maxCostMicros}`,
  ].filter((limit): limit is string => limit !== undefined);
  return limits.length > 0
    ? `Budget: ${limits.join(" · ")}`
    : "Budget: not set";
}

function formatPlanBudget(
  budget: BoardPlan["preview"]["budget"],
  taskNames: ReadonlyMap<string, string>,
): string {
  const totals = formatBudget(budget);
  const missing = [
    ...budget.workItemsMissingAgentTurnBudget.map(
      (id) => `${taskName(id, taskNames)} has no turn limit`,
    ),
    ...budget.workItemsMissingDurationBudget.map(
      (id) => `${taskName(id, taskNames)} has no duration limit`,
    ),
    ...budget.workItemsMissingCostBudget.map(
      (id) => `${taskName(id, taskNames)} has no cost limit`,
    ),
  ];
  return missing.length > 0 ? `${totals}. ${missing.join("; ")}.` : totals;
}
