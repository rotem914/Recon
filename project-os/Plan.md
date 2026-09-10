# Recon, build plan

The plan of record. Read it at task pickup, alongside `project-os/Backlog.md`.

Status: proposed, waiting on Rotem's verdicts in parts 1 and 2.
Written: 2026-09-10.
Product baseline: Rotem's product plan and Claude handoff dated 2026-09-10.

Nothing here is built yet. No code was run and no timing was measured in writing it.

Eight parts: the review of the product plan, the decisions that are Rotem's, the
proposed architecture, Stage 0, what Stage 0 is allowed to conclude, Stage 1, the later
stages, and what is measured against what is assumed.

## Part 1: the review of the product plan

Verdict vocabulary per finding: **fix** (change the plan now), **drop** (never mind),
**backlog** (record it and move on). Only `backlog` writes a row to
`project-os/Backlog.md`.

**In one line.** The plan is unusually sound on product behavior and unusually thin on
the two mechanisms that decide whether that behavior is achievable: how the exported
image is produced, and how the Stage 0 numbers are measured.

Sixteen findings: three blocking, seven important, six small.
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

Two render paths always drift, and the drift lands exactly on Hebrew wrapping,
font fallback and arrow geometry.
The failure is invisible in the editor: it appears only in the file the client gets.

Suggestion: one scene model and one renderer.
Display renders the scene at the current zoom; export renders the same scene at
original scale through the same code.
Add a pixel-diff test over a fixed set of documents so a drift becomes a failing check
rather than a client email.

Carried into: part 3 boundaries, S0.4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F2 · 🔴 The global hotkey is dead while an elevated window has focus**

Where: §2 principle 1, §3.1, §3.2.

Windows does not deliver a hotkey to a normal process while a window running as
administrator is in the foreground, and focus calls into that window are blocked too.

So "fast access is a core feature" fails silently whenever Task Manager, an elevated
terminal or an installer is in front, and Copy and Return can fail for the same
reason.

Suggestion: state the limitation in the plan, test it in Stage 0 against a real
elevated window, and decide whether v1 documents it or offers an optional elevated
launch.

Carried into: S0.7, decision 6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F3 · 🔴 Stage 0 asks for p95 numbers with no way to measure them**

Where: §9 Stage 0 acceptance.

A 95th percentile over 30 runs needs four timestamps per run: hotkey received, overlay
visible, editor first paint, editor accepting input.
Without a harness the gate degrades into a feeling, and a feeling cannot fail a stack.

Suggestion: make the instrumentation an explicit Stage 0 deliverable.
One run set writes one CSV, prints the median and the 95th percentile per leg, and
samples memory with the live document separated from retained history.

Carried into: S0.6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F4 · 🟠 Clipboard formats are unspecified, so the Stage 0 paste test can pass and still be wrong**

Where: §5 Image output, §9 Stage 0 acceptance.

Windows applications disagree about image formats.
Some read only the older device-independent bitmap and lose transparency; others
prefer PNG; a few take only the plain bitmap handle.
Publishing one format satisfies the two applications you happened to test.

Suggestion: publish several formats in one clipboard operation, from native code
rather than through the web view API, and name the five real destination applications
as the test list instead of "at least two".

Carried into: part 3 split, S0.4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F5 · 🟠 The selection overlay should not be a web view**

Where: §9 Stage 0, "evaluate Tauri + React + Rust".

The overlay is the latency-critical and DPI-critical surface, and a web view window
spread across mixed-scale monitors adds cold start, scaling and input-transparency
problems that have nothing to do with the editor.

Judging one stack for both surfaces risks failing the whole stack for a reason that
belongs to one component.

Suggestion: split the architecture before Stage 0 starts.
Native code owns the hotkey, the freeze, the overlay and the clipboard; the web view
owns the editor only.
Stage 0 then measures the editor's fidelity instead of the capture path's viability.

Carried into: part 3, the whole of it.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F6 · 🟠 Text direction is named but not defined**

Where: §4.3.

"Support Hebrew, English, and mixed-direction text" leaves three things open: the
base direction of a bubble, its alignment, and which way it grows as text is added.

This is the highest-risk area for your own daily use, and it decides the wrap
function that F1 depends on.

