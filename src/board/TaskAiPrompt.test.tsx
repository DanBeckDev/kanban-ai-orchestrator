import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TaskAiPrompt } from "./TaskAiPrompt";
import type { TicketEffectOperations } from "./ticketEffectOperations";
import type { TicketEffect } from "./ticketEffectTypes";

describe("task AI prompt", () => {
  it("asks for a typed, task-scoped evidence explanation", async () => {
    const operations = operationGateway();
    render(
      <TaskAiPrompt
        busy={false}
        hasOrganiser
        operations={operations}
        workItemId="task-1"
      />,
    );

    fireEvent.change(screen.getByLabelText("What do you need?"), {
      target: { value: "Explain the failing check." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ask task AI" }));

    await waitFor(() =>
      expect(operations.request).toHaveBeenCalledWith(
        expect.objectContaining({
          action: "explain_evidence",
          prompt: "Explain the failing check.",
          workItemId: "task-1",
        }),
      ),
    );
  });

  it("shows a manual proposal as named apply, reject, or dismiss choices", async () => {
    const operations = operationGateway([awaitingApprovalEffect()]);
    render(
      <TaskAiPrompt
        busy={false}
        hasOrganiser
        operations={operations}
        workItemId="task-1"
      />,
    );

    expect(await screen.findByText("Guide the worker")).toBeVisible();
    expect(
      screen.getByText("Worker guidance: Start with the focused test."),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));

    await waitFor(() =>
      expect(operations.resolve).toHaveBeenCalledWith({
        effectId: "effect-1",
        resolution: "apply",
      }),
    );
    expect(screen.getByRole("button", { name: "Reject" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Dismiss" })).toBeVisible();
  });

  it("shows proposed task details before a reviewer can apply a refinement", async () => {
    const effect: TicketEffect = {
      ...awaitingApprovalEffect(),
      action: "refine_specification",
      proposal: {
        title: "Clarify first run",
        description: "Explain the first safe step for a new board.",
        acceptanceCriteria: ["A user can finish setup without an ID."],
      },
    };
    render(
      <TaskAiPrompt
        busy={false}
        hasOrganiser
        operations={operationGateway([effect])}
        workItemId="task-1"
      />,
    );

    expect(
      await screen.findByText("Proposed title: Clarify first run"),
    ).toBeVisible();
    expect(
      screen.getByText("A user can finish setup without an ID."),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
  });

  it("keeps task AI unavailable until Settings has an orchestrator", () => {
    render(
      <TaskAiPrompt
        busy={false}
        hasOrganiser={false}
        operations={operationGateway()}
        workItemId="task-1"
      />,
    );

    expect(screen.getByText("Choose an orchestrator first")).toBeVisible();
    expect(screen.getByRole("button", { name: "Ask task AI" })).toBeDisabled();
  });
});

function operationGateway(
  effects: readonly TicketEffect[] = [],
): TicketEffectOperations {
  return {
    request: vi.fn().mockResolvedValue(undefined),
    resolve: vi.fn().mockResolvedValue(undefined),
    load: vi.fn().mockResolvedValue(effects),
  };
}

function awaitingApprovalEffect(): TicketEffect {
  return {
    id: "effect-1",
    boardId: "board-1",
    workItemId: "task-1",
    organiserProfileName: "organiser",
    action: "give_worker_guidance",
    promptSummary: "Tell the worker where to begin.",
    recommendation: "Start with the focused test.",
    rationale: "The task needs a clear first step.",
    proposal: {
      acceptanceCriteria: [],
      workerGuidance: "Start with the focused test.",
    },
    authorityMode: "manual",
    policyResult: "not_required",
    outcome: "awaiting_approval",
    expectedWorkItemSequence: 1,
    recordedAt: "2026-08-10T12:00:00Z",
  };
}
