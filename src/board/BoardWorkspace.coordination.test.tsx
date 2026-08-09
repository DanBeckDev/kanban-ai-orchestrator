import { fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard } from "./BoardWorkspace.test.helpers";

describe("board coordination", () => {
  afterEach(() => {
    window.localStorage.clear();
  });

  it("starts bounded coordination only after the user enables it", async () => {
    const boardGateway = gateway(snapshot([workItem("foundation", "inbox")]));
    await boardGateway.saveAgentProfile({
      name: "codex-cli",
      kind: "codex_cli",
      program: "codex",
      arguments: [],
    });
    await boardGateway.saveProjectAgentSettings({
      boardId: "board-1",
      ticketWorker: {
        agentProfileName: "codex-cli",
        model: { kind: "provider_default" },
        effort: "provider_default",
      },
    });

    await createBoard(boardGateway);
    fireEvent.click(screen.getByText("Kanban coordinates"));

    await waitFor(() =>
      expect(boardGateway.coordinateBoard).toHaveBeenCalledWith(
        "board-1",
        "codex-cli",
      ),
    );
  });
});
