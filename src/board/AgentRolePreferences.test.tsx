import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AgentRolePreferences } from "./AgentRolePreferences";

describe("AgentRolePreferences", () => {
  it("keeps a long model list inside a contained picker scroll region", async () => {
    render(
      <AgentRolePreferences
        effort="provider_default"
        idPrefix="worker"
        model={{ kind: "provider_default" }}
        models={Array.from({ length: 100 }, (_, index) => ({
          id: `model-${index}`,
          label: `Model ${index}`,
          efforts: [],
        }))}
        onEffortChange={vi.fn()}
        onModelChange={vi.fn()}
      />,
    );

    fireEvent.pointerDown(screen.getByLabelText("Model"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });

    const picker = await screen.findByRole("listbox");
    expect(picker).toHaveClass("max-h-80", "overscroll-contain");
    expect(
      await screen.findByRole("option", { name: "Model 99" }),
    ).toBeVisible();
  });

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

  it("keeps provider default as the only effort for a model without reasoning support", async () => {
    render(
      <AgentRolePreferences
        effort="provider_default"
        idPrefix="worker"
        model={{ kind: "named", name: "openai/gpt-4o" }}
        models={[
          {
            id: "openai/gpt-4o",
            label: "GPT-4o",
            efforts: [],
          },
        ]}
        onEffortChange={vi.fn()}
        onModelChange={vi.fn()}
      />,
    );

    fireEvent.pointerDown(screen.getByLabelText("Effort"), {
      button: 0,
      ctrlKey: false,
      pointerType: "mouse",
    });

    expect(
      await screen.findByRole("option", { name: "Provider default" }),
    ).toBeVisible();
    expect(screen.queryByRole("option", { name: "Focused (low)" })).toBeNull();
  });
});
