# Source structure gate

Clean Code review is a judgment exercise, but it must not let a large file pass without scrutiny. The repository therefore enforces a maximum of 400 meaningful lines for every changed production or test source file under `src/`, `src-tauri/src/`, and `scripts/`. Meaningful lines exclude blank lines and whole-line comments.

The limit is a navigation and cohesion guard, not a license to fragment code mechanically. A file that approaches the limit must be organized around independently understandable responsibilities: for example, plan validation, launch scheduling, public types, and focused test scenarios belong in separate modules when that makes them easier to find and change.

`npm run structure:check` checks the current working-tree change; CI checks every pull-request change against its base branch. The only temporary exceptions are listed in `code-structure-exceptions.json`. Each needs an owner work item and expiry date. No new exception is permitted without an accepted ADR and product-owner approval; expired entries fail the gate.

The current exceptions are inherited modules that predate this gate. `QUAL-004` removes every one of them by splitting code without changing behavior. Until then, any AI or human who touches one must keep the exception current and should move the relevant responsibility into a focused module instead of growing the legacy file.
