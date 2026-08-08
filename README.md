# Kanban AI Orchestrator

A local-first, cross-platform desktop application for planning, coordinating, and reviewing work performed by many AI coding agents.

The project is intentionally provider-neutral: an orchestrator plans and schedules work, while each task runs through an isolated worker-agent adapter in its own Git workspace. Linear is a first-class planning-system integration, rather than a one-way export.

## Start here

- [Product vision and principles](docs/product/vision.md)
- [Product requirements](docs/product/requirements.md)
- [System architecture](docs/architecture/overview.md)
- [Linear integration](docs/integrations/linear.md)
- [Reliability requirements](docs/quality/reliability.md)
- [Roadmap](docs/planning/roadmap.md)
- [Dependency-aware backlog](docs/planning/backlog.yaml)
- [Agent working agreement](AGENTS.md)

The documents above, together with accepted architecture decisions in `docs/decisions/`, are the source of truth. Code, tests, and the backlog must be kept consistent with them.

## Development quality gate

Install the repository hook once with `./scripts/install-git-hooks.sh`. Before handing off code-bearing work, run `npm run quality:verify` and add a completed receipt under `docs/quality/reviews/`. The hook rejects staged code without a valid receipt; CI reruns the full gate. Configure the `quality` workflow as a required status check when the GitHub remote is created.
