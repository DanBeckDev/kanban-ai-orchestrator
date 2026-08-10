import type { BoardSnapshot } from "./types";
import type { BoardGateway } from "./boardGateway";
import type {
  ResolveTicketEffectRequest,
  TicketEffect,
  TicketEffectPromptRequest,
} from "./ticketEffectTypes";

type RunBoardOperation = (
  operation: () => Promise<BoardSnapshot | undefined>,
) => Promise<void>;

export type TicketEffectOperations = Readonly<{
  request: (request: TicketEffectPromptRequest) => Promise<void>;
  resolve: (request: ResolveTicketEffectRequest) => Promise<void>;
  load: (workItemId: string) => Promise<readonly TicketEffect[]>;
}>;

export function ticketEffectOperations(
  gateway: BoardGateway,
  run: RunBoardOperation,
): TicketEffectOperations {
  return {
    request: async (request) => {
      await run(async () => {
        await gateway.requestTicketEffect(request);
        return undefined;
      });
    },
    resolve: (request) => run(() => gateway.resolveTicketEffect(request)),
    load: (workItemId) => gateway.ticketEffects(workItemId),
  };
}
