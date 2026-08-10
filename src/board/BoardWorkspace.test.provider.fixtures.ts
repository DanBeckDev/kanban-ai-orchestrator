import { vi } from "vitest";

import type { BoardGateway } from "./types";

export function providerGatewayMethods(): Pick<
  BoardGateway,
  | "agentProviderAvailability"
  | "providerModelCatalog"
  | "saveProviderCatalogCredential"
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
      status: "disconnected" as const,
      models: [],
    })),
    saveProviderCatalogCredential: vi
      .fn()
      .mockImplementation(async ({ providerKind }) => ({
        providerKind,
        status: "ready" as const,
        models: [
          {
            id: "gpt-5-codex",
            label: "GPT-5 Codex",
            efforts: ["focused", "balanced", "thorough"],
          },
        ],
      })),
  };
}
