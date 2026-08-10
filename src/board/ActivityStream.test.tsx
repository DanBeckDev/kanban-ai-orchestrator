import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ActivityStream } from "./ActivityStream";
import type { Execution, ExecutionActivityChunk } from "./types";

const execution: Execution = {
  id: "execution-1",
  workItemId: "task-1",
  role: "implementation",
  adapterName: "codex",
  status: "running",
  workspacePath: "/workspace/task-1",
  usage: { inputTokens: 0, outputTokens: 0 },
  lastEventSequence: 0,
};

function chunk(sequence: number): ExecutionActivityChunk {
  return {
    sequence,
    kind: "activity",
    summary: `Activity ${sequence}`,
    recordedAt: "2026-08-09T00:00:00Z",
  };
}

describe("ActivityStream", () => {
  it("loads bounded pages and renders only a virtualized window", async () => {
    const onLoad = vi.fn().mockImplementation(async (_id, after?: number) => {
      if (after === undefined) {
        return {
          chunks: Array.from({ length: 32 }, (_, index) => chunk(index + 1)),
          hasMore: true,
        };
      }
      return {
        chunks: Array.from({ length: 8 }, (_, index) => chunk(index + 33)),
        hasMore: false,
      };
    });

    render(<ActivityStream execution={execution} onLoad={onLoad} />);

    await screen.findByText("Activity 1");
    await waitFor(() => expect(onLoad).toHaveBeenCalledWith("execution-1", 32));
    expect(screen.queryByText("Activity 20")).not.toBeInTheDocument();

    fireEvent.scroll(screen.getByRole("log"), { target: { scrollTop: 960 } });

    expect(await screen.findByText("Activity 20")).toBeVisible();
  });

  it("shows a recoverable message when the local activity command is unavailable", async () => {
    const onLoad = vi.fn().mockRejectedValue(new Error("daemon restarting"));

    render(<ActivityStream execution={execution} onLoad={onLoad} />);

    expect(
      await screen.findByText(
        "Activity is temporarily unavailable. Keep this task open and Kanban will try again.",
      ),
    ).toBeVisible();
  });
});
