import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";

import type { AgentEffort, AgentModelPreference } from "./types";

export const providerDefaultModel: AgentModelPreference = {
  kind: "provider_default",
};

type AgentRolePreferencesProps = Readonly<{
  effort: AgentEffort;
  idPrefix: string;
  model: AgentModelPreference;
  onEffortChange: (effort: AgentEffort) => void;
  onModelChange: (model: AgentModelPreference) => void;
}>;

export function AgentRolePreferences({
  effort,
  idPrefix,
  model,
  onEffortChange,
  onModelChange,
}: AgentRolePreferencesProps) {
  const usesCustomModel = model.kind === "named";

  return (
    <>
      <Field>
        <FieldLabel htmlFor={`${idPrefix}-model`}>Model</FieldLabel>
        <Select
          onValueChange={(value) =>
            onModelChange(
              value === "custom"
                ? { kind: "named", name: "" }
                : providerDefaultModel,
            )
          }
          value={usesCustomModel ? "custom" : "provider_default"}
        >
          <SelectTrigger id={`${idPrefix}-model`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="provider_default">
                Provider default (recommended)
              </SelectItem>
              <SelectItem value="custom">Choose a specific model</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
        <FieldDescription>
          Use the installed provider's safe default unless you know the exact
          model name available to your account.
        </FieldDescription>
      </Field>
      {usesCustomModel && (
        <Field>
          <FieldLabel htmlFor={`${idPrefix}-custom-model`}>
            Specific model name
          </FieldLabel>
          <Input
            autoComplete="off"
            id={`${idPrefix}-custom-model`}
            name={`${idPrefix}-custom-model`}
            onChange={(event) =>
              onModelChange({ kind: "named", name: event.target.value })
            }
            value={model.name}
          />
        </Field>
      )}
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
