import { useMemo, useState, type FormEvent } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";

import {
  commentsAreAuthorized,
  connectedLinearDescription,
} from "./linearConnectionPresentation";
import type {
  BoardSnapshot,
  LinearConnectionStatus,
  QueueLinearCommentRequest,
} from "./types";

type LinearSyncPanelProps = Readonly<{
  busy: boolean;
  connectionStatus: LinearConnectionStatus;
  snapshot: BoardSnapshot;
  onDeliver: (outboxItemId: string) => Promise<void>;
  onQueue: (request: QueueLinearCommentRequest) => Promise<void>;
  onRefresh: (externalLinkId: string) => Promise<void>;
}>;

export function LinearSyncPanel({
  busy,
  connectionStatus,
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
  const canQueueComments = commentsAreAuthorized(connectionStatus);

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
      {!canQueueComments && (
        <Alert>
          <AlertTitle>Comments are not enabled</AlertTitle>
          <AlertDescription>
            {connectedLinearDescription(connectionStatus)} Enable manually sent
            Linear comments before you queue a public update.
          </AlertDescription>
        </Alert>
      )}
      {linkedWorkItems.length === 0 ? (
        <p>
          Import an issue in linked-execution mode before queuing a comment.
        </p>
      ) : canQueueComments ? (
        <form aria-label="Queue public Linear comment" onSubmit={queueComment}>
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="linear-comment-work-item">
                Linked task
              </FieldLabel>
              <select
                id="linear-comment-work-item"
                name="linear-comment-work-item"
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
            </Field>
            <Field>
              <FieldLabel htmlFor="linear-public-summary">
                Public update
              </FieldLabel>
              <Input
                autoComplete="off"
                id="linear-public-summary"
                maxLength={512}
                name="linear-public-summary"
                required
                value={publicSummary}
                onChange={(event) => setPublicSummary(event.target.value)}
              />
              <FieldDescription>
                This is the only text Kanban will include in the comment.
              </FieldDescription>
            </Field>
            <Button disabled={busy || workItemId === ""} type="submit">
              Queue Linear comment
            </Button>
          </FieldGroup>
        </form>
      ) : null}
      {linkedWorkItems.length > 0 && (
        <section aria-label="Refresh shared Linear fields">
          <h4>Refresh shared fields</h4>
          {linkedWorkItems.map(({ link }) => (
            <Button
              disabled={busy}
              key={link.id}
              type="button"
              onClick={() => void onRefresh(link.id)}
              variant="outline"
            >
              Refresh {link.displayIdentifier}
            </Button>
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
              <Button
                disabled={busy}
                type="button"
                onClick={() => void onDeliver(item.id)}
                size="sm"
              >
                Send comment
              </Button>
            )}
            {item.state === "delivery_uncertain" && (
              <p>
                The delivery result is unknown. Check Linear before deciding
                whether to send a new update; Kanban will not retry this one
                automatically.
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
              <>
                <p>
                  Choose which value to keep outside this view, then refresh
                  shared fields again. Kanban has not overwritten either value.
                </p>
                <dl>
                  <dt>Local</dt>
                  <dd>{item.localValue}</dd>
                  <dt>Linear</dt>
                  <dd>{item.remoteValue}</dd>
                </dl>
              </>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
