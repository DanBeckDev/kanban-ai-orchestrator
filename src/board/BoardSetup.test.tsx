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
        cloneDestinationPicker={async () => "/projects"}
        onBack={vi.fn()}
        onCloneGitHubRepository={vi.fn()}
        onCreate={onCreate}
        onInspectRepository={vi.fn()}
        repositoryPicker={async () => null}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder" }),
    );

    expect(
      await screen.findByText(
        "No project folder selected. No board has been created.",
      ),
    ).toBeVisible();
    expect(onCreate).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: "Set up workspace" }),
    ).toBeDisabled();
  });

  it("keeps ordinary setup compact and creates with the automatic starting point", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(
      <BoardSetup
        busy={false}
        cloneDestinationPicker={async () => "/projects"}
        onBack={vi.fn()}
        onCloneGitHubRepository={vi.fn()}
        onCreate={onCreate}
        onInspectRepository={vi.fn().mockResolvedValue(repository)}
        repositoryPicker={async () => repository.repositoryPath}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder" }),
    );
    expect(await screen.findByText("Selected project")).toBeVisible();
    expect(screen.getByLabelText("Board name")).toHaveValue("Reliable app");
    expect(
      screen.getByText(
        "Kanban will prepare a separate workspace for each task.",
      ),
    ).toBeVisible();
    expect(screen.queryByText("Policy: Standard")).not.toBeInTheDocument();
    expect(screen.queryByText("Base branch")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Set up workspace" }));

    expect(onCreate).toHaveBeenCalledWith({
      name: "Reliable app",
      repositoryPath: "/projects/reliable-app",
      baseRef: "release",
      policySetId: "standard",
    });
  });

  it("reveals a plain-language override only when someone asks for it", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(
      <BoardSetup
        busy={false}
        cloneDestinationPicker={async () => "/projects"}
        onBack={vi.fn()}
        onCloneGitHubRepository={vi.fn()}
        onCreate={onCreate}
        onInspectRepository={vi.fn().mockResolvedValue(repository)}
        repositoryPicker={async () => repository.repositoryPath}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder" }),
    );
    await screen.findByText("Use a different starting point");
    fireEvent.click(screen.getByText("Use a different starting point"));
    fireEvent.change(screen.getByLabelText("Start new work from"), {
      target: { value: "release/next" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Set up workspace" }));

    expect(onCreate).toHaveBeenCalledWith({
      name: "Reliable app",
      repositoryPath: "/projects/reliable-app",
      baseRef: "release/next",
      policySetId: "standard",
    });
  });

  it("reports a rejected repository without retaining it", async () => {
    render(
      <BoardSetup
        busy={false}
        cloneDestinationPicker={async () => "/projects"}
        onBack={vi.fn()}
        onCloneGitHubRepository={vi.fn()}
        onCreate={vi.fn()}
        onInspectRepository={vi
          .fn()
          .mockRejectedValue(
            new Error("Choose the Git repository root, not a subdirectory."),
          )}
        repositoryPicker={async () => "/projects/reliable-app/client"}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder" }),
    );

    expect(
      await screen.findByText(
        "Choose the top-level folder for your project, not a folder inside it.",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Set up workspace" }),
    ).toBeDisabled();
  });

  it("clones a GitHub repository into the chosen destination before setup", async () => {
    const onCloneGitHubRepository = vi.fn().mockResolvedValue(repository);
    render(
      <BoardSetup
        busy={false}
        cloneDestinationPicker={async () => "/projects"}
        onBack={vi.fn()}
        onCloneGitHubRepository={onCloneGitHubRepository}
        onCreate={vi.fn()}
        onInspectRepository={vi.fn()}
        repositoryPicker={async () => null}
      />,
    );

    fireEvent.click(
      screen.getByRole("radio", { name: "Link a GitHub repository" }),
    );
    fireEvent.change(screen.getByLabelText("GitHub repository URL"), {
      target: { value: "https://github.com/acme/reliable-app" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Choose clone destination" }),
    );
    await screen.findByText(
      "Kanban will create the repository folder in /projects.",
    );
    fireEvent.click(screen.getByRole("button", { name: "Clone repository" }));

    expect(await screen.findByText("Selected project")).toBeVisible();
    expect(onCloneGitHubRepository).toHaveBeenCalledWith({
      repositoryUrl: "https://github.com/acme/reliable-app",
      destinationParentPath: "/projects",
    });
    expect(screen.getByLabelText("Board name")).toHaveValue("Reliable app");
  });

  it("keeps a cancelled clone destination local", async () => {
    const onCloneGitHubRepository = vi.fn();
    render(
      <BoardSetup
        busy={false}
        cloneDestinationPicker={async () => null}
        onBack={vi.fn()}
        onCloneGitHubRepository={onCloneGitHubRepository}
        onCreate={vi.fn()}
        onInspectRepository={vi.fn()}
        repositoryPicker={async () => null}
      />,
    );

    fireEvent.click(
      screen.getByRole("radio", { name: "Link a GitHub repository" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Choose clone destination" }),
    );

    expect(
      await screen.findByText(
        "No clone destination selected. No repository has been cloned.",
      ),
    ).toBeVisible();
    expect(onCloneGitHubRepository).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: "Clone repository" }),
    ).toBeDisabled();
  });
});