Suggestion: base direction per bubble from its first strong character, with a manual
override on the selected bubble.
Alignment follows base direction.
The bubble's anchor-side edge stays fixed while the opposite edge grows, so a note
never walks away from the thing it points at.

Carried into: S0.5, decision 1.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F7 · 🟠 One undo stack, not two**

Where: §5, the Undo/Redo row.

Context-scoped undo means the same keystroke does different things depending on state
the user cannot see, and the first surprise costs a whole note instead of a word.

Suggestion: one document undo stack.
A text editing session coalesces into one entry when it ends, with per-word steps
while the caret is still live.

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

Carried into: part 3 split, S1.4.

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

Windows restricts which process may bring a window forward, and the only reliable
moment is while Recon still holds that right.
Some targets refuse anyway, elevated windows first among them.

Suggestion: capture the target window handle at capture time, restore it in the same
turn as the hide, and treat refusal as the plan's own fallback: hide without
activating anything.
Test it in Stage 0 against an elevated window and against an application that has
since closed.

Carried into: S0.7, S1.6.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F11 · 🟡 Numbering gaps travel to the client**

Where: §4.3.

Stable numbers with gaps are right internally and read as a mistake in a feedback
list that jumps 1, 2, 4, 7.

Suggestion: keep stable numbering while editing, and renumber at the moment of send
or export, or offer an explicit renumber action.

Carried into: S1.2, decision 3.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F12 · 🟡 "Keep forever" has an unstated cost**

Where: §6.3.

Thirty 4K captures a day is roughly 60 to 250 MB a day depending on content, so
"no expiry" is a real disk decision currently made without a number.

Suggestion: put the expected rate in the plan so the default is chosen knowingly.
Stage 2 already shows usage; revisit limits with data rather than a rule.

Carried into: part 7, Stage 2.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F13 · 🟡 In Stage 1, capture 1 is twenty-nine keypresses away**

Where: §6.2, §9 Stage 1.

The workload is defined as 20 to 30 captures with returns to earlier images, and
Stage 1 ships previous and next only.

Suggestion: add a position indicator and jump to first and last in Stage 1.
The thumbnail strip still waits for Stage 2.

Carried into: S1.5, decision 4.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F14 · 🟡 The minimum Windows version and the runtime distribution are undecided**

Where: §10 point 3.

The capture API sets the floor, and the web view runtime choice decides whether the
open-source release ships a small installer that downloads a runtime or a large one
that carries it.

Suggestion: name Windows 10 1903 as the floor and the downloading installer as the
default, then restate both after Stage 0 measures the capture path.

Carried into: S0.7, decision 5.

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

Carried into: S0.7.

Verdict: [ ] fix  [ ] drop  [ ] backlog

## Part 2: the six decisions that are yours

Parts 3 to 6 assume the suggestion in each case.
A different answer changes the step named beside it.

1) Bubble text direction: per bubble from its first strong character with a manual
   override, or something else? Changes S0.5.
2) Undo: one document stack with coalesced text edits, or keep the two contexts?
   Changes S1.3.
3) Numbering: renumber on send, an explicit renumber action, or keep the gaps?
   Changes S1.2.
4) Stage 1 navigation: add the position indicator and jump to first and last, or
   previous and next only? Changes S1.5.
5) Floor and installer: Windows 10 1903 and a downloading installer, or carry the
   runtime? Changes S0.7, then belongs in `project-os/Decisions.md`.
6) Elevated windows: document the hotkey limitation for v1, or plan an elevated mode?
   Changes S0.7.

## Part 3: the proposed architecture

The product plan's Stage 0 says "evaluate Tauri + React + Rust". This splits that into
two questions, because the two surfaces fail for different reasons and only one of them
is in doubt (F5).

| Layer | Runs as | Owns |
|---|---|---|
| Host | native process | tray, preferences, global hotkey, window lifecycle, the return-focus target |
| Capture | native | freezing the desktop, per-display bitmaps, virtual-desktop coordinates |
| Overlay | native, one borderless window per display | the dim, the selection rectangle, cancel, handing the region to the host |
| Store | native | one folder per capture, the untouched original, the versioned document, atomic writes |
| Clipboard | native | several image formats in one operation, success reported before any hide |
| Editor shell | web view | toolbar, capture strip, destination indicator, keyboard routing |
| Scene and composer | web view | the one renderer: display at the current zoom, export at original scale |
| Text editing | web view, an input layer above the canvas | caret, keyboard input, bidirectional typing, driven by the shared wrap function |

