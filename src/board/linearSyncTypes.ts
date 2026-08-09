export type ConnectorOutboxState =
  | "pending"
  | "delivering"
  | "delivered"
  | "delivery_uncertain";

export type ConnectorOutboxOperation = Readonly<{
  comment: Readonly<{ body: string }>;
}>;

export type ConnectorOutboxItem = Readonly<{
  id: string;
  workItemId: string;
  connectorId: string;
  externalLinkId: string;
  idempotencyKey: string;
  operation: ConnectorOutboxOperation;
  state: ConnectorOutboxState;
  createdAt: string;
  deliveredAt?: string;
}>;

export type ConnectorSharedField = "title" | "description" | "workflow_state";

export type ConnectorReconciliationState = "matched" | "needs_resolution";

export type ConnectorReconciliationItem = Readonly<{
  id: string;
  workItemId: string;
  connectorId: string;
  externalLinkId: string;
  field: ConnectorSharedField;
  localValue: string;
  remoteValue: string;
  remoteRevision: string;
  state: ConnectorReconciliationState;
  observedAt: string;
}>;

export type QueueLinearCommentRequest = Readonly<{
  outboxItemId: string;
  workItemId: string;
  idempotencyKey: string;
  publicSummary: string;
  recordedAt: string;
}>;

export type ObserveLinearSharedFieldRequest = Readonly<{
  reconciliationItemId: string;
  externalLinkId: string;
  field: ConnectorSharedField;
  remoteValue: string;
  remoteRevision: string;
  observedAt: string;
}>;
