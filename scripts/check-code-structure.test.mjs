import { describe, expect, it } from "vitest";

import {
  MAX_PRODUCTION_SOURCE_LINES,
  MAX_TEST_SOURCE_LINES,
  baselineExceptionsFor,
  changedPathsFor,
  loadExceptions,
  meaningfulLineCount,
  parseOptions,
  projectRootFor,
  runStructureCheck,
  sourceFileLimit,
  validateExceptionLedger,
  validateSourceStructure,
} from "./check-code-structure.mjs";

describe("source structure gate", () => {
  it("resolves its root in Node and virtual test-module environments", () => {
    expect(
      projectRootFor(
        "file:///workspace/scripts/check-code-structure.mjs",
        "/ignored",
      ),
    ).toBe("/workspace");
    expect(projectRootFor("vite://virtual-module", "/workspace")).toBe(
      "/workspace",
    );
  });

  it("classifies production, test, and non-source paths", () => {
    expect(sourceFileLimit("src-tauri/src/orchestration/plan.rs")).toBe(
      MAX_PRODUCTION_SOURCE_LINES,
    );
    expect(
      sourceFileLimit("src-tauri/src/orchestration/tests/plan_tests.rs"),
    ).toBe(MAX_TEST_SOURCE_LINES);
    expect(sourceFileLimit("scripts/check-code-structure.test.mjs")).toBe(
      MAX_TEST_SOURCE_LINES,
    );
    expect(sourceFileLimit("docs/quality/code-structure.md")).toBeUndefined();
  });

  it("counts code while ignoring blank lines and line comments", () => {
    expect(
      meaningfulLineCount(
        "\n// a comment\nconst value = 1;\n# shell comment\n",
      ),
    ).toBe(1);
  });

  it("rejects an oversized changed file without an active exception", () => {
    const path = "src-tauri/src/new_module.rs";
    const source = `${"let value = 1;\n".repeat(MAX_PRODUCTION_SOURCE_LINES + 1)}`;

    expect(
      validateSourceStructure({
        paths: [path],
        readFile: () => source,
      }),
    ).toEqual([
      `${path} has ${MAX_PRODUCTION_SOURCE_LINES + 1} meaningful lines; the limit is ${MAX_PRODUCTION_SOURCE_LINES}. Split independent responsibilities into cohesive modules before merging.`,
    ]);
  });

  it("does not read paths outside the source roots", () => {
    expect(
      validateSourceStructure({
        paths: ["docs/quality/code-structure.md"],
        readFile() {
          throw new Error("non-source paths must not be read");
        },
      }),
    ).toEqual([]);
  });

  it("ignores a source file that the current change deletes", () => {
    expect(
      validateSourceStructure({
        paths: ["src-tauri/src/removed.rs"],
        readFile() {
          const error = new Error("missing file");
          error.code = "ENOENT";
          throw error;
        },
      }),
    ).toEqual([]);
  });

  it("permits a temporary, owned exception only until its expiry and ceiling", () => {
    const path = "src-tauri/src/legacy.rs";
    const source = `${"let value = 1;\n".repeat(MAX_PRODUCTION_SOURCE_LINES + 1)}`;
    const exceptions = [
      {
        path,
        work_item: "QUAL-004",
        expires_on: "2026-08-22",
        maximum_meaningful_lines: MAX_PRODUCTION_SOURCE_LINES + 1,
      },
    ];

    expect(
      validateSourceStructure({
        paths: [path],
        readFile: () => source,
        exceptions,
        currentDate: "2026-08-08",
      }),
    ).toEqual([]);
    expect(
      validateSourceStructure({
        paths: [path],
        readFile: () =>
          `${"let value = 1;\n".repeat(MAX_PRODUCTION_SOURCE_LINES + 2)}`,
        exceptions,
        currentDate: "2026-08-08",
      }),
    ).toEqual([
      `${path} has ${MAX_PRODUCTION_SOURCE_LINES + 2} meaningful lines; the temporary exception ceiling is ${MAX_PRODUCTION_SOURCE_LINES + 1}. Split independent responsibilities into cohesive modules before merging.`,
    ]);
    expect(
      validateSourceStructure({
        paths: [path],
        readFile: () => source,
        exceptions,
        currentDate: "2026-08-23",
      }),
    ).toHaveLength(1);
  });

  it("parses supported change scopes and queries Git correctly", () => {
    expect(parseOptions([])).toEqual({ mode: "working-tree" });
    expect(parseOptions(["--staged"])).toEqual({ mode: "staged" });
    expect(parseOptions(["--all"])).toEqual({ mode: "all" });
    expect(parseOptions(["--base-ref", "origin/main"])).toEqual({
      baseRef: "origin/main",
      mode: "base-ref",
    });
    expect(() => parseOptions(["--unknown"])).toThrow("Usage:");

    const calls = [];
    expect(
      changedPathsFor(
        { mode: "base-ref", baseRef: "origin/main" },
        (command, args) => {
          calls.push({ args, command });
          return "src-tauri/src/orchestration/plan.rs\n";
        },
      ),
    ).toEqual(["src-tauri/src/orchestration/plan.rs"]);
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

    const sourcePaths = changedPathsFor({ mode: "all" }, (command, args) => {
      expect(command).toBe("git");
      expect(args).toEqual(["ls-files", "src", "src-tauri/src", "scripts"]);
      return "scripts/check-code-structure.mjs\n";
    });
    expect(sourcePaths).toEqual(["scripts/check-code-structure.mjs"]);
  });

  it("loads the base exception ledger only when the base contains one", () => {
    const content = JSON.stringify({
      exceptions: [
        {
          path: "src-tauri/src/legacy.rs",
          work_item: "QUAL-004",
          expires_on: "2026-08-22",
          maximum_meaningful_lines: 401,
        },
      ],
    });
    expect(
      baselineExceptionsFor(
        { baseRef: "origin/main", mode: "base-ref" },
        (command, args) => {
          expect(command).toBe("git");
          expect(args).toEqual([
            "show",
            "origin/main:docs/quality/code-structure-exceptions.json",
          ]);
          return content;
        },
      ),
    ).toEqual(JSON.parse(content).exceptions);
    expect(
      baselineExceptionsFor({ mode: "all" }, () => {
        throw new Error("all mode must not query Git history");
      }),
    ).toBeUndefined();
    expect(
      baselineExceptionsFor({ mode: "working-tree" }, () => {
        const error = new Error("exception ledger does not exist yet");
        error.status = 128;
        throw error;
      }),
    ).toBeUndefined();
  });

  it("rejects exception-ledger growth relative to its base branch", () => {
    const baseline = {
      path: "src-tauri/src/legacy.rs",
      work_item: "QUAL-004",
      expires_on: "2026-08-22",
      maximum_meaningful_lines: 401,
    };
    expect(
      validateExceptionLedger({
        exceptions: [baseline],
        baselineExceptions: [baseline],
      }),
    ).toEqual([]);
    expect(
      validateExceptionLedger({
        exceptions: [
          {
            ...baseline,
            maximum_meaningful_lines: baseline.maximum_meaningful_lines + 1,
          },
        ],
        baselineExceptions: [baseline],
      }),
    ).toEqual([
      `${baseline.path} cannot raise its source-structure exception ceiling above ${baseline.maximum_meaningful_lines}.`,
    ]);
    expect(
      validateExceptionLedger({
        exceptions: [{ ...baseline, expires_on: "2026-08-23" }],
        baselineExceptions: [baseline],
      }),
    ).toEqual([
      `${baseline.path} cannot extend its source-structure exception expiry beyond ${baseline.expires_on}.`,
    ]);
    expect(
      validateExceptionLedger({
        exceptions: [{ ...baseline, work_item: "QUAL-005" }],
        baselineExceptions: [baseline],
      }),
    ).toEqual([
      `${baseline.path} cannot change its source-structure exception owner from ${baseline.work_item} to QUAL-005.`,
    ]);
    expect(
      validateExceptionLedger({
        exceptions: [{ ...baseline, path: "src-tauri/src/new-legacy.rs" }],
        baselineExceptions: [baseline],
      }),
    ).toEqual([
      "src-tauri/src/new-legacy.rs is a new source-structure exception. New exceptions require product-owner approval and an ADR.",
    ]);
  });

  it("requires exceptions to have an owner, expiry, and fixed ceiling", () => {
    expect(() => loadExceptions(() => JSON.stringify({}))).toThrow(
      "must contain exceptions",
    );
    expect(() =>
      loadExceptions(() =>
        JSON.stringify({
          exceptions: [{ path: "legacy.rs", work_item: "QUAL-004" }],
        }),
      ),
    ).toThrow("YYYY-MM-DD expiry");
    expect(() =>
      loadExceptions(() =>
        JSON.stringify({
          exceptions: [
            {
              path: "legacy.rs",
              work_item: "QUAL-004",
              expires_on: "2026-08-22",
            },
          ],
        }),
      ),
    ).toThrow("maximum meaningful-line count");
    expect(() =>
      loadExceptions(() =>
        JSON.stringify({
          exceptions: [
            {
              path: "legacy.rs",
              work_item: "QUAL-004",
              expires_on: "2026-08-22",
              maximum_meaningful_lines: 401,
            },
            {
              path: "legacy.rs",
              work_item: "QUAL-004",
              expires_on: "2026-08-22",
              maximum_meaningful_lines: 401,
            },
          ],
        }),
      ),
    ).toThrow("must not duplicate");
    expect(
      loadExceptions(() =>
        JSON.stringify({
          exceptions: [
            {
              path: "legacy.rs",
              work_item: "QUAL-004",
              expires_on: "2026-08-22",
              maximum_meaningful_lines: 401,
            },
          ],
        }),
      ),
    ).toEqual([
      {
        path: "legacy.rs",
        work_item: "QUAL-004",
        expires_on: "2026-08-22",
        maximum_meaningful_lines: 401,
      },
    ]);
  });

  it("runs with injectable process and filesystem boundaries", () => {
    const logs = [];

    runStructureCheck({
      args: ["--staged"],
      currentDate: "2026-08-08",
      execute(command, args) {
        expect(command).toBe("git");
        expect(args).toEqual([
          "diff",
          "--cached",
          "--name-only",
          "--diff-filter=ACMR",
        ]);
        return "src-tauri/src/small.rs\n";
      },
      readFile() {
        return "let value = 1;\n";
      },
      loadAllowedExceptions() {
        return [];
      },
      loadBaselineExceptions() {
        return undefined;
      },
      log(message) {
        logs.push(message);
      },
    });

    expect(logs).toEqual(["Validated source structure for 1 changed path(s)."]);
  });

  it("fails the command before a structural violation can reach a merge", () => {
    expect(() =>
      runStructureCheck({
        args: ["--staged"],
        execute() {
          return "src-tauri/src/oversized.rs\n";
        },
        readFile() {
          return `${"let value = 1;\n".repeat(
            MAX_PRODUCTION_SOURCE_LINES + 1,
          )}`;
        },
        loadAllowedExceptions() {
          return [];
        },
        loadBaselineExceptions() {
          return undefined;
        },
      }),
    ).toThrow("Split independent responsibilities");
  });
});
