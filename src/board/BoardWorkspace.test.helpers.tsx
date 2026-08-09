import { fireEvent, render, screen } from "@testing-library/react";

import { App } from "../App";
import type { BoardGateway } from "./types";

export async function createBoard(boardGateway: BoardGateway) {
  render(<App gateway={boardGateway} />);
  fireEvent.change(screen.getByLabelText("Project ID"), {
    target: { value: "project-1" },
  });
  fireEvent.change(screen.getByLabelText("Project name"), {
    target: { value: "Project" },
  });
  fireEvent.change(screen.getByLabelText("Repository path"), {
    target: { value: "/projects/project" },
  });
  fireEvent.change(screen.getByLabelText("New board ID"), {
    target: { value: "board-1" },
  });
  fireEvent.change(screen.getByLabelText("New board name"), {
    target: { value: "MVP" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create local board" }));
  await screen.findByRole("heading", { name: "MVP" });
}
