import { useMemo, useState, type FormEvent } from "react";

import type { BoardSnapshot, QueueLinearCommentRequest } from "./types";

type LinearSyncPanelProps = Readonly<{
  busy: boolean;
  snapshot: BoardSnapshot;
  onDeliver: (outboxItemId: string) => Promise<void>;
  onQueue: (request: QueueLinearCommentRequest) => Promise<void>;
  onRefresh: (externalLinkId: string) => Promise<void>;
}>;

export function LinearSyncPanel({
  busy,
  snapshot,
  onDeliver,
  onQueue,
  onRefresh,
}: LinearSyncPanelProps) {
  const [workItemId, setWorkItemId] = useState("");
  const [publicSummary, setPublicSummary] = useState("");
  const links = useMemo(
    () =>
      snapshot.externalLinks.filter(
        (link) =>
          link.connectorId === "linear" &&
          link.connectionMode === "linked_execution",
      ),
    [snapshot.externalLinks],
  );
  const linkedWorkItems = links.flatMap((link) => {
    const workItem = snapshot.workItems.find(
      (candidate) => candidate.workItem.id === link.workItemId,
    )?.workItem;
    return workItem === undefined ? [] : [{ link, workItem }];
  });
  const outboxItems = snapshot.connectorOutboxItems.filter(
    (item) => item.connectorId === "linear",
  );
  const reconciliationItems = snapshot.connectorReconciliationItems.filter(
    (item) => item.connectorId === "linear",
  );

  async function queueComment(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (workItemId === "") return;
    const recordedAt = new Date().toISOString();
    const idempotencyKey = `linear-comment:${workItemId}:${recordedAt}`;
    await onQueue({
      outboxItemId: idempotencyKey,
      workItemId,
      idempotencyKey,
      publicSummary,
      recordedAt,
    });
    setPublicSummary("");
  }

  return (
    <section className="linear-sync-panel">
      <h3>Linear synchronization</h3>
      <p>
        Comments use a durable local outbox. Only this explicitly entered public
        update is sent; transcripts, command output, secrets, and diffs are not
        inputs to this form.
      </p>
      {linkedWorkItems.length === 0 ? (
        <p>
          Import an issue in linked-execution mode before queuing a comment.
        </p>
      ) : (
        <form aria-label="Queue public Linear comment" onSubmit={queueComment}>
          <label>
            Linked task
            <select
              required
              value={workItemId}
              onChange={(event) => setWorkItemId(event.target.value)}
            >
              <option value="">Select a linked task</option>
              {linkedWorkItems.map(({ link, workItem }) => (
                <option key={link.id} value={workItem.id}>
                  {link.displayIdentifier}: {workItem.title}
                </option>
              ))}
            </select>
          </label>
          <label>
            Public update
            <input
              maxLength={512}
              required
              value={publicSummary}
              onChange={(event) => setPublicSummary(event.target.value)}
            />
          </label>
          <button disabled={busy || workItemId === ""} type="submit">
            Queue Linear comment
          </button>
        </form>
      )}
      {linkedWorkItems.length > 0 && (
        <section aria-label="Refresh shared Linear fields">
          <h4>Refresh shared fields</h4>
          {linkedWorkItems.map(({ link }) => (
            <button
              disabled={busy}
              key={link.id}
              type="button"
              onClick={() => void onRefresh(link.id)}
            >
              Refresh {link.displayIdentifier}
            </button>
          ))}
        </section>
      )}
      <OutboxItems busy={busy} items={outboxItems} onDeliver={onDeliver} />
      <ReconciliationItems items={reconciliationItems} />
    </section>
  );
}

function OutboxItems({
  busy,
  items,
  onDeliver,
}: Readonly<{
  busy: boolean;
  items: readonly BoardSnapshot["connectorOutboxItems"][number][];
  onDeliver: (outboxItemId: string) => Promise<void>;
}>) {
  if (items.length === 0) return null;
  return (
    <section aria-label="Linear comment outbox">
      <h4>Comment outbox</h4>
      <ul>
        {items.map((item) => (
          <li key={item.id}>
            <strong>{item.state.replaceAll("_", " ")}</strong>
            {"comment" in item.operation && (
              <p>{item.operation.comment.body}</p>
            )}
            {item.state === "pending" && (
              <button
                disabled={busy}
                type="button"
                onClick={() => void onDeliver(item.id)}
              >
                Send comment
              </button>
            )}
            {item.state === "delivery_uncertain" && (
              <p>
                The delivery result is unknown. It will not retry automatically.
              </p>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

function ReconciliationItems({
  items,
}: Readonly<{
  items: readonly BoardSnapshot["connectorReconciliationItems"][number][];
}>) {
  if (items.length === 0) return null;
  return (
    <section aria-label="Linear reconciliation">
      <h4>Shared-field reconciliation</h4>
      <ul>
        {items.map((item) => (
          <li key={item.id}>
            <strong>{item.field.replaceAll("_", " ")}</strong>:{" "}
            {item.state.replaceAll("_", " ")}
            {item.state === "needs_resolution" && (
              <dl>
                <dt>Local</dt>
                <dd>{item.localValue}</dd>
                <dt>Linear</dt>
                <dd>{item.remoteValue}</dd>
              </dl>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
