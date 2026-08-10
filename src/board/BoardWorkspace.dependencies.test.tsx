import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard, openDependencies } from "./BoardWorkspace.test.helpers";

describe("dependency view", () => {
  it("offers graph and list navigation with a plain-language blocker explanation", async () => {
    const boardGateway = gateway(dependencySnapshot());
    await createBoard(boardGateway);
    openDependencies();

    expect(
      await screen.findByRole("heading", { name: "Visual dependency map" }),
    ).toBeVisible();
    const listCard = screen
      .getByRole("heading", { name: "Dependency list" })
      .closest('[data-slot="card"]');
    if (listCard === null) throw new Error("Dependency list card is missing.");
    const listTask = within(listCard).getByRole("button", {
      name: /Select Task ui/,
    });
    listTask.focus();
    fireEvent.click(listTask);

    expect(listTask).toHaveFocus();
    expect(screen.getByRole("heading", { name: "Task ui" })).toBeVisible();
    expect(screen.getByText("Waiting on prerequisite work")).toBeVisible();
    expect(
      screen.getByText("UI needs the API before it can render."),
    ).toBeVisible();
    expect(screen.getByText("platform")).toBeVisible();
    expect(screen.getByText("Complete API first.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Back to Tickets" }));
    await screen.findByRole("heading", { name: "Keep work moving" });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Tickets" })).toHaveFocus(),
    );
  });

  it("selects the same task from the visual map", async () => {
    const boardGateway = gateway(dependencySnapshot());
    await createBoard(boardGateway);
    openDependencies();

    fireEvent.click(
      await screen.findByRole("button", { name: "Select Task api, planned" }),
    );

    expect(screen.getByRole("heading", { name: "Task api" })).toBeVisible();
    expect(screen.getByText("Work affected next")).toBeVisible();
    expect(
      screen.getByText("This task must finish before the work below."),
    ).toBeVisible();
  });
});

function dependencySnapshot() {
  const current = snapshot([
    workItem("api", "planned"),
    workItem("ui", "planned"),
  ]);
  return {
    ...current,
    dependencies: [
      {
        id: "api-blocks-ui",
        upstreamWorkItemId: "api",
        downstreamWorkItemId: "ui",
        kind: "blocks" as const,
        reason: "UI needs the API before it can render.",
        owner: "platform",
        nextAction: "Complete API first.",
      },
    ],
  };
}
