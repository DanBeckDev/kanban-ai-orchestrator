# ADR 0012: Use a typed local board command boundary

- Status: Accepted
- Date: 2026-08-08

## Context

The desktop foundation previously displayed static product metadata despite an existing Rust domain core. A board UI needs durable projects, boards, work items, and dependencies, but must not acquire authority to mutate task state or schedule work.

## Decision

- Store local board records beside the event store in the platform application-data directory.
- Define application request/response types and a repository port for board use cases.
- Let the SQLite adapter implement that port and validate project/board ownership, dependency identity, cross-board edges, and hard-dependency cycles.
- Expose only typed Tauri commands that delegate to the application service. Keep one mutex-protected local service per desktop process; commands are synchronous local transactions and never hold the lock across external I/O.

## Consequences

- The React board can reload a durable snapshot without duplicating domain validation or state-transition rules.
- Adapter replacement and isolated application-service tests remain possible.
- Multi-device collaboration and Linear synchronization must enter through separate validated adapter paths rather than writing the local board tables directly.
