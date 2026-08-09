import { fireEvent, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard } from "./BoardWorkspace.test.helpers";

describe("board workflow view", () => {
  it("uses the top-left menu to move between board views", async () => {
    const boardGateway = gateway(snapshot([workItem("foundation", "planned")]));
    await createBoard(boardGateway);

    fireEvent.pointerDown(screen.getByRole("button", { name: "Workflow" }), {
      button: 0,
      ctrlKey: false,
    });
    fireEvent.click(
      screen.getByRole("menuitemradio", { name: "Dependencies" }),
    );
    expect(
      await screen.findByRole("heading", { name: "Dependencies" }),
    ).toBeVisible();

    fireEvent.pointerDown(
      screen.getByRole("button", { name: "Dependencies" }),
      { button: 0, ctrlKey: false },
    );
    fireEvent.click(screen.getByRole("menuitemradio", { name: "Workflow" }));
    expect(
      await screen.findByRole("heading", { name: "Prompt AI to orchestrate" }),
    ).toBeVisible();
  });
});
