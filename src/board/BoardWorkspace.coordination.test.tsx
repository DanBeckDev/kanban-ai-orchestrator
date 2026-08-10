import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { gateway, snapshot, workItem } from "./BoardWorkspace.test.fixtures";
import { createBoard } from "./BoardWorkspace.test.helpers";

describe("board coordination", () => {
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
      organiser: {
        plannerProfileName: "local organiser",
        model: { kind: "provider_default" },
        effort: "provider_default",
      },
      ticketWorker: {
        agentProfileName: "codex-cli",
        model: { kind: "provider_default" },
        effort: "provider_default",
      },
    });
    await boardGateway.savePlannerProfile({
      name: "local organiser",
      program: "planner",
      arguments: [],
    });

    await createBoard(boardGateway);
    fireEvent.click(screen.getByText("Kanban coordinates"));

    await waitFor(() =>
      expect(boardGateway.configureBoardSupervision).toHaveBeenCalledWith(
        "board-1",
        "autonomous",
      ),
    );
    expect(boardGateway.coordinateBoard).toHaveBeenCalledWith("board-1");
  });
});
