import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "../App";
import { gateway, snapshot } from "./BoardWorkspace.test.fixtures";
import type { BoardGateway } from "./types";

async function openBoard(boardGateway: BoardGateway) {
  render(<App gateway={boardGateway} />);
  fireEvent.change(screen.getByLabelText("Existing board ID"), {
    target: { value: "board-1" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Open board" }));
  await screen.findByRole("heading", { name: "MVP" });
}

describe("Linear connection in the board workspace", () => {
  it("starts OAuth after opening a board and shows its awaiting status", async () => {
    const boardGateway = gateway(snapshot());

    await openBoard(boardGateway);
    fireEvent.change(screen.getByLabelText("OAuth client ID"), {
      target: { value: "linear-client-id" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Connect Linear" }));

    await waitFor(() =>
      expect(boardGateway.beginLinearOAuth).toHaveBeenCalledWith({
        clientId: "linear-client-id",
        redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
      }),
    );
    expect(
      screen.getByText("Waiting for Linear authorization in your browser."),
    ).toBeVisible();
  });

  it("keeps board access available when reading a saved connection fails", async () => {
    const boardGateway = gateway(snapshot());
    boardGateway.linearConnectionStatus = vi
      .fn()
      .mockRejectedValue(new Error("Keychain is unavailable"));

    await openBoard(boardGateway);

    expect(
      screen.getByText("Connection failed: Keychain is unavailable"),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: "MVP" })).toBeVisible();
  });

  it("loads assigned Linear issues only after a connected account is confirmed", async () => {
    const boardGateway = gateway(snapshot());
    boardGateway.linearConnectionStatus = vi.fn().mockResolvedValue({
      kind: "connected",
      expiresAt: "2026-08-09T12:00:00Z",
      scopes: ["read"],
    });
    boardGateway.linearAssignedIssues = vi.fn().mockResolvedValue([
      {
        id: "d290f1ee-6c54-4b01-90e6-d701748f0851",
        identifier: "LIN-12",
        title: "Load the issue",
        url: "https://linear.app/example/issue/LIN-12",
      },
    ]);

    await openBoard(boardGateway);
    fireEvent.click(
      screen.getByRole("button", { name: "Load my assigned Linear issues" }),
    );

    await waitFor(() =>
      expect(boardGateway.linearAssignedIssues).toHaveBeenCalledOnce(),
    );
    expect(
      screen.getByRole("button", { name: "Use LIN-12: Load the issue" }),
    ).toBeVisible();
  });
});
