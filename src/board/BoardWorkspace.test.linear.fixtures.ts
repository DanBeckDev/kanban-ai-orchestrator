import { vi } from "vitest";

import type {
  BoardGateway,
  BoardSnapshot,
  LinearConnectionStatus,
  LinearIssueSummary,
  LinearOAuthConfiguration,
  ObserveLinearSharedFieldRequest,
  QueueLinearCommentRequest,
} from "./types";

type SnapshotAccess = Readonly<{
  current: () => BoardSnapshot;
  replace: (snapshot: BoardSnapshot) => void;
}>;

export function linearGatewayMethods(
  access: SnapshotAccess,
): Pick<
  BoardGateway,
  | "beginLinearOAuth"
  | "beginLinearCommentAccess"
  | "linearConnectionStatus"
  | "linearAssignedIssues"
  | "importLinearIssue"
  | "importLinearBlocker"
  | "queueLinearComment"
  | "observeLinearSharedField"
  | "syncLinearSharedFields"
  | "deliverLinearComment"
> {
  let linearConnectionStatus: LinearConnectionStatus = { kind: "disconnected" };
  const linearIssues: readonly LinearIssueSummary[] = [];
  return {
    beginLinearOAuth: vi
      .fn()
      .mockImplementation(async (_configuration: LinearOAuthConfiguration) => {
        linearConnectionStatus = { kind: "awaiting_authorization" };
        return linearConnectionStatus;
      }),
    beginLinearCommentAccess: vi.fn().mockImplementation(async () => {
      linearConnectionStatus = { kind: "awaiting_authorization" };
      return linearConnectionStatus;
    }),
    linearConnectionStatus: vi
      .fn()
      .mockImplementation(async () => linearConnectionStatus),
    linearAssignedIssues: vi.fn().mockImplementation(async () => linearIssues),
    importLinearIssue: vi.fn().mockImplementation(async (request) => {
      const current = access.current();
      const updated = {
        ...current,
        externalLinks: [
          ...current.externalLinks,
          {
            id: request.externalLinkId,
            workItemId: request.workItemId,
            connectorId: "linear",
            provenance: "imported" as const,
            externalId: request.issueId,
            displayIdentifier: request.displayIdentifier,
            url: request.url,
            connectionMode: request.connectionMode,
          },
        ],
      };
      access.replace(updated);
      return updated;
    }),
    importLinearBlocker: vi.fn().mockImplementation(async (request) => {
      const current = access.current();
      const upstream = current.externalLinks.find(
        (link) => link.externalId === request.upstreamIssueId,
      );
      const downstream = current.externalLinks.find(
        (link) => link.externalId === request.downstreamIssueId,
      );
      const updated = {
        ...current,
        dependencies: [
          ...current.dependencies,
          {
            id: request.dependencyId,
            upstreamWorkItemId: upstream?.workItemId ?? request.upstreamIssueId,
            downstreamWorkItemId:
              downstream?.workItemId ?? request.downstreamIssueId,
            kind: "blocks" as const,
            reason: request.reason,
            owner: request.owner,
            nextAction: request.nextAction,
          },
        ],
      };
      access.replace(updated);
      return updated;
    }),
    queueLinearComment: vi
      .fn()
      .mockImplementation(async (request: QueueLinearCommentRequest) => {
        const current = access.current();
        const updated = {
          ...current,
          connectorOutboxItems: [
            ...current.connectorOutboxItems,
            {
              id: request.outboxItemId,
              workItemId: request.workItemId,
              connectorId: "linear",
              externalLinkId:
                current.externalLinks.find(
                  (link) => link.workItemId === request.workItemId,
                )?.id ?? "linear-link",
              idempotencyKey: request.idempotencyKey,
              operation: {
                comment: {
                  body: `Public update: ${request.publicSummary}`,
                },
              },
              state: "pending" as const,
              createdAt: request.recordedAt,
            },
          ],
        };
        access.replace(updated);
        return updated;
      }),
    observeLinearSharedField: vi
      .fn()
      .mockImplementation(async (request: ObserveLinearSharedFieldRequest) => {
        const current = access.current();
        const link = current.externalLinks.find(
          (candidate) => candidate.id === request.externalLinkId,
        );
        const workItem = current.workItems.find(
          (candidate) => candidate.workItem.id === link?.workItemId,
        )?.workItem;
        const localValue =
          request.field === "title"
            ? workItem?.title
            : request.field === "description"
              ? workItem?.description
              : workItem?.state;
        const updated = {
          ...current,
          connectorReconciliationItems: [
            ...current.connectorReconciliationItems,
            {
              id: request.reconciliationItemId,
              workItemId: link?.workItemId ?? "unknown",
              connectorId: "linear",
              externalLinkId: request.externalLinkId,
              field: request.field,
              localValue: localValue ?? "",
              remoteValue: request.remoteValue,
              remoteRevision: request.remoteRevision,
              state:
                localValue === request.remoteValue
                  ? ("matched" as const)
                  : ("needs_resolution" as const),
              observedAt: request.observedAt,
            },
          ],
        };
        access.replace(updated);
        return updated;
      }),
    syncLinearSharedFields: vi
      .fn()
      .mockImplementation(async () => access.current()),
    deliverLinearComment: vi.fn().mockImplementation(async (outboxItemId) => {
      const current = access.current();
      const updated = {
        ...current,
        connectorOutboxItems: current.connectorOutboxItems.map((item) =>
          item.id === outboxItemId
            ? { ...item, state: "delivered" as const, deliveredAt: "now" }
            : item,
        ),
      };
      access.replace(updated);
      return updated;
    }),
  };
}
