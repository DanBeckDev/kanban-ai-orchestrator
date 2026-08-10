import { fireEvent, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway } from "./BoardWorkspace.test.fixtures";
import { createBoard, openPlan } from "./BoardWorkspace.test.helpers";

describe("plan input validation", () => {
  it("explains malformed plan JSON before it reaches the daemon", async () => {
    const boardGateway = gateway();
    await createBoard(boardGateway);
    openPlan();
    const proposalForm = screen.getByRole("form", {
      name: "Paste an existing plan",
    });
    fireEvent.change(within(proposalForm).getByLabelText("Plan JSON"), {
      target: { value: JSON.stringify({ dependencies: [] }) },
    });
    fireEvent.click(
      within(proposalForm).getByRole("button", {
        name: "Preview pasted plan",
      }),
    );

    expect(
      await screen.findAllByText(
        "Add a workItems list to the plan JSON, then preview it again.",
      ),
    ).toHaveLength(1);
    expect(boardGateway.proposePlan).not.toHaveBeenCalled();
  });
});
