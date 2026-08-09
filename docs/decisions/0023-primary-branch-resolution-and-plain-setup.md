# ADR 0023: Resolve the project starting point without exposing Git mechanics

- Status: Accepted
- Date: 2026-08-09

## Context

The first repository-first setup flow showed a tall card containing “Base
branch”, “Policy: Standard”, and a generic Advanced setup disclosure. The
implementation also used the selected directory's checked-out branch as the
suggested base reference. A person selecting a repository from a feature
worktree could therefore see that feature branch presented as the recommended
starting point, even when the project's primary line was `main`.

Those facts are needed by the workspace manager, not by someone creating their
first board. They add scrolling and force Git and policy knowledge on people
who may be coordinating work through natural-language coding tools rather than
managing a repository directly. BookCtx — *The Product-Minded Engineer*,
“Turn Unknown Unknowns into Known Unknowns” and “Interaction Design” supports
safe predictable defaults on the normal path, precise names for optional
controls, and deliberate friction only for less-common or higher-risk choices.

## Decision

- The normal board-creation path asks only for a project folder and editable
  board name. It does not display base-reference or policy terminology.
- Repository inspection resolves the workspace starting point from a configured
  remote's locally known `HEAD` reference when present. When that reference is
  unavailable, it prefers an existing local `main`, then `master`, then
  `trunk`; only then does it fall back to the checked-out branch. This requires
  no network access and the selected result remains revalidated before durable
  board creation.
- The daemon keeps ownership of the standard policy default. The normal UI
  neither displays nor accepts a policy identifier.
- A closed **Use a different starting point** control contains the base-reference
  override. Its help text says that Kanban normally selects the project's main
  line of work and that people should change it only when their team requires a
  different starting point.
- The normal selected-project state must fit, without vertical scrolling, in a
  720px-tall desktop viewport. Expanding the intentional override may add
  vertical content; that is a deliberate advanced action.

## Consequences

- First-time and vibe-coding users can create a valid board without knowing Git
  branch or policy concepts, while experienced users retain a narrowly scoped
  override.
- A feature checkout is no longer presented as the recommended base merely
  because it happens to be the current branch.
- The existing validation-before-lock and atomic-creation protections from ADR
  0020 remain unchanged. This decision supersedes ADR 0020 only where it made
  the checked-out branch visible by default and exposed a policy override in
  the ordinary setup UI.
