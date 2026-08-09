import { useState, type FormEvent } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

import { GoalPlanForm } from "./GoalPlanForm";
import { PlanDraftEditor } from "./PlanDraftEditor";
import { PlanPreview } from "./PlanPreview";
import { timestamp } from "./presentation";
import type {
  AgentProfile,
  BoardPlan,
  ConfirmPlanRequest,
  DependencyKind,
  GeneratePlanRequest,
  PlannerProfile,
  ProposePlanRequest,
  WorkItemBudget,
} from "./types";

type PlanProposalPanelProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  boardId: string;
  busy: boolean;
  defaultPlannerProfileName?: string;
  defaultTicketWorkerProfileName?: string;
  plan?: BoardPlan;
  plannerProfiles: readonly PlannerProfile[];
  onConfirm: (request: ConfirmPlanRequest) => Promise<void>;
  onGenerate: (request: GeneratePlanRequest) => Promise<void>;
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
  assignedAgentProfileName?: string;
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

export function PlanProposalPanel({
  agentProfiles,
  boardId,
  busy,
  defaultPlannerProfileName,
  defaultTicketWorkerProfileName,
  plan,
  plannerProfiles,
  onConfirm,
  onGenerate,
  onPropose,
}: PlanProposalPanelProps) {
  const [confirmedBy, setConfirmedBy] = useState("");
  const [draftText, setDraftText] = useState("");
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string>();

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

  async function saveRevision(request: ProposePlanRequest) {
    try {
      await onPropose(request);
      setError(undefined);
      setEditing(false);
    } catch (operationError) {
      setError(errorMessage(operationError));
      throw operationError;
    }
  }

  async function submitPastedPlan(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await onPropose(planRequest(boardId, draftText));
      setError(undefined);
    } catch (operationError) {
      setError(errorMessage(operationError));
    }
  }

  if (plan !== undefined && editing) {
    return (
      <section
        aria-labelledby="plan-proposal-title"
        className="panel form-panel"
      >
        <h3 id="plan-proposal-title">Plan with AI</h3>
        {error !== undefined && <ErrorNotice message={error} />}
        <PlanDraftEditor
          agentProfiles={agentProfiles}
          boardId={boardId}
          busy={busy}
          defaultTicketWorkerProfileName={defaultTicketWorkerProfileName}
          plan={plan}
          onCancel={() => setEditing(false)}
          onSave={saveRevision}
        />
      </section>
    );
  }

  return (
    <section aria-labelledby="plan-proposal-title" className="panel form-panel">
      <div>
        <h3 id="plan-proposal-title">Plan with AI</h3>
        <p className="field-hint">
          Ask the organiser to break an outcome into a reviewable plan. No task
          is created or started until you confirm the proposal.
        </p>
      </div>
      {error !== undefined && <ErrorNotice message={error} />}
      {plan === undefined ? (
        <>
          <GoalPlanForm
            boardId={boardId}
            busy={busy}
            defaultPlannerProfileName={defaultPlannerProfileName}
            hasProposal={false}
            onGenerate={onGenerate}
            profiles={plannerProfiles}
          />
          <details className="advanced-disclosure">
            <summary>Paste an existing plan</summary>
            <form
              aria-label="Paste an existing plan"
              onSubmit={submitPastedPlan}
            >
              <label>
                Plan JSON
                <textarea
                  required
                  value={draftText}
                  onChange={(event) => setDraftText(event.target.value)}
                />
              </label>
              <Button disabled={busy} type="submit" variant="outline">
                Preview pasted plan
              </Button>
            </form>
          </details>
        </>
      ) : (
        <PlanPreview
          agentProfiles={agentProfiles}
          busy={busy}
          confirmedBy={confirmedBy}
          plan={plan}
          onConfirmedByChange={setConfirmedBy}
          onConfirm={confirm}
          onEdit={() => setEditing(true)}
        />
      )}
    </section>
  );
}

function ErrorNotice({ message }: Readonly<{ message: string }>) {
  return (
    <Alert role="alert" variant="destructive">
      <AlertTitle>Kanban could not update the plan preview</AlertTitle>
      <AlertDescription>{message}</AlertDescription>
    </Alert>
  );
}

function planRequest(boardId: string, draftText: string): ProposePlanRequest {
  const draft = parseDraft(draftText);
  const proposedAt = timestamp();
  return {
    planId: `plan-${proposedAt}`,
    boardId,
    proposedBy: "user",
    proposedAt,
    workItems: draft.workItems.map((workItem) => ({
      workItemId: workItem.id,
      title: workItem.title,
      description: workItem.description,
      acceptanceCriteria: workItem.acceptanceCriteria,
      budget: workItem.budget ?? {},
      requiresHumanReview: workItem.requiresHumanReview ?? false,
      assignedAgentProfileName: workItem.assignedAgentProfileName,
      assignedAgentModel: undefined,
      assignedAgentEffort: undefined,
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
    throw new Error("Plan JSON must contain a workItems array.");
  }
  return parsed as PlanDraft;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