Four boundaries hold this together, and each one exists to stop a specific failure:

The web view never touches the screen or the clipboard, so a web view limitation can
never break capture (F5, F4).

The host never renders an annotation, so there is exactly one renderer.

The scene is the only source of geometry, and export is that renderer at original
scale, which is how the editor and the file stay identical (F1).

Everything is stored in virtual-desktop physical pixels, converted once at the edges,
so a display at 150% and a display placed left of the primary one cannot introduce
drift.

## Part 4: Stage 0, prove the foundations

Purpose: retire the risks that would change the architecture, and produce a go or
no-go recommendation per component rather than for the stack as a whole.

Each step carries a checkbox, a suggested model with a short reason, what it delivers,
and the evidence that closes it. The model is a suggestion for whoever picks the step
up, not a setting. A step is done when its evidence exists, not when its code runs.

Build nothing that is not on this list.

----
**[ ] S0.1 · Host shell: tray, hotkey, quit**

Model: Sonnet 5. Mechanical, documented APIs.

Delivers: a background process with a tray entry, a configurable global hotkey, and a
quit that leaves nothing running.

Evidence: the hotkey fires while the process has never had a window; a hotkey already
taken by another application is reported as a conflict rather than silently lost (F15);
quit leaves no process behind.

----
**[ ] S0.2 · Freeze, overlay, region selection**

Model: Opus 5. DPI, native APIs, irreversible shape.

Delivers: the frozen desktop image, one overlay window per display, drag to select,
cancel, and the region handed back in virtual-desktop pixels.
Both candidate freeze paths implemented far enough to measure: a single copy of the
whole virtual screen, and the modern per-display capture API.

Evidence: two displays at 100% and at 150% or 200%, with the secondary placed left of
the primary so coordinates go negative; the selected rectangle and the resulting pixels
agree exactly; the overlay and the editor never appear in the output; cancel restores
the previous context and creates no capture; a second hotkey press during selection is
ignored; an open menu is either captured or the limitation is written down.
Both freeze paths timed, with the recommendation stated.

----
**[ ] S0.3 · Editor window, scene, one callout**

Model: Opus 5. The composition contract is the product's spine.

Delivers: the editor window pre-created and hidden at startup, one image on the canvas,
and one callout that can be created, typed into, committed, selected, moved, deleted,
undone and redone. The anchor moves independently and the arrow follows.

Evidence: every one of those operations exercised by hand; an empty bubble is discarded
when editing ends and never reaches the output; undo and redo cover text and geometry.

----
**[ ] S0.4 · The composer and the clipboard**

Model: Opus 5. Fidelity, and the one place work can be lost.

Delivers: the composer, used for both display and export, and a native clipboard
operation publishing several image formats.

Evidence: six fixed documents exported at original scale and compared against stored
references, covering Hebrew, English, mixed direction, a long wrapped note, an anchor
against each edge, and a document that added a margin (F1); the captured pixels proved
unchanged by hashing the source region against the export's image area; a paste
verified in five named destination applications, not two (F4).

----
**[ ] S0.5 · Text direction and wrapping conformance**

Model: Opus 5. The highest-risk unknown in the product.

Depends on decision 1.

Delivers: one wrap function, used by the canvas renderer and by the input layer above
it, plus its test set.

Evidence: unit tests over twelve strings including mixed-direction lines and long
unbroken tokens; the input layer and the rendered text break at the same places; a
pixel comparison of a committed note rendered by both paths.

----
**[ ] S0.6 · Instrumentation and the thirty-run measurement**

Model: Sonnet 5. Mechanical, known shape.

Delivers: four timestamps per run (hotkey received, overlay visible, editor first paint,
editor accepting input), a CSV per run set, printed median and 95th percentile per leg,
and a memory sample that separates the live document from retained history (F3).

Evidence: thirty runs at 4K reported as numbers, against the plan's proposed targets of
250 ms to a usable selection and 500 ms to an editor ready for input, measured with the
process already in the background; process startup measured separately and reported
separately.

----
**[ ] S0.7 · The platform limits, written down**

Model: Sonnet 5. Reading, testing, recording.

