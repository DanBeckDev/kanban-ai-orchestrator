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
        installed: true,
      },
    ]),
    providerModelCatalog: vi.fn().mockImplementation(async (providerKind) => ({
      providerKind,
      status: "ready" as const,
      models:
        providerKind === "codex_cli"
          ? [
              {
                id: "gpt-5-codex",
                label: "GPT-5 Codex",
                efforts: ["focused", "balanced", "thorough"],
              },
            ]
          : providerKind === "claude_code"
            ? [
                {
                  id: "fable",
                  label: "Claude Fable",
                  efforts: [
                    "focused",
                    "balanced",
                    "thorough",
                    "extra_thorough",
                    "maximum",
                  ],
                },
                {
                  id: "sonnet",
                  label: "Claude Sonnet",
                  efforts: ["focused", "balanced", "thorough"],
                },
              ]
            : providerKind === "cline_pass_cli"
              ? [
                  {
                    id: "~anthropic/claude-opus-latest",
                    label: "Claude Opus Latest",
                    efforts: [
                      "focused",
                      "balanced",
                      "thorough",
                      "extra_thorough",
                    ],
                  },
                  {
                    id: "openai/gpt-4o",
                    label: "GPT-4o",
                    efforts: [],
                  },
                ]
              : [],
    })),
  };
}
