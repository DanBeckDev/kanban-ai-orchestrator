import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("describes the local board core and its execution authority", () => {
    render(<App />);

    expect(
      screen.getByRole("heading", { name: "Kanban AI Orchestrator" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Rust local core")).toBeInTheDocument();
    expect(screen.getByText("Local board core")).toBeInTheDocument();
  });
});
