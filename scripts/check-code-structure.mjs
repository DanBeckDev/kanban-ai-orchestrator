import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const MAX_PRODUCTION_SOURCE_LINES = 400;
export const MAX_TEST_SOURCE_LINES = 400;

export function projectRootFor(moduleUrl, workingDirectory) {
  return moduleUrl.startsWith("file:")
    ? resolve(fileURLToPath(new URL("..", moduleUrl)))
    : workingDirectory;
}

const projectRoot = projectRootFor(import.meta.url, process.cwd());
const exceptionsPath = "docs/quality/code-structure-exceptions.json";
const sourceRoots = ["src", "src-tauri/src", "scripts"];

export function sourceFileLimit(path) {
  const isSourceFile =
    (path.startsWith("src/") ||
      path.startsWith("src-tauri/src/") ||
      path.startsWith("scripts/")) &&
    /\.(?:[cm]?js|tsx?|rs)$/.test(path);

  if (!isSourceFile) {
    return undefined;
  }

  return /(?:^|\/)(?:tests?\/|[^/]+(?:\.test|_tests)\.)/.test(path)
    ? MAX_TEST_SOURCE_LINES
    : MAX_PRODUCTION_SOURCE_LINES;
}

export function meaningfulLineCount(source) {
  return source.split(/\r?\n/).filter((line) => {
    const trimmed = line.trim();
    return (
      trimmed.length > 0 &&
      !trimmed.startsWith("//") &&
      !trimmed.startsWith("#")
    );
  }).length;
}

export function parseOptions(args) {
  if (args.length === 0) {
    return { mode: "working-tree" };
  }
  if (args.length === 1 && args[0] === "--staged") {
    return { mode: "staged" };
  }
  if (args.length === 1 && args[0] === "--all") {
    return { mode: "all" };
  }
  if (args.length === 2 && args[0] === "--base-ref") {
    return { baseRef: args[1], mode: "base-ref" };
  }

  throw new Error(
    "Usage: check-code-structure.mjs [--staged | --all | --base-ref <ref>]",
  );
}

export function changedPathsFor(options, execute = execFileSync) {
  const executeGit = (args) =>
    execute("git", args, { cwd: projectRoot, encoding: "utf8" })
      .split("\n")
      .filter(Boolean);
  let args;
  if (options.mode === "all") {
    args = [
      "ls-files",
      "--cached",
      "--others",
      "--exclude-standard",
      "--",
      ...sourceRoots,
    ];
  } else if (options.mode === "staged") {
    args = ["diff", "--cached", "--name-only", "--diff-filter=ACMR"];
  } else if (options.mode === "base-ref") {
    args = [
      "diff",
      "--name-only",
      "--diff-filter=ACMR",
      `${options.baseRef}...HEAD`,
    ];
  } else if (options.mode === "working-tree") {
    args = ["diff", "--name-only", "--diff-filter=ACMR", "HEAD"];
    const modifiedPaths = executeGit(args);
    const untrackedPaths = executeGit([
      "ls-files",
      "--others",
      "--exclude-standard",
      "--",
      ...sourceRoots,
    ]);
    return [...new Set([...modifiedPaths, ...untrackedPaths])];
  }

  return executeGit(args);
}

export function loadExceptions(readFile = readFileSync) {
  const ledger = JSON.parse(
    readFile(resolve(projectRoot, exceptionsPath), "utf8"),
  );
  if (!Array.isArray(ledger.exceptions)) {
    throw new Error(`${exceptionsPath} must declare an exceptions array.`);
  }
  if (ledger.exceptions.length > 0) {
    throw new Error(
      `${exceptionsPath} must not contain source-structure exceptions. Split every oversized file before merging.`,
    );
  }
}

export function validateSourceStructure({ paths, readFile = readFileSync }) {
  return paths.flatMap((path) => {
    const limit = sourceFileLimit(path);
    if (limit === undefined) {
      return [];
    }

    let source;
    try {
      source = readFile(resolve(projectRoot, path), "utf8");
    } catch (error) {
      if (error.code === "ENOENT") {
        return [];
      }
      throw error;
    }
    const lineCount = meaningfulLineCount(source);
    if (lineCount <= limit) {
      return [];
    }

    return [
      `${path} has ${lineCount} meaningful lines; the limit is ${limit}. Split independent responsibilities into cohesive modules before merging.`,
    ];
  });
}

export function runStructureCheck({
  args = process.argv.slice(2),
  execute = execFileSync,
  validateExceptionLedger = loadExceptions,
  log = console.log,
  readFile = readFileSync,
} = {}) {
  const options = parseOptions(args);
  const paths = changedPathsFor(options, execute);
  validateExceptionLedger(readFile);
  const violations = validateSourceStructure({
    paths,
    readFile,
  });

  if (violations.length > 0) {
    throw new Error(violations.join("\n"));
  }

  const scope =
    options.mode === "all" ? "repository path(s)" : "changed path(s)";
  log(`Validated source structure for ${paths.length} ${scope}.`);
}

/* v8 ignore next -- narrow CLI bootstrap; runStructureCheck is unit tested. */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runStructureCheck();
}