Depends on decisions 5 and 6.

Delivers: the tested Windows version, hardware, display arrangement, and the
limitations found.

Evidence: the hotkey tested against a real elevated window and the result recorded
(F2); return-focus tested against an elevated window and against an application that
has since closed (F10); the minimum Windows version and the installer shape stated
(F14); HDR behavior determined, with "not supported in v1, detected rather than
silently wrong" as an acceptable answer (F16).

----
**[ ] S0.8 · The go or no-go report**

Model: Opus 5. Judgment on evidence.

Delivers: one short document separating measured facts, documentation-supported claims
and assumptions, with a recommendation per component and the named fallback for any
component that failed.

Evidence: it exists, it names numbers, and it recommends keeping or replacing a
specific component rather than the whole stack.

## Part 5: what Stage 0 is allowed to conclude

The stack question is four questions. Each has a fallback that does not restart the
project.

| If this fails | Then |
|---|---|
| The freeze path is wrong on DPI or misses window content | Swap the freeze path for the other candidate. The rest of the architecture is untouched. |
| A usable selection is still slow after pre-warming | The overlay is the wrong technology, not the editor. Keep the split and rebuild that one window. |
| The export drifts from the display and the shared wrap cannot close it | Move the composer to native code and let the editor display the composer's output. The editor stays. |
| The clipboard cannot satisfy the five destinations from the web view | The clipboard was already native in this plan. This is the expected outcome, not a failure. |
| Editing is unusable at 4K | Tile the canvas, or reconsider the editor surface, with the measurement in hand. |

No Stage 0 result justifies building the capture library or the Rogers integration to
compensate for an unproven capture path.

## Part 6: Stage 1, the daily-use loop

Dependency order, not dates. Each item stays usable on its own, and the stage ends with
real work rather than a feature count.

- [ ] S1.1 Capture lifecycle: repeated captures, hide and show, the remembered return
      target, no overwriting of a previous capture, and the empty-state line (F15).
- [ ] S1.2 Callout completion: the deterministic candidate positions, the neutral margin
      when a bubble cannot fit, and numbering. Depends on decision 3. Model: Opus 5.
- [ ] S1.3 The keyboard contract, routed by context, with the stage column added to the
      table so nothing promises a key that does not exist (F9). Depends on decision 2.
- [ ] S1.4 The document store: debounced autosave, forced saves on blur, navigation,
      hide and quit, reopen after restart, and a visible failure that never claims to
      have saved (F8). Model: Opus 5, it touches stored work.
- [ ] S1.5 Navigation: previous and next, plus the position indicator and jump to first
      and last. Depends on decision 4.
- [ ] S1.6 Copy and Return, including every failure path: a failed clipboard, a failed
      save, a closed target application, and a user who moved on mid-operation (F10).
- [ ] S1.7 The trial: thirty captures of real client feedback, with the friction recorded
      in `project-os/History.md` and anything deferred sent to `project-os/Backlog.md`.

Stage 1 is the first release that replaces the current Snagit workflow. It ships without
file export, without the thumbnail strip, and without Rogers.

## Part 7: the later stages

Kept as a dependency outline only, per the product plan's sections 9 and 10.

Stage 2, history and file output: the strip, older captures inside the same editor, PNG
export, intentional deletion, visible storage use (F12). Depends on S1.4.

Stage 3, secondary tools: arrow, rectangle, text, blur, highlight, ordered by what daily
use actually demanded. Depends on the scene and composer being stable.

Stage 4, Rogers delivery: inspect Rogers first, then one narrow inbound operation, the
submission record, and safe retry. The submission record can be defined during Stage 1
as a local queue row, so Stage 4 only adds transport.

## Part 8: measured, supported, assumed

Measured: nothing. This project has no code yet, so no timing and no memory figure in
this document was produced by running anything.

Supported by platform documentation: the capture API's Windows floor, the hotkey and
foreground restrictions around windows running as administrator, and the disagreement
between applications over clipboard image formats.

Assumed until Stage 0 says otherwise: that pre-creating the overlay and the editor at
startup is enough to reach the proposed latency targets, that one shared wrap function
can keep the editor and the export identical, and that a familiar stack is the right
one here.

Not inspected: Copy Ninja and Rogers. Both are yours to point me at, in a task that
names them.
