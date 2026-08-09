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
  fireEvent.click(screen.getByRole("button", { name: "Create board" }));
  await screen.findByRole("heading", { name: "MVP" });
}

export function selectBoardControlTab(name: string) {
  fireEvent.mouseDown(screen.getByRole("tab", { name }), {
    button: 0,
    ctrlKey: false,
  });
}
