import { describe, expect, it } from "vitest";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import {
  changedPathsFor,
  isCodeBearingPath,
  parseOptions,
  projectRootFor,
  runReceiptVerification,
  validateReceiptContent,
  validateSourceStructureEvidence,
  verifyQualityReceipts,
} from "./verify-quality-receipt.mjs";

const validReceipt = `work_item: QUAL-002
review:
  skill: clean-code-review
  reviewer: Codex
  unresolved_actionable_findings: 0
remediation: []
verification:
  commands:
    - command: npm run quality:verify
      result: passed
  coverage:
    threshold_met: true
`;

const sourceStructureReceipt = `${validReceipt}source_structure:
  reviewed_files:
    - path: src/App.tsx
      meaningful_lines: 1
      responsibilities: Renders the desktop application shell.
      decision: kept cohesive
`;

const oneMeaningfulLine = "export const app = true;\n";

describe("quality-review receipt gate", () => {
  it("resolves its root in Node and virtual test-module environments", () => {
    const workspace = resolve("workspace");
    expect(
      projectRootFor(
        pathToFileURL(
          resolve(workspace, "scripts", "verify-quality-receipt.mjs"),
        ).href,
        "ignored",
      ),
    ).toBe(workspace);
    expect(projectRootFor("vite://virtual-module", workspace)).toBe(workspace);
  });

  it("recognizes code-bearing paths", () => {
    expect(isCodeBearingPath("src/App.tsx")).toBe(true);
    expect(isCodeBearingPath("src-tauri/src/lib.rs")).toBe(true);
    expect(isCodeBearingPath("src-tauri/capabilities/default.json")).toBe(true);
    expect(isCodeBearingPath(".github/workflows/quality.yml")).toBe(true);
    expect(isCodeBearingPath("index.html")).toBe(true);
    expect(isCodeBearingPath("docs/product/vision.md")).toBe(false);
  });

  it("requires complete review evidence", () => {
    expect(validateReceiptContent(validReceipt)).toEqual([]);
    expect(validateReceiptContent("work_item: QUAL-002")).toContain(
      "zero unresolved actionable findings",
    );
  });

  it("requires a changed valid receipt for code-bearing work", () => {
    expect(() =>
      verifyQualityReceipts({
        changedPaths: ["src/App.tsx"],
        mode: "staged",
        readFile: () => validReceipt,
      }),
    ).toThrow("requires a changed quality-review receipt");

    expect(
      verifyQualityReceipts({
        changedPaths: ["src/App.tsx", "docs/quality/reviews/QUAL-002.yaml"],
        mode: "staged",
        readFile: () => sourceStructureReceipt,
        readSourceFile: () => oneMeaningfulLine,
      }),
    ).toBe("Validated 1 quality-review receipt(s).");

    expect(() =>
      verifyQualityReceipts({
        changedPaths: ["src/App.tsx", "docs/quality/reviews/QUAL-002.yaml"],
        mode: "staged",
        readFile: () => "work_item: QUAL-002",
        readSourceFile: () => oneMeaningfulLine,
      }),
    ).toThrow("is missing the Clean Code review skill");
  });

  it("does not require a receipt for documentation-only work", () => {
    expect(
      verifyQualityReceipts({
        changedPaths: ["docs/product/vision.md"],
        mode: "staged",
        readFile: () => validReceipt,
      }),
    ).toBe("No code-bearing changes; no quality-review receipt is required.");
  });

  it("requires the receipt to prove every changed source file was inspected", () => {
    expect(
      validateSourceStructureEvidence({
        receiptContents: [validReceipt],
        readSourceFile: () => oneMeaningfulLine,
        sourcePaths: ["src/App.tsx"],
      }),
    ).toEqual([
      "Code-bearing work requires a source_structure.reviewed_files inventory in its quality-review receipt.",
      "src/App.tsx must appear in source_structure.reviewed_files with its actual 1 meaningful lines, responsibilities, and decision.",
    ]);

    expect(
      validateSourceStructureEvidence({
        receiptContents: [sourceStructureReceipt],
        readSourceFile: () => oneMeaningfulLine,
        sourcePaths: ["src/App.tsx"],
      }),
    ).toEqual([]);

    expect(
      validateSourceStructureEvidence({
        receiptContents: [sourceStructureReceipt],
        readSourceFile: () => `${oneMeaningfulLine}${oneMeaningfulLine}`,
        sourcePaths: ["src/App.tsx"],
      }),
    ).toEqual([
      "src/App.tsx must appear in source_structure.reviewed_files with its actual 2 meaningful lines, responsibilities, and decision.",
    ]);
  });

  it("validates the recorded receipts outside a pull request", () => {
    expect(
      verifyQualityReceipts({
        changedPaths: [],
        mode: "all",
      }),
    ).toMatch(/^Validated \d+ quality-review receipt\(s\)\.$/);
  });

  it("accepts both supported receipt extensions when reading all receipts", () => {
    expect(
      verifyQualityReceipts({
        changedPaths: [],
        mode: "all",
        readDirectory: () => ["ignored.txt", "LEGACY.yml"],
        readFile: () => validReceipt,
      }),
    ).toBe("Validated 1 quality-review receipt(s).");
  });

  it("parses supported change scopes and queries Git correctly", () => {
    expect(parseOptions(["--staged"])).toEqual({ mode: "staged" });
    expect(parseOptions(["--all"])).toEqual({ mode: "all" });
    expect(parseOptions(["--base-ref", "origin/main"])).toEqual({
      baseRef: "origin/main",
      mode: "base-ref",
    });
    expect(() => parseOptions([])).toThrow("Usage:");

    const calls = [];
    expect(
      changedPathsFor(
        { mode: "base-ref", baseRef: "origin/main" },
        (command, args) => {
          calls.push({ args, command });
          return "src/App.tsx\ndocs/quality/reviews/QUAL-002.yaml\n";
        },
      ),
    ).toEqual(["src/App.tsx", "docs/quality/reviews/QUAL-002.yaml"]);
    expect(calls).toEqual([
      {
        args: [
          "diff",
          "--name-only",
          "--diff-filter=ACMR",
          "origin/main...HEAD",
        ],
        command: "git",
      },
    ]);

    expect(changedPathsFor({ mode: "all" }, () => "unexpected")).toEqual([]);
    expect(
      changedPathsFor({ mode: "staged" }, (command, args) => {
        expect(command).toBe("git");
        expect(args).toEqual([
          "diff",
          "--cached",
          "--name-only",
          "--diff-filter=ACMR",
        ]);
        return "src/App.tsx\n";
      }),
    ).toEqual(["src/App.tsx"]);
  });

  it("runs receipt verification with injectable process boundaries", () => {
    const logs = [];
    const options = [];

    runReceiptVerification({
      args: ["--staged"],
      findChangedPaths(option) {
        options.push(option);
        return ["src/App.tsx"];
      },
      log(message) {
        logs.push(message);
      },
      verifyReceipts(input) {
        expect(input).toEqual({
          changedPaths: ["src/App.tsx"],
          mode: "staged",
        });
        return "Validated 1 quality-review receipt(s).";
      },
    });

    expect(options).toEqual([{ mode: "staged" }]);
    expect(logs).toEqual(["Validated 1 quality-review receipt(s)."]);
  });
});
