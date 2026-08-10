import { useCallback, useEffect, useState } from "react";
import type { RepositoryPicker } from "./BoardSetup";
import { BoardWorkspaceScreen } from "./BoardWorkspaceScreen";
import { useBoardSnapshotRefresh } from "./useBoardSnapshotRefresh";
import { errorMessage, useBoardOperation } from "./useBoardOperation";
import { tauriBoardGateway } from "./gateway";
import { ticketEffectOperations } from "./ticketEffectOperations";
import { selectCloneDestination, selectRepository } from "./repositoryPicker";
import type {
  AddDependencyRequest,
  AgentProfile,
  AgentProviderAvailability,
  BoardGateway,
  BoardLibraryEntry,
  BoardSupervision,
  BoardSupervisionMode,
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
  ProjectAgentSettings,
  RecordCleanCodeReviewRequest,
  RecordReviewCheckRequest,
  RecordReviewDecisionRequest,
  StartExecutionRequest,
  SupervisionDecision,
  SaveProjectAgentSettingsRequest,
  TransitionWorkItemRequest,
} from "./types";

type BoardWorkspaceProps = Readonly<{
  gateway?: BoardGateway;
  repositoryPicker?: RepositoryPicker;
}>;

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
  const [projectAgentSettings, setProjectAgentSettings] = useState<
    ProjectAgentSettings | undefined
  >();
  const [boardSupervision, setBoardSupervision] = useState<
    BoardSupervision | undefined
  >();
  const [supervisionDecisions, setSupervisionDecisions] = useState<
    readonly SupervisionDecision[]
  >([]);
  const [plannerProfiles, setPlannerProfiles] = useState<
    readonly PlannerProfile[]
  >([]);
  const [boardPlan, setBoardPlan] = useState<BoardPlan>();
  const [linearConnectionStatus, setLinearConnectionStatus] =
    useState<LinearConnectionStatus>({ kind: "disconnected" });
  const [linearIssues, setLinearIssues] = useState<
    readonly LinearIssueSummary[]
  >([]);

  const run = useBoardOperation({
    onError: setError,
    onSnapshot: setSnapshot,
    setBusy,
  });

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

  async function saveProjectAgentSettings(
    request: SaveProjectAgentSettingsRequest,
  ) {
    setBusy(true);
    setError(undefined);
    try {
      setProjectAgentSettings(await gateway.saveProjectAgentSettings(request));
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

  async function configureBoardSupervision(mode: BoardSupervisionMode) {
    if (snapshot === undefined) return;
    setBusy(true);
    setError(undefined);
    try {
      setBoardSupervision(
        await gateway.configureBoardSupervision(snapshot.board.id, mode),
      );
    } catch (operationError) {
      setError(errorMessage(operationError));
      throw operationError;
    } finally {
      setBusy(false);
    }
  }

  async function coordinateBoard(boardId: string) {
    await run(async () => {
      const updatedSnapshot = await gateway.coordinateBoard(boardId);
      setSupervisionDecisions(await gateway.supervisionDecisions(boardId));
      return updatedSnapshot;
    });
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
    const [
      profiles,
      planners,
      plan,
      providers,
      settings,
      supervision,
      decisions,
    ] = await Promise.all([
      gateway.agentProfiles(),
      gateway.plannerProfiles(),
      gateway.boardPlan(boardId),
      gateway.agentProviderAvailability(),
      gateway.projectAgentSettings(boardId),
      gateway.boardSupervision(boardId),
      gateway.supervisionDecisions(boardId),
    ]);
    setAgentProfiles(profiles);
    setPlannerProfiles(planners);
    setBoardPlan(plan);
    setProviderAvailability(providers);
    setProjectAgentSettings(settings);
    setBoardSupervision(supervision);
    setSupervisionDecisions(decisions);
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

  const activeBoardProps =
    snapshot === undefined
      ? { snapshot: undefined, boardViewProps: undefined }
      : {
          snapshot,
          boardViewProps: {
            busy,
            agentProfiles,
            projectAgentSettings,
            boardSupervision,
            providerAvailability,
            onAddDependency: addDependency,
            boardPlan,
            onConfirmPlan: confirmPlan,
            onCreateWorkItem: createWorkItem,
            onImportLinearBlocker: importLinearBlocker,
            onImportLinearIssue: importLinearIssue,
            linearConnectionStatus,
            linearIssues,
            onConnectLinear: beginLinearOAuth,
            onEnableLinearCommentAccess: beginLinearCommentAccess,
            onLoadLinearIssues: loadLinearAssignedIssues,
            onQueueLinearComment: queueLinearComment,
            onDeliverLinearComment: deliverLinearComment,
            onRefreshLinearSharedFields: refreshLinearSharedFields,
            onProposePlan: proposePlan,
            onGeneratePlan: generatePlan,
            onSaveAgentProfile: saveAgentProfile,
            onSaveProjectAgentSettings: saveProjectAgentSettings,
            onSavePlannerProfile: savePlannerProfile,
            ticketEffects: ticketEffectOperations(gateway, run),
            onCoordinateBoard: coordinateBoard,
            supervisionDecisions,
            onConfigureBoardSupervision: configureBoardSupervision,
            onStartExecution: startExecution,
            onStopExecution: stopExecution,
            onLoadExecutionActivity: gateway.executionActivity,
            onRecordReviewCheck: recordReviewCheck,
            onRecordReviewDecision: recordReviewDecision,
            onRecordCleanCodeReview: recordCleanCodeReview,
            onTransition: transitionWorkItem,
            snapshot,
            plannerProfiles,
          },
        };

  return (
    <BoardWorkspaceScreen
      boardLibraryLoadFailed={boardLibraryLoadFailed}
      boardLibraryLoaded={boardLibrary !== undefined}
      boardLibraryProps={{
        boards: boardLibrary ?? [],
        busy,
        onCreateBoard: () => setShowBoardSetup(true),
        onOpenBoard: (boardId) => void openBoard(boardId),
      }}
      boardSetupProps={{
        busy,
        cloneDestinationPicker: selectCloneDestination,
        repositoryPicker,
        onCloneGitHubRepository: gateway.cloneGitHubRepository,
        onInspectRepository: gateway.inspectRepository,
        onBack: () => {
          setShowBoardSetup(false);
          void loadBoardLibrary();
        },
        onCreate: createBoard,
      }}
      error={error}
      onRetryBoardLibrary={() => void loadBoardLibrary()}
      showBoardSetup={showBoardSetup}
      {...activeBoardProps}
    />
  );
}
