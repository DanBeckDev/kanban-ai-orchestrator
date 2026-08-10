import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

import { AgentProfileForm } from "./AgentProfileForm";
import { BoardAutomation } from "./BoardAutomation";
import { BoardSupportDetails } from "./BoardSupportDetails";
import { LinearBoardModeNotice } from "./LinearBoardModeNotice";
import { LinearConnectionPanel } from "./LinearConnectionPanel";
import { LinearImportForm } from "./LinearImportForm";
import { LinearSyncPanel } from "./LinearSyncPanel";
import { productManagedLinearOAuthConfiguration } from "./linearConnectionPresentation";
import { PlannerProfileForm } from "./PlannerProfileForm";
import { ProjectAgentDefaultsForm } from "./ProjectAgentDefaultsForm";
import { SurfaceHeader } from "./BoardManagement";
import type {
  AgentProfile,
  AgentProviderAvailability,
  BoardSnapshot,
  BoardSupervision,
  BoardSupervisionMode,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  PlannerProfile,
  ProjectAgentSettings,
  ProviderModelCatalog,
  QueueLinearCommentRequest,
  SaveProjectAgentSettingsRequest,
  SupervisionDecision,
} from "./types";

type BoardSettingsProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  boardSupervision?: BoardSupervision;
  busy: boolean;
  linearConnectionStatus: LinearConnectionStatus;
  linearIssues: readonly LinearIssueSummary[];
  plannerProfiles: readonly PlannerProfile[];
  projectAgentSettings?: ProjectAgentSettings;
  providerAvailability: readonly AgentProviderAvailability[];
  snapshot: BoardSnapshot;
  supervisionDecisions: readonly SupervisionDecision[];
  onBack: () => void;
  onConfigureBoardSupervision: (mode: BoardSupervisionMode) => Promise<void>;
  onConnectLinear: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onCoordinateBoard: (boardId: string) => Promise<void>;
  onDeliverLinearComment: (outboxItemId: string) => Promise<void>;
  onEnableLinearCommentAccess: () => Promise<void>;
  onImportLinearBlocker: (request: ImportLinearBlockerRequest) => Promise<void>;
  onImportLinearIssue: (request: ImportLinearIssueRequest) => Promise<void>;
  onLoadLinearIssues: () => Promise<void>;
  onQueueLinearComment: (request: QueueLinearCommentRequest) => Promise<void>;
  onRefreshLinearSharedFields: (externalLinkId: string) => Promise<void>;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<boolean>;
  onLoadProviderCatalog: (
    provider: AgentProviderAvailability,
  ) => Promise<ProviderModelCatalog>;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
  onSaveProjectAgentSettings: (
    request: SaveProjectAgentSettingsRequest,
  ) => Promise<void>;
}>;

export function BoardSettings({
  agentProfiles,
  boardSupervision,
  busy,
  linearConnectionStatus,
  linearIssues,
  plannerProfiles,
  projectAgentSettings,
  providerAvailability,
  snapshot,
  supervisionDecisions,
  onBack,
  onConfigureBoardSupervision,
  onConnectLinear,
  onCoordinateBoard,
  onDeliverLinearComment,
  onEnableLinearCommentAccess,
  onImportLinearBlocker,
  onImportLinearIssue,
  onLoadLinearIssues,
  onQueueLinearComment,
  onRefreshLinearSharedFields,
  onSaveAgentProfile,
  onLoadProviderCatalog,
  onSavePlannerProfile,
  onSaveProjectAgentSettings,
}: BoardSettingsProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);
  const hasConfiguredRoles =
    projectAgentSettings?.organiser !== undefined &&
    projectAgentSettings.ticketWorker !== undefined &&
    agentProfiles.some(
      ({ name }) =>
        name === projectAgentSettings.ticketWorker?.agentProfileName,
    );

  return (
    <section aria-label="Settings" className="workspace-surface">
      <SurfaceHeader
        backLabel="Back to Tickets"
        description="Choose the AI roles and connected tools for this project."
        onBack={onBack}
        title="Settings"
      />
      <Tabs defaultValue="ai" orientation="vertical">
        <TabsList aria-label="Settings sections" variant="line">
          <TabsTrigger value="ai">AI</TabsTrigger>
          <TabsTrigger value="automation">Automation</TabsTrigger>
          <TabsTrigger value="linear">Linear</TabsTrigger>
          <TabsTrigger value="project">Project</TabsTrigger>
        </TabsList>
        <TabsContent value="ai">
          <ProjectAgentDefaultsForm
            agentProfiles={agentProfiles}
            boardId={snapshot.board.id}
            busy={busy}
            plannerProfiles={plannerProfiles}
            providerAvailability={providerAvailability}
            settings={projectAgentSettings}
            onSaveAgentProfile={onSaveAgentProfile}
            onLoadProviderCatalog={onLoadProviderCatalog}
            onSavePlannerProfile={onSavePlannerProfile}
            onSaveSettings={onSaveProjectAgentSettings}
          />
          <details className="advanced-disclosure">
            <summary>Add an orchestrator</summary>
            <p className="field-hint">
              Use an approved planning bridge that returns a validated plan.
            </p>
            <PlannerProfileForm
              busy={busy}
              onSave={onSavePlannerProfile}
              profiles={plannerProfiles}
            />
          </details>
          <details className="advanced-disclosure">
            <summary>Set up a custom ticket worker</summary>
            <p className="field-hint">
              Use this for a team-managed bridge or an existing advanced setup.
            </p>
            <AgentProfileForm
              busy={busy}
              onSave={onSaveAgentProfile}
              profiles={agentProfiles}
            />
          </details>
        </TabsContent>
        <TabsContent value="automation">
          <BoardAutomation
            decisions={supervisionDecisions}
            hasConfiguredRoles={hasConfiguredRoles}
            snapshot={snapshot}
            supervision={boardSupervision}
            onConfigure={onConfigureBoardSupervision}
            onCoordinate={onCoordinateBoard}
          />
        </TabsContent>
        <TabsContent value="linear">
          <section className="settings-section">
            <div>
              <h3>Linear</h3>
              <p>
                Connect and import only when Linear belongs in this workflow.
              </p>
            </div>
            <LinearBoardModeNotice snapshot={snapshot} />
            <LinearConnectionPanel
              busy={busy}
              productManagedConfiguration={productManagedLinearOAuthConfiguration()}
              status={linearConnectionStatus}
              onConnect={onConnectLinear}
              onEnableCommentAccess={onEnableLinearCommentAccess}
            />
            <Separator />
            <LinearImportForm
              busy={busy}
              connectionStatus={linearConnectionStatus}
              issues={linearIssues}
              workItems={workItems}
              onImportBlocker={onImportLinearBlocker}
              onImportIssue={onImportLinearIssue}
              onLoadIssues={onLoadLinearIssues}
            />
            <Separator />
            <LinearSyncPanel
              busy={busy}
              connectionStatus={linearConnectionStatus}
              snapshot={snapshot}
              onDeliver={onDeliverLinearComment}
              onQueue={onQueueLinearComment}
              onRefresh={onRefreshLinearSharedFields}
            />
          </section>
        </TabsContent>
        <TabsContent value="project">
          <section className="settings-section">
            <div>
              <h3>Project details</h3>
              <p>
                These details help with support and do not change existing work.
              </p>
            </div>
            <BoardSupportDetails board={snapshot.board} />
          </section>
        </TabsContent>
      </Tabs>
    </section>
  );
}
