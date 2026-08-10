# ClinePass worker profile

## Purpose

The **Cline CLI (ClinePass)** agent profile runs a task through Cline's native JSON event stream while preserving the board's provider-neutral lifecycle contract. ClinePass is selected and authenticated by Cline itself, so the board never stores an API key, OAuth token, account token, subscription state, or model entitlement.

## One-time setup

Install the Cline CLI using Cline's [installation instructions](https://github.com/cline/cline/tree/main/apps/cli), then authenticate it outside the desktop app:

```sh
cline auth cline
```

Sign in and choose an available ClinePass model in Cline's setup flow. Cline documents that account tokens are generated and managed automatically after sign-in; do not copy a token into this app.

## Add the board profile

Select **Cline CLI (ClinePass)** in the normal AI-role settings. Its default
program is `cline`; the adapter owns `--json`, `--provider cline`, and
`--auto-approve true` for a ticket worker. The settings surface can pass a
deliberately named model and a provider-neutral effort choice as Cline's native
model/thinking options. It never shows provider, credential, approval,
worktree, or protocol flags.

Advanced profile arguments may tune only non-reserved behaviour. The adapter
rejects profile arguments that can supply a key, select another provider or
model, change approvals, detach to the Cline hub, replace the assigned
worktree, or alter the native event protocol.

The adapter passes the task brief over standard input, sets Cline's current directory to the assigned worktree, and accepts only bounded newline-delimited lifecycle events. It retains fixed lifecycle summaries and token/cost totals, never assistant text, tool content, account credentials, or raw Cline transcripts.

## Approval and safety boundary

Cline's non-interactive mode denies tool requests when approvals are required. The worker profile therefore owns its non-interactive auto-approval setting so a policy-authorized task can complete inside its assigned worktree. That setting is not user-configurable through the profile and does not claim a Cline filesystem sandbox. The desktop policy gate, worktree boundary, quality checks, and human review rules continue to govern whether a task launches and whether it can reach `Done`.

Feedback, session resume, and safe process-tree cancellation remain unavailable for this profile until there is Cline-specific, platform-tested support.
