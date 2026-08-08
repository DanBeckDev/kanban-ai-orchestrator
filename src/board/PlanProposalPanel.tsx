import { useState, type FormEvent } from "react";

import { timestamp } from "./presentation";
import type {
  BoardPlan,
  ConfirmPlanRequest,
  DependencyKind,
  ProposePlanRequest,
  WorkItemBudget,
} from "./types";

type PlanProposalPanelProps = Readonly<{
  boardId: string;
  busy: boolean;
  plan?: BoardPlan;
  onConfirm: (request: ConfirmPlanRequest) => Promise<void>;
  onPropose: (request: ProposePlanRequest) => Promise<void>;
}>;

type PlanDraft = Readonly<{
  workItems: readonly PlanDraftWorkItem[];
  dependencies?: readonly PlanDraftDependency[];
  unresolvedAssumptions?: readonly string[];
}>;

type PlanDraftWorkItem = Readonly<{
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget?: WorkItemBudget;
  requiresHumanReview?: boolean;
}>;

type PlanDraftDependency = Readonly<{
  id: string;
  upstreamWorkItemId: string;
  downstreamWorkItemId: string;
  kind: DependencyKind;
  reason: string;
  owner: string;
  nextAction: string;
}>;

const draftExample = `{
  "workItems": [
    {
      "id": "foundation",
      "title": "Define the contract",
      "description": "Create the shared data contract.",
      "acceptanceCriteria": ["Contract tests pass."],
      "budget": { "maxAgentTurns": 8 },
      "requiresHumanReview": true
    }
  ],
  "dependencies": [],
  "unresolvedAssumptions": []
}`;

export function PlanProposalPanel({
  boardId,
  busy,
  plan,
  onConfirm,
  onPropose,
}: PlanProposalPanelProps) {
  const [proposedBy, setProposedBy] = useState("orchestrator");
  const [draftText, setDraftText] = useState(draftExample);
  const [confirmedBy, setConfirmedBy] = useState("");
  const [draftError, setDraftError] = useState<string>();
  const [editing, setEditing] = useState(false);

  async function propose(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      const proposedAt = timestamp();
      await onPropose(
        planRequest({ boardId, draftText, proposedAt, proposedBy }),
      );
      setDraftError(undefined);
      setEditing(false);
    } catch (error) {
      setDraftError(errorMessage(error));
    }
  }

  async function confirm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (plan === undefined) return;
    await onConfirm({
      boardId,
      planId: plan.preview.id,
      confirmedBy,
      confirmedAt: timestamp(),
    });
  }

  return (
    <section aria-labelledby="plan-proposal-title" className="panel form-panel">
      <div>
        <h3 id="plan-proposal-title">Orchestrator plan</h3>
        <p className="field-hint">
          Paste the provider-neutral plan JSON produced by your orchestrator.
          The daemon validates dependencies and records the exact preview before
          any task exists.
        </p>
      </div>
      {plan === undefined || editing ? (
        <form
          aria-label={
            plan === undefined ? "Propose board plan" : "Revise board plan"
          }
          onSubmit={propose}
        >
          {plan !== undefined && (
            <p className="field-hint">
              Replace the unconfirmed proposal with a revised complete plan. Its
              earlier tasks will not be created.
            </p>
          )}
          <label>
            Planner identity
            <input
              required
              value={proposedBy}
              onChange={(event) => setProposedBy(event.target.value)}
            />
          </label>
          <label>
            Plan draft JSON
            <textarea
              required
              value={draftText}
              onChange={(event) => setDraftText(event.target.value)}
            />
          </label>
          {draftError !== undefined && (
            <p className="inline-error" role="alert">
              {draftError}
            </p>
          )}
          <button disabled={busy} type="submit">
            {plan === undefined ? "Preview plan" : "Preview revised plan"}
          </button>
          {plan !== undefined && (
            <button
              disabled={busy}
              onClick={() => setEditing(false)}
              type="button"
            >
              Cancel revision
            </button>
          )}
        </form>
      ) : (
        <PlanPreview
          busy={busy}
          confirmedBy={confirmedBy}
          onConfirmedByChange={setConfirmedBy}
          onConfirm={confirm}
          onEdit={() => setEditing(true)}
          plan={plan}
        />
      )}
    </section>
  );
}

