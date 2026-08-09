import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LinearConnectionPanel } from "./LinearConnectionPanel";

describe("LinearConnectionPanel", () => {
  it("submits only the OAuth client configuration and explains the pending browser step", async () => {
    const onConnect = vi.fn().mockResolvedValue(undefined);
    const { rerender } = render(
      <LinearConnectionPanel
        busy={false}
        status={{ kind: "disconnected" }}
        onConnect={onConnect}
      />,
    );

    fireEvent.change(screen.getByLabelText("OAuth client ID"), {
      target: { value: "linear-client-id" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Connect Linear" }));

    await waitFor(() =>
      expect(onConnect).toHaveBeenCalledWith({
        clientId: "linear-client-id",
        redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
      }),
    );
    rerender(
      <LinearConnectionPanel
        busy={false}
        status={{ kind: "awaiting_authorization" }}
        onConnect={onConnect}
      />,
    );
    expect(
      screen.getByText("Waiting for Linear authorization in your browser."),
    ).toBeVisible();
  });

  it("shows the connected access scope without presenting a token input", () => {
    render(
      <LinearConnectionPanel
        busy={false}
        status={{
          kind: "connected",
          expiresAt: "2026-08-09T12:00:00Z",
          scopes: ["read"],
        }}
        onConnect={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.getByText(
        "Connected with read access; token expires 2026-08-09T12:00:00Z.",
      ),
    ).toBeVisible();
    expect(screen.queryByLabelText(/client secret|access token/i)).toBeNull();
  });

  it("offers narrowly scoped comment permission after a read-only connection", () => {
    const onEnableCommentAccess = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearConnectionPanel
        busy={false}
        status={{
          kind: "connected",
          expiresAt: "2026-08-09T12:00:00Z",
          scopes: ["read"],
        }}
        onConnect={vi.fn().mockResolvedValue(undefined)}
        onEnableCommentAccess={onEnableCommentAccess}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Enable manually sent Linear comments",
      }),
    );

    expect(onEnableCommentAccess).toHaveBeenCalledOnce();
  });

  it("shows a connection failure and lets the user try again", () => {
    render(
      <LinearConnectionPanel
        busy={false}
        status={{ kind: "failed", message: "Authorization timed out" }}
        onConnect={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.getByText("Connection failed: Authorization timed out"),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Connect Linear" }),
    ).toBeEnabled();
  });
});
