import { useEffect, useState, type FormEvent } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
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
  installationGuideFor,
} from "./agentProfilePresentation";
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
  onSaveSettings: (request: SaveProjectAgentSettingsRequest) => Promise<void>;
}>;

const noSelection = "__none__";
const providerDefaultModel: AgentModelPreference = {
  kind: "provider_default",
};

export function ProjectAgentDefaultsForm({
  agentProfiles,
  boardId,
  busy,
  plannerProfiles,
  providerAvailability,
  settings,
  onSaveAgentProfile,
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

  async function saveDefaults(nextWorkerName: string) {
    await onSaveSettings({
      boardId,
      organiser: organiserDefaults(
        organiserName,
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
          <RolePreferences
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
          <RolePreferences
            effort={workerEffort}
            idPrefix="ticket-worker"
            onEffortChange={setWorkerEffort}
            onModelChange={setWorkerModel}
            model={workerModel}
          />
        </FieldSet>
      </FieldGroup>
      <InstalledProviders
        agentProfiles={agentProfiles}
        busy={busy}
        providers={providerAvailability}
        selectedWorkerName={workerName}
        onUse={useInstalledProvider}
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

function RolePreferences({
  effort,
  idPrefix,
  onEffortChange,
  onModelChange,
  model,
}: Readonly<{
  effort: AgentEffort;
  idPrefix: string;
  onEffortChange: (effort: AgentEffort) => void;
  onModelChange: (model: AgentModelPreference) => void;
  model: AgentModelPreference;
}>) {
  return (
    <>
      <Field>
        <FieldLabel htmlFor={`${idPrefix}-model`}>
          Model name (optional)
        </FieldLabel>
        <Input
          autoComplete="off"
          id={`${idPrefix}-model`}
          name={`${idPrefix}-model`}
          onChange={(event) =>
            onModelChange(
              event.target.value.trim().length === 0
                ? providerDefaultModel
                : { kind: "named", name: event.target.value },
            )
          }
          value={model.kind === "named" ? model.name : ""}
        />
        <FieldDescription>
          Leave this blank to use the provider default. Otherwise, enter a model
          name supported by the selected AI connection.
        </FieldDescription>
      </Field>
      <Field>
        <FieldLabel htmlFor={`${idPrefix}-effort`}>Effort</FieldLabel>
        <Select onValueChange={onEffortChange} value={effort}>
          <SelectTrigger id={`${idPrefix}-effort`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="provider_default">Provider default</SelectItem>
              <SelectItem value="focused">Focused</SelectItem>
              <SelectItem value="balanced">Balanced</SelectItem>
              <SelectItem value="thorough">Thorough</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </Field>
    </>
  );
}

function InstalledProviders({
  agentProfiles,
  busy,
  providers,
  selectedWorkerName,
  onUse,
}: Readonly<{
  agentProfiles: readonly AgentProfile[];
  busy: boolean;
  providers: readonly AgentProviderAvailability[];
  selectedWorkerName: string;
  onUse: (provider: AgentProviderAvailability) => Promise<void>;
}>) {
  if (providers.length === 0) {
    return <p className="field-hint">Checking available ticket workers…</p>;
  }

  return (
    <section aria-labelledby="installed-workers-title">
      <h4 id="installed-workers-title">Available on this computer</h4>
      <ul aria-label="Available ticket workers" className="provider-list">
        {providers.map((provider) => {
          const selected = agentProfiles.some(
            (profile) =>
              profile.name === selectedWorkerName &&
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
                  <Button
                    disabled={busy}
                    onClick={() => void onUse(provider)}
                    type="button"
                    variant={selected ? "default" : "outline"}
                  >
                    {selected ? "Chosen" : "Use as worker"}
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
    </section>
  );
}
