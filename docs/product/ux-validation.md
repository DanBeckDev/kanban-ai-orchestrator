# Representative developer validation study kit

- Status: Product-owner wireframe handoff accepted; study ready for UX-007 and UI-004; no participant results recorded
- Date: 2026-08-10
- Scope: first use, returning use, planning, dependencies, recovery, and visual/accessibility validation; no production source changes

This file separates what the implementation currently does from what people
actually need. The current-state audit is an implementer baseline, **not user
research**. On 2026-08-09, the product owner supplied and clarified the intended
four-screen wireframes; their decisions are recorded in [Approved wireframe
specification](approved-wireframes.md). They are an implementation handoff, not
representative-user evidence. The study protocol below moves to UX-007, after a
current build can support an honest end-to-end walkthrough.

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

Recruit five developers who are plausible users of an agent-coordinated local
desktop tool. Seek variation rather than statistical representation:

1. A developer who has coordinated multiple repository tasks or AI agents.
2. A developer setting up a local project/workflow for the first time.
3. A developer who often creates software through AI-assisted or low-code/vibe
   coding workflows and may not know Git terminology.
4. A keyboard-first or assistive-technology user where possible.
5. A developer who regularly reviews or coordinates other contributors' work.

If a keyboard-first or assistive-technology user is unavailable, record that
limitation honestly and schedule a dedicated accessibility walkthrough. Do not
replace it with an implementer self-review.

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

### Scenario D — understand a blocker

**Set-up:** “The sample board has a task called Publish release notes waiting
for Design the release checklist. Another task, Update internal documentation,
can continue safely.”

**Tasks:** Use the board view menu to find why Publish release notes is waiting,
identify what it affects, and decide what could progress while the blocker is
unresolved. Open the related task only if it helps the decision.

**Success without moderator assistance:** Finds the dedicated Dependencies view,
uses the selected-task explanation or accessible list rather than inferring a
line direction or colour, correctly identifies the upstream work and next
action, and can name one parallel-safe task.

### Scenario E — recover safely

**Set-up:** “The sample board contains a task returned from review and a
separate Linear comment whose delivery result is unknown. The facilitator has
checked these are visible before the participant begins.”

**Tasks:** Explain what has and has not happened, identify the next permitted
action for each item, and say whether the app will automatically retry or send
anything.

**Success without moderator assistance:** Does not mistake either state for
completion, finds the named recovery or inspection route, and states that a
new public update requires an explicit person decision after checking Linear.

### Keyboard and assistive-technology additions

Ask the keyboard-first/AT participant to use their normal configuration. Observe
whether focus location, labels, state announcements, and errors are predictable.
Do not prescribe a technique. Record functional barriers, not medical details or
tool configuration beyond what the participant volunteers as relevant.

## UI-004 visual and assistive-settings addendum

Use the same current desktop build and anonymized study IDs. These checks assess
the product as it is used; automated tests, a design review, or the moderator's
opinion do not count as participant evidence.

For at least three of the five participants, ask neutral comprehension questions
before explaining the interface:

1. “What product are you in, and what do you think this first screen is for?”
2. “How would you change the appearance, and what do you expect to change?”
3. “What would you do first to start or continue work?”

Then have each participant visit the board library, workspace setup, Workflow,
Dependencies, Settings, and task detail in both Dark and Light appearance.
Record whether keyboard focus is visible, each control has a usable name, and
task/status meaning remains understandable without colour.

Run one dedicated forced-colours or operating-system high-contrast walkthrough
on a supported desktop platform. Confirm that controls, boundaries, focus, and
warning/success/status meaning remain visible with the authored palette
overridden. If that environment cannot be supplied, record it as an open UI-004
constraint; do not mark the criterion complete.

For a screen-reader walkthrough, let the participant use their normal setup.
Capture only the affected screen, control, state, and outcome—not personal
assistive-technology settings or a transcript. A failure to reach, identify, or
operate a required control is critical when it prevents the scenario.

### Facilitator capture sheet

Copy this short form once per participant. It deliberately captures behaviour
and decisions rather than a person's identity, private project data, screen
recording, or a verbatim transcript. A participant can omit any answer.

```text
Study ID: P-01 / P-02 / P-03 / P-04 / P-05
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

Scenario D — understand a blocker
- Completed without moderator assistance: yes / partly / no
- Upstream blocker and next action identified:
- Parallel-safe work identified:

Scenario E — recover safely
- Completed without moderator assistance: yes / partly / no
- What the participant believed had happened remotely:
- Recovery action and reason:

Visual and assistive settings (at least three participants)
- Product name and first-screen purpose understood: yes / partly / no
- Appearance control and expected effect understood: yes / partly / no
- First next action understood: yes / partly / no
- Dark and Light appearance result:
- Keyboard/screen-reader or high-contrast result, if applicable:

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

Complete one minimally anonymized entry per participant during UX-007. Use a
neutral study ID only. A repeated or critical finding must have a disposition;
create a linked backlog item for a product/engineering change.

| Study ID | Scenario completion | Observed confusion, question, or barrier | Severity | Proposed disposition | Linked backlog item | Status |
| --- | --- | --- | --- | --- | --- | --- |
| Pending-01 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-02 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-03 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-04 | Pending | — | — | — | — | Awaiting walkthrough |
| Pending-05 | Pending | — | — | — | — | Awaiting walkthrough |

Severity definitions:

- **Critical:** prevents creating/reopening a board, knowing whether work is
  safe to start, or finding a required review/recovery action.
- **Major:** causes a wrong expectation, an avoidable workaround, or sustained
  confusion in a core scenario.
- **Refinement:** does not block the scenario, but reduces clarity, confidence,
  accessibility, or efficiency.

## Completion gate

UX-008 is complete because the product owner provided a clear visual direction,
clarified the material interaction decisions, and the resulting implementation
work is represented in the backlog. It does **not** claim usability validation.
UX-007 owns the five-person current-build walkthrough, scenario findings, and
release gate. UI-004 owns the theme and assistive-settings checks. Until both
have real participant evidence and any critical findings have a disposition, no
document may present this handoff as validated user experience.

## Research basis

BookCtx — _User Story Mapping_, Jeff Patton, “Validate the Problem” (chunk 5)
frames product ideas as hypotheses and recommends sketches, scenarios, and user
testing as a lower-cost learning loop before full implementation. BookCtx —
_User Story Mapping_, Jeff Patton, “We’re Wrong Most of the Time” (chunk 14)
warns that a polished prototype is not validation; it requires real people and
explicit risky assumptions. These are the reasons this record distinguishes code
audit from participant evidence and leaves the findings table pending rather than
fabricating results.

BookCtx — _User Story Mapping_, Jeff Patton, “Keep Talking as You Build” (chunk
11), argues that a meaningful flow is enough for users to complete a real goal
and give corrective feedback. Project inference: the five scenarios test whole
decisions—not isolated labels—and feed concrete findings back to named backlog
items with an owner and disposition.
