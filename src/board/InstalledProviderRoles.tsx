import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import { installationGuideFor } from "./agentProfilePresentation";
import type {
  AgentProfile,
  AgentProviderAvailability,
  PlannerProfile,
} from "./types";

type InstalledProviderRolesProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  plannerProfiles: readonly PlannerProfile[];
  providers: readonly AgentProviderAvailability[];
  selectedOrganiserName: string;
  selectedWorkerName: string;
  onUseForOrchestrator: (provider: AgentProviderAvailability) => Promise<void>;
  onUseForWorker: (provider: AgentProviderAvailability) => Promise<void>;
}>;

export function InstalledProviderRoles({
  agentProfiles,
  busy,
  plannerProfiles,
  providers,
  selectedOrganiserName,
  selectedWorkerName,
  onUseForOrchestrator,
  onUseForWorker,
}: InstalledProviderRolesProps) {
  if (providers.length === 0) {
    return <p className="field-hint">Checking available AI providers…</p>;
  }

  return (
    <section aria-labelledby="installed-providers-title">
      <h4 id="installed-providers-title">Available on this computer</h4>
      <ul aria-label="Available AI providers" className="provider-list">
        {providers.map((provider) => {
          const workerSelected = agentProfiles.some(
            (profile) =>
              profile.name === selectedWorkerName &&
              profile.kind === provider.kind,
          );
          const organiserSelected = plannerProfiles.some(
            (profile) =>
              profile.name === selectedOrganiserName &&
              profile.kind === provider.kind,
          );
          return (
            <li key={provider.kind}>
              <div>
                <strong>{provider.label}</strong>
                <p>{provider.installed ? "Ready to add" : "Install to add"}</p>
              </div>
              <div className="provider-actions">
                <Badge variant={provider.installed ? "secondary" : "outline"}>
                  {provider.installed ? "Installed" : "Not installed"}
                </Badge>
                {provider.installed ? (
                  <>
                    <Button
                      disabled={busy}
                      onClick={() => void onUseForOrchestrator(provider)}
                      type="button"
                      variant={organiserSelected ? "default" : "outline"}
                    >
                      {organiserSelected
                        ? "Orchestrator chosen"
                        : "Use as orchestrator"}
                    </Button>
                    <Button
                      disabled={busy}
                      onClick={() => void onUseForWorker(provider)}
                      type="button"
                      variant={workerSelected ? "default" : "outline"}
                    >
                      {workerSelected ? "Worker chosen" : "Use as worker"}
                    </Button>
                  </>
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
    </section>
  );
}
