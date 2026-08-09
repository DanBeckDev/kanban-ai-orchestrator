import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

import { AgentProfileForm } from "./AgentProfileForm";
import { BoardSupportDetails } from "./BoardSupportDetails";
import {
  defaultNativeAgentProfile,
  installationGuideFor,
} from "./agentProfilePresentation";
import { LinearConnectionPanel } from "./LinearConnectionPanel";
import { LinearImportForm } from "./LinearImportForm";
import { LinearSyncPanel } from "./LinearSyncPanel";
import { PlannerProfileForm } from "./PlannerProfileForm";
import { SurfaceHeader } from "./BoardManagement";
import type {
  AgentProfile,
  AgentProviderAvailability,
  BoardSnapshot,
  ImportLinearBlockerRequest,
  ImportLinearIssueRequest,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  PlannerProfile,
  QueueLinearCommentRequest,
} from "./types";

type BoardSettingsProps = Readonly<{
  busy: boolean;
  snapshot: BoardSnapshot;
  agentProfiles: readonly AgentProfile[];
  defaultAgentProfileName?: string;
  providerAvailability: readonly AgentProviderAvailability[];
  plannerProfiles: readonly PlannerProfile[];
  linearConnectionStatus: LinearConnectionStatus;
  linearIssues: readonly LinearIssueSummary[];
  onBack: () => void;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<boolean>;
  onSelectDefaultAgentProfile: (profileName: string) => void;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
  onConnectLinear: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onEnableLinearCommentAccess: () => Promise<void>;
  onImportLinearBlocker: (request: ImportLinearBlockerRequest) => Promise<void>;
  onImportLinearIssue: (request: ImportLinearIssueRequest) => Promise<void>;
  onQueueLinearComment: (request: QueueLinearCommentRequest) => Promise<void>;
  onDeliverLinearComment: (outboxItemId: string) => Promise<void>;
  onRefreshLinearSharedFields: (externalLinkId: string) => Promise<void>;
  onLoadLinearIssues: () => Promise<void>;
}>;

export function BoardSettings({
  busy,
  snapshot,
  agentProfiles,
  defaultAgentProfileName,
  providerAvailability,
  plannerProfiles,
  linearConnectionStatus,
  linearIssues,
  onBack,
  onSaveAgentProfile,
  onSelectDefaultAgentProfile,
  onSavePlannerProfile,
  onConnectLinear,
  onEnableLinearCommentAccess,
  onImportLinearBlocker,
  onImportLinearIssue,
  onQueueLinearComment,
  onDeliverLinearComment,
  onRefreshLinearSharedFields,
  onLoadLinearIssues,
}: BoardSettingsProps) {
  const workItems = snapshot.workItems.map(({ workItem }) => workItem);

  return (
    <section aria-label="Settings" className="workspace-surface">
      <SurfaceHeader
        description="Choose the tools that help this board without interrupting the work itself."
        onBack={onBack}
        title="Settings"
      />
      <Tabs defaultValue="agent" orientation="vertical">
        <TabsList aria-label="Settings sections" variant="line">
          <TabsTrigger value="agent">Agent</TabsTrigger>
          <TabsTrigger value="planning">Organiser</TabsTrigger>
          <TabsTrigger value="linear">Linear</TabsTrigger>
          <TabsTrigger value="project">Project</TabsTrigger>
        </TabsList>
        <TabsContent value="agent">
          <AgentSettings
            busy={busy}
            defaultAgentProfileName={defaultAgentProfileName}
            profiles={agentProfiles}
            providerAvailability={providerAvailability}
            onSave={onSaveAgentProfile}
            onSelectDefault={onSelectDefaultAgentProfile}
          />
        </TabsContent>
        <TabsContent value="planning">
          <section className="settings-section">
            <div>
              <h3>Plan organiser</h3>
              <p>
                Connect the AI that turns an outcome into a reviewable plan. It
                never starts a task until you confirm the plan.
              </p>
            </div>
            <PlannerProfileForm
              busy={busy}
              onSave={onSavePlannerProfile}
              profiles={plannerProfiles}
            />
          </section>
        </TabsContent>
        <TabsContent value="linear">
          <section className="settings-section">
            <div>
              <h3>Linear</h3>
              <p>
                Connect and import only when Linear belongs in this workflow.
              </p>
            </div>
            <LinearConnectionPanel
              busy={busy}
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
                These details help with support. They do not change the work
                already on this board.
              </p>
            </div>
            <BoardSupportDetails board={snapshot.board} />
          </section>
        </TabsContent>
      </Tabs>
    </section>
  );
}

