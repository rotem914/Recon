# Recon product plan, review

Date: 2026-09-10.
Reviewed: the product plan and Claude handoff dated 2026-09-10, as given in chat.
Reviewer: the assistant, at Rotem's request.

Verdict vocabulary per finding: **fix** (change the plan now), **drop** (never mind),
**backlog** (record it and move on). Only `backlog` writes a row to
`project-os/Backlog.md`.

## Verdict in one line

The plan is unusually sound on product behavior and unusually thin on the two
mechanisms that decide whether that behavior is achievable: how the exported image
is produced, and how the Stage 0 numbers are measured.

Sixteen findings: three blocking, seven important, six small.
Nothing here changes the product definition or the stage order.

## What is strong

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

## Findings

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
Display renders the scene at zoom Z; export renders the same scene at Z=1 through the
same code.
Add a pixel-diff test over a fixed set of documents so a drift becomes a failing check
rather than a client email.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F2 · 🔴 The global hotkey is dead while an elevated window has focus**

Where: §2 principle 1, §3.1, §3.2.

Windows does not deliver a hotkey to a normal process while a UAC-elevated window is
in the foreground, and focus-related calls into that window are blocked too.

So "fast access is a core feature" fails silently whenever Task Manager, an elevated
terminal or an installer is in front, and Copy and Return can fail for the same
reason.

Suggestion: state the limitation in the plan, test it in Stage 0 against a real
elevated window, and decide whether v1 documents it or offers an optional elevated
launch.

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 6 below

----
**F3 · 🔴 Stage 0 asks for p95 numbers with no way to measure them**

Where: §9 Stage 0 acceptance.

A p95 over 30 runs needs four timestamps per run: hotkey received, overlay visible,
editor first paint, editor accepting input.
Without a harness the gate degrades into a feeling, and a feeling cannot fail a stack.

Suggestion: make the instrumentation an explicit Stage 0 deliverable.
One run set writes one CSV, prints p50 and p95 per leg, and samples memory with the
live document separated from retained history.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F4 · 🟠 Clipboard formats are unspecified, so the Stage 0 paste test can pass and still be wrong**

Where: §5 Image output, §9 Stage 0 acceptance.

Windows applications disagree about image formats.
Some read only the older device-independent bitmap and lose transparency; others
prefer PNG; a few take only the plain bitmap handle.
Publishing one format satisfies the two applications you happened to test.

Suggestion: publish several formats in one clipboard operation, from native code
rather than through the webview API, and name the five real destination applications
as the test list instead of "at least two".

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

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 1 below

----
**F7 · 🟠 One undo stack, not two**

Where: §5, the Undo/Redo row.

Context-scoped undo means the same keystroke does different things depending on state
the user cannot see, and the first surprise costs a whole note instead of a word.

Suggestion: one document undo stack.
A text editing session coalesces into one entry when it ends, with per-word steps
while the caret is still live.

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 2 below

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

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F9 · 🟠 Ctrl+S sits in the v1 contract while file export is Stage 2**

Where: §5 table, against §9 Stage 2.

The shortcut table reads as the shipping contract, so Stage 1 promises a key that
does nothing.
The Rogers row is marked conditional; the export row is not.

Suggestion: add a stage column to the table, or mark the two deferred rows the way
the Rogers row is marked.

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

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F11 · 🟡 Numbering gaps travel to the client**

Where: §4.3.

Stable numbers with gaps are right internally and read as a mistake in a feedback
list that jumps 1, 2, 4, 7.

Suggestion: keep stable numbering while editing, and renumber at the moment of send
or export, or offer an explicit renumber action.

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 3 below

----
**F12 · 🟡 "Keep forever" has an unstated cost**

Where: §6.3.

Thirty 4K captures a day is roughly 60 to 250 MB a day depending on content, so
"no expiry" is a real disk decision currently made without a number.

Suggestion: put the expected rate in the plan so the default is chosen knowingly.
Stage 2 already shows usage; revisit limits with data rather than a rule.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F13 · 🟡 In Stage 1, capture 1 is twenty-nine keypresses away**

Where: §6.2, §9 Stage 1.

The workload is defined as 20 to 30 captures with returns to earlier images, and
Stage 1 ships previous and next only.

Suggestion: add a position indicator and jump to first and last in Stage 1.
The thumbnail strip still waits for Stage 2.

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 4 below

----
**F14 · 🟡 The minimum Windows version and the runtime distribution are undecided**

Where: §10 point 3.

The capture API sets the floor, and the web view runtime choice decides whether the
open-source release ships a small installer that downloads a runtime or a large one
that carries it.

Suggestion: name Windows 10 1903 as the floor and the downloading installer as the
default, then restate both after Stage 0 measures the capture path.

Verdict: [ ] fix  [ ] drop  [ ] backlog  ·  also decision 5 below

----
**F15 · 🟡 First run and the empty editor are missing**

Where: §3.1, §3.3.

For an open-source release the first two minutes are the product: no capture exists
yet, the hotkey may be taken, and the tray icon is undiscovered.

Suggestion: one empty-state line in the editor naming the current hotkey, and a
first-run check that reports a hotkey conflict rather than failing quietly.

Verdict: [ ] fix  [ ] drop  [ ] backlog

----
**F16 · 🟡 HDR is left as an open question**

Where: §9 Stage 0.

Tone mapping HDR content is a project of its own, and leaving it open invites Stage 0
to spend a day on it.

Suggestion: make "not supported in v1, and documented" an acceptable Stage 0 exit,
with detection so the output is never silently washed out.

Verdict: [ ] fix  [ ] drop  [ ] backlog

## The six decisions that are yours

The technical plan assumes the suggestion in each case.
A different answer changes the steps marked in `project-os/Plan.md`.

1) Bubble text direction: per bubble from its first strong character with a manual
   override, or something else?
2) Undo: one document stack with coalesced text edits, or keep the two contexts?
3) Numbering: renumber on send, an explicit renumber action, or keep the gaps?
4) Stage 1 navigation: add the position indicator and jump to first and last, or
   previous and next only?
5) Floor and installer: Windows 10 1903 and a downloading installer, or carry the
   runtime?
6) Elevated windows: document the hotkey limitation for v1, or plan an elevated mode?

## What I did not check

No code exists in this repository yet, so nothing here was measured or benchmarked.

Every timing figure in these findings is either your own target or an expectation from
platform documentation, and Stage 0 exists to replace both with numbers.

The Windows behaviors in F2, F4 and F10 are documented platform rules, not results I
reproduced on this machine.

I did not open Copy Ninja and I did not inspect Rogers.
Both are yours to point me at, in a task that names them.
