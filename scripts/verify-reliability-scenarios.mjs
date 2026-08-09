import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { projectRootFor } from "./project-root.mjs";

export { projectRootFor } from "./project-root.mjs";

const projectRoot = projectRootFor(import.meta.url, process.cwd());

export const reliabilityScenarios = [
  {
    id: "restart-recovery",
    test: "persistence::sqlite_event_store_tests::restart_reconciliation_preserves_history_and_interrupts_unconfirmed_work",
  },
  {
    id: "dependency-blockers",
    test: "domain::dependency_graph::tests::finds_ready_unblocked_items_and_critical_path",
  },
  {
    id: "dependency-cycle",
    test: "persistence::board_store_tests::validates_dependency_cycles_and_reuses_matching_dependency_commands",
  },
  {
    id: "worktree-race-guard",
    test: "desktop_daemon_lock::tests::prevents_another_daemon_from_using_the_same_data_directory",
  },
  {
    id: "worktree-recovery",
    test: "workspace::tests::recovers_an_interrupted_empty_target_by_attaching_the_precreated_task_branch",
  },
  {
    id: "connector-conflict",
    test: "application::linear_sync_service_tests::records_an_intentionally_empty_linear_description_as_a_conflict",
  },
  {
    id: "ambiguous-connector-delivery",
    test: "persistence::connector_sync_store_tests::converts_in_flight_delivery_to_uncertain_during_restart_recovery",
  },
  {
    id: "scope-escape",
    test: "workspace::tests::denies_base_repository_writes_and_undeclared_paths_but_allows_assigned_workspace_paths",
  },
  {
    id: "direct-process-cancellation",
    platform: "unix",
    test: "desktop_execution_runtime_tests::stops_a_live_direct_process_and_records_an_interrupted_attempt",
  },
];

export function scenariosForPlatform(platform) {
  return reliabilityScenarios.filter(
    (scenario) => scenario.platform !== "unix" || platform !== "win32",
  );
}

export function cargoArgumentsFor(test) {
  return [
    "test",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    test,
    "--",
    "--exact",
  ];
}

export function runReliabilityScenarios({
  execute = execFileSync,
  log = console.log,
  platform = process.platform,
} = {}) {
  return scenariosForPlatform(platform).map((scenario) => {
    log(`Verifying reliability scenario: ${scenario.id}`);
    execute("cargo", cargoArgumentsFor(scenario.test), {
      cwd: projectRoot,
      stdio: "inherit",
    });
    return scenario.id;
  });
}

export function printReliabilityResult({
  log = console.log,
  runScenarios = runReliabilityScenarios,
} = {}) {
  const scenarios = runScenarios({ log });
  log(`Verified ${scenarios.length} reliability release scenarios.`);
}

/* v8 ignore next -- narrow CLI bootstrap; printReliabilityResult is unit tested. */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  printReliabilityResult();
}
