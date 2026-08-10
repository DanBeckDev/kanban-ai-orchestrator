import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LinearSyncPanel } from "./LinearSyncPanel";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

const connectedWithCommentAccess = {
  kind: "connected" as const,
  expiresAt: "2026-08-10T12:00:00Z",
  scopes: ["read", "comments:create"],
};

function linkedSnapshot() {
  return {
    ...snapshot([workItem("task-1")]),
    externalLinks: [
      {
        id: "linear-link-1",
        workItemId: "task-1",
        connectorId: "linear",
        provenance: "imported" as const,
        externalId: "7d64b2ce-45d7-4c2b-a55b-7b1929dc89ad",
        displayIdentifier: "LIN-42",
        url: "https://linear.app/example/issue/LIN-42/sync",
        connectionMode: "linked_execution" as const,
      },
    ],
  };
}

describe("LinearSyncPanel", () => {
  it("queues only an explicit public summary for a linked execution task", () => {
    const onQueue = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearSyncPanel
        busy={false}
        connectionStatus={connectedWithCommentAccess}
        snapshot={linkedSnapshot()}
        onDeliver={vi.fn().mockResolvedValue(undefined)}
        onQueue={onQueue}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    const form = screen.getByRole("form", {
      name: "Queue public Linear comment",
    });
    fireEvent.change(within(form).getByLabelText("Linked task"), {
      target: { value: "task-1" },
    });
    fireEvent.change(within(form).getByLabelText("Public update"), {
      target: { value: "Checks passed; ready for review." },
    });
    fireEvent.submit(form);

    expect(onQueue).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "task-1",
        publicSummary: "Checks passed; ready for review.",
      }),
    );
    const request = onQueue.mock.calls[0][0];
    expect(request.outboxItemId).toBe(request.idempotencyKey);
  });

  it("makes each conflicting shared field visible without claiming it was overwritten", () => {
    const board = {
      ...linkedSnapshot(),
      connectorReconciliationItems: [
        {
          id: "reconciliation-1",
          workItemId: "task-1",
          connectorId: "linear",
          externalLinkId: "linear-link-1",
          field: "title" as const,
          localValue: "Local title",
          remoteValue: "Linear title",
          remoteRevision: "2026-08-09T12:00:00.000Z",
          state: "needs_resolution" as const,
          observedAt: "2026-08-09T12:00:01Z",
        },
      ],
    };
    render(
      <LinearSyncPanel
        busy={false}
        connectionStatus={connectedWithCommentAccess}
        snapshot={board}
        onDeliver={vi.fn().mockResolvedValue(undefined)}
        onQueue={vi.fn().mockResolvedValue(undefined)}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    const reconciliation = screen.getByRole("region", {
      name: "Linear reconciliation",
    });
    expect(within(reconciliation).getByText("Local title")).toBeInTheDocument();
    expect(
      within(reconciliation).getByText("Linear title"),
    ).toBeInTheDocument();
    expect(
      within(reconciliation).getByText(/needs resolution/),
    ).toBeInTheDocument();
    expect(
      within(reconciliation).getByText(
        "Choose which value to keep outside this view, then refresh shared fields again. Kanban has not overwritten either value.",
      ),
    ).toBeVisible();
  });

  it("refreshes one selected linked issue through the explicit sync command", () => {
    const onRefresh = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearSyncPanel
        busy={false}
        connectionStatus={connectedWithCommentAccess}
        snapshot={linkedSnapshot()}
        onDeliver={vi.fn().mockResolvedValue(undefined)}
        onQueue={vi.fn().mockResolvedValue(undefined)}
        onRefresh={onRefresh}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Refresh LIN-42" }));

    expect(onRefresh).toHaveBeenCalledWith("linear-link-1");
  });

  it("only exposes delivery for a pending outbox item", () => {
    const onDeliver = vi.fn().mockResolvedValue(undefined);
    const board = {
      ...linkedSnapshot(),
      connectorOutboxItems: [
        {
          id: "outbox-1",
          workItemId: "task-1",
          connectorId: "linear",
          externalLinkId: "linear-link-1",
          idempotencyKey: "task-1:review:1",
          operation: { comment: { body: "A safe public update." } },
          state: "pending" as const,
          createdAt: "2026-08-09T12:00:00Z",
        },
        {
          id: "outbox-2",
          workItemId: "task-1",
          connectorId: "linear",
          externalLinkId: "linear-link-1",
          idempotencyKey: "task-1:review:2",
          operation: { comment: { body: "Unknown external outcome." } },
          state: "delivery_uncertain" as const,
          createdAt: "2026-08-09T12:01:00Z",
        },
      ],
    };
    render(
      <LinearSyncPanel
        busy={false}
        connectionStatus={connectedWithCommentAccess}
        snapshot={board}
        onDeliver={onDeliver}
        onQueue={vi.fn().mockResolvedValue(undefined)}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Send comment" }));

    expect(onDeliver).toHaveBeenCalledWith("outbox-1");
    expect(
      screen.getByText(
        /Check Linear before deciding whether to send a new update/,
      ),
    ).toBeInTheDocument();
  });

  it("does not offer a comment form when comment scope is unavailable", () => {
    render(
      <LinearSyncPanel
        busy={false}
        connectionStatus={{
          kind: "connected",
          expiresAt: "2026-08-10T12:00:00Z",
          scopes: ["read"],
        }}
        snapshot={linkedSnapshot()}
        onDeliver={vi.fn().mockResolvedValue(undefined)}
        onQueue={vi.fn().mockResolvedValue(undefined)}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.queryByRole("form", { name: "Queue public Linear comment" }),
    ).toBeNull();
    expect(screen.getByText("Comments are not enabled")).toBeVisible();
  });
});
