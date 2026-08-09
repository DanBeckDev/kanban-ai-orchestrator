import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AgentLaunchForm } from "./AgentLaunchForm";
import type { WorkItem } from "./types";

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

describe("AgentLaunchForm", () => {
  it("creates a distinct independent-review execution request", async () => {
    const onStart = vi.fn().mockResolvedValue(undefined);
    render(
      <AgentLaunchForm
        busy={false}
        executionRole="independent_review"
        formLabel="Start independent reviewer for Task one"
        profiles={[
          {
            name: "reviewer",
            kind: "structured_process",
            program: "review-agent",
            arguments: [],
          },
        ]}
        workItem={workItem}
        onStart={onStart}
      />,
    );

    const form = screen.getByRole("form", {
      name: "Start independent reviewer for Task one",
    });
    fireEvent.pointerDown(screen.getByLabelText("Agent profile"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });
    fireEvent.click(await screen.findByRole("option", { name: /reviewer/ }));
    fireEvent.submit(form);

    await vi.waitFor(() => expect(onStart).toHaveBeenCalledOnce());
    expect(onStart).toHaveBeenCalledWith(
      expect.objectContaining({
        agentProfileName: "reviewer",
        executionRole: "independent_review",
        taskBrief: expect.stringMatching(/Do not edit files/),
      }),
    );
  });
});
