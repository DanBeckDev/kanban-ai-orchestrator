import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BoardSetup } from "./BoardSetup";

const repository = {
  repositoryPath: "/projects/reliable-app",
  suggestedBoardName: "Reliable app",
  baseRef: "release",
};

describe("BoardSetup", () => {
  it("keeps cancellation local and creates nothing", async () => {
    const onCreate = vi.fn();
    render(
      <BoardSetup
        busy={false}
        onBack={vi.fn()}
        onCreate={onCreate}
        onInspectRepository={vi.fn()}
        repositoryPicker={async () => null}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose repository" }));

    expect(
      await screen.findByText(
        "No repository selected. No board has been created.",
      ),
    ).toBeVisible();
    expect(onCreate).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Create board" })).toBeDisabled();
  });

  it("shows safe defaults, allows advanced overrides, and creates only after confirmation", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(
      <BoardSetup
        busy={false}
        onBack={vi.fn()}
        onCreate={onCreate}
        onInspectRepository={vi.fn().mockResolvedValue(repository)}
        repositoryPicker={async () => repository.repositoryPath}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose repository" }));
    expect(await screen.findByText("Git root")).toBeVisible();
    expect(screen.getByLabelText("Board name")).toHaveValue("Reliable app");
    expect(
      screen.getByText("Base branch: release · Policy: Standard"),
    ).toBeVisible();
    fireEvent.click(screen.getByText("Advanced setup"));
    fireEvent.change(screen.getByLabelText("Base branch"), {
      target: { value: "staging" },
    });
    fireEvent.change(screen.getByLabelText("Policy"), {
      target: { value: "restricted" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create board" }));

    expect(onCreate).toHaveBeenCalledWith({
      name: "Reliable app",
      repositoryPath: "/projects/reliable-app",
      baseRef: "staging",
      policySetId: "restricted",
    });
  });

  it("reports a rejected repository without retaining it", async () => {
    render(
      <BoardSetup
        busy={false}
        onBack={vi.fn()}
        onCreate={vi.fn()}
        onInspectRepository={vi
          .fn()
          .mockRejectedValue(
            new Error("Choose the Git repository root, not a subdirectory."),
          )}
        repositoryPicker={async () => "/projects/reliable-app/client"}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose repository" }));

    expect(
      await screen.findByText(
        "Choose the Git repository root, not a subdirectory.",
      ),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Create board" })).toBeDisabled();
  });
});