function AgentSettings({
  busy,
  defaultAgentProfileName,
  profiles,
  providerAvailability,
  onSave,
  onSelectDefault,
}: Readonly<{
  busy: boolean;
  defaultAgentProfileName?: string;
  profiles: readonly AgentProfile[];
  providerAvailability: readonly AgentProviderAvailability[];
  onSave: (profile: AgentProfile) => Promise<boolean>;
  onSelectDefault: (profileName: string) => void;
}>) {
  const defaultProfile = profiles.find(
    ({ name }) => name === defaultAgentProfileName,
  );

  async function selectProvider(provider: AgentProviderAvailability) {
    if (!provider.installed) return;
    const profile =
      profiles.find(({ kind }) => kind === provider.kind) ??
      defaultNativeAgentProfile(provider.kind);
    if (
      !profiles.some(({ name }) => name === profile.name) &&
      !(await onSave(profile))
    ) {
      return;
    }
    onSelectDefault(profile.name);
  }

  return (
    <section className="settings-section">
      <div>
        <h3>Default task agent</h3>
        <p>
          Choose an installed agent for new task runs. Kanban only checks
          whether the app is available on this computer; it does not start it or
          inspect your account.
        </p>
      </div>
      {providerAvailability.length === 0 ? (
        <p aria-live="polite" className="field-hint">
          Checking which agents are available…
        </p>
      ) : (
        <ul aria-label="Available task agents" className="provider-list">
          {providerAvailability.map((provider) => {
            const isSelected = defaultProfile?.kind === provider.kind;
            return (
              <li key={provider.kind}>
                <div>
                  <strong>{provider.label}</strong>
                  <p>{providerSummary(provider.kind)}</p>
                </div>
                <div className="provider-actions">
                  <Badge variant={provider.installed ? "secondary" : "outline"}>
                    {provider.installed ? "Installed" : "Not installed"}
                  </Badge>
                  {provider.installed ? (
                    <Button
                      aria-pressed={isSelected}
                      disabled={busy}
                      onClick={() => void selectProvider(provider)}
                      type="button"
                      variant={isSelected ? "default" : "outline"}
                    >
                      {isSelected ? "Selected" : "Use for tasks"}
                    </Button>
                  ) : (
                    <Button asChild variant="outline">
                      <a
                        href={installationGuideFor(provider.kind)}
                        rel="noreferrer"
                        target="_blank"
                      >
                        How to install
                      </a>
                    </Button>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      )}
      {defaultProfile !== undefined && (
        <p aria-live="polite" className="local-status">
          New task runs use {defaultProfile.name} by default.
        </p>
      )}
      <details className="advanced-disclosure">
        <summary>Set up a custom agent</summary>
        <p className="field-hint">
          Use this only for a team-managed bridge or an existing advanced setup.
        </p>
        <AgentProfileForm busy={busy} onSave={onSave} profiles={profiles} />
      </details>
    </section>
  );
}

function providerSummary(kind: AgentProviderAvailability["kind"]): string {
  switch (kind) {
    case "codex_cli":
      return "Use the OpenAI Codex app already installed on this computer.";
    case "claude_code":
      return "Use Claude Code already installed on this computer.";
    case "cline_pass_cli":
      return "Use the Cline CLI already installed on this computer.";
  }
}
