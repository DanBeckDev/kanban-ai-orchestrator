import { useEffect, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import {
  defaultNativeAgentProfile,
  defaultNativePlannerProfile,
} from "./agentProfilePresentation";
import {
  AgentRolePreferences,
  providerDefaultModel,
} from "./AgentRolePreferences";
import { InstalledProviderRoles } from "./InstalledProviderRoles";
import type {
  AgentEffort,
  AgentModelPreference,
  AgentProfile,
  AgentProviderAvailability,
  PlannerProfile,
  ProjectAgentSettings,
  SaveProjectAgentSettingsRequest,
} from "./types";

type ProjectAgentDefaultsFormProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  boardId: string;
  busy: boolean;
  plannerProfiles: readonly PlannerProfile[];
  providerAvailability: readonly AgentProviderAvailability[];
  settings?: ProjectAgentSettings;
  onSaveAgentProfile: (profile: AgentProfile) => Promise<boolean>;
  onSavePlannerProfile: (profile: PlannerProfile) => Promise<void>;
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
  onSaveAgentProfile,
  onSavePlannerProfile,
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
    await saveDefaults(workerName);
  }

  async function saveDefaults(
    nextWorkerName: string,
    nextOrganiserName = organiserName,
  ) {
    await onSaveSettings({
      boardId,
      organiser: organiserDefaults(
        nextOrganiserName,
        organiserModel,
        organiserEffort,
      ),
      ticketWorker: ticketWorkerDefaults(
        nextWorkerName,
        workerModel,
        workerEffort,
      ),
    });
  }

  async function useInstalledProvider(provider: AgentProviderAvailability) {
    if (!provider.installed) return;
    const profile =
      agentProfiles.find(({ kind }) => kind === provider.kind) ??
      defaultNativeAgentProfile(provider.kind);
    if (
      !agentProfiles.some(({ name }) => name === profile.name) &&
      !(await onSaveAgentProfile(profile))
    ) {
      return;
    }
    setWorkerName(profile.name);
    await saveDefaults(profile.name);
  }

  async function useInstalledProviderForOrchestrator(
    provider: AgentProviderAvailability,
  ) {
    if (!provider.installed) return;
    const profile =
      plannerProfiles.find(({ kind }) => kind === provider.kind) ??
      defaultNativePlannerProfile(provider.kind);
    if (!plannerProfiles.some(({ name }) => name === profile.name)) {
      await onSavePlannerProfile(profile);
    }
    setOrganiserName(profile.name);
    await saveDefaults(workerName, profile.name);
  }

  return (
    <form
      aria-label="Project AI defaults"
      className="settings-section"
      onSubmit={save}
    >
      <div>
        <h3>AI roles</h3>
        <p>
          Pick who plans the work and who works on each new ticket. Kanban
          remembers these choices for every board in this project.
        </p>
      </div>
      <FieldGroup>
        <FieldSet>
          <FieldLegend>Orchestrator</FieldLegend>
          <FieldDescription>
            Turns your outcome into a reviewable plan. It never creates tickets
            or starts workers until you confirm.
          </FieldDescription>
          <Field>
            <FieldLabel htmlFor="organiser-profile">AI connection</FieldLabel>
            <Select onValueChange={setOrganiserName} value={organiserName}>
              <SelectTrigger id="organiser-profile">
                <SelectValue placeholder="Choose an orchestrator" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem value={noSelection}>
                    No orchestrator yet
                  </SelectItem>
                  {plannerProfiles.map((profile) => (
                    <SelectItem key={profile.name} value={profile.name}>
                      {profile.name}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
          <AgentRolePreferences
            effort={organiserEffort}
            idPrefix="organiser"
            onEffortChange={setOrganiserEffort}
            onModelChange={setOrganiserModel}
            model={organiserModel}
          />
        </FieldSet>
        <FieldSet>
          <FieldLegend>Ticket workers</FieldLegend>
          <FieldDescription>
            The default worker for manually created tickets and AI-proposed
            plans. You can reassign an individual ticket before it is created.
          </FieldDescription>
          <Field>
            <FieldLabel htmlFor="ticket-worker-profile">Worker</FieldLabel>
            <Select onValueChange={setWorkerName} value={workerName}>
              <SelectTrigger id="ticket-worker-profile">
                <SelectValue placeholder="Choose a ticket worker" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem value={noSelection}>
                    No default worker yet
                  </SelectItem>
                  {agentProfiles.map((profile) => (
                    <SelectItem key={profile.name} value={profile.name}>
                      {profile.name}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
          <AgentRolePreferences
            effort={workerEffort}
            idPrefix="ticket-worker"
            onEffortChange={setWorkerEffort}
            onModelChange={setWorkerModel}
            model={workerModel}
          />
        </FieldSet>
      </FieldGroup>
      <InstalledProviderRoles
        agentProfiles={agentProfiles}
        busy={busy}
        plannerProfiles={plannerProfiles}
        providers={providerAvailability}
        selectedOrganiserName={organiserName}
        selectedWorkerName={workerName}
        onUseForOrchestrator={useInstalledProviderForOrchestrator}
        onUseForWorker={useInstalledProvider}
      />
      <Button disabled={busy} type="submit">
        Save AI defaults
      </Button>
    </form>
  );
}

function organiserDefaults(
  name: string,
  model: AgentModelPreference,
  effort: AgentEffort,
) {
  return name === noSelection || name.trim().length === 0
    ? undefined
    : { plannerProfileName: name, model: modelPreference(model), effort };
}

function ticketWorkerDefaults(
  name: string,
  model: AgentModelPreference,
  effort: AgentEffort,
) {
  return name === noSelection || name.trim().length === 0
    ? undefined
    : { agentProfileName: name, model: modelPreference(model), effort };
}

function modelPreference(model: AgentModelPreference): AgentModelPreference {
  if (model.kind === "provider_default") return model;
  const name = model.name.trim();
  return name.length === 0 ? providerDefaultModel : { kind: "named", name };
}
