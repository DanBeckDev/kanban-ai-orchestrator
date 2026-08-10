import { useCallback, useEffect, useState } from "react";
import { BotIcon, CheckIcon, XIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

import type { TicketEffectOperations } from "./ticketEffectOperations";
import type {
  TicketEffect,
  TicketEffectAction,
  TicketEffectResolution,
} from "./ticketEffectTypes";

type TaskAiPromptProps = Readonly<{
  busy: boolean;
  hasOrganiser: boolean;
  operations: TicketEffectOperations;
  workItemId: string;
}>;

const actions: readonly Readonly<{
  value: TicketEffectAction;
  label: string;
  hint: string;
}>[] = [
  {
    value: "refine_specification",
    label: "Improve task details",
    hint: "Propose clearer task details and success checks.",
  },
  {
    value: "give_worker_guidance",
    label: "Guide the worker",
    hint: "Add focused guidance to the next worker brief.",
  },
  {
    value: "prepare_start",
    label: "Start this task",
    hint: "Prepare a safe worker start.",
  },
  {
    value: "prepare_restart",
    label: "Restart this task",
    hint: "Prepare a safe retry for blocked or failed work.",
  },
  {
    value: "explain_evidence",
    label: "Explain the evidence",
    hint: "Explain what the recorded checks and review say.",
  },
  {
    value: "return_for_correction",
    label: "Return for correction",
    hint: "Prepare a return when review evidence has failed.",
  },
  {
    value: "recover_interrupted",
    label: "Recover an interruption",
    hint: "Prepare interrupted work to be picked up again.",
  },
];

export function TaskAiPrompt({
  busy,
  hasOrganiser,
  operations,
  workItemId,
}: TaskAiPromptProps) {
  const [action, setAction] = useState<TicketEffectAction>("explain_evidence");
  const [prompt, setPrompt] = useState("");
  const [effects, setEffects] = useState<readonly TicketEffect[]>([]);
  const activeAction =
    actions.find(({ value }) => value === action) ?? actions[0];
  const loadEffects = useCallback(
    async () => setEffects(await operations.load(workItemId)),
    [operations, workItemId],
  );

  useEffect(() => {
    void loadEffects();
  }, [loadEffects]);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!hasOrganiser || prompt.trim().length === 0) return;
    await operations.request({
      requestId: crypto.randomUUID(),
      workItemId,
      action,
      prompt,
    });
    setPrompt("");
    await loadEffects();
  }

  async function resolve(effectId: string, resolution: TicketEffectResolution) {
    await operations.resolve({ effectId, resolution });
    await loadEffects();
  }

  return (
    <Card className="task-ai-prompt">
      <CardHeader>
        <CardTitle as="h3">Ask task AI</CardTitle>
        <CardDescription>
          Ask the orchestrator about this task. Kanban records the safe
          proposal; changes wait for your decision unless they are already
          authorised.
        </CardDescription>
      </CardHeader>
      <CardContent className="task-ai-prompt-content">
        {!hasOrganiser && (
          <Alert>
            <BotIcon aria-hidden="true" />
            <AlertTitle>Choose an orchestrator first</AlertTitle>
            <AlertDescription>
              Set an orchestrator default in Settings before asking task AI.
            </AlertDescription>
          </Alert>
        )}
        <form aria-label="Ask task AI" onSubmit={(event) => void submit(event)}>
          <Label htmlFor={`ticket-ai-action-${workItemId}`}>
            How can task AI help?
          </Label>
          <Select onValueChange={setAction} value={action}>
            <SelectTrigger id={`ticket-ai-action-${workItemId}`}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {actions.map(({ label, value }) => (
                <SelectItem key={value} value={value}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="form-hint">{activeAction.hint}</p>
          <Label htmlFor={`ticket-ai-prompt-${workItemId}`}>
            What do you need?
          </Label>
          <Textarea
            disabled={!hasOrganiser || busy}
            id={`ticket-ai-prompt-${workItemId}`}
            onChange={(event) => setPrompt(event.target.value)}
            placeholder="For example: explain why the latest review failed and what to fix first."
            value={prompt}
          />
          <Button
            disabled={!hasOrganiser || busy || prompt.trim().length === 0}
            type="submit"
          >
            <BotIcon data-icon="inline-start" /> Ask task AI
          </Button>
        </form>
        <TicketEffectHistory effects={effects} onResolve={resolve} />
      </CardContent>
    </Card>
  );
}

function TicketEffectHistory({
  effects,
  onResolve,
}: Readonly<{
  effects: readonly TicketEffect[];
  onResolve: (
    effectId: string,
    resolution: TicketEffectResolution,
  ) => Promise<void>;
}>) {
  if (effects.length === 0) return null;
  return (
    <section aria-label="Task AI decisions" className="task-ai-history">
      <h4>Task AI decisions</h4>
      <ol className="task-history-list">
        {effects.map((effect) => (
          <li key={effect.id}>
            <p>{actionLabel(effect.action)}</p>
            <span>{effect.recommendation}</span>
            <span>{effect.rationale}</span>
            <RefinementProposal proposal={effect.proposal} />
            {effect.proposal.evidenceExplanation && (
              <span>{effect.proposal.evidenceExplanation}</span>
            )}
            {effect.proposal.workerGuidance && (
              <span>Worker guidance: {effect.proposal.workerGuidance}</span>
            )}
            <span>Result: {effect.outcome.replaceAll("_", " ")}</span>
            {effect.outcome === "awaiting_approval" && (
              <div className="task-ai-decision-actions">
                <Button
                  onClick={() => void onResolve(effect.id, "apply")}
                  size="sm"
                  type="button"
                >
                  <CheckIcon data-icon="inline-start" /> Apply
                </Button>
                <Button
                  onClick={() => void onResolve(effect.id, "reject")}
                  size="sm"
                  type="button"
                  variant="outline"
                >
                  <XIcon data-icon="inline-start" /> Reject
                </Button>
                <Button
                  onClick={() => void onResolve(effect.id, "cancel")}
                  size="sm"
                  type="button"
                  variant="ghost"
                >
                  Dismiss
                </Button>
              </div>
            )}
          </li>
        ))}
      </ol>
    </section>
  );
}

function RefinementProposal({
  proposal,
}: Readonly<{ proposal: TicketEffect["proposal"] }>) {
  if (proposal.title === undefined) return null;
  return (
    <>
      <span>Proposed title: {proposal.title}</span>
      {proposal.description && (
        <span>Proposed details: {proposal.description}</span>
      )}
      {proposal.acceptanceCriteria.length > 0 && (
        <>
          <span>Proposed success checks:</span>
          <ul className="criteria-list">
            {proposal.acceptanceCriteria.map((criterion) => (
              <li key={criterion}>{criterion}</li>
            ))}
          </ul>
        </>
      )}
    </>
  );
}

function actionLabel(action: TicketEffectAction): string {
  return (
    actions.find(({ value }) => value === action)?.label ?? "Task AI request"
  );
}
