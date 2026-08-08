# Coverage exclusions

Coverage excludes only framework plumbing with no product behavior.

| Path | Exclusion | Reason |
| --- | --- | --- |
| `src/main.tsx` | Frontend coverage | Vite/React root mounting boilerplate; the product UI is tested through `App`. |
| `src/vite-env.d.ts` | Frontend coverage | Type-only Vite declaration. |
| `src/test/**` | Frontend coverage | Test support code is not production behavior. |
| `src-tauri/src/main.rs` | Rust coverage | Tauri's thin platform entry point delegates directly to the tested library. |
| `src-tauri/src/lib.rs::run` | Rust coverage | Tauri framework bootstrap; domain/command behavior remains in tested functions. |
| `githooks/pre-commit` | JavaScript/Rust coverage | Thin Git launcher that delegates to the covered quality scripts. |
| `scripts/install-git-hooks.sh` | JavaScript/Rust coverage | Thin one-time Git configuration launcher; its behavior is visible and relies on Git's exit status. |
| CLI-bootstrap conditionals in `scripts/*.mjs` | JavaScript coverage | One-line process-entry checks delegate directly to separately tested injectable command functions. |

Rust's LLVM coverage calls its executable-source unit a **region**, rather than a JavaScript statement. The quality gate uses Rust region coverage as the statement-equivalent metric and enforces the same 80% floor.

When a metric has no executable items (for example, a package without a conditional branch), its coverage is treated as 100%. Once executable items exist, the gate calculates and enforces the metric normally; missing instrumentation is still an error.
