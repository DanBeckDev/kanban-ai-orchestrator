import type { AgentProfile, AgentProfileKind, PlannerProfile } from "./types";

export type AgentProfilePresentation = Readonly<{
  kind: AgentProfileKind;
  label: string;
  defaultProgram: string;
  argumentHint: string;
  protocolSummary: string;
}>;

const noninteractiveCapabilitySummary =
  "Streams structured lifecycle events. Feedback, session resume, and safe process-tree cancellation are not available yet.";

const presentationByKind: Record<AgentProfileKind, AgentProfilePresentation> = {
  structured_process: {
    kind: "structured_process",
    label: "Structured JSONL bridge",
    defaultProgram: "agent-worker",
    argumentHint: "--jsonl",
    protocolSummary:
      "Runs an approved executable that accepts a task brief on stdin and emits normalized JSONL events on stdout.",
  },
  codex_cli: {
    kind: "codex_cli",
    label: "Codex CLI",
    defaultProgram: "codex",
    argumentHint: "--model\ngpt-5",
    protocolSummary:
      "Runs Codex through its native structured event protocol; this profile cannot override the desktop protocol or sandbox controls.",
  },
  claude_code: {
    kind: "claude_code",
    label: "Claude Code",
    defaultProgram: "claude",
    argumentHint: "--model\nsonnet",
    protocolSummary:
      "Runs Claude Code through its native structured event protocol; this profile cannot override the desktop protocol or permission controls.",
  },
  cline_pass_cli: {
    kind: "cline_pass_cli",
    label: "Cline CLI (ClinePass)",
    defaultProgram: "cline",
    argumentHint: "--thinking\nhigh",
    protocolSummary:
      "Runs Cline's native structured event protocol with the locally configured ClinePass account; this profile cannot override provider, credentials, approval, worktree, or protocol controls.",
  },
};

export const agentProfilePresentations = Object.values(presentationByKind);

export function agentProfilePresentation(
  kind: AgentProfileKind,
): AgentProfilePresentation {
  return presentationByKind[kind];
}

export function defaultNativeAgentProfile(
  kind: Exclude<AgentProfileKind, "structured_process">,
): AgentProfile {
  const presentation = agentProfilePresentation(kind);
  return {
    name: `Default ${presentation.label}`,
    kind,
    program: presentation.defaultProgram,
    arguments: [],
  };
}

export function defaultNativePlannerProfile(
  kind: Exclude<AgentProfileKind, "structured_process">,
): PlannerProfile {
  const presentation = agentProfilePresentation(kind);
  return {
    name: `Default ${presentation.label} orchestrator`,
    kind,
    program: presentation.defaultProgram,
    arguments: [],
  };
}

export function installationGuideFor(
  kind: AgentProfileKind,
): string | undefined {
  switch (kind) {
    case "codex_cli":
      return "https://developers.openai.com/codex/cli/";
    case "claude_code":
      return "https://code.claude.com/docs/en/overview";
    case "cline_pass_cli":
      return "https://docs.cline.bot/cli";
    case "structured_process":
      return undefined;
  }
}

export { noninteractiveCapabilitySummary };
