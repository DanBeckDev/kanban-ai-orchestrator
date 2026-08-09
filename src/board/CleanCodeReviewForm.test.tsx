import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { CleanCodeReviewForm } from "./CleanCodeReviewForm";
import type { Execution, WorkItem } from "./types";

const workItem: WorkItem = {
  id: "task-1",
  boardId: "board-1",
  title: "Task one",
  description: "A bounded task.",
  acceptanceCriteria: ["The task is reviewed."],
  budget: {},
  state: "review",
  requiresHumanReview: true,
};

const completedReview: Execution = {
  id: "review-execution-1",
  workItemId: "task-1",
  role: "independent_review",
  adapterName: "reviewer",
  status: "completed",
  sessionId: "review-session",
  workspacePath: "/workspaces/task-1",
  usage: { inputTokens: 0, outputTokens: 0 },
  lastEventSequence: 1,
};

describe("CleanCodeReviewForm", () => {
  it("requires a completed independent reviewer run before allowing a decision", () => {
    render(
      <CleanCodeReviewForm
        busy={false}
        reviewExecutions={[]}
        workItem={workItem}
        onRecord={vi.fn()}
      />,
    );

    expect(
      screen.getByText(/Start and complete an independent review agent/i),
    ).toBeVisible();
  });

  it("records a structured zero-finding decision for the selected reviewer", async () => {
    const onRecord = vi.fn().mockResolvedValue(undefined);
    render(
      <CleanCodeReviewForm
        busy={false}
        reviewExecutions={[completedReview]}
        workItem={workItem}
        onRecord={onRecord}
      />,
    );

    const form = screen.getByRole("form", {
      name: "Record Clean Code review for Task one",
    });
    fireEvent.pointerDown(screen.getByLabelText("Completed reviewer run"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });
    fireEvent.click(
      await screen.findByRole("option", { name: /review-execution-1/ }),
    );
    fireEvent.change(screen.getByLabelText("Decision summary"), {
      target: { value: "No actionable findings." },
    });
    fireEvent.submit(form);

    await vi.waitFor(() => expect(onRecord).toHaveBeenCalledOnce());
    expect(onRecord).toHaveBeenCalledWith(
      expect.objectContaining({
        workItemId: "task-1",
        reviewExecutionId: "review-execution-1",
        actionableFindingCount: 0,
        summary: "No actionable findings.",
      }),
    );
  });
});
