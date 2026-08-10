import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AgentRolePreferences } from "./AgentRolePreferences";

describe("AgentRolePreferences", () => {
  it("keeps a saved maximum effort selectable when its model is no longer listed", async () => {
    const onEffortChange = vi.fn();

    render(
      <AgentRolePreferences
        effort="maximum"
        idPrefix="worker"
        model={{ kind: "named", name: "retired-claude" }}
        models={[]}
        onEffortChange={onEffortChange}
        onModelChange={vi.fn()}
      />,
    );

    expect(
      screen.getByText("retired-claude (not in the current model list)"),
    ).toBeVisible();
    fireEvent.pointerDown(screen.getByLabelText("Effort"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });
    expect(
      await screen.findByRole("option", { name: "Maximum (max)" }),
    ).toBeVisible();
    fireEvent.click(
      await screen.findByRole("option", {
        name: "Extra thorough (xhigh)",
      }),
    );

    expect(onEffortChange).toHaveBeenCalledWith("extra_thorough");
  });
});
