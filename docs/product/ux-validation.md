# UX-008 journey-validation record

- Status: Study prepared; participant walkthroughs pending
- Date: 2026-08-09
- Scope: first use, returning use, and first plan; no production source changes

This file separates what the implementation currently does from what people
actually need. The current-state audit is an implementer baseline, **not user
research**. Findings become evidence only after an opt-in representative
walkthrough is completed and minimally recorded below.

## Current journey (“now map”)

Source inspection on 2026-08-09 provides this baseline. It is linked to the
current React entry and workspace components so later observations can confirm or
contradict the assumptions rather than inherit them as facts.

| Activity | Current path | Observed pain / likely question | Workaround or consequence | Evidence |
| --- | --- | --- | --- | --- |
| Find work | Enter an existing board ID in `Open an existing board` | “Which ID is mine, and where do I find it?” | Store or obtain an opaque ID outside the app | `src/board/BoardSetup.tsx` |
| Set up safely | Choose a project folder, accept a feature branch as the suggested starting point, then scan technical setup details | “Why is that branch recommended, and what does the policy setting mean?” | Override a value without context, or scroll around a form that should be simple | `src/board/BoardSetup.tsx` |
| Describe an outcome | Open a board, then find the proposal panel among many side-panel forms | “Do I need to configure an agent, Linear, a task, and a dependency before I can plan?” | Interpret a dense configuration surface as the onboarding path | `src/board/BoardView.tsx` |
| Decide the order | Read cards and dependency form in the board workspace | “What is blocked, why, and what can safely run together?” | Infer from task data and dependency notation | `src/board/BoardView.tsx` |
| Supervise work | Use cards and execution controls after profiles are configured | “What needs my attention right now?” | Scan board columns and configuration panels | `src/board/BoardView.tsx` |
| Review or recover | Use task-card controls and review/evidence interfaces | “Is this done, waiting, failed, or safe to retry?” | Inspect implementation-level state and evidence controls | `src/board/WorkItemCard.tsx` |

### Assumptions to test, not conclusions

| Hypothesis | Risk if wrong | Validation signal |
| --- | --- | --- |
| A named recent-board library lets returning developers resume without external notes. | People may instead use repository context, branch, issue tracker, or team cues. | Participant finds the intended board and explains why they selected it. |
| A native repository chooser feels safer and clearer than a typed path. | The chooser may make repository-root status or cancellation less understandable. | Participant selects a valid repository, recovers from cancellation or invalid selection, and explains persistence state. |
| Outcome-first planning makes the next action obvious. | People may need task-level control or profile context earlier. | Participant finds proposal creation and correctly predicts that no worker has started. |
| An automatic primary starting point removes a normal-user decision while retaining a deliberate plain-language override. | A fallback could select the wrong line for an unusual repository. | Participant creates a board without Git vocabulary, then finds and explains the override when given a team-specific scenario. |

## Moderated walkthrough protocol

### Participants

Recruit three developers who are plausible users of an agent-coordinated local
desktop tool. Seek variation rather than statistical representation:

1. A developer who has coordinated multiple repository tasks or AI agents.
2. A developer setting up a local project/workflow for the first time.
3. A keyboard-first or assistive-technology user where possible. If that is not
   possible in the initial three, record the limitation and schedule a dedicated
   accessibility walkthrough before UX-005 is considered complete.

Use a throwaway sample repository and fictitious board names. Do not ask for or
record credentials, real repository paths, private diffs, tickets, agent
transcripts, or screen recordings without explicit separate consent. Participation
is opt-in and a participant may skip any question or stop at any time.

### Moderator opening

“We are testing the flow, not you. Please think aloud where comfortable. This
is an early text prototype, so say what you would expect to happen. We will not
record private project data; please use the sample scenario rather than your own
repository. You can stop or skip any question.”

Do not teach the desired sequence before a participant attempts it. Ask neutral
follow-ups such as “What are you looking for?”, “What do you expect next?”, and
“What makes you say that?” Avoid “Wouldn’t you click…?” or defending a design.

### Scenario A — return to work

**Set-up:** “Yesterday you used a board called Website reliability for a local
repository. It has two decisions waiting and one active agent. Today the folder
for a different board is no longer where the app expects it.”

**Tasks:** Open Website reliability; describe why you chose it and what the
empty outcome-planning state means; then explain what you would do with the
unavailable board.

**Success without moderator assistance:** Select the intended board from visible
recognition cues, identify the attention summary and first outcome action, and
choose or describe the Locate/Retry recovery path without requesting or
inventing an ID. If the application encounters an unexpected rendering failure,
the participant sees a clear Try again state rather than an empty window.

### Scenario B — create safely

**Set-up:** “You want to coordinate work in a sample project. Use the project
chooser, name the board, and create it. Then imagine you selected a folder
inside the project rather than the project folder itself.”

**Tasks:** Create the board; explain what is created and what has not started;
recover from invalid selection or chooser cancellation. Then imagine the team
asks for a different starting point and find the optional control.

