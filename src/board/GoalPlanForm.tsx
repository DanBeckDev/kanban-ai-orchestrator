import { useState, type FormEvent } from "react";
import { SparklesIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

import type { GeneratePlanRequest, PlannerProfile } from "./types";

type GoalPlanFormProps = Readonly<{
  boardId: string;
  busy: boolean;
  hasProposal: boolean;
  label?: string;
  profiles: readonly PlannerProfile[];
  onGenerate: (request: GeneratePlanRequest) => Promise<void>;
}>;

export function GoalPlanForm({
  boardId,
  busy,
  hasProposal,
  label,
  profiles,
  onGenerate,
}: GoalPlanFormProps) {
  const [goal, setGoal] = useState("");
  const [plannerProfileName, setPlannerProfileName] = useState("");
  const [generationError, setGenerationError] = useState<string>();
  const selectedProfile = plannerProfileName || profiles[0]?.name || "";
  const formLabel =
    label ?? (hasProposal ? "Revise plan with AI" : "Plan with AI");

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await onGenerate({ boardId, goal, plannerProfileName: selectedProfile });
      setGenerationError(undefined);
    } catch (error) {
      setGenerationError(errorMessage(error));
    }
  }

  if (profiles.length === 0) {
    return (
      <Alert>
        <AlertTitle>Set up an organiser first</AlertTitle>
        <AlertDescription>
          Choose an organiser connection in Settings before you create a plan.
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <form aria-label={formLabel} className="goal-plan-form" onSubmit={submit}>
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="planner-profile">Organiser</FieldLabel>
          <Select onValueChange={setPlannerProfileName} value={selectedProfile}>
            <SelectTrigger id="planner-profile">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {profiles.map((profile) => (
                  <SelectItem key={profile.name} value={profile.name}>
                    {profile.name}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
        <Field>
          <FieldLabel htmlFor="planning-goal">
            What do you want to achieve?
          </FieldLabel>
          <Textarea
            id="planning-goal"
            name="goal"
            onChange={(event) => setGoal(event.target.value)}
            required
            value={goal}
          />
          <FieldDescription>
            Kanban will propose tasks and their order. Nothing starts until you
            review and confirm the proposal.
          </FieldDescription>
        </Field>
        {generationError !== undefined && (
          <Alert role="alert" variant="destructive">
            <AlertTitle>Kanban could not create a plan preview</AlertTitle>
            <AlertDescription>{generationError}</AlertDescription>
          </Alert>
        )}
        <Button disabled={busy} type="submit">
          <SparklesIcon data-icon="inline-start" />
          {hasProposal ? "Create revised preview" : "Create plan preview"}
        </Button>
      </FieldGroup>
    </form>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
