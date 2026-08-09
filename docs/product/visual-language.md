# Visual and content language

## Purpose

Kanban helps a developer plan, oversee, and review work done by AI agents. The
interface should make the next safe decision easy to find—not ask a person to
decode the local implementation.

## Product name

Use **Kanban** in visible product chrome, window titles, and document titles.
The repository, package, bundle identifier, and local data contracts retain
their technical names so this presentation decision does not change a durable
interface.

## Meaningful copy rule

Every persistent string earns its place by doing one of four jobs:

1. Orient: say where the person is or what they are looking at.
2. Act: name the specific action they can take now.
3. Explain: state what happened, why it matters, or what will happen next.
4. Support: provide a necessary diagnostic fact where it helps recovery.

Remove a string that does none of these jobs. Prefer familiar task vocabulary,
active voice, and a specific action. Put implementation terminology, IDs, and
raw configuration in support or advanced disclosure rather than first-use
chrome.

Examples:

| Avoid | Use when it is true |
| --- | --- |
| Execution authority | No first-screen label; the local daemon is explained only in support or an actionable error. |
| Current milestone | No first-screen label; show the person their board or a current action instead. |
| Local-first agent coordination | Plan & oversee agent work. |
| Continue | Open board, with an accessible name that includes the board name. |

## Theme and palette

Dark is the default appearance. A person can always select Dark or Light, and
the choice is saved locally on the device. The themes share semantic roles;
components use those roles rather than hard-coded colour values.

| Role | Visual intent |
| --- | --- |
| Background and surfaces | Calm slate structure with enough separation to read groups without nested boxes. |
| Primary and focus | Punchy iris for a clear next action and keyboard focus. |
| Success | Jade/green, always paired with a state label. |
| Warning | Amber, always paired with a state label and next action where one exists. |
| Destructive | Ruby/red, reserved for risk, failure, or irreversible actions. |

## Accessibility checks

- Normal text maintains at least 4.5:1 contrast with its actual background;
  large text maintains at least 3:1.
- The base token pairs are checked at or above 4.5:1: dark body text 10.14:1,
  dark secondary text 7.33:1, dark primary-action text 5.70:1, light body
  text 6.11:1, light secondary text 4.97:1, and light primary-action text
  5.15:1. Recheck a pair whenever either token changes.
- Never communicate status, validation, or selection with colour alone.
- Keep a visible focus indicator for keyboard users. Do not remove outlines
  without a clear replacement.
- Honour forced-colours and high-contrast settings by keeping native semantic
  controls, borders, text labels, and structure meaningful without the authored
  palette.
- Test long board names, repository names, task titles, and error text in both
  appearances.
