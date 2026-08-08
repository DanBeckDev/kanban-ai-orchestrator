# Source structure gate

Clean Code review is a judgment exercise, but it must not let a large file pass without scrutiny. The repository therefore enforces a maximum of 400 meaningful lines for every production or test source file under `src/`, `src-tauri/src/`, and `scripts/`. Meaningful lines exclude blank lines and whole-line comments.

The limit is a navigation and cohesion guard, not a license to fragment code mechanically. A file that approaches the limit must be organized around independently understandable responsibilities: for example, plan validation, launch scheduling, public types, and focused test scenarios belong in separate modules when that makes them easier to find and change.

`npm run structure:check` checks the current working-tree change. `npm run structure:verify` scans the entire repository, including untracked source files; it runs in every full-quality verification. The exception ledger is intentionally retained as an auditable empty record, but any entry fails the gate. An agent must split the code instead of recording an exception.
