import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { gateway, snapshot } from "./BoardWorkspace.test.fixtures";
import { useLinearBoardConnection } from "./useLinearBoardConnection";

function deferred<Value>() {
  let resolve: (value: Value) => void = () => undefined;
  const promise = new Promise<Value>((nextResolve) => {
    resolve = nextResolve;
  });
  return { promise, resolve };
}

describe("useLinearBoardConnection", () => {
  it("keeps a newer connection status when an older refresh resolves later", async () => {
    const boardGateway = gateway(snapshot());
    const firstRefresh = deferred<{
      kind: "disconnected";
    }>();
    const secondRefresh = deferred<{
      kind: "connected";
      expiresAt: string;
      scopes: readonly string[];
    }>();
    boardGateway.linearConnectionStatus = vi
      .fn()
      .mockReturnValueOnce(firstRefresh.promise)
      .mockReturnValueOnce(secondRefresh.promise);
    const { result } = renderHook(() =>
      useLinearBoardConnection({
        clearError: vi.fn(),
        gateway: boardGateway,
        run: vi.fn().mockResolvedValue(undefined),
        setBusy: vi.fn(),
      }),
    );

    void result.current.refreshConnectionStatus();
    void result.current.refreshConnectionStatus();

    await act(async () => {
      secondRefresh.resolve({
        kind: "connected",
        expiresAt: "2026-08-10T12:00:00Z",
        scopes: ["read"],
      });
      await secondRefresh.promise;
    });
    expect(result.current.connectionStatus.kind).toBe("connected");

    await act(async () => {
      firstRefresh.resolve({ kind: "disconnected" });
      await firstRefresh.promise;
    });
    expect(result.current.connectionStatus.kind).toBe("connected");
  });

  it("keeps the newest issue list when requests finish out of order", async () => {
    const boardGateway = gateway(snapshot());
    const firstLoad =
      deferred<
        readonly {
          id: string;
          identifier: string;
          title: string;
          url: string;
        }[]
      >();
    const secondLoad =
      deferred<
        readonly {
          id: string;
          identifier: string;
          title: string;
          url: string;
        }[]
      >();
    boardGateway.linearAssignedIssues = vi
      .fn()
      .mockReturnValueOnce(firstLoad.promise)
      .mockReturnValueOnce(secondLoad.promise);
    const { result } = renderHook(() =>
      useLinearBoardConnection({
        clearError: vi.fn(),
        gateway: boardGateway,
        run: vi.fn().mockResolvedValue(undefined),
        setBusy: vi.fn(),
      }),
    );

    void result.current.loadAssignedIssues();
    void result.current.loadAssignedIssues();

    await act(async () => {
      secondLoad.resolve([
        {
          id: "linear-new",
          identifier: "LIN-2",
          title: "Newest issue",
          url: "https://linear.app/example/issue/LIN-2",
        },
      ]);
      await secondLoad.promise;
    });
    expect(result.current.issues).toHaveLength(1);
    expect(result.current.issues[0]?.id).toBe("linear-new");

    await act(async () => {
      firstLoad.resolve([
        {
          id: "linear-old",
          identifier: "LIN-1",
          title: "Older issue",
          url: "https://linear.app/example/issue/LIN-1",
        },
      ]);
      await firstLoad.promise;
    });
    expect(result.current.issues[0]?.id).toBe("linear-new");
  });
});
