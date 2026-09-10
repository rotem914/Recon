# Recon, build plan

The plan of record. Read it at task pickup, alongside `project-os/Backlog.md`.

Status: revision 2, after Rotem's correction pass. Waiting on part 3 only.
Written: 2026-09-10.
Product baseline: Rotem's product plan and Claude handoff dated 2026-09-10.

Nothing here is built yet. No code was run and no timing was measured in writing it.

Ten parts: the review of the product plan, the six decisions and their answers, what
must be settled before Stage 0 starts, the architecture, Stage 0, what Stage 0 may
conclude, Stage 1, the later stages, the evidence status of every claim here, and the
sources behind it.

### What changed in revision 2

F2 lost its blocking rating and its documentation claim: the elevated-window hotkey
limitation is now something to verify, not something known.

F3 gained the timestamp it was missing, so the second latency target can actually be
measured.

F11 flipped. Numbering gaps stay in every output, and renumbering is rejected, on
Rotem's argument that a shifting number breaks a reference already shared.

The coordinate contract in part 4 is now four named spaces instead of one slogan.

The fidelity tests in part 5 are three separate checks, because the old single hash
check was wrong: annotations change pixels inside the image area by design.

Stage 0 shrank to one capture path, two real destination applications, and no
packaging decision. Undo moved to Stage 1 in full.

Stage 1 no longer prepares anything for Rogers.

## Part 1: the review of the product plan

Verdict vocabulary per finding: **fix** (change the plan now), **drop** (never mind),
**backlog** (record it and move on). Only `backlog` writes a row to
`project-os/Backlog.md`.

**In one line.** The plan is unusually sound on product behavior and unusually thin on
the two mechanisms that decide whether that behavior is achievable: how the exported
image is produced, and how the Stage 0 numbers are measured.

Sixteen findings: two blocking, eight important, six small.
Nothing in them changes the product definition or the stage order.

### What is strong

Stage 0 as a real gate, with acceptance evidence and an explicit "these are proposed
targets, not validated promises". Most plans skip the gate and keep the promise.

The callout as one object, with selection, movement, deletion and undo required from
the first usable version. That is the correct primitive and the correct floor.

Idempotent Send: a persisted submission identifier reused on retry, a snapshot taken
at Send, and a refusal to read a timeout as either success or failure.

"Extend the canvas with a neutral margin rather than crop the captured pixels."
Non-obvious, and the right way round.

The focus discipline: never steal foreground on a background completion, and never
close work the user has moved on from.

Honest positioning against Snagit, including the refusal to claim missing features.
A plan built on a false premise fails later and expensively.

The out-of-scope list, and "no Done or Apply step" as an invariant rather than a
preference.

### Findings

----
**F1 · 🔴 Nothing guarantees the exported image matches the editor**

Where: §4.3 last line, §5 Image output.

