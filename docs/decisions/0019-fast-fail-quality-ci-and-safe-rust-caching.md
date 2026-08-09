# ADR 0019: Fail quality policy early and cache stable Cargo artifacts safely

- Status: Accepted
- Date: 2026-08-09

## Context

The required quality workflow ran source-structure and review-receipt policy only after it had
installed coverage tooling and run the full frontend and Rust quality suite. A missing receipt
could therefore waste several minutes of hosted-runner time before failing. Every platform also
started from an empty Cargo dependency and build-artifact cache, making stable `cargo test` and
Clippy needlessly slow. Rust coverage uses a nightly instrumented build, so it must not reuse or
overwrite the stable artifacts used by the portable test matrix.

## Decision

- Run source-structure and review-receipt validation in a lightweight `policy` job before any
  platform or full-quality job. The expensive `quality` job requires that job to pass.
- Keep the Linux, macOS, and Windows portable-core matrix and the Linux full quality/coverage job;
  performance work must not reduce cross-platform or coverage evidence.
- Restore only Cargo registry, Git dependency, and stable `src-tauri/target` artifacts. Key that
  cache by operating system, architecture, installed Rust compiler identity, and the Cargo lockfile
  and manifest. Never cache credentials or a cross-platform target directory.
- Put nightly `cargo-llvm-cov` output in `src-tauri/coverage-target`, outside the stable cache.
  Install the already-version-pinned coverage executable through its maintained verified release
  installer rather than compiling that executable from source in every run.
- Cancel obsolete runs for the same pull request or branch when a newer commit arrives.

## Consequences

- Policy errors stop in seconds and do not consume platform capacity.
- A warmed cache from `main` or a previous run avoids repeat downloads and recompilation for
  stable `cargo test` and Clippy, while Cargo still validates cache artifacts against the exact
  compiler and dependency inputs.
- The first run after a compiler or dependency change remains a cache miss by design. Nightly
  coverage remains independent and authoritative, rather than being made unsafely incremental.
