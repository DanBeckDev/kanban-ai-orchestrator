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
  FieldLegend,
  FieldSet,
  FieldTitle,
} from "@/components/ui/field";
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
  onEffortChange,
  onModelChange,
  onRoleChange,
}: ProviderConfigurationCardProps) {
  const [catalog, setCatalog] = useState<ProviderModelCatalog>();
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
                  busy={busy || catalogBusy}
                  catalog={catalog}
                  error={catalogError}
                  provider={provider}
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
              Kanban uses this AI's existing local session. It never asks for or
              stores an API key.
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
  busy,
  catalog,
  error,
  provider,
  onLoad,
}: Readonly<{
  busy: boolean;
  catalog?: ProviderModelCatalog;
  error?: string;
  provider: AgentProviderAvailability;
  onLoad: () => void;
}>) {
  if (error !== undefined) {
    return (
      <FieldSet>
        <FieldLegend variant="label">Model list</FieldLegend>
        <FieldDescription>{error}</FieldDescription>
        <Button
          disabled={busy}
          onClick={onLoad}
          type="button"
          variant="outline"
        >
          Refresh models
        </Button>
      </FieldSet>
    );
  }

  if (catalog === undefined) {
    return (
      <Field>
        <FieldDescription>
          Checking models available in your installed {provider.label} session…
        </FieldDescription>
      </Field>
    );
  }

  if (catalog?.status === "ready") {
    return (
      <Field orientation="horizontal">
        <FieldDescription>
          {catalog.models.length === 0
            ? "This installed AI did not return any selectable models."
            : "Model list loaded from your installed AI session."}
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
        {catalog.status === "unavailable"
          ? `Kanban could not read models from ${provider.label}. Sign in or update it, then try again.`
          : `${provider.label} manages its models in its own app. Kanban will use that provider's default.`}
      </FieldDescription>
      {catalog.status === "unavailable" && (
        <Button
          disabled={busy}
          onClick={onLoad}
          type="button"
          variant="outline"
        >
          Refresh models
        </Button>
      )}
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
