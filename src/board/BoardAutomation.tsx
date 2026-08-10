import { BotIcon, CirclePauseIcon, UserRoundCheckIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import type {
  BoardSnapshot,
  BoardSupervision,
  BoardSupervisionMode,
  SupervisionDecision,
} from "./types";

type BoardAutomationProps = Readonly<{
  snapshot: BoardSnapshot;
  supervision?: BoardSupervision;
  decisions: readonly SupervisionDecision[];
  hasConfiguredRoles: boolean;
  onConfigure: (mode: BoardSupervisionMode) => Promise<void>;
  onCoordinate: (boardId: string) => Promise<void>;
}>;

export function BoardAutomation({
  snapshot,
  supervision,
  decisions = [],
  hasConfiguredRoles,
  onConfigure,
  onCoordinate,
}: BoardAutomationProps) {
  const mode = supervision?.mode ?? "manual";
  const isAutonomous = mode === "autonomous";
  const latestDecision = decisions[0];

  async function chooseMode(value: string) {
    if (isBoardSupervisionMode(value)) {
      await onConfigure(value);
      if (value === "autonomous") await onCoordinate(snapshot.board.id);
    }
  }

  return (
    <Card className="board-automation" size="sm">
      <CardHeader>
        <CardTitle as="h3">How Kanban moves work</CardTitle>
        <CardDescription>
          Choose how much of the next step you want Kanban to handle.
        </CardDescription>
      </CardHeader>
      <CardContent className="board-automation-content">
        <ToggleGroup
          aria-label="How Kanban moves work"
          onValueChange={(value) => void chooseMode(value)}
          spacing={0}
          type="single"
          value={mode}
          variant="outline"
        >
          <ToggleGroupItem value="manual">
            <UserRoundCheckIcon data-icon="inline-start" />
            You approve actions
          </ToggleGroupItem>
          <ToggleGroupItem disabled={!hasConfiguredRoles} value="autonomous">
            <BotIcon data-icon="inline-start" />
            Kanban coordinates
          </ToggleGroupItem>
        </ToggleGroup>
        {isAutonomous ? (
          <>
            <Alert>
              <BotIcon aria-hidden="true" />
              <AlertTitle>Coordination is on</AlertTitle>
              <AlertDescription>
                Kanban can prepare dependency-ready work, start one worker, and
                retry once. It never marks work done or performs external
                actions. <strong>Pause automation</strong> takes effect in the
                immediately.
              </AlertDescription>
            </Alert>
            <Button
              onClick={() => void onConfigure("manual")}
              type="button"
              variant="outline"
            >
              <CirclePauseIcon data-icon="inline-start" />
              Pause automation
            </Button>
          </>
        ) : (
          <p className="board-automation-summary">
            <CirclePauseIcon aria-hidden="true" /> You decide when each task
            starts. Ask Kanban for a recorded recommendation whenever you want.
          </p>
        )}
        {!isAutonomous && hasConfiguredRoles && (
          <Button
            onClick={() => void onCoordinate(snapshot.board.id)}
            type="button"
            variant="outline"
          >
            <BotIcon data-icon="inline-start" /> Ask Kanban what to do next
          </Button>
        )}
        {!hasConfiguredRoles && (
          <Alert>
            <CirclePauseIcon aria-hidden="true" />
            <AlertTitle>Choose a task agent first</AlertTitle>
            <AlertDescription>
              Select both an orchestrator and an installed ticket worker in
              Settings before you turn on coordination.
            </AlertDescription>
          </Alert>
        )}
        <p className="board-automation-safeguard">
          Kanban stays within your saved limits. It never marks a task done or
          sends Linear updates here. Task agents retain the permissions in their
          own profile.
        </p>
        {latestDecision && (
          <Alert>
            <BotIcon aria-hidden="true" />
            <AlertTitle>Latest orchestrator decision</AlertTitle>
            <AlertDescription>
              {latestDecision.recommendation} {latestDecision.rationale} Result:{" "}
              {decisionOutcome(latestDecision)}.
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}

function isBoardSupervisionMode(value: string): value is BoardSupervisionMode {
  return value === "manual" || value === "autonomous";
}

function decisionOutcome(decision: SupervisionDecision): string {
  return decision.outcome.replaceAll("_", " ");
}
