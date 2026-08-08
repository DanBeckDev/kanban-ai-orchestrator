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
      aria-label={hasProposal ? "Regenerate board plan" : "Generate board plan"}
      onSubmit={submit}
    >
      <h4>Generate a proposal</h4>
      <p className="field-hint">
        The planner can only return an unconfirmed proposal. Review the exact
        tasks and dependencies before creating anything.
      </p>
      {profiles.length === 0 ? (
        <p className="field-hint">
          Save a planner profile below before generating a plan.
        </p>
      ) : (
        <>
          <label>
            Planner profile
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
            Goal
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
            {hasProposal ? "Generate revised preview" : "Generate preview"}
          </button>
        </>
      )}
    </form>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
