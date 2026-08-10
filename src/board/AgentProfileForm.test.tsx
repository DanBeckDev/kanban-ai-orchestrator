import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AgentProfileForm } from "./AgentProfileForm";

describe("AgentProfileForm", () => {
  it("gives every advanced configuration field a stable name and autocomplete policy", () => {
    render(<AgentProfileForm busy={false} onSave={vi.fn()} profiles={[]} />);

    expect(screen.getByLabelText("Profile name")).toHaveAttribute(
      "name",
      "agent-profile-name",
    );
    expect(screen.getByLabelText("Profile name")).toHaveAttribute(
      "autocomplete",
      "off",
    );
    expect(screen.getByLabelText("Adapter")).toHaveAttribute(
      "name",
      "agent-profile-kind",
    );
    expect(screen.getByLabelText("Program")).toHaveAttribute(
      "name",
      "agent-program",
    );
    expect(screen.getByLabelText("Program")).toHaveAttribute(
      "autocomplete",
      "off",
    );
    expect(screen.getByLabelText("Arguments (one per line)")).toHaveAttribute(
      "name",
      "agent-arguments",
    );
    expect(screen.getByLabelText("Arguments (one per line)")).toHaveAttribute(
      "autocomplete",
      "off",
    );
  });
});
