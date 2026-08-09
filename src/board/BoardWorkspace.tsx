import { useCallback, useEffect, useState } from "react";
import { AlertCircleIcon, RefreshCwIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { BoardLibrary } from "./BoardLibrary";
import { BoardSetup, type RepositoryPicker } from "./BoardSetup";
import { BoardView } from "./BoardView";
import { useBoardSnapshotRefresh } from "./useBoardSnapshotRefresh";
import { useDefaultAgentProfileName } from "./agentPreferences";
import { tauriBoardGateway } from "./gateway";
import { selectRepository } from "./repositoryPicker";
import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  BoardGateway,
  BoardLibraryEntry,
  BoardPlan,
  BoardSnapshot,
  ConfirmPlanRequest,
  CreateLocalBoardRequest,
  CreateWorkItemRequest,
  GeneratePlanRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  QueueLinearCommentRequest,
  ProposePlanRequest,
  PlannerProfile,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  TransitionWorkItemRequest,
} from "./types";

type BoardWorkspaceProps = Readonly<{
  gateway?: BoardGateway;
  repositoryPicker?: RepositoryPicker;
}>;

const disconnectedLinearStatus: LinearConnectionStatus = {
  kind: "disconnected",
};

export function BoardWorkspace({
  gateway = tauriBoardGateway,
  repositoryPicker = selectRepository,
}: BoardWorkspaceProps) {
  const [snapshot, setSnapshot] = useState<BoardSnapshot>();
  const [boardLibrary, setBoardLibrary] =
    useState<readonly BoardLibraryEntry[]>();
  const [showBoardSetup, setShowBoardSetup] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [agentProfiles, setAgentProfiles] = useState<readonly AgentProfile[]>(
    [],
  );
  const [providerAvailability, setProviderAvailability] = useState<
    readonly AgentProviderAvailability[]
  >([]);
  const { defaultAgentProfileName, selectDefaultAgentProfile } =
    useDefaultAgentProfileName();
  const [plannerProfiles, setPlannerProfiles] = useState<
    readonly PlannerProfile[]
  >([]);
  const [boardPlan, setBoardPlan] = useState<BoardPlan>();
  const [linearConnectionStatus, setLinearConnectionStatus] =
    useState<LinearConnectionStatus>(disconnectedLinearStatus);
  const [linearIssues, setLinearIssues] = useState<
    readonly LinearIssueSummary[]
  >([]);

  async function run(operation: () => Promise<BoardSnapshot | undefined>) {
    setBusy(true);
    setError(undefined);
    try {
      const updatedSnapshot = await operation();
      if (updatedSnapshot !== undefined) {
        setSnapshot(updatedSnapshot);
      }
    } catch (operationError) {
      setError(errorMessage(operationError));
    } finally {
      setBusy(false);
    }
  }

  async function createBoard(input: CreateLocalBoardRequest) {
    await run(async () => {
      const boardSnapshot = await gateway.createLocalBoard(input);
      await loadBoardContext(boardSnapshot.board.id);
      return boardSnapshot;
    });
  }

  async function openBoard(boardId: string) {
    await run(async () => {
      const boardSnapshot = await gateway.openBoard(boardId);
      await loadBoardContext(boardId);
      return boardSnapshot;
    });
  }

  const loadBoardLibrary = useCallback(async () => {
    setError(undefined);
    try {
      setBoardLibrary(await gateway.boardLibrary());
    } catch (libraryError) {
      setError(errorMessage(libraryError));
    }
  }, [gateway]);

  async function createWorkItem(request: CreateWorkItemRequest) {
    await run(() => gateway.createWorkItem(request));
  }

  async function addDependency(request: AddDependencyRequest) {
    await run(() => gateway.addDependency(request));
  }

  async function proposePlan(request: ProposePlanRequest) {
    setBusy(true);
    setError(undefined);
    try {
      setBoardPlan(await gateway.proposePlan(request));
    } catch (operationError) {
      setError(errorMessage(operationError));
      throw operationError;
    } finally {
      setBusy(false);
    }
  }

  async function generatePlan(request: GeneratePlanRequest) {
    setBusy(true);
    setError(undefined);
    try {
      setBoardPlan(await gateway.generatePlan(request));
    } catch (operationError) {
      setError(errorMessage(operationError));
      throw operationError;
    } finally {
      setBusy(false);
    }
  }

  async function confirmPlan(request: ConfirmPlanRequest) {
    setBusy(true);
    setError(undefined);
    try {
      const updatedSnapshot = await gateway.confirmPlan(request);
      setSnapshot(updatedSnapshot);
      setBoardPlan(await gateway.boardPlan(request.boardId));
    } catch (operationError) {
      setError(errorMessage(operationError));
    } finally {
      setBusy(false);
    }
  }

  async function transitionWorkItem(request: TransitionWorkItemRequest) {
    await run(() => gateway.transitionWorkItem(request));
  }

  async function saveAgentProfile(profile: AgentProfile) {
    setBusy(true);
    setError(undefined);
    try {
      await gateway.saveAgentProfile(profile);
      setAgentProfiles(await gateway.agentProfiles());
      return true;
    } catch (operationError) {
      setError(errorMessage(operationError));
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function savePlannerProfile(profile: PlannerProfile) {
    setBusy(true);
    setError(undefined);
    try {
      await gateway.savePlannerProfile(profile);
      setPlannerProfiles(await gateway.plannerProfiles());
    } catch (operationError) {
      setError(errorMessage(operationError));
      throw operationError;
    } finally {
      setBusy(false);
    }
  }

  async function startExecution(request: StartExecutionRequest) {
    await run(() => gateway.startExecution(request));
  }

  async function coordinateBoard(boardId: string, agentProfileName: string) {
    await run(() => gateway.coordinateBoard(boardId, agentProfileName));
  }

  async function stopExecution(executionId: string) {
    await run(() => gateway.stopExecution(executionId));
  }

  async function recordReviewCheck(request: RecordReviewCheckRequest) {
    await run(() => gateway.recordReviewCheck(request));
  }

  async function recordReviewDecision(request: RecordReviewDecisionRequest) {
    await run(() => gateway.recordReviewDecision(request));
  }

  async function recordCleanCodeReview(request: RecordCleanCodeReviewRequest) {
    await run(() => gateway.recordCleanCodeReview(request));
  }

  async function importLinearIssue(request: ImportLinearIssueRequest) {
    await run(() => gateway.importLinearIssue(request));
  }

  async function importLinearBlocker(request: ImportLinearBlockerRequest) {
    await run(() => gateway.importLinearBlocker(request));
  }

  const refreshLinearConnectionStatus = useCallback(async () => {
    try {
      setLinearConnectionStatus(await gateway.linearConnectionStatus());
    } catch (connectionError) {
      setLinearConnectionStatus({
        kind: "failed",
        message: errorMessage(connectionError),
      });
    }
  }, [gateway]);

  async function loadBoardContext(boardId: string) {
    const [profiles, planners, plan, providers] = await Promise.all([
      gateway.agentProfiles(),
      gateway.plannerProfiles(),
      gateway.boardPlan(boardId),
      gateway.agentProviderAvailability(),
    ]);
    setAgentProfiles(profiles);
    setPlannerProfiles(planners);
    setBoardPlan(plan);
    setProviderAvailability(providers);
    setLinearIssues([]);
    await refreshLinearConnectionStatus();
  }

  async function beginLinearOAuth(configuration: LinearOAuthConfiguration) {
    setBusy(true);
    setError(undefined);
    try {
      setLinearConnectionStatus(await gateway.beginLinearOAuth(configuration));
    } catch (connectionError) {
      setError(errorMessage(connectionError));
    } finally {
      setBusy(false);
    }
  }

  async function beginLinearCommentAccess() {
    setBusy(true);
    setError(undefined);
    try {
      setLinearConnectionStatus(await gateway.beginLinearCommentAccess());
    } catch (connectionError) {
      setError(errorMessage(connectionError));
    } finally {
      setBusy(false);
    }
  }

  async function queueLinearComment(request: QueueLinearCommentRequest) {
    await run(() => gateway.queueLinearComment(request));
  }

  async function deliverLinearComment(outboxItemId: string) {
    await run(() => gateway.deliverLinearComment(outboxItemId));
  }

  async function refreshLinearSharedFields(externalLinkId: string) {
    await run(() => gateway.syncLinearSharedFields(externalLinkId));
  }

  async function loadLinearAssignedIssues() {
    setBusy(true);
    setError(undefined);
    try {
      setLinearIssues(await gateway.linearAssignedIssues());
    } catch (linearError) {
      setError(errorMessage(linearError));
    } finally {
      setBusy(false);
    }
  }

  const loadExecutionActivity = useCallback(
    (executionId: string, afterSequence?: number) =>
      gateway.executionActivity(executionId, afterSequence),
    [gateway],
  );

  const boardLibraryLoadFailed = !snapshot && !boardLibrary && Boolean(error);
  useEffect(() => {
    void loadBoardLibrary();
  }, [loadBoardLibrary]);

  useBoardSnapshotRefresh({
    boardId: snapshot?.board.id,
    gateway,
    isAwaitingLinearAuthorization:
      linearConnectionStatus.kind === "awaiting_authorization",
    onLinearStatusRefresh: refreshLinearConnectionStatus,
    onSnapshot: setSnapshot,
  });

  return (
    <section className="board-shell">
      {error !== undefined && !boardLibraryLoadFailed && (
        <Alert
          aria-live="polite"
          className="error-notice"
          variant="destructive"
        >
          <AlertCircleIcon aria-hidden="true" />
          <AlertTitle>Kanban could not complete that request.</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      {snapshot === undefined && boardLibraryLoadFailed ? (
        <Empty className="board-library-loading">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <AlertCircleIcon />
            </EmptyMedia>
            <EmptyTitle aria-level={2} role="heading">
              Kanban could not load your boards
            </EmptyTitle>
            <EmptyDescription>
              Try again. If it keeps happening, restart Kanban.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={() => void loadBoardLibrary()} type="button">
              <RefreshCwIcon data-icon="inline-start" />
              Try again
            </Button>
          </EmptyContent>
        </Empty>
      ) : snapshot === undefined && boardLibrary === undefined ? (
        <Empty aria-live="polite" className="board-library-loading">
          <EmptyHeader>
            <EmptyTitle>Loading your local boards…</EmptyTitle>
            <EmptyDescription>
              Reading the boards stored on this device.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : snapshot === undefined && showBoardSetup ? (
        <BoardSetup
          busy={busy}
          repositoryPicker={repositoryPicker}
          onInspectRepository={gateway.inspectRepository}
          onBack={() => {
            setShowBoardSetup(false);
            void loadBoardLibrary();
          }}
          onCreate={createBoard}
        />
      ) : snapshot === undefined ? (
        <BoardLibrary
          boards={boardLibrary ?? []}
          busy={busy}
          onCreateBoard={() => setShowBoardSetup(true)}
          onOpenBoard={(boardId) => void openBoard(boardId)}
        />
      ) : (
        <BoardView
          busy={busy}
          agentProfiles={agentProfiles}
          defaultAgentProfileName={defaultAgentProfileName}
          providerAvailability={providerAvailability}
          onAddDependency={addDependency}
          boardPlan={boardPlan}
          onConfirmPlan={confirmPlan}
          onCreateWorkItem={createWorkItem}
          onImportLinearBlocker={importLinearBlocker}
          onImportLinearIssue={importLinearIssue}
          linearConnectionStatus={linearConnectionStatus}
          linearIssues={linearIssues}
          onConnectLinear={beginLinearOAuth}
          onEnableLinearCommentAccess={beginLinearCommentAccess}
          onLoadLinearIssues={loadLinearAssignedIssues}
          onQueueLinearComment={queueLinearComment}
          onDeliverLinearComment={deliverLinearComment}
          onRefreshLinearSharedFields={refreshLinearSharedFields}
          onProposePlan={proposePlan}
          onGeneratePlan={generatePlan}
          onSaveAgentProfile={saveAgentProfile}
          onSelectDefaultAgentProfile={selectDefaultAgentProfile}
          onSavePlannerProfile={savePlannerProfile}
          onCoordinateBoard={coordinateBoard}
          onStartExecution={startExecution}
          onStopExecution={stopExecution}
          onLoadExecutionActivity={loadExecutionActivity}
          onRecordReviewCheck={recordReviewCheck}
          onRecordReviewDecision={recordReviewDecision}
          onRecordCleanCodeReview={recordCleanCodeReview}
          onTransition={transitionWorkItem}
          snapshot={snapshot}
          plannerProfiles={plannerProfiles}
        />
      )}
    </section>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
