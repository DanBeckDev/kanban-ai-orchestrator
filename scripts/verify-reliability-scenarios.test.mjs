import { describe, expect, it } from "vitest";

import {
  cargoArgumentsFor,
  printReliabilityResult,
  reliabilityScenarios,
  runReliabilityScenarios,
  scenariosForPlatform,
} from "./verify-reliability-scenarios.mjs";

describe("reliability release-scenario gate", () => {
  it("keeps every required reliability scenario explicitly named", () => {
    expect(reliabilityScenarios.map((scenario) => scenario.id)).toEqual([
      "restart-recovery",
      "dependency-blockers",
      "dependency-cycle",
      "worktree-race-guard",
      "worktree-recovery",
      "connector-conflict",
      "ambiguous-connector-delivery",
      "scope-escape",
      "direct-process-cancellation",
    ]);
  });

  it("omits the Unix-only direct-process scenario on Windows", () => {
    expect(scenariosForPlatform("win32")).not.toContainEqual(
      expect.objectContaining({ id: "direct-process-cancellation" }),
    );
    expect(scenariosForPlatform("linux")).toContainEqual(
      expect.objectContaining({ id: "direct-process-cancellation" }),
    );
  });

  it("runs each selected scenario as an exact Rust test", () => {
    const calls = [];
    const verified = runReliabilityScenarios({
      execute(command, args, options) {
        calls.push({ args, command, options });
      },
      log() {},
      platform: "win32",
    });

    expect(verified).toHaveLength(8);
    expect(calls).toHaveLength(8);
    expect(calls[0]).toEqual({
      args: cargoArgumentsFor(reliabilityScenarios[0].test),
      command: "cargo",
      options: expect.objectContaining({
        cwd: expect.any(String),
        stdio: "inherit",
      }),
    });
  });

  it("reports the scenario count after the runner finishes", () => {
    const messages = [];
    printReliabilityResult({
      log(message) {
        messages.push(message);
      },
      runScenarios() {
        return ["restart-recovery", "scope-escape"];
      },
    });

    expect(messages).toEqual(["Verified 2 reliability release scenarios."]);
  });
});
