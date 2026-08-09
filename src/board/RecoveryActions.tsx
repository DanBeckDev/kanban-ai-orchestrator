import { useState } from "react";

import { timestamp } from "./presentation";
import type { Execution, TransitionWorkItemRequest, WorkItem } from "./types";

type RecoveryActionsProps = Readonly<{
  busy: boolean;
  executions: readonly Execution[];
  workItem: WorkItem;
  onTransition: (request: TransitionWorkItemRequest) => Promise<void>;
}>;

export function RecoveryActions({
  busy,
  executions,
  workItem,
  onTransition,
}: RecoveryActionsProps) {
  const [showAttempt, setShowAttempt] = useState(false);

  async function transition(nextState: "ready" | "cancelled", reason: string) {
    const recordedAt = timestamp();
    await onTransition({
      eventId: `recovery-${workItem.id}-${nextState}-${recordedAt}`,
      workItemId: workItem.id,
      nextState,
      reason,
      recordedAt,
    });
  }

  return (
    <section
      aria-label={`Recovery actions for ${workItem.title}`}
      className="recovery-actions"
    >
      <h5>Recovery actions</h5>
      <div>
        <button
          disabled={busy}
          type="button"
          onClick={() => setShowAttempt((current) => !current)}
        >
          Inspect last attempt
        </button>
        <button
          disabled={busy}
          type="button"
          onClick={() =>
            void transition(
              "ready",
              "Recovery approved; task is ready to retry.",
            )
          }
        >
          Recover to Ready
        </button>
        <button
          disabled={busy}
          type="button"
          onClick={() =>
            void transition(
              "cancelled",
              "Task cancelled after recovery review.",
            )
          }
        >
          Cancel task
        </button>
      </div>
      {showAttempt && <LatestAttempt executions={executions} />}
    </section>
  );
}

function LatestAttempt({
  executions,
}: Readonly<{ executions: readonly Execution[] }>) {
  const execution = executions.at(-1);
  if (execution === undefined) {
    return <p>No durable agent attempt has been recorded for this task.</p>;
  }
  return (
    <dl className="latest-attempt">
      <dt>Adapter</dt>
      <dd>{execution.adapterName}</dd>
      <dt>Outcome</dt>
      <dd>{execution.status.replaceAll("_", " ")}</dd>
      <dt>Workspace</dt>
      <dd>{execution.workspacePath}</dd>
    </dl>
  );
}
