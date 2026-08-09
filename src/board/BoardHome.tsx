import { BotIcon, CircleAlertIcon, CircleDotDashedIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import { executionsFor } from "./presentation";
import type { BoardSnapshot, WorkItem, WorkItemState } from "./types";

type BoardHomeProps = Readonly<{
  snapshot: BoardSnapshot;
  onOpenTask: (workItemId: string) => void;
}>;

type BoardHomeTask = Readonly<{
  workItem: WorkItem;
  action: string;
  detail: string;
}>;

type AttentionState = Extract<
  WorkItemState,
  "awaiting_input" | "review" | "failed" | "interrupted" | "blocked"
>;

const attentionStates: readonly AttentionState[] = [
  "awaiting_input",
  "review",
  "failed",
  "interrupted",
  "blocked",
];

export function BoardHome({ snapshot, onOpenTask }: BoardHomeProps) {
  const workItems = workItemsFor(snapshot);
  const attention = attentionFor(workItems);
  const activeWork = activeWorkFor(snapshot);
  const delivery = deliveryPicture(workItems);

  return (
    <section aria-labelledby="board-home-title" className="board-home">
      <header className="board-home-header">
        <div>
          <p className="eyebrow">Board home</p>
          <h3 id="board-home-title">Your next decisions</h3>
          <p>{boardHomeSummary(attention, activeWork, delivery.ready)}</p>
        </div>
        <Badge variant={attention.length > 0 ? "destructive" : "secondary"}>
          {attention.length > 0 ? "Decision needed" : "On track"}
        </Badge>
      </header>
      <div className="board-home-focus">
        <FocusList
          emptyCopy="No task needs a decision right now."
          headingId="needs-attention-title"
          icon={CircleAlertIcon}
          items={attention}
          onOpenTask={onOpenTask}
          title="Needs your attention"
        />
        <FocusList
          emptyCopy="No agents are working right now."
          headingId="work-in-motion-title"
          icon={BotIcon}
          items={activeWork}
          onOpenTask={onOpenTask}
          title="Work in motion"
        />
      </div>
      <section
        aria-labelledby="delivery-picture-title"
        className="delivery-picture"
      >
        <div className="delivery-picture-heading">
          <CircleDotDashedIcon aria-hidden="true" />
          <h4 id="delivery-picture-title">Delivery picture</h4>
        </div>
        <dl>
          {delivery.entries.map(({ count, label }) => (
            <div key={label}>
              <dt>{label}</dt>
              <dd>{count}</dd>
            </div>
          ))}
        </dl>
      </section>
    </section>
  );
}

function FocusList({
  emptyCopy,
  headingId,
  icon: Icon,
  items,
  onOpenTask,
  title,
}: Readonly<{
  emptyCopy: string;
  headingId: string;
  icon: typeof CircleAlertIcon;
  items: readonly BoardHomeTask[];
  onOpenTask: (workItemId: string) => void;
  title: string;
}>) {
  return (
    <section aria-labelledby={headingId}>
      <div className="board-home-section-heading">
        <Icon aria-hidden="true" />
        <h4 id={headingId}>{title}</h4>
        <Badge variant="outline">{items.length}</Badge>
      </div>
      {items.length === 0 ? (
        <p className="board-home-empty">{emptyCopy}</p>
      ) : (
        <ul className="board-home-task-list">
          {items.map(({ action, detail, workItem }) => (
            <li key={workItem.id}>
              <div>
                <strong>{workItem.title}</strong>
                <p>{detail}</p>
              </div>
              <Button
                aria-label={`${action} ${workItem.title}`}
                onClick={() => onOpenTask(workItem.id)}
                size="sm"
                type="button"
                variant="outline"
              >
                {action}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function workItemsFor(snapshot: BoardSnapshot): readonly WorkItem[] {
  return snapshot.workItems.map(({ workItem }) => workItem);
}

function attentionFor(
  workItems: readonly WorkItem[],
): readonly BoardHomeTask[] {
  return workItems
    .filter((workItem) => isAttentionState(workItem.state))
    .sort(
      (left, right) =>
        attentionStates.indexOf(left.state) -
        attentionStates.indexOf(right.state),
    )
    .map((workItem) => ({
      workItem,
      ...attentionCopy(workItem.state),
    }));
}

function isAttentionState(state: WorkItemState): state is AttentionState {
  return attentionStates.includes(state as AttentionState);
}

function activeWorkFor(snapshot: BoardSnapshot): readonly BoardHomeTask[] {
  return snapshot.workItems
    .map(({ workItem }) => workItem)
    .filter((workItem) => workItem.state === "running")
    .map((workItem) => {
      const worker = executionsFor(snapshot, workItem.id).find(
        (execution) => execution.status === "running",
      );
      return {
        workItem,
        action: "View work",
        detail:
          worker === undefined
            ? "An agent is working on this task."
            : `${worker.adapterName} is working on this task.`,
      };
    });
}

function deliveryPicture(workItems: readonly WorkItem[]) {
  return {
    ready: countWorkItems(workItems, ["ready"]),
    entries: [
      {
        label: "Planned",
        count: countWorkItems(workItems, ["inbox", "planned"]),
      },
      { label: "Ready", count: countWorkItems(workItems, ["ready"]) },
      { label: "In review", count: countWorkItems(workItems, ["review"]) },
      { label: "Completed", count: countWorkItems(workItems, ["done"]) },
      {
        label: "Recovery",
        count: countWorkItems(workItems, ["blocked", "failed", "interrupted"]),
      },
    ],
  };
}

function countWorkItems(
  workItems: readonly WorkItem[],
  states: readonly WorkItemState[],
): number {
  return workItems.filter((workItem) => states.includes(workItem.state)).length;
}

function attentionCopy(state: AttentionState): Omit<BoardHomeTask, "workItem"> {
  switch (state) {
    case "awaiting_input":
      return {
        action: "Inspect",
        detail: "This agent needs a decision to continue.",
      };
    case "review":
      return { action: "Review", detail: "This work is ready for review." };
    case "failed":
      return {
        action: "Recover",
        detail: "The last attempt failed and needs a recovery choice.",
      };
    case "interrupted":
      return {
        action: "Recover",
        detail: "The last attempt was interrupted and needs a recovery choice.",
      };
    case "blocked":
      return {
        action: "Unblock",
        detail: "This task is blocked and needs an owner or next step.",
      };
  }
}

function boardHomeSummary(
  attention: readonly BoardHomeTask[],
  activeWork: readonly BoardHomeTask[],
  readyCount: number,
): string {
  if (attention.length > 0) {
    return `${taskCount(attention.length)} needs your decision before work can move forward.`;
  }
  if (activeWork.length > 0) {
    return `${taskCount(activeWork.length)} is in progress; you can monitor it here.`;
  }
  if (readyCount > 0) {
    return `${taskCount(readyCount)} is ready for the next approved action.`;
  }
  return "No work needs action right now. Plan the next outcome when you are ready.";
}

function taskCount(count: number): string {
  return count === 1 ? "1 task" : `${count} tasks`;
}
