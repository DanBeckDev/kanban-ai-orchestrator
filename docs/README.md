# Documentation map

This documentation is deliberately small, versioned, and task-oriented. It is designed to retain product context between incremental implementation sessions.

| Location | Purpose | Update when |
| --- | --- | --- |
| `product/vision.md` | Long-lived product intent and boundaries | The target user or product promise changes |
| `product/requirements.md` | Testable product requirements | A user-visible behavior changes |
| `product/user-experience.md` | Interaction model, onboarding, information architecture, and usability scenarios | Changing a user journey, navigation, or UI priority |
| `product/ux-prototype.md` | Low-fidelity later-map prototype for the next UX slice | Testing or changing the first-use, returning-user, or first-plan flow |
| `product/ux-validation.md` | Current-journey audit, moderated-study protocol, and anonymized findings record | Running or recording UX-008 validation |
| `architecture/overview.md` | System components, authority boundaries, and data model | A component or interface changes |
| `integrations/cline-pass.md` | ClinePass worker setup, credential boundary, and supported profile options | Changing the ClinePass adapter |
| `integrations/linear.md` | Linear connector contract and sync rules | Linear-facing behavior changes |
| `quality/code-requirements.md` | Mandatory code quality, remediation, coverage, and verification policy | Any quality gate or coverage rule changes |
| `quality/review-receipt.template.yaml` | Required evidence format for code-bearing work | The quality evidence contract changes |
| `quality/reliability.md` | Failure modes, reliability targets, and test scenarios | A risk or quality bar changes |
| `quality/platform-release.md` | Cross-platform CI and release-specific evidence | Modifying packaging, credentials, or provider process behavior |
| `planning/roadmap.md` | Phase outcomes and release gates | Sequencing changes |
| `planning/backlog.yaml` | Smallest executable work packages and dependencies | Starting, completing, splitting, or blocking work |
| `decisions/` | Immutable records of consequential decisions | A durable technical choice is made |

## How to use the backlog

`backlog.yaml` is the operational plan until the product can manage its own work. A task may start only when its `depends_on` tasks are done or its documented exception is accepted. Each task includes acceptance criteria that must be demonstrable, not merely claimed by an agent.

When the app is ready, this file can be imported into its own board and then maintained there. Keep a repository export so the build remains understandable without access to an external service.
