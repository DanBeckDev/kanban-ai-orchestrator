import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LinearConnectionPanel } from "./LinearConnectionPanel";

describe("LinearConnectionPanel", () => {
  it("keeps unavailable managed OAuth truthful and hides self-managed setup until requested", () => {
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={undefined}
        status={{ kind: "disconnected" }}
        onConnect={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.getByText(
        "Managed Linear connection is not available in this build",
      ),
    ).toBeVisible();
    expect(screen.queryByLabelText("OAuth client ID")).toBeNull();
    expect(
      screen.getByText(
        "No Linear account is connected. Existing local links are unchanged; connect Linear to load issues or choose linked execution.",
      ),
    ).toBeVisible();
  });

  it("connects with a product-managed client in one action", async () => {
    const onConnect = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={{
          clientId: "linear-product-client",
          redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
        }}
        status={{ kind: "disconnected" }}
        onConnect={onConnect}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Connect Linear" }));

    await waitFor(() =>
      expect(onConnect).toHaveBeenCalledWith({
        clientId: "linear-product-client",
        redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
      }),
    );
    expect(screen.queryByLabelText("OAuth client ID")).toBeNull();
  });

  it("submits self-managed setup only after the user opens advanced setup", async () => {
    const onConnect = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={undefined}
        status={{ kind: "disconnected" }}
        onConnect={onConnect}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Use a self-managed Linear app" }),
    );
    fireEvent.change(screen.getByLabelText("OAuth client ID"), {
      target: { value: "linear-client-id" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Connect self-managed app" }),
    );

    await waitFor(() =>
      expect(onConnect).toHaveBeenCalledWith({
        clientId: "linear-client-id",
        redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
      }),
    );
    expect(screen.getByLabelText("Callback URL")).toHaveValue(
      "http://127.0.0.1:38471/linear/oauth/callback",
    );
    expect(
      screen.getByText(
        /requests read access first, refreshes it only for an action/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/revoke access in Linear at any time/),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Linear OAuth setup guide" }),
    ).toHaveAttribute(
      "href",
      "https://linear.app/developers/oauth-2-0-authentication",
    );
  });

  it("explains the connected read-only next action without presenting a token input", () => {
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={undefined}
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
        "Linear is connected in read-only mode. You can load issues; Kanban will not send updates.",
      ),
    ).toBeVisible();
    expect(screen.queryByLabelText(/client secret|access token/i)).toBeNull();
    expect(
      screen.getByText(/Your existing connection remains available/),
    ).toBeVisible();
  });

  it("offers narrowly scoped comment permission after a read-only connection", () => {
    const onEnableCommentAccess = vi.fn().mockResolvedValue(undefined);
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={undefined}
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

  it("explains how to recover from a connection failure", () => {
    render(
      <LinearConnectionPanel
        busy={false}
        productManagedConfiguration={undefined}
        status={{ kind: "failed", message: "Authorization timed out" }}
        onConnect={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.getByText(
        "Kanban could not connect Linear. Reopen setup, check the app details, then connect again.",
      ),
    ).toBeVisible();
  });
});
