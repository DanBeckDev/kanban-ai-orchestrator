import { vi } from "vitest";

import type { BoardGateway, BoardSnapshot } from "./types";

type FixtureContext = Readonly<{
  current: () => BoardSnapshot;
  replace: (snapshot: BoardSnapshot) => void;
}>;

export function executionGatewayMethods({
  current,
  replace,
}: FixtureContext): Pick<
  BoardGateway,
  | "coordinateBoard"
  | "executionActivity"
  | "recordCleanCodeReview"
  | "recordReviewCheck"
  | "recordReviewDecision"
  | "startExecution"
  | "stopExecution"
> {
  return {
    startExecution: vi.fn().mockImplementation(async (request) => {
      const snapshot = current();
      const updated = {
        ...snapshot,
        workItems: snapshot.workItems.map((materializedWorkItem) =>
          materializedWorkItem.workItem.id === request.workItemId
            ? {
                ...materializedWorkItem,
                workItem: {
                  ...materializedWorkItem.workItem,
                  state: "running" as const,
                },
              }
            : materializedWorkItem,
        ),
      };
      replace(updated);
      return updated;
    }),
    coordinateBoard: vi.fn().mockImplementation(async () => current()),
    stopExecution: vi.fn().mockImplementation(async () => current()),
    executionActivity: vi
      .fn()
      .mockResolvedValue({ chunks: [], hasMore: false }),
    recordReviewCheck: vi.fn().mockImplementation(async (request) => {
      const snapshot = evidenceSnapshot(current(), {
        id: request.evidenceId,
        workItemId: request.workItemId,
        kind: "quality_gate",
        result: request.passed ? "passed" : "failed",
        summary: request.summary,
        recordedAt: request.recordedAt,
      });
      replace(snapshot);
      return snapshot;
    }),
    recordReviewDecision: vi.fn().mockImplementation(async (request) => {
      const snapshot = evidenceSnapshot(current(), {
        id: request.evidenceId,
        workItemId: request.workItemId,
        kind: "review_decision",
        result: request.accepted ? "passed" : "failed",
        summary: `${request.reviewer}: ${request.summary}`,
        recordedAt: request.recordedAt,
      });
      replace(snapshot);
      return snapshot;
    }),
    recordCleanCodeReview: vi.fn().mockImplementation(async (request) => {
      const snapshot = evidenceSnapshot(current(), {
        id: request.evidenceId,
        workItemId: request.workItemId,
        kind: "clean_code_review",
        result: request.actionableFindingCount === 0 ? "passed" : "failed",
        summary: request.summary,
        recordedAt: request.recordedAt,
      });
      replace(snapshot);
      return snapshot;
    }),
  };
}

function evidenceSnapshot(
  snapshot: BoardSnapshot,
  evidence: BoardSnapshot["evidence"][number],
): BoardSnapshot {
  return { ...snapshot, evidence: [...snapshot.evidence, evidence] };
}