function PlanPreview({
  busy,
  confirmedBy,
  onConfirmedByChange,
  onConfirm,
  onEdit,
  plan,
}: Readonly<{
  busy: boolean;
  confirmedBy: string;
  onConfirmedByChange: (value: string) => void;
  onConfirm: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onEdit: () => void;
  plan: BoardPlan;
}>) {
  const { preview } = plan;
  return (
    <div className="plan-preview">
      <p>
        Plan <strong>{preview.id}</strong> · {preview.workItems.length} tasks
      </p>
      <p>Critical path: {preview.criticalPath.join(" → ") || "No hard path"}</p>
      <ol aria-label="Plan tasks" className="criteria-list">
        {preview.workItems.map((workItem) => (
          <li key={workItem.id}>
            <strong>{workItem.id}</strong> — {workItem.title}
            <span className="budget-summary">
              {formatBudget(workItem.budget)}
            </span>
            <ul>
              {workItem.acceptanceCriteria.map((criterion) => (
                <li key={criterion}>{criterion}</li>
              ))}
            </ul>
          </li>
        ))}
      </ol>
      <p>Parallel stages: {formatStages(preview.parallelStages)}</p>
      <p className="budget-summary">{formatPlanBudget(preview.budget)}</p>
      {preview.dependencies.length > 0 && (
        <ul aria-label="Plan dependencies" className="criteria-list">
          {preview.dependencies.map((dependency) => (
            <li key={dependency.id}>
              {dependency.upstreamWorkItemId} {dependency.kind} →{" "}
              {dependency.downstreamWorkItemId}
            </li>
          ))}
        </ul>
      )}
      {preview.unresolvedAssumptions.length > 0 && (
        <ul aria-label="Unresolved plan assumptions" className="criteria-list">
          {preview.unresolvedAssumptions.map((assumption) => (
            <li key={assumption}>{assumption}</li>
          ))}
        </ul>
      )}
      {plan.confirmation === undefined ? (
        <>
          <button disabled={busy} onClick={onEdit} type="button">
            Revise proposal
          </button>
          <form aria-label="Confirm board plan" onSubmit={onConfirm}>
            <label>
              Confirm as
              <input
                required
                value={confirmedBy}
                onChange={(event) => onConfirmedByChange(event.target.value)}
              />
            </label>
            <button disabled={busy} type="submit">
              Confirm and create tasks
            </button>
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

function planRequest({
  boardId,
  draftText,
  proposedAt,
  proposedBy,
}: Readonly<{
  boardId: string;
  draftText: string;
  proposedAt: string;
  proposedBy: string;
}>): ProposePlanRequest {
  const draft = parseDraft(draftText);
  return {
    planId: `plan-${proposedAt}`,
    boardId,
    proposedBy,
    proposedAt,
    workItems: draft.workItems.map((workItem) => ({
      workItemId: workItem.id,
      title: workItem.title,
      description: workItem.description,
      acceptanceCriteria: workItem.acceptanceCriteria,
      budget: workItem.budget ?? {},
      requiresHumanReview: workItem.requiresHumanReview ?? false,
    })),
    dependencies: (draft.dependencies ?? []).map((dependency) => ({
      dependencyId: dependency.id,
      upstreamWorkItemId: dependency.upstreamWorkItemId,
      downstreamWorkItemId: dependency.downstreamWorkItemId,
      kind: dependency.kind,
      reason: dependency.reason,
      owner: dependency.owner,
      nextAction: dependency.nextAction,
    })),
    unresolvedAssumptions: draft.unresolvedAssumptions ?? [],
  };
}

function parseDraft(draftText: string): PlanDraft {
  const parsed: unknown = JSON.parse(draftText);
  if (
    parsed === null ||
    typeof parsed !== "object" ||
    !("workItems" in parsed) ||
    !Array.isArray(parsed.workItems)
  ) {
    throw new Error("Plan draft JSON must contain a workItems array.");
  }
  return parsed as PlanDraft;
}

function formatStages(stages: readonly (readonly string[])[]): string {
  return stages
    .map((stage, index) => `${index + 1}: ${stage.join(", ")}`)
    .join(" · ");
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

function formatPlanBudget(budget: BoardPlan["preview"]["budget"]): string {
  const totals = formatBudget(budget);
  const missing = [
    ...budget.workItemsMissingAgentTurnBudget.map(
      (id) => `${id} has no turn limit`,
    ),
    ...budget.workItemsMissingDurationBudget.map(
      (id) => `${id} has no duration limit`,
    ),
    ...budget.workItemsMissingCostBudget.map((id) => `${id} has no cost limit`),
  ];
  return missing.length > 0 ? `${totals}. ${missing.join("; ")}.` : totals;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
