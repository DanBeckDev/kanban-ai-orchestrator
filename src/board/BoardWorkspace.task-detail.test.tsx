import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard, openTask } from "./BoardWorkspace.test.helpers";

describe("focused task detail", () => {
  it("leads with the review decision and keeps task context in labelled disclosures", async () => {
    const boardGateway = gateway(
      snapshot(
        [workItem("review-task", "review")],
        [],
        [],
        [
          {
            id: "check-1",
            workItemId: "review-task",
            kind: "check",
            result: "passed",
            summary: "Unit tests passed.",
            recordedAt: "2026-08-09T00:00:00Z",
          },
        ],
      ),
    );

    await createBoard(boardGateway);
    openTask("Task review-task");

    expect(
      screen.getByRole("heading", { name: "Review this task" }),
    ).toBeVisible();
    expect(
      screen.getByText("Record checks and make a review decision."),
    ).toBeVisible();
    expect(screen.queryByText("Tests pass.")).toBeNull();

    const details = screen.getByRole("button", {
      name: "Task details and success checks",
    });
    details.focus();
    fireEvent.click(details);

    expect(details).toHaveFocus();
    expect(screen.getByText("Tests pass.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Review evidence" }));
    expect(screen.getByText("Unit tests passed.")).toBeVisible();
  });

  it("makes recovery visibly different from completion and restores workflow focus", async () => {
    const boardGateway = gateway(
      snapshot([workItem("recovery", "interrupted")]),
    );

    await createBoard(boardGateway);
    openTask("Task recovery");

    expect(
      screen.getByRole("heading", {
        name: "The last attempt was interrupted",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("region", {
        name: "Recovery actions for Task recovery",
      }),
    ).toBeVisible();
    expect(screen.queryByText("Completion is recorded")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
    await screen.findByRole("heading", { name: "Prompt AI to orchestrate" });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Workflow" })).toHaveFocus(),
    );
  });
});
