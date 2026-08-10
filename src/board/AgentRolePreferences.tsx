import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";

import type { AgentEffort, AgentModelPreference, ProviderModel } from "./types";

export const providerDefaultModel: AgentModelPreference = {
  kind: "provider_default",
};

type AgentRolePreferencesProps = Readonly<{
  effort: AgentEffort;
  idPrefix: string;
  model: AgentModelPreference;
  models: readonly ProviderModel[];
  onEffortChange: (effort: AgentEffort) => void;
  onModelChange: (model: AgentModelPreference) => void;
}>;

export function AgentRolePreferences({
  effort,
  idPrefix,
  model,
  models,
  onEffortChange,
  onModelChange,
}: AgentRolePreferencesProps) {
  const catalogModel =
    model.kind === "named"
      ? models.find(({ id }) => id === model.name)
      : undefined;
  const previouslySelectedModel =
    model.kind === "named" && catalogModel === undefined
      ? {
          id: model.name,
          label: `${model.name} (not in the current model list)`,
          efforts: defaultEfforts,
        }
      : undefined;
  const selectedModel = catalogModel ?? previouslySelectedModel;
  const selectableModels =
    previouslySelectedModel === undefined
      ? models
      : [...models, previouslySelectedModel];
  const availableEfforts = selectedModel?.efforts ?? defaultEfforts;

  return (
    <>
      <Field>
        <FieldLabel htmlFor={`${idPrefix}-model`}>Model</FieldLabel>
        <Select
          onValueChange={(value) =>
            onModelChange(
              value === "provider_default"
                ? providerDefaultModel
                : { kind: "named", name: value },
            )
          }
          value={model.kind === "named" ? model.name : "provider_default"}
        >
          <SelectTrigger id={`${idPrefix}-model`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="provider_default">
                Provider default (recommended)
              </SelectItem>
              {selectableModels.map((availableModel) => (
                <SelectItem key={availableModel.id} value={availableModel.id}>
                  {availableModel.label}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <FieldDescription>
          {previouslySelectedModel !== undefined
            ? "This saved model is not in the current account list. Refresh or choose another model before starting work."
            : models.length === 0
              ? "Connect this provider to load the models available to your account."
              : "Only models returned by this provider are available here."}
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
              {availableEfforts.includes("focused") && (
                <SelectItem value="focused">Focused</SelectItem>
              )}
              {availableEfforts.includes("balanced") && (
                <SelectItem value="balanced">Balanced</SelectItem>
              )}
              {availableEfforts.includes("thorough") && (
                <SelectItem value="thorough">Thorough</SelectItem>
              )}
            </SelectGroup>
          </SelectContent>
        </Select>
      </Field>
    </>
  );
}

const defaultEfforts: readonly AgentEffort[] = [
  "focused",
  "balanced",
  "thorough",
];
