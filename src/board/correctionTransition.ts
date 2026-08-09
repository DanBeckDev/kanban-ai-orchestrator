import type { TransitionWorkItemRequest, WorkItem } from "./types";

export function correctionTransition(
  workItem: WorkItem,
  summary: string,
  recordedAt: string,
): TransitionWorkItemRequest {
  return {
    eventId: `return-for-correction-${workItem.id}-${recordedAt}`,
    workItemId: workItem.id,
    nextState: "ready",
    reason: summary,
    recordedAt,
  };
}
