import { useEffect, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { FieldGroup } from "@/components/ui/field";

import {
  defaultNativeAgentProfile,
  defaultNativePlannerProfile,
} from "./agentProfilePresentation";
import { providerDefaultModel } from "./AgentRolePreferences";
import { ProviderConfigurationCard } from "./ProviderConfigurationCard";
import type {
  AgentEffort,
  AgentModelPreference,
  AgentProfile,
  AgentProviderAvailability,
  PlannerProfile,
  ProjectAgentSettings,
  ProviderModelCatalog,
  SaveProjectAgentSettingsRequest,
} from "./types";

type ProjectAgentDefaultsFormProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  boardId: string;
  busy: boolean;
  plannerProfiles: readonly PlannerProfile[];
  providerAvailability: readonly AgentProviderAvailability[];
  settings?: ProjectAgentSettings;
  onLoadProviderCatalog: (
    provider: AgentProviderAvailability,
  ) => Promise<ProviderModelCatalog>;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<boolean>;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
  onSaveProviderCatalogCredential: (
    provider: AgentProviderAvailability,
    apiKey: string,
  ) => Promise<ProviderModelCatalog>;
  onSaveSettings: (request: SaveProjectAgentSettingsRequest) => Promise<void>;
}>;

const noSelection = "__none__";

export function ProjectAgentDefaultsForm({
  agentProfiles,
  boardId,
  busy,
  plannerProfiles,
  providerAvailability,
  settings,
  onLoadProviderCatalog,
  onSaveAgentProfile,
  onSavePlannerProfile,
  onSaveProviderCatalogCredential,
  onSaveSettings,
}: ProjectAgentDefaultsFormProps) {
  const [organiserName, setOrganiserName] = useState(noSelection);
  const [organiserEffort, setOrganiserEffort] =
    useState<AgentEffort>("provider_default");
  const [organiserModel, setOrganiserModel] =
    useState<AgentModelPreference>(providerDefaultModel);
  const [workerName, setWorkerName] = useState(noSelection);
  const [workerEffort, setWorkerEffort] =
    useState<AgentEffort>("provider_default");
  const [workerModel, setWorkerModel] =
    useState<AgentModelPreference>(providerDefaultModel);

  useEffect(() => {
    setOrganiserName(settings?.organiser?.plannerProfileName ?? noSelection);
    setOrganiserEffort(settings?.organiser?.effort ?? "provider_default");
    setOrganiserModel(settings?.organiser?.model ?? providerDefaultModel);
    setWorkerName(settings?.ticketWorker?.agentProfileName ?? noSelection);
    setWorkerEffort(settings?.ticketWorker?.effort ?? "provider_default");
    setWorkerModel(settings?.ticketWorker?.model ?? providerDefaultModel);
  }, [settings]);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveSettings({
      boardId,
      organiser: organiserDefaults(
        organiserName,
        organiserModel,
        organiserEffort,
      ),
      ticketWorker: ticketWorkerDefaults(workerName, workerModel, workerEffort),
    });
  }

  async function changeRole(
    provider: AgentProviderAvailability,
    role: "organiser" | "worker",
    enabled: boolean,
  ) {
    if (!enabled) {
      if (role === "organiser") setOrganiserName(noSelection);
      else setWorkerName(noSelection);
      return;
    }
    if (role === "organiser") {
      const profile = await ensurePlannerProfile(provider);
      if (profile === undefined) return;
      if (!hasPlannerKind(organiserName, provider.kind))
        resetOrganiserPreferences();
      setOrganiserName(profile.name);
      return;
    }
    const profile = await ensureWorkerProfile(provider);
    if (profile === undefined) return;
    if (!hasWorkerKind(workerName, provider.kind)) resetWorkerPreferences();
    setWorkerName(profile.name);
  }

  async function ensurePlannerProfile(provider: AgentProviderAvailability) {
    const profile =
      plannerProfiles.find(({ kind }) => kind === provider.kind) ??
      defaultNativePlannerProfile(provider.kind);
    if (!plannerProfiles.some(({ name }) => name === profile.name)) {
      await onSavePlannerProfile(profile);
    }
    return profile;
  }

  async function ensureWorkerProfile(provider: AgentProviderAvailability) {
    const profile =
      agentProfiles.find(({ kind }) => kind === provider.kind) ??
      defaultNativeAgentProfile(provider.kind);
    if (
      !agentProfiles.some(({ name }) => name === profile.name) &&
      !(await onSaveAgentProfile(profile))
    ) {
      return undefined;
    }
    return profile;
  }

  function hasPlannerKind(
    name: string,
    kind: AgentProviderAvailability["kind"],
  ) {
    return plannerProfiles.some(
      (profile) => profile.name === name && profile.kind === kind,
    );
  }

  function hasWorkerKind(
    name: string,
    kind: AgentProviderAvailability["kind"],
  ) {
    return agentProfiles.some(
      (profile) => profile.name === name && profile.kind === kind,
    );
  }

  function resetOrganiserPreferences() {
    setOrganiserModel(providerDefaultModel);
    setOrganiserEffort("provider_default");
  }

  function resetWorkerPreferences() {
    setWorkerModel(providerDefaultModel);
    setWorkerEffort("provider_default");
  }

  return (
    <form
      aria-label="Project AI defaults"
      className="settings-section"
      onSubmit={save}
    >
      <div>
        <h3>AI for this project</h3>
        <p>
          Choose which installed AI plans work and works on tickets. Keep its
          model and effort choices with that provider.
        </p>
      </div>
      <FieldGroup>
        {providerAvailability.length === 0 ? (
          <p className="field-hint">Checking available AI providers…</p>
        ) : (
          providerAvailability.map((provider) => (
            <ProviderConfigurationCard
              busy={busy}
              key={provider.kind}
              organiser={
                hasPlannerKind(organiserName, provider.kind)
                  ? { effort: organiserEffort, model: organiserModel }
                  : undefined
              }
              provider={provider}
              worker={
                hasWorkerKind(workerName, provider.kind)
                  ? { effort: workerEffort, model: workerModel }
                  : undefined
              }
              onCatalogConnect={(selectedProvider, apiKey) =>
                onSaveProviderCatalogCredential(selectedProvider, apiKey)
              }
              onCatalogLoad={onLoadProviderCatalog}
              onEffortChange={(role, effort) => {
                if (role === "organiser") setOrganiserEffort(effort);
                else setWorkerEffort(effort);
              }}
              onModelChange={(role, model) => {
                if (role === "organiser") setOrganiserModel(model);
                else setWorkerModel(model);
              }}
              onRoleChange={(role, enabled) =>
                changeRole(provider, role, enabled)
              }
            />
          ))
        )}
      </FieldGroup>
      <Button disabled={busy} type="submit">
        Save AI setup
      </Button>
    </form>
  );
}

function organiserDefaults(
  name: string,
  model: AgentModelPreference,
  effort: AgentEffort,
) {
  return name === noSelection
    ? undefined
    : { plannerProfileName: name, model, effort };
}

function ticketWorkerDefaults(
  name: string,
  model: AgentModelPreference,
  effort: AgentEffort,
) {
  return name === noSelection
    ? undefined
    : { agentProfileName: name, model, effort };
}
