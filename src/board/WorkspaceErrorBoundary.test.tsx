import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { WorkspaceErrorBoundary } from "./WorkspaceErrorBoundary";

function BrokenWorkspace() {
  throw new Error("rendering failed");
}

describe("WorkspaceErrorBoundary", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("replaces a failed workspace with a meaningful retry state", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(
      <WorkspaceErrorBoundary>
        <BrokenWorkspace />
      </WorkspaceErrorBoundary>,
    );

    expect(
      screen.getByRole("heading", {
        name: "Kanban couldn't show this workspace",
      }),
    ).toBeVisible();
    expect(
      screen.getByText("Your saved boards and work have not been changed."),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Reload workspace" }));
    expect(screen.getByRole("alert")).toBeVisible();
  });
});
