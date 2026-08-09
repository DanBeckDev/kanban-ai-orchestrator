import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Toggle } from "./toggle";

describe("Toggle", () => {
  it("reports its pressed state with its accessible name", () => {
    render(<Toggle aria-label="Pin board" />);

    const toggle = screen.getByRole("button", { name: "Pin board" });
    fireEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-pressed", "true");
  });
});
