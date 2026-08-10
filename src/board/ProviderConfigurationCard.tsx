import { useCallback, useEffect, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
  FieldTitle,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import { AgentRolePreferences } from "./AgentRolePreferences";
import { installationGuideFor } from "./agentProfilePresentation";
import type {
  AgentEffort,
  AgentModelPreference,
  AgentProviderAvailability,
  ProviderModelCatalog,
} from "./types";

type ProviderRole = "organiser" | "worker";

type RolePreferences = Readonly<{
  effort: AgentEffort;
  model: AgentModelPreference;
}>;

type ProviderConfigurationCardProps = Readonly<{
  busy: boolean;
  organiser?: RolePreferences;
  provider: AgentProviderAvailability;
  worker?: RolePreferences;
  onCatalogLoad: (
    provider: AgentProviderAvailability,
  ) => Promise<ProviderModelCatalog>;
  onCatalogConnect: (
    provider: AgentProviderAvailability,
    apiKey: string,
  ) => Promise<ProviderModelCatalog>;
  onEffortChange: (role: ProviderRole, effort: AgentEffort) => void;
  onModelChange: (role: ProviderRole, model: AgentModelPreference) => void;
  onRoleChange: (role: ProviderRole, enabled: boolean) => Promise<void>;
}>;

export function ProviderConfigurationCard({
  busy,
  organiser,
  provider,
  worker,
  onCatalogLoad,
  onCatalogConnect,
  onEffortChange,
  onModelChange,
  onRoleChange,
}: ProviderConfigurationCardProps) {
  const [catalog, setCatalog] = useState<ProviderModelCatalog>();
  const [apiKey, setApiKey] = useState("");
  const [catalogBusy, setCatalogBusy] = useState(false);
  const [catalogError, setCatalogError] = useState<string>();
  const [catalogLoadAttempted, setCatalogLoadAttempted] = useState(false);
  const roles = [
    ...(organiser === undefined ? [] : ["organiser"]),
    ...(worker === undefined ? [] : ["worker"]),
  ];

  const loadCatalog = useCallback(async () => {
    setCatalogLoadAttempted(true);
    setCatalogBusy(true);
    try {
      setCatalogError(undefined);
      setCatalog(await onCatalogLoad(provider));
    } catch {
      setCatalogError(
        "Could not load models. Check this provider and try again.",
      );
    } finally {
      setCatalogBusy(false);
    }
  }, [onCatalogLoad, provider]);

  async function connectCatalog() {
    setCatalogBusy(true);
    try {
      setCatalogError(undefined);
      setCatalog(await onCatalogConnect(provider, apiKey));
      setApiKey("");
    } catch {
      setCatalogError("Could not connect. Check the API key and try again.");
    } finally {
      setCatalogBusy(false);
    }
  }

  function changeRoles(nextRoles: string[]) {
    (["organiser", "worker"] as const).forEach((role) => {
      const enabled = nextRoles.includes(role);
      if (enabled !== roles.includes(role)) {
        void onRoleChange(role, enabled).catch(() => undefined);
      }
    });
  }

  useEffect(() => {
    if (roles.length === 0 || catalog !== undefined || catalogLoadAttempted)
      return;
    void loadCatalog();
  }, [catalog, catalogLoadAttempted, loadCatalog, roles.length]);

  return (
    <Card size="sm">
      <CardHeader>
        <CardTitle as="h4">{provider.label}</CardTitle>
        <CardDescription>
          {provider.installed
            ? "Choose what this AI does in this project."
            : "Install this AI before using it in this project."}
        </CardDescription>
        <CardAction>
          <Badge variant={provider.installed ? "secondary" : "outline"}>
            {provider.installed ? "Installed" : "Not installed"}
          </Badge>
        </CardAction>
      </CardHeader>
      {provider.installed ? (
        <>
          <CardContent>
            <FieldGroup>
              <Field>
                <FieldTitle id={`${provider.kind}-roles`}>Use for</FieldTitle>
                <ToggleGroup
                  aria-labelledby={`${provider.kind}-roles`}
                  disabled={busy}
                  onValueChange={changeRoles}
                  type="multiple"
                  value={roles}
                  variant="outline"
                >
                  <ToggleGroupItem value="organiser">Plan work</ToggleGroupItem>
                  <ToggleGroupItem value="worker">
                    Work on tickets
                  </ToggleGroupItem>
                </ToggleGroup>
              </Field>
              {roles.length > 0 && (
                <CatalogControls
                  apiKey={apiKey}
                  busy={busy || catalogBusy}
                  catalog={catalog}
                  error={catalogError}
                  provider={provider}
                  onApiKeyChange={setApiKey}
                  onConnect={() => void connectCatalog()}
                  onLoad={() => void loadCatalog()}
                />
              )}
              {organiser !== undefined && (
                <RoleConfiguration
                  catalog={catalog}
                  effort={organiser.effort}
                  idPrefix={`${provider.kind}-organiser`}
                  legend="Plan work"
                  model={organiser.model}
                  onEffortChange={(effort) =>
                    onEffortChange("organiser", effort)
                  }
                  onModelChange={(model) => onModelChange("organiser", model)}
                />
              )}
              {worker !== undefined && (
                <RoleConfiguration
                  catalog={catalog}
                  effort={worker.effort}
                  idPrefix={`${provider.kind}-worker`}
                  legend="Work on tickets"
                  model={worker.model}
                  onEffortChange={(effort) => onEffortChange("worker", effort)}
                  onModelChange={(model) => onModelChange("worker", model)}
                />
              )}
            </FieldGroup>
          </CardContent>
          <CardFooter>
            <p className="field-hint">
              When you connect, the API key is stored in this device's keychain,
              never with this board.
            </p>
          </CardFooter>
        </>
      ) : (
        <CardFooter>
          <Button asChild variant="outline">
            <a
              href={installationGuideFor(provider.kind)}
              rel="noreferrer"
              target="_blank"
            >
              How to install
            </a>
          </Button>
        </CardFooter>
      )}
    </Card>
  );
}

function CatalogControls({
  apiKey,
  busy,
  catalog,
  error,
  provider,
  onApiKeyChange,
  onConnect,
  onLoad,
}: Readonly<{
  apiKey: string;
  busy: boolean;
  catalog?: ProviderModelCatalog;
  error?: string;
  provider: AgentProviderAvailability;
  onApiKeyChange: (value: string) => void;
  onConnect: () => void;
  onLoad: () => void;
}>) {
  if (catalog?.status === "ready") {
    return (
      <Field orientation="horizontal">
        <FieldDescription>
          {catalog.models.length === 0
            ? "This account did not return any selectable models."
            : "Model list loaded from this provider account."}
        </FieldDescription>
        <Button
          disabled={busy}
          onClick={onLoad}
          type="button"
          variant="outline"
        >
          Refresh models
        </Button>
      </Field>
    );
  }

  return (
    <FieldSet>
      <FieldLegend variant="label">Model list</FieldLegend>
      <FieldDescription>
        {catalog?.status === "unavailable"
          ? "Kanban could not refresh this model list. Check the key and connection."
          : "Connect this provider API to choose from models available to your account."}
      </FieldDescription>
      {catalog?.status === "unavailable" && (
        <Button
          disabled={busy}
          onClick={onLoad}
          type="button"
          variant="outline"
        >
          Refresh models
        </Button>
      )}
      {catalog?.status === "disconnected" && (
        <Button
          disabled={busy}
          onClick={onLoad}
          type="button"
          variant="outline"
        >
          Load saved model list
        </Button>
      )}
      <details>
        <summary>Connect provider API</summary>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor={`${provider.kind}-api-key`}>
              {provider.label} API key
            </FieldLabel>
            <Input
              autoComplete="off"
              id={`${provider.kind}-api-key`}
              onChange={(event) => onApiKeyChange(event.target.value)}
              type="password"
              value={apiKey}
            />
          </Field>
          {error !== undefined && <FieldDescription>{error}</FieldDescription>}
          <Button
            disabled={busy || apiKey.trim().length === 0}
            onClick={onConnect}
            type="button"
          >
            Connect and load models
          </Button>
        </FieldGroup>
      </details>
    </FieldSet>
  );
}

function RoleConfiguration({
  catalog,
  effort,
  idPrefix,
  legend,
  model,
  onEffortChange,
  onModelChange,
}: Readonly<{
  catalog?: ProviderModelCatalog;
  effort: AgentEffort;
  idPrefix: string;
  legend: string;
  model: AgentModelPreference;
  onEffortChange: (effort: AgentEffort) => void;
  onModelChange: (model: AgentModelPreference) => void;
}>) {
  return (
    <FieldSet>
      <FieldLegend>{legend}</FieldLegend>
      <AgentRolePreferences
        effort={effort}
        idPrefix={idPrefix}
        model={model}
        models={catalog?.models ?? []}
        onEffortChange={onEffortChange}
        onModelChange={onModelChange}
      />
    </FieldSet>
  );
}
