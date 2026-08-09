# ADR 0020: Validate repository setup before atomically creating a local board

- Status: Accepted
- Date: 2026-08-09

## Context

Normal board creation must be understandable without internal IDs, but a board
still needs a canonical Git-root path, a valid worktree base reference, and
generated project and board identifiers. The native directory picker is a
cross-platform interaction boundary, while repository inspection runs external
Git commands and local-board creation mutates SQLite state. Holding the daemon
mutex during Git I/O would violate ADR 0012 and could block unrelated board
operations. Splitting project, board, and recency writes could also leave an
unopenable partial board after a failure.

## Decision

- Use Tauri's native dialog plugin with only `dialog:allow-open` capability.
  The UI can select one directory but receives no general filesystem permission.
- Inspect and revalidate the chosen directory in Rust before taking the local
  board-service mutex. The path must be the Git repository root; the selected
  or detected base reference must resolve to a commit. The checked-out branch
  is the visible default, including `HEAD` for a detached checkout.
- Generate project and board UUIDs inside the local service rather than accept
  them from the normal UI.
- Persist the project, board, and initial board-access record in one SQLite
  transaction, then load the created board snapshot. A cancellation, failed
  inspection, invalid reference, or failed transaction leaves no board state.
- Keep base reference and policy overrides optional at the typed command
  boundary. The normal UI exposes them only in collapsed Advanced setup; the
  command defaults policy to `standard` when invoked without an override.

## Consequences

- The first-board flow works naturally on macOS, Windows, and Linux without
  duplicating setup logic or widening browser authority.
- The service mutex protects only short local state changes, not filesystem or
  Git work; commands must continue to revalidate external inputs before lock
  acquisition.
- Generated identifiers stay available in collapsed support details for
  diagnostics but cannot become a normal-user prerequisite.
