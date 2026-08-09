import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ExternalLinks } from "./ExternalLinks";

describe("ExternalLinks", () => {
  it("renders a Linear issue identifier and connection mode", () => {
    render(
      <ExternalLinks
        links={[
          {
            id: "link-1",
            workItemId: "task-1",
            connectorId: "linear",
            provenance: "imported",
            externalId: "immutable-uuid",
            displayIdentifier: "LIN-12",
            url: "https://linear.app/example/issue/LIN-12",
            connectionMode: "linked_execution",
          },
        ]}
      />,
    );

    expect(screen.getByRole("link", { name: "LIN-12" })).toHaveAttribute(
      "href",
      "https://linear.app/example/issue/LIN-12",
    );
    expect(screen.getByText("linked execution")).toBeVisible();
  });
});
