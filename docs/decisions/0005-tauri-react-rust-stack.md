# ADR 0005: Tauri 2 shell with React/TypeScript and a Rust core

- Status: Accepted
- Date: 2026-08-08

## Context

The first release is a macOS desktop app, but its board, orchestration, agent-adapter, and repository logic must remain portable to Windows and Linux. It needs native process/worktree access, secure credential storage, durable local persistence, a responsive visual board, and reliable test coverage in both application languages.

## Decision

Use the following stack:

| Concern | Choice |
| --- | --- |
| Desktop shell and IPC | Tauri 2 |
| User interface | React with strict TypeScript and Vite |
| Authoritative local core | Rust, hosted by the Tauri application process |
| Concurrency | Tokio; isolate process/PTY and persistence mechanics from domain state |
| Durable data | SQLite behind a Rust repository interface |
| Agent process boundary | Rust adapter interface with structured protocol first and constrained PTY fallback |
| Frontend tests | Vitest with V8 coverage and Testing Library |
| Rust tests | `cargo test` plus `cargo llvm-cov` on nightly for branch instrumentation |
| JavaScript quality | Biome format/lint and TypeScript strict checking |
| Rust quality | `cargo fmt`, Clippy with warnings denied, and Cargo tests |

The core stays inside the Tauri process for the initial local release. If future requirements need an independently restartable background daemon, extract the Rust core behind the existing command/event boundary rather than putting domain rules in React.

## Coverage enforcement

Vitest enforces 80% line, statement, function, and branch coverage for the frontend package. Rust coverage runs with `cargo llvm-cov` and the nightly toolchain because branch instrumentation is not stable in Rust's normal coverage workflow. A repository script parses the Rust coverage report and enforces the same four 80% thresholds.

Framework entry points and generated sources are excluded only when they contain no product behavior; each exclusion is documented in `docs/quality/coverage-exclusions.md`.

## Consequences

- One frontend codebase serves macOS, Windows, and Linux; Tauri provides native shell/invoke capabilities without a browser-hosted runtime.
- Rust is a required developer and CI toolchain. The project records exact setup commands and keeps stable build tooling separate from nightly-only coverage instrumentation.
- The first quality-gate implementation must cover both JavaScript and Rust. A frontend-only green build is insufficient.
- React is a view layer. Scheduling, policy, state transitions, worktree management, and connector sync remain Rust domain concerns.
