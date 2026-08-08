import { describe, expect, it } from "vitest";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import {
  projectRootFor,
  printCoverageResult,
  runRustCoverage,
  validateCoverageTotals,
} from "./check-rust-coverage.mjs";

const completeTotals = {
  branches: { count: 0, covered: 0 },
  functions: { count: 2, covered: 2 },
  lines: { count: 4, covered: 4 },
  regions: { count: 4, covered: 4 },
};

describe("Rust coverage gate", () => {
  it("resolves its root in Node and virtual test-module environments", () => {
    const workspace = resolve("workspace");
    expect(
      projectRootFor(
        pathToFileURL(resolve(workspace, "scripts", "check-rust-coverage.mjs"))
          .href,
        "ignored",
      ),
    ).toBe(workspace);
    expect(projectRootFor("vite://virtual-module", workspace)).toBe(workspace);
  });

  it("accepts complete coverage and treats no executable branches as complete", () => {
    expect(validateCoverageTotals(completeTotals)).toEqual([
      "Rust branches: 100.00% (no executable items)",
      "Rust functions: 100.00%",
      "Rust lines: 100.00%",
      "Rust statements: 100.00%",
    ]);
  });

  it("rejects unavailable or insufficient coverage", () => {
    expect(() => validateCoverageTotals(undefined)).toThrow(
      "does not contain totals",
    );
    expect(() =>
      validateCoverageTotals({
        ...completeTotals,
        lines: { count: 4, covered: 3 },
      }),
    ).toThrow("Rust lines coverage is 75.00%; 80% is required.");
    expect(() =>
      validateCoverageTotals({
        ...completeTotals,
        functions: { count: -1, covered: 0 },
      }),
    ).toThrow("Rust functions coverage is unavailable.");
  });

  it("runs cargo coverage before validating its JSON report", () => {
    const calls = [];
    const report = JSON.stringify({ data: [{ totals: completeTotals }] });

    const messages = runRustCoverage({
      execute(command, args, options) {
        calls.push({ args, command, options });
      },
      makeDirectory(path, options) {
        calls.push({ options, path });
      },
      outputPath: "/tmp/rust-coverage.json",
      readFile(path, encoding) {
        calls.push({ encoding, path });
        return report;
      },
    });

    expect(messages).toHaveLength(4);
    expect(calls).toContainEqual({
      args: [
        "+nightly",
        "llvm-cov",
        "--manifest-path",
        "src-tauri/Cargo.toml",
        "--branch",
        "--json",
        "--output-path",
        "/tmp/rust-coverage.json",
      ],
      command: "cargo",
      options: expect.objectContaining({ stdio: "inherit" }),
    });
  });

  it("prints each result returned by the coverage runner", () => {
    const messages = [];

    printCoverageResult({
      log(message) {
        messages.push(message);
      },
      runCoverage() {
        return ["Rust lines: 100.00%"];
      },
    });

    expect(messages).toEqual(["Rust lines: 100.00%"]);
  });
});
