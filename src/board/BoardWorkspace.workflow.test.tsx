import { fireEvent, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard } from "./BoardWorkspace.test.helpers";

describe("board workflow view", () => {
  it("uses the top-left menu to keep Home and Tickets separate", async () => {
    const boardGateway = gateway(snapshot([workItem("foundation", "planned")]));
    await createBoard(boardGateway);

    fireEvent.pointerDown(screen.getByRole("button", { name: "Home" }), {
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
      {
        button: 0,
        ctrlKey: false,
      },
    );
    fireEvent.click(screen.getByRole("menuitemradio", { name: "Tickets" }));
    expect(
      await screen.findByRole("heading", { name: "Keep work moving" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("heading", { name: "Prompt AI to orchestrate" }),
    ).not.toBeInTheDocument();
  });
});
