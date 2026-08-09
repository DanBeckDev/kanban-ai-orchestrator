# Platform release evidence

The initial release targets macOS, while the shared React, Rust, SQLite, Git-worktree, and adapter boundaries remain portable to Windows and Linux. This document separates continuously verified portability from release-specific evidence that cannot safely be faked in a generic CI job.

## Continuous verification

The `core` GitHub Actions job runs `npm run test:platform` on fresh Linux, macOS, and Windows runners. That command type-checks the shared TypeScript UI, runs the frontend tests, and runs the Rust suite, including the state machine, dependency graph, SQLite persistence, Linear connector, portable workspace-path validation, and Git worktree boundary tests.

The Linux-only `quality` job retains the full coverage, lint, formatting, source-structure, and receipt checks. Coverage instrumentation does not need to run three times to prove that the core compiles and behaves on each supported operating system.

## Platform boundary coverage

| Concern | Continuous evidence | Release evidence before distribution |
| --- | --- | --- |
| Command construction | Worker and planner profiles use direct executable plus structured arguments; briefs use standard input rather than command-line text. Rust tests cover profile validation and process boundaries, and the three-platform core job compiles the same boundary. | Run one real provider profile with a representative long brief on each supported platform. |
| Credential storage | OAuth, refresh, and credential-store error behavior are unit tested without secrets. The production dependency selects Keychain Services, Credential Manager, or Secret Service by platform. | Connect then disconnect a test Linear OAuth app on the target platform; confirm credentials are not present in SQLite, diagnostics, or logs. |
| Process and PTY behavior | Provider capability discovery exposes unsupported feedback, resume, and process-tree cancellation instead of silently claiming support. Unix direct-process conformance tests cover the currently implemented process path. | Verify the selected provider's documented lifecycle and cancellation behavior on the target platform before enabling that capability in a release profile. |
| ClinePass worker | The adapter fixes Cline's JSON, Cline-provider, and noninteractive approval controls, while Cline retains its own account authentication and model selection. Unit and conformance tests cover the normalized event boundary without a real account. | Run an authenticated ClinePass profile with a representative long brief, record the Cline version and selected model without credentials, and confirm the board retains no transcript or account data. |
| Packaging | Tauri is configured to produce all native bundle targets from one codebase. | Build and install each target bundle; macOS distribution additionally needs signing and notarization, Windows needs the chosen signing identity, and Linux needs package-format validation for the supported distribution family. |

## Release decision

No release may claim a provider capability or platform package that lacks the corresponding evidence above. A failing cross-platform core job must be resolved before merge; repository branch protection should require all three `core` checks before a release branch can merge. A missing release-specific manual check blocks distribution of that platform artifact, not the portable core.

## Manual evidence register

The **release lead** owns this register for every distributable platform artifact. Evidence belongs in the release PR or its linked release record, must be dated, and must exclude credentials, provider transcripts, and project source.

| Check | Accountable owner | Required evidence | Distribution effect if absent |
| --- | --- | --- | --- |
| Representative long provider brief | Release lead | Selected profile, platform, outcome, and CI/release-record link. | Do not distribute that platform/provider profile. |
| Linear credential lifecycle | Release lead with the test Linear-app owner | Connect/disconnect result confirming no credential appears in SQLite, diagnostics, or logs. | Do not distribute Linear connectivity on that platform. |
| Provider lifecycle and cancellation | Provider-profile owner | Documented capability matrix plus a target-platform start/stop observation. | Do not claim unsupported resume, feedback, or process-tree cancellation. |
| ClinePass worker profile | ClinePass profile owner | Cline version, selected ClinePass model, redacted long-brief outcome, and proof that board data contains no credentials or provider transcript. | Do not distribute ClinePass worker support on that platform. |
| Package installation and signing | Platform release owner | Installed bundle version, package format, signing/notarization identity status, and install result. | Do not distribute that platform artifact. |
