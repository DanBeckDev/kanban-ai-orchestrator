# ADR 0013: Use bounded provider-process adapters at the execution boundary

- Status: Accepted
- Date: 2026-08-08

## Context

The core accepts normalized agent lifecycle events, but a provider-neutral desktop also needs a concrete outer boundary for local agent CLIs and future provider wrappers. Terminal prose is neither a reliable completion signal nor a safe process protocol. Unbounded stdout can also freeze or exhaust the desktop process.

## Decision

- A configured process adapter starts a declared executable with structured argument values, never through a shell string. It uses the assigned workspace as its current directory and supplies the task brief over standard input, which avoids operating-system command-line length limits.
- The `structured_process` profile is the provider-neutral bridge: it reads normalized newline-delimited JSON lifecycle events from standard output. Each line has a monotonic `sequence` and a flattened normalized event shape, for example `{"sequence":1,"type":"activity","summary":"Inspecting the repository"}`.
- The `codex_cli` profile invokes `codex exec --json --sandbox workspace-write … -`, passing the brief on standard input. Its native JSONL lifecycle events are translated at the edge into safe normalized summaries. This command shape follows the [Codex non-interactive CLI documentation](https://developers.openai.com/codex/noninteractive/).
- The `claude_code` profile invokes `claude --print --output-format stream-json --verbose --permission-mode acceptEdits …`, again passing the brief on standard input. Its stream events are translated at the edge into the same normalized summaries. This command shape follows the [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage).
- The `cline_pass_cli` profile invokes `cline --json --provider cline --auto-approve true …`, passing the brief on standard input. The user authenticates separately with `cline auth cline` and chooses a ClinePass model in Cline; account credentials remain in Cline's local credential store, never in the board profile or SQLite. Cline's native `agent_event` NDJSON is translated at the edge into fixed progress, usage, completion, and failure summaries. The non-interactive approval mode is adapter-owned because Cline denies tool requests in a non-TTY run when approvals are required; the desktop's existing start policy and assigned worktree remain the launch boundary. This command shape follows the [Cline CLI reference](https://github.com/cline/cline/tree/main/apps/cli).
- Native profiles reject user-supplied arguments that would replace the adapter-owned protocol, working-directory, sandbox, permission, resume, or unsafe-bypass controls. They may still accept provider options such as a model choice.
- A single event line is limited to 64 KiB and a session retains at most 1,000 events. Malformed, out-of-order, oversized, or excessive output becomes one normalized `failed` event and stops the child process.
- Standard error is not retained by this adapter. Provider wrappers must publish selected safe lifecycle summaries instead of leaking raw transcripts or secrets into durable board data.
- All current profiles expose structured streaming but honestly report feedback, session resume, and process-tree interruption as unsupported. They may terminate their direct child for shutdown, but may not claim that this cancels descendants. Platform-specific adapters can add process-tree cancellation only with a tested implementation.
- If a current adapter emits an input or approval request, the runtime records that request and then fails the attempt with an explicit capability reason. It must not show a card as interactively waiting when no feedback channel exists.

## Consequences

- Codex CLI, Claude Code, ClinePass CLI, and the provider-neutral structured bridge pass the same non-interactive conformance suite. The suite verifies explicit capability discovery, non-resumable sessions, rejected feedback/resume/interruption, sequential safe events, and a terminal review-ready completion.
- Other providers can add a small protocol decoder and wrapper without changing the daemon state machine.
- A provider `completed` event still requests only `Review`; `Done` remains subject to evidence and human-review rules.
- The desktop runtime durably saves a profile, records and audits a start decision, provisions and verifies the assigned worktree, starts the direct child, and atomically attaches its session while moving the task to `Running`. It monitors the process independently of React; a `completed` event stops at `Review`.
- Platform process-tree cancellation and interactive feedback/resume remain separate execution-layer work. A future adapter must advertise each only after its own conformance and platform tests prove the behavior.
