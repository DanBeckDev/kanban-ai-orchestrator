import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const boardStyles = readFileSync("src/styles/board.css", "utf8").replace(
  /\r\n/g,
  "\n",
);

describe("task detail styles", () => {
  it("uses a decision grid on wider screens and one column on narrow screens", () => {
    expect(boardStyles).toContain(
      ".task-decision-content > dl,\n.task-dependency-list dl {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));",
    );
    expect(boardStyles).toContain("@media (max-width: 48rem) {");
    expect(boardStyles).toContain(
      ".task-decision-content > dl,\n  .task-dependency-list dl {\n    grid-template-columns: 1fr;",
    );
  });
});
