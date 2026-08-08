import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LinearImportForm } from "./LinearImportForm";
import type { WorkItem } from "./types";

const workItems: readonly WorkItem[] = [
  {
    id: "task-1",
    boardId: "board-1",
    title: "Task one",
    description: "First task.",
    acceptanceCriteria: [],
    budget: {},
    state: "inbox",
    requiresHumanReview: false,
  },
];

describe("LinearImportForm", () => {
  it("imports an immutable issue link with its selected connection mode", async () => {
    const onImportIssue = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearImportForm
        busy={false}
        connectionStatus={{ kind: "disconnected" }}
        issues={[]}
        workItems={workItems}
        onImportBlocker={vi.fn().mockResolvedValue(undefined)}
        onImportIssue={onImportIssue}
        onLoadIssues={vi.fn().mockResolvedValue(undefined)}
      />,
    );
    const form = screen.getByRole("form", { name: "Import Linear issue" });
    fireEvent.change(within(form).getByLabelText("Local task"), {
      target: { value: "task-1" },
    });
    fireEvent.change(within(form).getByLabelText("Linear issue UUID"), {
      target: { value: "immutable-uuid" },
    });
    fireEvent.change(within(form).getByLabelText("Linear issue identifier"), {
      target: { value: "LIN-12" },
    });
    fireEvent.change(within(form).getByLabelText("Linear issue URL"), {
      target: { value: "https://linear.app/example/issue/LIN-12" },
    });
    fireEvent.change(within(form).getByLabelText("Connection mode"), {
      target: { value: "linked_execution" },
    });
    fireEvent.click(
      within(form).getByRole("button", { name: "Import Linear issue" }),
    );

    await waitFor(() =>
      expect(onImportIssue).toHaveBeenCalledWith(
        expect.objectContaining({
          workItemId: "task-1",
          issueId: "immutable-uuid",
          displayIdentifier: "LIN-12",
          connectionMode: "linked_execution",
        }),
      ),
    );
  });

  it("loads and uses a selected assigned issue without putting a token in the form", async () => {
    const onLoadIssues = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearImportForm
        busy={false}
        connectionStatus={{
          kind: "connected",
          expiresAt: "2026-08-09T12:00:00Z",
          scopes: ["read"],
        }}
        issues={[
          {
            id: "d290f1ee-6c54-4b01-90e6-d701748f0851",
            identifier: "LIN-12",
            title: "Load the issue",
            url: "https://linear.app/example/issue/LIN-12",
          },
        ]}
        workItems={workItems}
        onImportBlocker={vi.fn().mockResolvedValue(undefined)}
        onImportIssue={vi.fn().mockResolvedValue(undefined)}
        onLoadIssues={onLoadIssues}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Load my assigned Linear issues" }),
    );
    await waitFor(() => expect(onLoadIssues).toHaveBeenCalledOnce());
    fireEvent.click(
      screen.getByRole("button", { name: "Use LIN-12: Load the issue" }),
    );

    expect(screen.getByLabelText("Linear issue UUID")).toHaveValue(
      "d290f1ee-6c54-4b01-90e6-d701748f0851",
    );
    expect(screen.queryByLabelText(/access token/i)).toBeNull();
  });
});
