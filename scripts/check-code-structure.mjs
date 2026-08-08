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
  let args;
  if (options.mode === "all") {
    args = ["ls-files", "src", "src-tauri/src", "scripts"];
  } else if (options.mode === "staged") {
    args = ["diff", "--cached", "--name-only", "--diff-filter=ACMR"];
  } else if (options.mode === "base-ref") {
    args = [
      "diff",
      "--name-only",
      "--diff-filter=ACMR",
      `${options.baseRef}...HEAD`,
    ];
  } else {
    args = ["diff", "--name-only", "--diff-filter=ACMR", "HEAD"];
  }

  return execute("git", args, { cwd: projectRoot, encoding: "utf8" })
    .split("\n")
    .filter(Boolean);
}

export function loadExceptions(readFile = readFileSync) {
  return parseExceptions(
    readFile(resolve(projectRoot, exceptionsPath), "utf8"),
  );
}

export function parseExceptions(content) {
  const parsed = JSON.parse(content);

  if (
    !Array.isArray(parsed.exceptions) ||
    parsed.exceptions.some(
      (exception) =>
        typeof exception.path !== "string" ||
        typeof exception.work_item !== "string" ||
        !/^\d{4}-\d{2}-\d{2}$/.test(exception.expires_on) ||
        !Number.isInteger(exception.maximum_meaningful_lines) ||
        exception.maximum_meaningful_lines <= MAX_TEST_SOURCE_LINES,
    )
  ) {
    throw new Error(
      `${exceptionsPath} must contain exceptions with a path, work item, YYYY-MM-DD expiry, and a maximum meaningful-line count over ${MAX_TEST_SOURCE_LINES}.`,
    );
  }

  const paths = new Set(parsed.exceptions.map((exception) => exception.path));
  if (paths.size !== parsed.exceptions.length) {
    throw new Error(`${exceptionsPath} must not duplicate an exception path.`);
  }

  return parsed.exceptions;
}

function activeException(path, exceptions, currentDate) {
  return exceptions.find(
    (exception) =>
      exception.path === path && exception.expires_on >= currentDate,
  );
}

export function baselineExceptionsFor(options, execute = execFileSync) {
  const baseRef =
    options.mode === "base-ref"
      ? options.baseRef
      : options.mode === "all"
        ? undefined
        : "HEAD";
  if (baseRef === undefined) {
    return undefined;
  }

  try {
    return parseExceptions(
      execute("git", ["show", `${baseRef}:${exceptionsPath}`], {
        cwd: projectRoot,
        encoding: "utf8",
      }),
    );
  } catch (error) {
    if (error.status === 128) {
      return undefined;
    }
    throw error;
  }
}

export function validateExceptionLedger({ exceptions, baselineExceptions }) {
  if (baselineExceptions === undefined) {
    return [];
  }

  const baselineByPath = new Map(
    baselineExceptions.map((exception) => [exception.path, exception]),
  );
  return exceptions.flatMap((exception) => {
    const baseline = baselineByPath.get(exception.path);
    if (baseline === undefined) {
      return [
        `${exception.path} is a new source-structure exception. New exceptions require product-owner approval and an ADR.`,
      ];
    }
    if (baseline.work_item !== exception.work_item) {
      return [
        `${exception.path} cannot change its source-structure exception owner from ${baseline.work_item} to ${exception.work_item}.`,
      ];
    }
    if (baseline.expires_on < exception.expires_on) {
      return [
        `${exception.path} cannot extend its source-structure exception expiry beyond ${baseline.expires_on}.`,
      ];
    }
    if (
      baseline.maximum_meaningful_lines < exception.maximum_meaningful_lines
    ) {
      return [
        `${exception.path} cannot raise its source-structure exception ceiling above ${baseline.maximum_meaningful_lines}.`,
      ];
    }

    return [];
  });
}

export function validateSourceStructure({
  paths,
  readFile = readFileSync,
  exceptions = [],
  currentDate = new Date().toISOString().slice(0, 10),
}) {
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
    const exception = activeException(path, exceptions, currentDate);
    if (
      lineCount <= limit ||
      (exception && lineCount <= exception.maximum_meaningful_lines)
    ) {
      return [];
    }

    const ceiling = exception
      ? `the temporary exception ceiling is ${exception.maximum_meaningful_lines}`
      : `the limit is ${limit}`;

    return [
      `${path} has ${lineCount} meaningful lines; ${ceiling}. Split independent responsibilities into cohesive modules before merging.`,
    ];
  });
}

export function runStructureCheck({
  args = process.argv.slice(2),
  currentDate,
  execute = execFileSync,
  loadBaselineExceptions = baselineExceptionsFor,
  loadAllowedExceptions = loadExceptions,
  log = console.log,
  readFile = readFileSync,
} = {}) {
  const options = parseOptions(args);
  const paths = changedPathsFor(options, execute);
  const exceptions = loadAllowedExceptions(readFile);
  const violations = validateSourceStructure({
    paths,
    readFile,
    exceptions,
    currentDate,
  });
  violations.push(
    ...validateExceptionLedger({
      exceptions,
      baselineExceptions: loadBaselineExceptions(options, execute),
    }),
  );

  if (violations.length > 0) {
    throw new Error(violations.join("\n"));
  }

  log(`Validated source structure for ${paths.length} changed path(s).`);
}

/* v8 ignore next -- narrow CLI bootstrap; runStructureCheck is unit tested. */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runStructureCheck();
}
