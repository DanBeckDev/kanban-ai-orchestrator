import { useCallback, useEffect, useState } from "react";
import { BoardSetup, type CreateBoardInput } from "./BoardSetup";
import { BoardView } from "./BoardView";
import { tauriBoardGateway } from "./gateway";
import type {
  AddDependencyRequest,
  AgentProfile,
  BoardGateway,
  BoardPlan,
  BoardSnapshot,
  ConfirmPlanRequest,
  CreateWorkItemRequest,
  GeneratePlanRequest,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
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
}>;

const disconnectedLinearStatus: LinearConnectionStatus = {
  kind: "disconnected",
};

export function BoardWorkspace({
  gateway = tauriBoardGateway,
}: BoardWorkspaceProps) {
  const [snapshot, setSnapshot] = useState<BoardSnapshot>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [agentProfiles, setAgentProfiles] = useState<readonly AgentProfile[]>(
    [],
  );
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

  async function createBoard(input: CreateBoardInput) {
    await run(async () => {
      await gateway.createProject({
        projectId: input.projectId,
        name: input.projectName,
        repositoryPath: input.repositoryPath,
        baseRef: input.baseRef,
        policySetId: input.policySetId,
      });
      const boardSnapshot = await gateway.createBoard({
        boardId: input.boardId,
        projectId: input.projectId,
        name: input.boardName,
      });
      await loadBoardContext(input.boardId);
      return boardSnapshot;
    });
  }

  async function openBoard(boardId: string) {
    await run(async () => {
      const boardSnapshot = await gateway.boardSnapshot(boardId);
      await loadBoardContext(boardId);
      return boardSnapshot;
    });
  }

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
    } catch (operationError) {
      setError(errorMessage(operationError));
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
    setAgentProfiles(await gateway.agentProfiles());
    setPlannerProfiles(await gateway.plannerProfiles());
    setBoardPlan(await gateway.boardPlan(boardId));
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

  const boardId = snapshot?.board.id;
  useEffect(() => {
    if (boardId === undefined) return undefined;
    const refresh = () => {
      void gateway
        .boardSnapshot(boardId)
        .then(setSnapshot)
        .catch(() => undefined);
      if (linearConnectionStatus.kind === "awaiting_authorization") {
        void refreshLinearConnectionStatus();
      }
    };
    const intervalId = window.setInterval(refresh, 1_000);
    return () => window.clearInterval(intervalId);
  }, [
    boardId,
    gateway,
    linearConnectionStatus.kind,
    refreshLinearConnectionStatus,
  ]);

  return (
    <section className="board-shell">
      {error !== undefined && (
        <div aria-live="polite" className="error-notice" role="alert">
          <strong>The local daemon rejected that request.</strong> {error}
        </div>
      )}
      {snapshot === undefined ? (
        <BoardSetup busy={busy} onCreate={createBoard} onOpen={openBoard} />
      ) : (
        <BoardView
          busy={busy}
          agentProfiles={agentProfiles}
          onAddDependency={addDependency}
          boardPlan={boardPlan}
          onConfirmPlan={confirmPlan}
          onCreateWorkItem={createWorkItem}
          onImportLinearBlocker={importLinearBlocker}
          onImportLinearIssue={importLinearIssue}
          linearConnectionStatus={linearConnectionStatus}
          linearIssues={linearIssues}
          onConnectLinear={beginLinearOAuth}
          onLoadLinearIssues={loadLinearAssignedIssues}
          onProposePlan={proposePlan}
          onGeneratePlan={generatePlan}
          onSaveAgentProfile={saveAgentProfile}
          onSavePlannerProfile={savePlannerProfile}
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
