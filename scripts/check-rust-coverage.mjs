import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { projectRootFor } from "./project-root.mjs";

export { projectRootFor } from "./project-root.mjs";

const projectRoot = projectRootFor(import.meta.url, process.cwd());
const reportPath = resolve(projectRoot, "coverage", "rust-coverage.json");
export const REQUIRED_PERCENTAGE = 80;

function coveragePercentage(name, metric) {
  if (
    metric === undefined ||
    !Number.isFinite(metric.count) ||
    !Number.isFinite(metric.covered) ||
    metric.count < 0 ||
    metric.covered < 0 ||
    metric.covered > metric.count
  ) {
    throw new Error(`Rust ${name} coverage is unavailable.`);
  }

  return metric.count === 0 ? 100 : (metric.covered / metric.count) * 100;
}

export function validateCoverageTotals(
  totals,
  requiredPercentage = REQUIRED_PERCENTAGE,
) {
  if (totals === undefined) {
    throw new Error("Rust coverage report does not contain totals.");
  }

  const metrics = {
    branches: totals.branches,
    functions: totals.functions,
    lines: totals.lines,
    statements: totals.regions,
  };

  return Object.entries(metrics).map(([name, metric]) => {
    const percentage = coveragePercentage(name, metric);

    if (percentage < requiredPercentage) {
      throw new Error(
        `Rust ${name} coverage is ${percentage.toFixed(2)}%; ${requiredPercentage}% is required.`,
      );
    }

    const eligibleItems = metric.count === 0 ? " (no executable items)" : "";
    return `Rust ${name}: ${percentage.toFixed(2)}%${eligibleItems}`;
  });
}

export function runRustCoverage({
  execute = execFileSync,
  makeDirectory = mkdirSync,
  outputPath = reportPath,
  readFile = readFileSync,
} = {}) {
  makeDirectory(dirname(outputPath), { recursive: true });

  execute(
    "cargo",
    [
      "+nightly",
      "llvm-cov",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--branch",
      "--json",
      "--output-path",
      outputPath,
    ],
    {
      cwd: projectRoot,
      stdio: "inherit",
    },
  );

  const report = JSON.parse(readFile(outputPath, "utf8"));
  return validateCoverageTotals(report.data?.[0]?.totals);
}

export function printCoverageResult({
  log = console.log,
  runCoverage = runRustCoverage,
} = {}) {
  for (const message of runCoverage()) {
    log(message);
  }
}

/* v8 ignore next -- narrow CLI bootstrap; printCoverageResult is unit tested. */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  printCoverageResult();
}
