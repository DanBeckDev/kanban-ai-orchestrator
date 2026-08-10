import { CircleAlertIcon, CircleCheckIcon, UserRoundIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import type { TaskDecision } from "./taskDetailPresentation";

type TaskDecisionSummaryProps = Readonly<{
  decision: TaskDecision;
}>;

export function TaskDecisionSummary({ decision }: TaskDecisionSummaryProps) {
  const hasUnresolvedBlocker = decision.blockers.some(
    ({ unresolved }) => unresolved,
  );

  return (
    <Card className="task-decision-card" data-state={decision.state}>
      <CardHeader>
        <Badge variant={stateVariant(decision.state)}>
          {decision.stateLabel}
        </Badge>
        <CardTitle as="h3">{decision.title}</CardTitle>
        <CardDescription>{decision.description}</CardDescription>
      </CardHeader>
      <CardContent className="task-decision-content">
        <dl>
          <div>
            <dt>Next permitted action</dt>
            <dd>{decision.nextAction}</dd>
          </div>
          <div>
            <dt>Worker</dt>
            <dd>{decision.worker}</dd>
          </div>
          <div>
            <dt>Evidence</dt>
            <dd>{decision.evidenceStatus}</dd>
          </div>
        </dl>
        {hasUnresolvedBlocker && (
          <Alert variant="destructive">
            <CircleAlertIcon aria-hidden="true" />
            <AlertTitle>Waiting on prerequisite work</AlertTitle>
            <AlertDescription>
              The dependencies section explains the owner and next action. The
              Kanban checks whether this task can start.
            </AlertDescription>
          </Alert>
        )}
        {decision.state === "done" && (
          <Alert>
            <CircleCheckIcon aria-hidden="true" />
            <AlertTitle>Completion is recorded</AlertTitle>
            <AlertDescription>
              Review evidence and task history remain available below.
            </AlertDescription>
          </Alert>
        )}
        <p className="task-decision-actor">
          <UserRoundIcon aria-hidden="true" />
          Kanban applies the task state and permission rules for this board.
        </p>
      </CardContent>
    </Card>
  );
}

function stateVariant(
  state: TaskDecision["state"],
): "default" | "secondary" | "destructive" | "outline" {
  if (["blocked", "failed", "interrupted"].includes(state)) {
    return "destructive";
  }
  if (["done", "cancelled"].includes(state)) return "secondary";
  if (state === "review") return "default";
  return "outline";
}