The requirement is stated ("the editor and rendered output must agree on wrapping,
alignment, font rendering, and arrow positions") and no mechanism is named to keep
it true.

The specific risk being tested: an input layer for typing sits above the canvas that
composes the output, so the same note is laid out twice while the caret is live, and
Hebrew wrapping, font fallback and arrow geometry are where the two can disagree.
A shared wrapping function narrows that risk. It does not prove the two agree.

The failure is invisible in the editor: it appears only in the file the client gets.

Suggestion: one scene model and one composer, used for display and for export, plus
the three checks in S0.4, one of which exercises copying while the caret is still
active. The product forbids an Apply step, so that case is the contract, not an edge.

Carried into: part 4 boundaries, S0.4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F2 · 🟠 The elevated-window behavior is unknown, and the plan treats fast access as certain**

Where: §2 principle 1, §3.1, §3.2.

Revised: the earlier version of this finding claimed that Windows stops delivering a
registered global hotkey while an elevated window is in the foreground, and cited that
as documented. It is not. Registering a system hotkey, installing a keyboard hook,
injecting input and activating a foreground window are four different mechanisms, and
the documented restriction covers injection, not hotkey registration.

What remains: nobody here knows what happens when an application running as
administrator holds the foreground, and "fast access is a core feature" depends on the
answer.

Two cases, and they are not the same. An elevated application on the normal desktop is
a question to answer by test. The consent prompt's own secure desktop is out of scope
by design: nothing on the user desktop reaches it, and no product decision follows.

Suggestion: keep the test in Stage 0, record what it finds, and decide nothing about an
elevated launch mode until a problem is demonstrated.

Carried into: S0.6, part 9 under behavior to verify, decision 6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F3 · 🔴 Stage 0 asks for two latency numbers and the timestamps cannot produce one of them**

Where: §9 Stage 0 acceptance.

The second target is 500 ms from a completed selection to an editor ready for input, and
the earlier timestamp list had no selection-completed mark, so that interval could not be
computed at all. The drag itself is the user's time and must not be inside the number.

Visible is also not interactive. An overlay that has painted but is not yet taking the
mouse looks finished and measures fast.

Suggestion: six marks per run, and two reported intervals.

Marks: hotkey received · overlay displayed · overlay accepting input · selection
completed · editor displayed · editor accepting annotation input.

Intervals: hotkey received to overlay accepting input, and selection completed to editor
accepting annotation input. Everything else is diagnostic.

Report the raw runs and the percentile method, not only a summary: with thirty runs the
95th percentile is the second-worst run, so the maximum and the list carry more
information than the label does.

Carried into: S0.5.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F4 · 🟠 Clipboard formats are unspecified, so a passing paste test proves little**

Where: §5 Image output, §9 Stage 0 acceptance.

Windows applications disagree about image formats.
Some read only the older device-independent bitmap and lose transparency; others
prefer PNG; a few take only the plain bitmap handle.
Publishing one format satisfies whichever applications happened to be tested.

Suggestion: publish several formats in one clipboard operation, from native code rather
than through the web view API, and name the destinations by actual use.
Two real destinations are the acceptance boundary. A longer list is not a stronger test,
it is a longer test.

Carried into: part 4 split, S0.4, part 3 decision B.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F5 · 🟠 The selection overlay should not be a web view**

Where: §9 Stage 0, "evaluate Tauri + React + Rust".

The overlay is the latency-critical and DPI-critical surface, and a web view window
spread across mixed-scale displays adds cold start, scaling and input-transparency
problems that have nothing to do with the editor.

Judging one stack for both surfaces risks failing the whole stack for a reason that
belongs to one component.

Suggestion: split the architecture before Stage 0 starts.
Native code owns the hotkey, the freeze, the overlay and the clipboard; the web view
owns the editor only.
Stage 0 then measures the editor's fidelity instead of the capture path's viability.

Carried into: part 4, the whole of it.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F6 · 🟠 Text direction is named but not defined**

Where: §4.3.

"Support Hebrew, English, and mixed-direction text" leaves three things open: the
base direction of a bubble, its alignment, and which way it grows as text is added.

This is the highest-risk area for your own daily use, and it shapes the wrapping that
F1 depends on.

Suggestion: base direction per bubble from its first strong character, with a manual
override on the selected bubble.
Alignment follows base direction.
The bubble's anchor-side edge stays fixed while the opposite edge grows, so a note
never walks away from the thing it points at.

Carried into: S0.4, decision 1.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F7 · 🟠 Undo is specified by context, and the user cannot see the context**

Where: §5, the Undo/Redo row.

Context-scoped undo means the same keystroke does different things depending on state
nobody can see, and the first surprise costs a whole note instead of a word.

Suggestion: specify the observable behavior instead of the number of stacks.
While a note is being edited, undo works on the typing and never reaches into object
operations. Once editing ends, that text change takes its place in document history as
one grouped step.

Its acceptance sequence is in S1.3, and it is the test that settles this.

Carried into: S1.3, decision 2.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F8 · 🟠 The document format is deferred, but three stated requirements already depend on it**

Where: §6.1.

Autosave with no Apply step, recovery to the last successful save, and "no coupling to
a rendering library's serialization" cannot be judged while the shape is unnamed.

Suggestion, as a starting position: one folder per capture, holding the original
screenshot as a PNG that is never rewritten, plus a JSON document with a schema
version.
An index database, if any, holds pointers and never the images.
Writes go to a temporary file and are renamed into place, per `project-os/QA.md` §5.
Autosave is debounced while typing and forced on blur, navigation, hide and quit.

Carried into: part 4 split, part 4 coordinates, S1.4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F9 · 🟠 Ctrl+S sits in the v1 contract while file export is Stage 2**

Where: §5 table, against §9 Stage 2.

The shortcut table reads as the shipping contract, so Stage 1 promises a key that
does nothing.
The Rogers row is marked conditional; the export row is not.

Suggestion: add a stage column to the table, or mark the two deferred rows the way
the Rogers row is marked.

Carried into: S1.3.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F10 · 🟠 Returning focus to the previous application needs a named mechanism and a failure path**

Where: §3.1, §3.3.

Windows restricts which process may bring a window forward, and it may refuse the
request outright. Doing it next to the hide improves the odds and guarantees nothing.

Suggestion: store the target window at capture time, attempt activation on the way out
as best effort, and treat refusal as the plan's own fallback: hide without activating
anything, and never report a return that did not happen.
Test it in Stage 0 against an elevated window and against an application that has
since closed.

Carried into: S0.6, S1.6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F11 · 🟡 Numbering: gaps stay, renumbering is rejected**

Where: §4.3.

The original finding said that a feedback list jumping 1, 2, 4, 7 reads as a mistake to
the client, and suggested renumbering at send or export.

Rotem rejected that, and the argument is stronger than the finding: a number that
changes on export breaks every reference already made in the image, in a chat message
or in a Rogers task, and a second export could quietly change what a shared number
means.

Resolved: stable numbers everywhere, gaps included, identical in the editor, the
clipboard, the file and the Rogers text.
An explicit Renumber action stays a backlog idea and enters nothing now.
Recorded in `project-os/Decisions.md`.

Carried into: S1.2, decision 3.

Verdict: resolved, kept as the record.

----
**F12 · 🟡 "Keep forever" has an unstated cost**

Where: §6.3.

Thirty 4K captures a day is roughly 60 to 250 MB a day depending on content, so
"no expiry" is a real disk decision currently made without a number.

Suggestion: put the expected rate in the plan so the default is chosen knowingly.
Stage 2 already shows usage; revisit limits with data rather than a rule.

Carried into: part 8, Stage 2.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F13 · 🟡 In Stage 1, capture 1 is twenty-nine keypresses away**

Where: §6.2, §9 Stage 1.

The workload is defined as 20 to 30 captures with returns to earlier images, and
Stage 1 ships previous and next only.

Suggestion: add a position indicator and shortcuts to the first and last capture in
Stage 1. The thumbnail strip still waits for Stage 2.

Carried into: S1.5, decision 4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F14 · 🟡 Three different questions are being answered as one**

Where: §10 point 3.

A capture API's minimum Windows version, the versions Recon claims to support and
tests, and how the web view runtime reaches a user are three separate decisions, and
the plan treats them as one.

The per-display capture call documents Windows 10 version 1903 as its own minimum.
That is a constraint on the API, not a product support baseline, and not an installer
strategy.

Suggestion: state the API minimum where the API is chosen, state the supported and
tested versions as a product decision, and keep runtime packaging as a distribution
decision that does not block the experiment.
Recorded in `project-os/Decisions.md`.

Carried into: S0.6, decision 5.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F15 · 🟡 First run and the empty editor are missing**

Where: §3.1, §3.3.

For an open-source release the first two minutes are the product: no capture exists
yet, the hotkey may be taken, and the tray icon is undiscovered.

Suggestion: one empty-state line in the editor naming the current hotkey, and a
first-run check that reports a hotkey conflict rather than failing quietly.

Carried into: S0.1, S1.1.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F16 · 🟡 HDR is left as an open question**

Where: §9 Stage 0.

Tone mapping high dynamic range content is a project of its own, and leaving it open
invites Stage 0 to spend a day on it.

Suggestion: make "not supported in v1, and documented" an acceptable Stage 0 exit,
with detection so the output is never silently washed out.

Carried into: S0.6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

## Part 2: the six decisions, answered

Rotem's answers, as given. The rest of this plan follows them.

| Decision | Answer | Where it lands |
|---|---|---|
| 1 · Text direction | Detect from the first strong character, with a manual override. | S0.4, F6 |
| 2 · Undo | Specify observable behavior, not a stack count. Typing undoes typing; once editing ends, the text change joins document history as one grouped step. | S1.3, F7 |
| 3 · Numbering | Stable numbers and gaps in every output. Renumbering rejected. | S1.2, F11, `project-os/Decisions.md` |
| 4 · Stage 1 navigation | Position indicator, plus shortcuts to the first and last capture. | S1.5, F13 |
| 5 · Windows support | Separate the API minimum from the supported and tested versions. Runtime packaging is its own distribution decision. | S0.6, F14, `project-os/Decisions.md` |
| 6 · Elevated windows | Test first. No elevated mode on an unverified assumption. | S0.6, F2 |

## Part 3: what must be settled before Stage 0 starts

Three answers are needed to begin. Everything else in this plan can be resolved as the
step arrives.

**A · Which single capture path is built first.**
Stage 0 implements one, and a second only if the first demonstrably fails.
My recommendation: start with one copy of the whole virtual screen, because it is a
single synchronous call that already spans every display and negative coordinate, so it
retires the foundation risk at the lowest cost.
Move to the modern per-display capture API if that copy misses window content, mangles
a hardware-composited surface, or measures slow against the target in F3.
Say if you would rather start from the per-display API instead.

**B · Which two destination applications are the acceptance boundary for output.**
Name the two you paste into most. They decide S0.4, and I do not want to guess them.

**C · The reference environment for the fidelity checks.**
One machine, one display scale, and a pinned font for the reference images, so a
comparison failure means the renderer changed rather than the font did.
I propose your own main machine and its current scale, recorded in the S0.6 report.

Details that wait for their step: the exact document schema (S1.4), the placement
candidate set (S1.2), the shortcut table's stage column (S1.3), the storage rate note
(Stage 2), the Rogers submission record (Stage 4).

## Part 4: the proposed architecture

The product plan's Stage 0 says "evaluate Tauri + React + Rust". This splits that into
two questions, because the two surfaces fail for different reasons and only one of them
is in doubt (F5).

| Layer | Runs as | Owns |
|---|---|---|
| Host | native process | tray, preferences, global hotkey, window lifecycle, the return-focus target |
| Capture | native | freezing the desktop, the captured region, physical desktop coordinates |
| Overlay | native, one borderless window per display | the dim, the selection rectangle, cancel, handing the region to the host |
| Store | native | one folder per capture, the untouched original, the versioned document, atomic writes |
| Clipboard | native | several image formats in one operation, success reported before any hide |
| Editor shell | web view | toolbar, capture strip, destination indicator, keyboard routing |
| Scene and composer | web view | the one composer: display at the current zoom, export at original scale |
| Text editing | web view, an input layer above the canvas | caret, keyboard input, bidirectional typing, sharing the composer's wrapping |

Three boundaries hold this together, and each one exists to stop a specific failure:

The web view never touches the screen or the clipboard, so a web view limitation can
never break capture (F5, F4).

The host never renders an annotation, so there is one composer and the export is that
composer at original scale (F1).

The document never stores a screen coordinate, so it cannot be invalidated by moving a
monitor (below).

### Coordinate spaces, and the transforms between them

Four spaces, named, with the conversion written down at each boundary. One conversion
at the edges is not a guarantee against drift; naming the spaces is what makes a wrong
conversion findable.

| Space | Units | Used by |
|---|---|---|
| Desktop | physical pixels across all displays, negative positions included | the freeze, the overlay, the selected rectangle |
| Image | pixels local to the captured image, origin at its top-left, original scale | the stored document: anchors, bubbles, arrows, text boxes |
| Canvas | image space plus the annotation margin, as an offset record | the composer, the clipboard, the exported file |
| View | canvas space through zoom, pan and display scale | what the editor draws and what the pointer hits |

The conversions: desktop to image happens once, at capture, and is then discarded.
Image to canvas is the margin offset, stored as four edge values so a growing margin
never rewrites a single annotation coordinate. Canvas to view is the display transform,
never persisted. Nothing in the document refers to a display, a scale factor or a
monitor arrangement, so yesterday's capture opens the same way after the monitors move.

## Part 5: Stage 0, the smallest experiment that settles the foundations

Purpose: retire the risks that would change the architecture, on the smallest build that
can produce evidence. Six steps. Anything not on this list is not Stage 0, including
packaging, the second capture path, and full undo.

Each step carries a checkbox, a suggested model with a short reason, what it delivers,
and the evidence that closes it. The model is a suggestion for whoever picks the step
up, not a setting. A step is done when its evidence exists, not when its code runs.

----
**[ ] S0.1 · Host shell: tray, hotkey, quit**

Model: Sonnet 5. Mechanical, documented APIs.

Delivers: a background process with a tray entry, a configurable global hotkey, and a
quit that leaves nothing running.

Evidence: the hotkey fires while the process has never shown a window; a hotkey already
taken by another application is reported as a conflict rather than silently lost (F15);
quit leaves no process behind.

----
**[ ] S0.2 · Freeze, overlay, region selection**

Model: Opus 5. DPI, native APIs, irreversible shape.

Depends on decision A.

Delivers: the frozen desktop image by the chosen path, one overlay window per display,
drag to select, cancel, and the region handed back in desktop coordinates and converted
once into image space.

Evidence: two displays at 100% and at 150% or 200%, with the secondary placed left of
the primary so coordinates go negative; the selected rectangle and the resulting pixels
agree exactly; the overlay and the editor never appear in the output; cancel restores
the previous context and creates no capture; a second hotkey press during selection is
ignored; an open menu is either captured or the limitation is written down.

----
**[ ] S0.3 · Editor window, scene, one callout**

Model: Opus 5. The composition contract is the product's spine.

Delivers: the editor window pre-created and hidden at startup, one image on the canvas,
and one callout that can be created, typed into, committed, selected, moved and deleted.
The anchor moves independently and the arrow follows.

Evidence: every one of those operations exercised by hand; an empty bubble is discarded
when editing ends and never reaches the output.

Full undo and redo are Stage 1 (S1.3), not a Stage 0 gate.

----
**[ ] S0.4 · Output fidelity and the clipboard**

Model: Opus 5. Fidelity, and the one place work can be lost.

Depends on decisions B, C and 1.

Delivers: the composer used for both display and export, and a native clipboard
operation publishing several image formats.

Three separate checks, because one comparison cannot answer all three questions:

Original pixels survive. Export a capture with no annotations, decode it, and compare
its image area against the captured source at the margin offset. Encoding is lossless,
so this one is exact equality, not a tolerance.

Annotated output is correct. Compare six documents against reviewed reference images
covering Hebrew, English, mixed direction, a long wrapped note, an anchor against each
edge, and a document that added a margin. State the reference environment and the
comparison tolerance, since font rendering is what the tolerance is for.

The editor and the output agree while typing. Copy with the caret still active, mid
note, in each of the three text cases, and compare against the composed output. The
product forbids an Apply step, so this is the case that matters, and committed text is
the easy half (F1).

Plus: a paste verified in the two destination applications from decision B.

----
**[ ] S0.5 · Instrumentation and the thirty-run measurement**

Model: Sonnet 5. Mechanical, known shape.

Delivers: the six marks per run from F3, a CSV per run set, the two reported intervals,
and a memory sample that separates the live document from retained history.

Evidence: thirty runs at 4K with the process already in the background, reported as raw
runs plus the maximum and the percentile method, against the plan's proposed targets of
250 ms to a usable selection and 500 ms to an editor ready for input; the user's drag
time excluded from both intervals; process startup measured and reported separately.

----
**[ ] S0.6 · The limits, and the go or no-go report**

Model: Opus 5. Judgment on evidence.

Depends on decisions 5, 6 and C.

Delivers: what the platform actually did here, and one short report.

Evidence: the hotkey tested against a real elevated application on the normal desktop,
with the result recorded either way and no product decision attached to a guess (F2);
best-effort return-focus tested against that same window and against an application that
has since closed (F10); the API's own minimum Windows version stated separately from the
versions Recon supports and tests (F14); HDR behavior determined, with "not supported in
v1, detected rather than silently wrong" an acceptable answer (F16); the reference
environment recorded (decision C).

The report separates measured facts, documented behavior and assumptions, and recommends
keeping or replacing a named component rather than the stack.

## Part 6: what Stage 0 is allowed to conclude

Each failure has a next move, and none of them is a verdict on the whole stack. A
failure is diagnosed before anything is replaced, and a failure that is not resolved is
reported as a limitation rather than absorbed.

| If this fails | Then |
|---|---|
| The chosen freeze path is wrong on DPI, or misses window content | Locate the cause, then implement the other capture path and compare. This is the only condition that earns a second path. |
| Activation is slower than the target | Find where the time goes first: process wake, window show, the freeze, the first paint, or input readiness. Replace the component the measurement accuses, not the one that is easiest to blame. |
| The clipboard cannot satisfy the two destinations | Diagnose it: which format, which application, which failure. Fix it, or report an explicit limitation with the applications named. It does not pass the gate on the grounds that the clipboard is native by design. |
| The export drifts from the display and shared wrapping cannot close it | Moving the composer to native code is a candidate, not a remedy: it must be revalidated for editing and output together, including the active-caret case, before it counts. |
| Editing is unusable at 4K | Tile the canvas, or reconsider the editor surface, with the measurement in hand. |

No Stage 0 result justifies building the capture library or the Rogers integration to
compensate for an unproven capture path.

## Part 7: Stage 1, the daily-use loop

Dependency order, not dates. Each item stays usable on its own, and the stage ends with
real work rather than a feature count.

- [ ] S1.1 Capture lifecycle: repeated captures, hide and show, the remembered return
      target, no overwriting of a previous capture, and the empty-state line (F15).
- [ ] S1.2 Callout completion: the deterministic candidate positions, the neutral margin
      when a bubble cannot fit, and stable numbering with gaps, identical in the editor
      and in every output (decision 3). Model: Opus 5.
- [ ] S1.3 The keyboard contract, routed by context, with the stage column added to the
      table so nothing promises a key that does not exist (F9), and undo and redo in
      full (decision 2).
      Its acceptance sequence: edit an existing bubble, leave text editing, move the
      bubble, undo restores its position, undo again restores its previous text.
- [ ] S1.4 The document store: the schema from F8, debounced autosave, forced saves on
      blur, navigation, hide and quit, reopen after restart, and a visible failure that
      never claims to have saved. Model: Opus 5, it touches stored work.
- [ ] S1.5 Navigation: previous and next, the position indicator, and shortcuts to the
      first and last capture (decision 4).
- [ ] S1.6 Copy and Return, including every failure path: a failed clipboard, a failed
      save, a refused activation, a closed target application, and a user who moved on
      mid-operation (F10).
- [ ] S1.7 The trial: thirty captures of real client feedback, with the friction recorded
      in `project-os/History.md` and anything deferred sent to `project-os/Backlog.md`.

Stage 1 is the first release that replaces the current Snagit workflow. It ships without
file export, without the thumbnail strip, and without Rogers.

Stage 1 prepares nothing for Rogers. It persists editable capture documents, and that is
all the foundation Stage 4 is entitled to assume.

## Part 8: the later stages

Kept as a dependency outline only, per the product plan's sections 9 and 10.

Stage 2, history and file output: the strip, older captures inside the same editor, PNG
export, intentional deletion, visible storage use with the expected daily rate stated
(F12). Depends on S1.4.

Stage 3, secondary tools: arrow, rectangle, text, blur, highlight, ordered by what daily
use actually demanded. Depends on the scene and composer being stable.

Stage 4, Rogers delivery: inspect Rogers first, then design the submission record against
what is actually there. The work is destination validation, item and attachment behavior,
what counts as success, duplicate prevention, retry with a reused identifier, and error
handling. Calling any of that transport understates it. Explicit retry only, with no
background delivery queue in v1.

## Part 9: the evidence status of every claim here

**Measured:** nothing. This project has no code yet, so no timing and no memory figure in
this document was produced by running anything.

**Documented platform behavior:** the per-display capture call's own minimum Windows
version; that injected input is subject to privilege restrictions; that a request to
bring a window to the foreground can be refused. Sources in part 10.

**Behavior to verify, currently unknown:** whether a registered global hotkey is
delivered while an application running as administrator holds the foreground (F2, S0.6);
whether best-effort activation succeeds against such a window and against a closed one
(F10, S0.6); which clipboard formats the two chosen destinations actually accept (F4,
S0.4); what the freeze and the first paint really cost on this machine (F3, S0.5).

**Assumed until Stage 0 says otherwise:** that pre-creating the overlay and the editor at
startup is enough to approach the proposed latency targets; that shared wrapping plus the
three S0.4 checks are enough to keep the editor and the export in agreement; that a
familiar stack is the right one here.

**Not inspected:** Copy Ninja and Rogers. Both are yours to point me at, in a task that
names them.

## Part 10: sources

- [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey) · what hotkey registration does and does not promise.
- [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) · the documented privilege restriction, on injection.
- [SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) · when activation is allowed and when it is refused.
- [CreateForMonitor](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createformonitor) · the per-display capture call and its Windows 10 1903 minimum.
- [High DPI development on Windows](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows) · DPI contexts and coordinate handling.
