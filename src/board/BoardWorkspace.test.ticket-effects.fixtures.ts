import { vi } from "vitest";

import type { BoardGateway, BoardSnapshot } from "./types";
import type { TicketEffect } from "./ticketEffectTypes";

export function ticketEffectGatewayMethods(
  context: Readonly<{
    current: () => BoardSnapshot;
  }>,
): Pick<
  BoardGateway,
  "requestTicketEffect" | "resolveTicketEffect" | "ticketEffects"
> {
  let effects: readonly TicketEffect[] = [];
  return {
    requestTicketEffect: vi.fn().mockImplementation(async (request) => {
      const effect: TicketEffect = {
        id: request.requestId,
        boardId: context.current().board.id,
        workItemId: request.workItemId,
        organiserProfileName: "local organiser",
        action: request.action,
        promptSummary: request.prompt,
        recommendation: "Kanban prepared a safe task decision.",
        rationale: "The request is scoped to the selected task.",
        proposal: proposalFor(request.action),
        authorityMode: "manual",
        policyResult: "not_required",
        outcome:
          request.action === "explain_evidence"
            ? "applied"
            : "awaiting_approval",
        expectedWorkItemSequence: 1,
        recordedAt: "2026-08-10T12:00:00Z",
      };
      effects = [effect, ...effects];
      return effect;
    }),
    resolveTicketEffect: vi.fn().mockImplementation(async (request) => {
      effects = effects.map((effect) =>
        effect.id === request.effectId
          ? {
              ...effect,
              outcome:
                request.resolution === "apply"
                  ? "applied"
                  : request.resolution === "reject"
                    ? "rejected"
                    : "cancelled",
            }
          : effect,
      );
      return context.current();
    }),
    ticketEffects: vi
      .fn()
      .mockImplementation(async (workItemId: string) =>
        effects.filter((effect) => effect.workItemId === workItemId),
      ),
  };
}

function proposalFor(action: TicketEffect["action"]): TicketEffect["proposal"] {
  if (action === "give_worker_guidance") {
    return {
      acceptanceCriteria: [],
      workerGuidance: "Start with the focused test suite.",
    };
  }
  if (action === "explain_evidence") {
    return {
      acceptanceCriteria: [],
      evidenceExplanation: "The latest evidence needs a correction.",
    };
  }
  return { acceptanceCriteria: [] };
}
