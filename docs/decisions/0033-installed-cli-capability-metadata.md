# ADR 0033: Read documented installed-CLI capability metadata

- Status: Accepted
- Date: 2026-08-10

## Context

ADR 0032 correctly removed the second, app-owned API-key connection. Its first
adapter obtains an account-specific Codex model catalogue through Codex's local
app server. During live testing, however, an installed Claude Code card could
only show **Provider default**. Its official CLI has no local account catalogue
endpoint, but its own `--help` command advertises the model aliases and effort
levels that its installed version accepts.

Leaving the card with empty selectors is worse than either deliberate option:
it does not let a person choose the official CLI selections that the app will
pass at execution time, and it incorrectly suggests they must configure a
second API account.

## Decision

- A provider adapter may read **documented, local, read-only CLI capability
  metadata** when that is the provider's supported way to advertise its model
  aliases or effort values. It remains behind the same
  `ProviderModelCatalogClient` boundary as SDK and app-server discovery.
- The call must use a non-session command, send no input, discard stderr,
  bound process duration and stdout size, and never inspect credential files,
  settings files, keychains, or private account data.
- Claude Code's adapter invokes only `claude --help`. It extracts only the
  aliases stated in that output and only its stated effort tokens. It does not
  invent full model identifiers or claim that every alias is entitled for the
  person's account; the installed Claude Code runtime remains the authority at
  execution.
- The durable effort preference preserves every displayed native value. Native
  adapters pass it through without reducing `xhigh`, `max`, or `ultra` to a
  different level.
- The settings card shows model and effort selectors only after a runtime has
  supplied selectable capabilities. Otherwise it presents one clear
  **Provider default** state, with a recovery action for any saved override.

## Consequences

- No provider API key, account duplication, or direct REST catalogue is added.
- The app can offer a usable Claude Code chooser today, while retaining a
  future account-specific SDK/app-server catalogue when the provider exposes
  one.
- UI copy says the installed runtime supplied the choices; it does not promise
  account-specific availability when the provider only advertises aliases.