**Success without moderator assistance:** Uses the picker rather than seeking a
path field, identifies the editable board name and the immediate result of
creation without needing Git or policy knowledge, finds the intentional override
when needed, and states that no partial board or AI work exists before the
explicit create step.

### Scenario C — make the first plan

**Set-up:** “The board is new. Your desired outcome is: reduce failed
deployments by making the release check reliable.”

**Tasks:** Find the next action; enter or describe the outcome; create a
proposal; identify what to inspect and when an agent could start; choose whether
to confirm, revise, or reject the shown proposal.

**Success without moderator assistance:** Finds outcome planning before manual
task configuration, says the proposal is not yet running work, and identifies
tasks/dependencies/assumptions/budgets as review information before confirmation.

### Keyboard and assistive-technology additions

Ask the keyboard-first/AT participant to use their normal configuration. Observe
whether focus location, labels, state announcements, and errors are predictable.
Do not prescribe a technique. Record functional barriers, not medical details or
tool configuration beyond what the participant volunteers as relevant.

### Facilitator capture sheet

Copy this short form once per participant. It deliberately captures behaviour
and decisions rather than a person's identity, private project data, screen
recording, or a verbatim transcript. A participant can omit any answer.

```text
Study ID: P-01 / P-02 / P-03
Relevant working style volunteered by participant: keyboard-first / assistive technology / neither stated

Scenario A — return to work
- Completed without moderator assistance: yes / partly / no
- Recognition cue used to choose the board:
- What the participant believed the attention summary and first action meant:
- Recovery choice for unavailable board:

Scenario B — create safely
- Completed without moderator assistance: yes / partly / no
- What the participant believed Create board would do:
- Invalid-folder or cancelled-picker recovery:
- Could they find and explain the optional different starting point: yes / no

Scenario C — make the first plan
- Completed without moderator assistance: yes / partly / no
- First action the participant found:
- What they believed proposal confirmation would do:
- What they chose to inspect, revise, reject, or confirm:

Observed confusion or barrier (one row per finding)
- Finding:
- Severity: critical / major / refinement
- Repeated in another study: yes / no / not known
- Proposed disposition and linked backlog item:
```

Record only the condensed findings and dispositions in the table below. The
facilitator notes remain outside the repository unless the participant has
explicitly consented to sharing them.

## Decision framework

| Category | Prototype decision | Rationale to test |
| --- | --- | --- |
| Safe local default | Suggested board name from repository folder | Easy to edit; does not change execution behaviour. |
| Safe local default | Resolved project starting point and standard safety policy | Removes a normal-user decision; the narrowly scoped override is discoverable only when needed. |
| Generated implementation detail | Project and board identifiers | Required by the system, but not a normal-user decision. Retain only in support details. |
| Explicit confirmation | Create board | Creates durable local records; before confirmation the app persists nothing. |
| Explicit confirmation | Confirm plan | Materialises the inspected local tasks/dependencies exactly once; it does not itself start a worker. |
| Intentional advanced control | Different starting point | Needed by some teams, but distracting in ordinary local setup. Reveal with effect text. |
| Intentional deferral | Linear connection and graph visualisation | Useful later, but neither is required to form a valid local board or understand the first plan. |

If testing shows a proposed default can cause a surprising repository, policy, or
provider effect, change it to an informed choice or confirmation before
implementation. Defaults never authorise agent execution, external updates, or
weaken daemon policy.

## Findings log

Complete one minimally anonymized entry per participant. Use a neutral study ID
only. A repeated or critical finding must have a disposition before UX-008 can be
marked done; create a linked backlog item for a product/engineering change.

| Study ID | Scenario completion | Observed confusion, question, or barrier | Severity | Proposed disposition | Linked backlog item | Status |
| --- | --- | --- | --- | --- | --- | --- |
| Pending-01 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-02 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-03 | Pending | — | — | — | — | Awaiting walkthrough |

Severity definitions:

- **Critical:** prevents creating/reopening a board, knowing whether work is
  safe to start, or finding a required review/recovery action.
- **Major:** causes a wrong expectation, an avoidable workaround, or sustained
  confusion in a core scenario.
- **Refinement:** does not block the scenario, but reduces clarity, confidence,
  accessibility, or efficiency.

## Completion gate

UX-008 is complete only when three walkthroughs are recorded, including a
keyboard-first or assistive-technology participant where possible; every critical
or recurring finding has an explicit disposition; and required follow-up work is
represented in the backlog. Until then, UX-002 and later production UI work stay
blocked by this task.

## Research basis

BookCtx — _User Story Mapping_, Jeff Patton, “Validate the Problem” (chunk 5)
frames product ideas as hypotheses and recommends sketches, scenarios, and user
testing as a lower-cost learning loop before full implementation. BookCtx —
_User Story Mapping_, Jeff Patton, “We’re Wrong Most of the Time” (chunk 14)
warns that a polished prototype is not validation; it requires real people and
explicit risky assumptions. These are the reasons this record distinguishes code
audit from participant evidence and leaves the findings table pending rather than
fabricating results.
