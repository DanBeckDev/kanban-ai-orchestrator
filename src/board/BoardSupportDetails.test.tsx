import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { BoardSupportDetails } from "./BoardSupportDetails";

describe("BoardSupportDetails", () => {
  it("keeps generated identifiers in a collapsed support section", () => {
    render(
      <BoardSupportDetails
        board={{ id: "board-1", projectId: "project-1", name: "MVP" }}
      />,
    );

    expect(
      screen.getByText("Support details").closest("details"),
    ).not.toHaveAttribute("open");
    expect(screen.getByText("Board ID")).toBeInTheDocument();
    expect(screen.getByText("project-1")).toBeInTheDocument();
  });
});
