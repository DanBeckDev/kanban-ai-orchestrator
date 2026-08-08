# ADR 0006: External, recoverable task worktrees

- Status: Accepted
- Date: 2026-08-08

## Context

Worker agents must edit isolated task copies without accidentally changing a user's base repository or following a symlink outside the approved project boundary. Provisioning can be interrupted after a branch or directory has been created, so repeating a request must be safe. Projects also differ in whether dependencies should be installed independently, use a managed cache, or use an explicitly approved link.

## Decision

- A workspace manager receives the declared project repository root and a separately declared workspace root. It resolves existing ancestors before creating the root and rejects any overlap with the repository, including through symlinks.
- It derives a task branch as `kanban/<safe-work-item-id>` and always uses the project's declared `base_ref`; a task request cannot override that ref. Manager initialization requires that the declared base resolves to a commit before it creates a workspace root. Safe IDs use portable ASCII directory characters and reject Windows device names even on another operating system.
- Provisioning inspects Git's registered worktrees. A matching root and branch is reused; an unregistered task branch can be reattached after interruption only when it still resolves to the declared base commit. Conflicting or divergent branches, non-empty directories, and symlink targets fail closed. The manager removes only the exact, empty derived task directory to recover an interrupted setup; it never uses Git force flags or removes user content.
- The manager alone constructs workspace assignments. Before an execution launches, it verifies that the execution's work item and path match the assignment, that the assignment belongs to this manager's root, and that Git reports the expected root and branch.
- Worker path authorization allows only the assigned worktree. Writes to the base repository and reads or writes to every undeclared path are denied. Git is invoked through structured process arguments, never a shell string, and clears inherited Git environment variables and configuration injection that can redirect repository, worktree, index, object, namespace, or discovery context.
- Dependency sharing is declared as `isolated_install`, `managed_shared_cache`, or `explicit_project_approved_link`. This milestone records the strategy but does not create a symlink; project-specific link materialization requires a recorded approval and compatibility check.

## Consequences

- A recoverable-but-ambiguous directory or branch conflict becomes an actionable error instead of an automatic cleanup. Users retain control over any non-empty data.
- Workspaces consume additional disk space and projects must select a sharing strategy explicitly. No project is silently placed into a symlink-based dependency mode.
- The Git CLI remains a small outer boundary around a policy-focused manager, so future platform-specific Git integration can replace that boundary without moving safety rules into UI or agent adapters.
