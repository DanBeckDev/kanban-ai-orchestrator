import { vi } from "vitest";

import type { BoardGateway } from "./types";

export function providerGatewayMethods(): Pick<
  BoardGateway,
  "agentProviderAvailability" | "providerModelCatalog"
> {
  return {
    agentProviderAvailability: vi.fn().mockResolvedValue([
      {
        kind: "codex_cli",
        label: "Codex",
        program: "codex",
        installed: true,
      },
      {
        kind: "claude_code",
        label: "Claude Code",
        program: "claude",
        installed: true,
      },
      {
        kind: "cline_pass_cli",
        label: "Cline",
        program: "cline",
        installed: false,
      },
    ]),
    providerModelCatalog: vi.fn().mockImplementation(async (providerKind) => ({
      providerKind,
      status:
        providerKind === "codex_cli"
          ? ("ready" as const)
          : ("uses_provider_default" as const),
      models:
        providerKind === "codex_cli"
          ? [
              {
                id: "gpt-5-codex",
                label: "GPT-5 Codex",
                efforts: ["focused", "balanced", "thorough"],
              },
            ]
          : [],
    })),
  };
}
