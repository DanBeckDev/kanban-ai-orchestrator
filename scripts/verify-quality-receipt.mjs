import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export function projectRootFor(moduleUrl, workingDirectory) {
  return moduleUrl.startsWith("file:")
    ? resolve(fileURLToPath(new URL("..", moduleUrl)))
    : workingDirectory;
}

const projectRoot = projectRootFor(import.meta.url, process.cwd());
const reviewDirectory = "docs/quality/reviews";
const receiptPathPattern = /^docs\/quality\/reviews\/[^/]+\.ya?ml$/;

export function isCodeBearingPath(path) {
  return (
    path.startsWith("src/") ||
    path.startsWith("src-tauri/") ||
    path.startsWith("scripts/") ||
    path.startsWith("githooks/") ||
    path.startsWith(".github/workflows/") ||
    [
      "package.json",
      "package-lock.json",
      ".nvmrc",
      "index.html",
      "biome.json",
      "vite.config.ts",
      "vitest.config.ts",
      "tsconfig.json",
      "tsconfig.app.json",
      "tsconfig.node.json",
      "src-tauri/Cargo.toml",
      "src-tauri/Cargo.lock",
    ].includes(path)
  );
}

export function validateReceiptContent(content) {
  const requirements = [
    ["a work item", /^work_item:\s*\S+/m],
    ["the Clean Code review skill", /^\s*skill:\s*clean-code-review\s*$/m],
    ["a reviewer", /^\s*reviewer:\s*\S+/m],
    ["a remediation record", /^remediation:\s*(?:\[\])?\s*$/m],
    [
      "zero unresolved actionable findings",
      /^\s*unresolved_actionable_findings:\s*0\s*$/m,
    ],
    [
      "a passing quality:verify command",
      /^\s*-\s*command:\s*npm run quality:verify\s*\n\s*result:\s*passed\s*$/m,
    ],
    ["a met coverage threshold", /^\s*threshold_met:\s*true\s*$/m],
  ];

  return requirements
    .filter(([, pattern]) => !pattern.test(content))
    .map(([description]) => description);
}

function validateReceiptPaths(receiptPaths, readFile) {
  const errors = receiptPaths.flatMap((receiptPath) =>
    validateReceiptContent(
      readFile(resolve(projectRoot, receiptPath), "utf8"),
    ).map((requirement) => `${receiptPath} is missing ${requirement}.`),
  );

  if (errors.length > 0) {
    throw new Error(errors.join("\n"));
  }
}

export function verifyQualityReceipts({
  changedPaths,
  mode,
  readDirectory = readdirSync,
  readFile = readFileSync,
}) {
  const codeBearingPaths = changedPaths.filter(isCodeBearingPath);

  if (mode !== "all" && codeBearingPaths.length === 0) {
    return "No code-bearing changes; no quality-review receipt is required.";
  }

  const receiptPaths =
    mode === "all"
      ? readDirectory(resolve(projectRoot, reviewDirectory))
          .filter(
            (fileName) =>
              fileName.endsWith(".yaml") || fileName.endsWith(".yml"),
          )
          .map((fileName) => `${reviewDirectory}/${fileName}`)
      : changedPaths.filter((path) => receiptPathPattern.test(path));

  if (receiptPaths.length === 0) {
    throw new Error(
      "Code-bearing work requires a changed quality-review receipt.",
    );
  }

  validateReceiptPaths(receiptPaths, readFile);
  return `Validated ${receiptPaths.length} quality-review receipt(s).`;
}

export function parseOptions(args) {
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
    "Usage: verify-quality-receipt.mjs --staged | --all | --base-ref <ref>",
  );
}

export function changedPathsFor(options, execute = execFileSync) {
  if (options.mode === "all") {
    return [];
  }

  const args =
    options.mode === "staged"
      ? ["diff", "--cached", "--name-only", "--diff-filter=ACMR"]
      : [
          "diff",
          "--name-only",
          "--diff-filter=ACMR",
          `${options.baseRef}...HEAD`,
        ];
  const output = execute("git", args, { cwd: projectRoot, encoding: "utf8" });

  return output.split("\n").filter(Boolean);
}

export function runReceiptVerification({
  args = process.argv.slice(2),
  findChangedPaths = changedPathsFor,
  log = console.log,
  verifyReceipts = verifyQualityReceipts,
} = {}) {
  const options = parseOptions(args);
  const changedPaths = findChangedPaths(options);
  log(verifyReceipts({ changedPaths, mode: options.mode }));
}

/* v8 ignore next -- narrow CLI bootstrap; runReceiptVerification is unit tested. */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runReceiptVerification();
}
