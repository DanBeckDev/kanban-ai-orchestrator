import { fireEvent, render, screen } from "@testing-library/react";

import { App } from "../App";
import type { BoardGateway } from "./types";

export async function createBoard(boardGateway: BoardGateway) {
  render(
    <App
      gateway={boardGateway}
      repositoryPicker={async () => "/projects/project"}
    />,
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Create a board" }),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Choose project folder" }),
  );
  await screen.findByText("Selected project");
  fireEvent.change(screen.getByLabelText("Board name"), {
    target: { value: "MVP" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Set up workspace" }));
  await screen.findByRole("heading", { name: "MVP" });
}

export function openPlan() {
  fireEvent.click(screen.getByRole("button", { name: "Plan with AI" }));
}

export function openNewTask() {
  fireEvent.click(screen.getByRole("button", { name: "New task" }));
}

export function openDependencies() {
  openNewTask();
  selectTab("Dependencies");
}

export function openSettings(
  section?: "Agent" | "Organiser" | "Linear" | "Project",
) {
  fireEvent.click(screen.getByRole("button", { name: "Settings" }));
  if (section !== undefined) {
    selectTab(section);
  }
}

export function openTask(title: string) {
  fireEvent.click(screen.getByRole("button", { name: `Open task ${title}` }));
}

function selectTab(name: string) {
  // Radix Tabs selects on a primary pointer press, rather than a synthetic click.
  fireEvent.mouseDown(screen.getByRole("tab", { name }), { button: 0 });
}
