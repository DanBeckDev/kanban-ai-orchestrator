import { useEffect, useMemo, useRef } from "react";
import { BotIcon, CirclePauseIcon, UserRoundCheckIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import {
  type CoordinationMode,
  useCoordinationMode,
} from "./orchestrationPreferences";
import type { BoardSnapshot } from "./types";

type BoardAutomationProps = Readonly<{
  defaultAgentProfileName?: string;
  hasDefaultAgent: boolean;
  snapshot: BoardSnapshot;
  onCoordinate: (boardId: string, agentProfileName: string) => Promise<void>;
}>;

export function BoardAutomation({
  defaultAgentProfileName,
  hasDefaultAgent,
  snapshot,
  onCoordinate,
}: BoardAutomationProps) {
  const { mode, selectMode } = useCoordinationMode(snapshot.board.id);
  const lastRequestedSignature = useRef<string>();
  const eligibleWorkSignature = useMemo(
    () =>
      snapshot.workItems
        .filter(({ workItem }) =>
          ["inbox", "planned", "ready"].includes(workItem.state),
        )
        .map(({ workItem }) => `${workItem.id}:${workItem.state}`)
        .join(","),
    [snapshot.workItems],
  );
  const canCoordinate =
    defaultAgentProfileName !== undefined && hasDefaultAgent;
  const isAutonomous = mode === "autonomous" && canCoordinate;

  useEffect(() => {
    if (!canCoordinate && mode === "autonomous") selectMode("manual");
  }, [canCoordinate, mode, selectMode]);

  useEffect(() => {
    if (
      mode !== "autonomous" ||
      defaultAgentProfileName === undefined ||
      !hasDefaultAgent ||
      eligibleWorkSignature === ""
    ) {
      return;
    }
    const signature = `${snapshot.board.id}:${defaultAgentProfileName}:${eligibleWorkSignature}`;
    if (lastRequestedSignature.current === signature) return;
    lastRequestedSignature.current = signature;
    void onCoordinate(snapshot.board.id, defaultAgentProfileName);
  }, [
    defaultAgentProfileName,
    eligibleWorkSignature,
    hasDefaultAgent,
    mode,
    onCoordinate,
    snapshot.board.id,
  ]);

  useEffect(() => {
    if (mode === "manual") lastRequestedSignature.current = undefined;
  }, [mode]);

  function chooseMode(value: string) {
    if (isCoordinationMode(value)) selectMode(value);
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
          onValueChange={chooseMode}
          spacing={0}
          type="single"
          value={mode}
          variant="outline"
        >
          <ToggleGroupItem value="manual">
            <UserRoundCheckIcon data-icon="inline-start" />
            You approve actions
          </ToggleGroupItem>
          <ToggleGroupItem disabled={!canCoordinate} value="autonomous">
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
                Kanban prepares tasks in dependency order and starts one ready
                task at a time with <strong>{defaultAgentProfileName}</strong>.
                Completed work always returns to Review.
              </AlertDescription>
            </Alert>
            <Button
              onClick={() => selectMode("manual")}
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
            starts. Kanban still shows what is blocked and ready.
          </p>
        )}
        {!canCoordinate && (
          <Alert>
            <CirclePauseIcon aria-hidden="true" />
            <AlertTitle>Choose a task agent first</AlertTitle>
            <AlertDescription>
              Select an installed agent in Settings before you turn on
              coordination.
            </AlertDescription>
          </Alert>
        )}
        <p className="board-automation-safeguard">
          <Badge variant="secondary">Bounded</Badge> Kanban never marks a task
          done or sends Linear updates here. Task agents retain the permissions
          configured for their own profile.
        </p>
      </CardContent>
    </Card>
  );
}

function isCoordinationMode(value: string): value is CoordinationMode {
  return value === "manual" || value === "autonomous";
}
