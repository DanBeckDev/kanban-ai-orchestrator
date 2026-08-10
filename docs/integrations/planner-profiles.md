# Planner profiles

## Purpose

A planner profile connects a local AI tool to the board's **Generate a proposal** action. The normal Settings path can create a safe native Codex, Claude Code, or Cline/ClinePass orchestrator profile with one action. The advanced bridge remains provider-neutral: it can call a self-hosted model or internal service. Every route can return only an unconfirmed proposal.

## Configure a profile

In the desktop board, select an installed provider for the orchestrator role to use the normal safe path. It uses the provider default model unless you deliberately choose a named model, and maps the provider-neutral effort choice inside the native adapter. It never asks for a program, arguments, credentials, or approval flags.

Advanced setup saves a bridge with a name, program, and one argument per line. The app starts that program directly with the selected project's repository as its working directory; it does not build or evaluate a shell command. The bridge reads standard input and writes standard output.

Treat a profile program as trusted local code. It runs with your user account's normal permissions; the generic profile mechanism is not a filesystem sandbox. Do not put API keys, access tokens, or other credentials in its arguments: the profile configuration is stored in the local board database. Configure credentials using the provider's normal local credential mechanism instead.

## Bridge contract

The app writes one JSON object followed by a newline to a bridge's standard input. It has this shape:

```json
{
  "goal": "Plan a dependable local-first planning workflow.",
  "outputContract": "Return exactly one JSON object with workItems, dependencies, and unresolvedAssumptions."
}
```

The bridge must write exactly one JSON object to standard output—no Markdown fences, prose, logging, or additional fields. Native adapters send an equivalent constrained prompt through their documented non-interactive protocol and extract only its final JSON response. A valid response looks like this:

```json
{
  "workItems": [
    {
      "key": "planner-contract",
      "title": "Define the planner contract",
      "description": "Specify the request, response, and error boundary.",
      "acceptanceCriteria": [
        "The contract is documented and tested."
      ],
      "budget": {
        "maxAgentTurns": 8,
        "maxDurationSeconds": 1800,
        "maxCostMicros": 250000
      },
      "requiresHumanReview": true
    },
    {
      "key": "planner-ui",
      "title": "Show the plan preview",
      "description": "Present the generated proposal before creating work.",
      "acceptanceCriteria": [
        "A user can inspect and confirm the exact proposal."
      ]
    }
  ],
  "dependencies": [
    {
      "upstreamKey": "planner-contract",
      "downstreamKey": "planner-ui",
      "kind": "blocks",
      "reason": "The interface depends on the agreed contract.",
      "owner": "orchestrator",
      "nextAction": "Complete and review the contract task."
    }
  ],
  "unresolvedAssumptions": [
    "The selected provider is available locally."
  ]
}
```

Every work item needs a unique non-blank `key`, title, description, and at least one non-blank acceptance criterion. Dependencies must reference those keys and include a reason, owner, and next action. The only accepted budget fields are `maxAgentTurns`, `maxDurationSeconds`, and `maxCostMicros`; every other field is rejected. `requiresHumanReview` defaults to `true`.

The app accepts at most 50 work items, 100 dependencies, 50 assumptions, 8,000 bytes of goal text, 65,536 bytes of response text, and 45 seconds of direct-child runtime. It discards standard error and never persists raw goal text or raw model output.

## Safety and confirmation

The model cannot select a board, create identifiers, set lifecycle state, assert a confirmation, or launch a worker. The daemon derives those values, validates the typed proposal, and shows the existing task/dependency graph, critical path, parallel stages, budgets, and assumptions. A person must explicitly confirm that exact proposal before any task or dependency is stored.

Malformed, oversized, invalid, or unknown-field responses leave the board unchanged and return an error that can be shown to the user. See [ADR 0015](../decisions/0015-bounded-planner-profile-boundary.md) for the enduring boundary decision.
