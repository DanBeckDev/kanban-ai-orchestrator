import { describe, expect, it } from "vitest";

import {
  boardLinearMode,
  commentsAreAuthorized,
  productManagedLinearOAuthConfiguration,
} from "./linearConnectionPresentation";
import { snapshot, workItem } from "./BoardWorkspace.test.fixtures";

describe("Linear connection presentation", () => {
  it("only exposes a managed OAuth configuration when the release provides a client ID", () => {
    expect(productManagedLinearOAuthConfiguration(" ")).toBeUndefined();
    expect(
      productManagedLinearOAuthConfiguration(" linear-public-client "),
    ).toEqual({
      clientId: "linear-public-client",
      redirectUri: "http://127.0.0.1:38471/linear/oauth/callback",
    });
  });

  it("derives local-only, read-only, and linked-execution board states from durable links", () => {
    const localBoard = snapshot([workItem("task-1")]);
    expect(boardLinearMode(localBoard).mode).toBe("local_only");

    const readOnlyBoard = {
      ...localBoard,
      externalLinks: [
        {
          id: "linear-link-1",
          workItemId: "task-1",
          connectorId: "linear",
          provenance: "imported" as const,
          externalId: "linear-1",
          displayIdentifier: "LIN-1",
          url: "https://linear.app/example/issue/LIN-1",
          connectionMode: "read_only" as const,
        },
      ],
    };
    expect(boardLinearMode(readOnlyBoard).mode).toBe("read_only");

    expect(
      boardLinearMode({
        ...readOnlyBoard,
        externalLinks: [
          {
            ...readOnlyBoard.externalLinks[0],
            connectionMode: "linked_execution" as const,
          },
        ],
      }).mode,
    ).toBe("linked_execution");
  });

  it("only permits linked execution after comment scope is confirmed", () => {
    expect(
      commentsAreAuthorized({
        kind: "connected",
        expiresAt: "2026-08-10T12:00:00Z",
        scopes: ["read"],
      }),
    ).toBe(false);
    expect(
      commentsAreAuthorized({
        kind: "connected",
        expiresAt: "2026-08-10T12:00:00Z",
        scopes: ["read", "comments:create"],
      }),
    ).toBe(true);
  });
});
