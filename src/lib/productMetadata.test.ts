import { describe, expect, it } from "vitest";
import { foundationMessage, productMetadata } from "./productMetadata";

describe("foundationMessage", () => {
  it("uses the supplied product name", () => {
    expect(foundationMessage(productMetadata)).toBe(
      "Kanban AI Orchestrator is ready for its durable local core.",
    );
  });
});
