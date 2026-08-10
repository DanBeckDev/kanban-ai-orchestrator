import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const appShellStyles = readFileSync("src/styles/app-shell.css", "utf8");
const formStyles = readFileSync("src/styles/forms.css", "utf8");

describe("interface-guideline styles", () => {
  it("keeps focus treatment keyboard-specific and honours reduced motion", () => {
    expect(formStyles).toContain(":focus-visible");
    expect(formStyles).not.toMatch(/:focus(?!-visible)/);
    expect(appShellStyles).toContain("text-wrap: balance");
    expect(appShellStyles).toContain("@media (prefers-reduced-motion: reduce)");
    expect(appShellStyles).toContain("transition-duration: 0.01ms !important");
  });
});
