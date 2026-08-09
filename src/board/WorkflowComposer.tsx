import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import { GoalPlanForm } from "./GoalPlanForm";
import type { GeneratePlanRequest, PlannerProfile } from "./types";

type WorkflowComposerProps = Readonly<{
  boardId: string;
  busy: boolean;
  defaultPlannerProfileName?: string;
  plannerProfiles: readonly PlannerProfile[];
  onGeneratePlan: (request: GeneratePlanRequest) => Promise<void>;
}>;

export function WorkflowComposer({
  boardId,
  busy,
  defaultPlannerProfileName,
  plannerProfiles,
  onGeneratePlan,
}: WorkflowComposerProps) {
  return (
    <section aria-labelledby="workflow-composer-title">
      <Card className="workflow-composer">
        <CardHeader>
          <CardTitle as="h3" id="workflow-composer-title">
            Prompt AI to orchestrate
          </CardTitle>
          <CardDescription>
            Describe the outcome. Kanban will create a reviewable proposal
            before any ticket or worker is started.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <GoalPlanForm
            boardId={boardId}
            busy={busy}
            defaultPlannerProfileName={defaultPlannerProfileName}
            hasProposal={false}
            label="Prompt AI to orchestrate"
            onGenerate={onGeneratePlan}
            profiles={plannerProfiles}
          />
        </CardContent>
      </Card>
    </section>
  );
}
