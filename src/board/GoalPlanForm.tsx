import { useState, type FormEvent } from "react";

import type { GeneratePlanRequest, PlannerProfile } from "./types";

type GoalPlanFormProps = Readonly<{
  boardId: string;
  busy: boolean;
  hasProposal: boolean;
  profiles: readonly PlannerProfile[];
  onGenerate: (request: GeneratePlanRequest) => Promise<void>;
}>;

export function GoalPlanForm({
  boardId,
  busy,
  hasProposal,
  profiles,
  onGenerate,
}: GoalPlanFormProps) {
  const [goal, setGoal] = useState("");
  const [plannerProfileName, setPlannerProfileName] = useState("");
  const [generationError, setGenerationError] = useState<string>();
  const selectedProfile = plannerProfileName || profiles[0]?.name || "";

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await onGenerate({ boardId, goal, plannerProfileName: selectedProfile });
      setGenerationError(undefined);
    } catch (error) {
      setGenerationError(errorMessage(error));
    }
  }

  return (
    <form
      aria-label={hasProposal ? "Revise plan with AI" : "Plan with AI"}
      onSubmit={submit}
    >
      <h4>Ask the organiser</h4>
      <p className="field-hint">
        Describe the outcome in your own words. Kanban will suggest tasks and
        their order; nothing is created until you review and confirm it.
      </p>
      {profiles.length === 0 ? (
        <p className="field-hint">
          Set up an organiser connection in Settings before you create a plan.
        </p>
      ) : (
        <>
          <label>
            Use this organiser
            <select
              required
              value={selectedProfile}
              onChange={(event) => setPlannerProfileName(event.target.value)}
            >
              {profiles.map((profile) => (
                <option key={profile.name} value={profile.name}>
                  {profile.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            What do you want to achieve?
            <textarea
              required
              value={goal}
              onChange={(event) => setGoal(event.target.value)}
            />
          </label>
          {generationError !== undefined && (
            <p className="inline-error" role="alert">
              {generationError}
            </p>
          )}
          <button disabled={busy} type="submit">
            {hasProposal ? "Create revised preview" : "Create plan preview"}
          </button>
        </>
      )}
    </form>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
