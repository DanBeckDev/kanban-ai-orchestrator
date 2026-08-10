import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";

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
  fireEvent.click(screen.getByRole("button", { name: "Plan work with AI" }));
}

export function openNewTask() {
  fireEvent.click(screen.getByRole("button", { name: "Create task" }));
}

export function openDependencies() {
  selectBoardView("Dependencies");
}

export function openSettings(section?: "AI" | "Linear" | "Project") {
  selectBoardView("Settings");
  if (section !== undefined) {
    selectTab(section);
  }
}

export function openTask(title: string) {
  const action = ["Open task", "Inspect", "Review", "Recover", "Unblock"]
    .map((label) => screen.queryByRole("button", { name: `${label} ${title}` }))
    .find((button) => button !== null);
  if (action === undefined) {
    throw new Error(`No task action is available for ${title}.`);
  }
  fireEvent.click(action);
}

export async function configurePlanner() {
  openSettings("AI");
  const profileForm = screen.getByRole("form", {
    name: "Save orchestrator connection",
  });
  fireEvent.change(within(profileForm).getByLabelText("Connection name"), {
    target: { value: "local planner" },
  });
  fireEvent.click(
    within(profileForm).getByRole("button", {
      name: "Save orchestrator connection",
    }),
  );
  await waitFor(() => {
    if (screen.queryAllByText("local planner").length === 0) {
      throw new Error("The orchestrator connection has not been saved yet.");
    }
  });
  fireEvent.click(screen.getByRole("button", { name: "Back to board" }));
}

export async function selectOption(label: string, name: string | RegExp) {
  fireEvent.pointerDown(screen.getByLabelText(label), {
    button: 0,
    ctrlKey: false,
    pointerType: "mouse",
  });
  fireEvent.click(await screen.findByRole("option", { name }));
}

function selectBoardView(name: "Dependencies" | "Settings") {
  fireEvent.pointerDown(screen.getByRole("button", { name: "Workflow" }), {
    button: 0,
    ctrlKey: false,
  });
  fireEvent.click(screen.getByRole("menuitemradio", { name }));
}

function selectTab(name: string) {
  // Radix Tabs selects on a primary pointer press, rather than a synthetic click.
  fireEvent.mouseDown(screen.getByRole("tab", { name }), { button: 0 });
}
