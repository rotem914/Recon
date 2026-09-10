# Recon, build plan

The plan of record. Read it at task pickup, alongside `project-os/Backlog.md`.

Status: proposed, waiting on Rotem's review.
Written: 2026-09-10.
Product baseline: the product plan and Claude handoff dated 2026-09-10.
Findings and open decisions: `notes/2026-09-10-plan-review.md`.

Nothing here is built yet. No code was run and no timing was measured in writing it.

## How to read this

Each step carries a checkbox, a suggested model with a short reason, what it delivers,
and the evidence that closes it.
The model is a suggestion for whoever picks the step up, not a setting: the capable
model for design, native platform work and anything that touches stored data, the
faster one for mechanical wiring.

A step is done when its evidence exists, not when its code runs.

Steps marked **[D1]** to **[D6]** assume a decision from the review that Rotem has not
given yet. Its answer changes that step.

## The proposed split

The plan's Stage 0 says "evaluate Tauri + React + Rust". This splits that into two
questions, because the two surfaces fail for different reasons and only one of them is
in doubt.

| Layer | Runs as | Owns |
|---|---|---|
| Host | native process | tray, preferences, global hotkey, window lifecycle, the return-focus target |
| Capture | native | freezing the desktop, per-monitor bitmaps, virtual-desktop coordinates |
| Overlay | native, one borderless window per display | the dim, the selection rectangle, cancel, handing the region to the host |
| Store | native | one folder per capture, the untouched original, the versioned document, atomic writes |
| Clipboard | native | several image formats in one operation, success reported before any hide |
| Editor shell | web view | toolbar, capture strip, destination indicator, keyboard routing |
| Scene and composer | web view | the one renderer: display at the current zoom, export at original scale |
| Text editing | web view, an input layer above the canvas | caret, keyboard input, bidirectional typing, driven by the shared wrap function |

Four boundaries hold the design together, and each one exists to stop a specific
failure:

The web view never touches the screen or the clipboard, so a web view limitation can
never break capture.

The host never renders an annotation, so there is exactly one renderer.

The scene is the only source of geometry, and export is that renderer at original
scale, which is how the editor and the file stay identical (review F1).

Everything is stored in virtual-desktop physical pixels, converted once at the edges,
so a display at 150% and a display placed left of the primary one cannot introduce
drift.

## Stage 0: prove the foundations

Purpose: retire the four risks that would change the architecture, and produce a
go or no-go recommendation per component rather than for the stack as a whole.

Build nothing that is not on this list.

----
**[ ] S0.1 · Host shell: tray, hotkey, quit**

Model: Sonnet 5. Mechanical, documented APIs.

Delivers: a background process with a tray entry, a configurable global hotkey, and a
quit that leaves nothing running.

Evidence: the hotkey fires while the process has never had a window; a hotkey already
taken by another application is reported as a conflict rather than silently lost; quit
leaves no process behind.

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
against each edge, and a document that added a margin; the captured pixels are proved
unchanged by hashing the source region against the export's image area; a paste
verified in five named destination applications, not two (review F4).

----
**[ ] S0.5 · Text direction and wrapping conformance  [D1]**

Model: Opus 5. The highest-risk unknown in the product.

Delivers: one wrap function, used by the canvas renderer and by the input layer above
it, plus its test set.

Evidence: unit tests over twelve strings including mixed-direction lines and long
unbroken tokens; the input layer and the rendered text break at the same places; a
pixel comparison of a committed note rendered by both paths.

Assumes decision 1: base direction per bubble from its first strong character, with a
manual override.

----
**[ ] S0.6 · Instrumentation and the thirty-run measurement**

Model: Sonnet 5. Mechanical, known shape.

Delivers: four timestamps per run (hotkey received, overlay visible, editor first paint,
editor accepting input), a CSV per run set, printed median and 95th percentile per leg,
and a memory sample that separates the live document from retained history (review F3).

Evidence: thirty runs at 4K reported as numbers, against the plan's proposed targets of
250 ms to a usable selection and 500 ms to an editor ready for input, measured with the
process already in the background; process startup measured separately and reported
separately.

----
**[ ] S0.7 · The platform limits, written down  [D6]**

Model: Sonnet 5. Reading, testing, recording.

Delivers: the tested Windows version, hardware, display arrangement, and the
limitations found.

Evidence: the hotkey tested against a real elevated window and the result recorded
(review F2); return-focus tested against an elevated window and against an application
that has since closed (review F10); HDR behavior determined, with "not supported in v1,
detected rather than silently wrong" as an acceptable answer (review F16).

----
**[ ] S0.8 · The go or no-go report**

Model: Opus 5. Judgment on evidence.

Delivers: one short document separating measured facts, documentation-supported claims
and assumptions, with a recommendation per component and the named fallback for any
component that failed.

Evidence: it exists, it names numbers, and it recommends keeping or replacing a
specific component rather than the whole stack.

## What Stage 0 is allowed to conclude

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

## Stage 1: the daily-use loop

Dependency order, not dates. Each item stays usable on its own, and the stage ends with
real work rather than a feature count.

- [ ] S1.1 Capture lifecycle: repeated captures, hide and show, the remembered return
      target, no overwriting of a previous capture.
- [ ] S1.2 Callout completion: the deterministic candidate positions, the neutral margin
      when a bubble cannot fit, and numbering. Model: Opus 5. **[D3]** on renumbering.
- [ ] S1.3 The keyboard contract, routed by context, with the stage column added to the
      table so nothing promises a key that does not exist (review F9). **[D2]** on undo.
- [ ] S1.4 The document store: debounced autosave, forced saves on blur, navigation,
      hide and quit, reopen after restart, and a visible failure that never claims to
      have saved. Model: Opus 5, it touches stored work.
- [ ] S1.5 Navigation: previous and next, plus the position indicator and jump to first
      and last. **[D4]**
- [ ] S1.6 Copy and Return, including every failure path: a failed clipboard, a failed
      save, a closed target application, and a user who moved on mid-operation.
- [ ] S1.7 The trial: thirty captures of real client feedback, with the friction recorded
      in `project-os/History.md` and anything deferred sent to `project-os/Backlog.md`.

Stage 1 is the first release that replaces the current Snagit workflow. It ships without
file export, without the thumbnail strip, and without Rogers.

## Later stages

Kept as a dependency outline only, per the product plan's sections 9 and 10.

Stage 2, history and file output: the strip, older captures inside the same editor, PNG
export, intentional deletion, visible storage use. Depends on S1.4.

Stage 3, secondary tools: arrow, rectangle, text, blur, highlight, ordered by what daily
use actually demanded. Depends on the scene and composer being stable.

Stage 4, Rogers delivery: inspect Rogers first, then one narrow inbound operation, the
submission record, and safe retry. The submission record can be defined during Stage 1
as a local queue row, so Stage 4 only adds transport.

## Open decisions

Answers live in `notes/2026-09-10-plan-review.md`, section "The six decisions that are
yours". This plan assumes the suggestion in each case.

D1 bubble text direction · D2 undo model · D3 numbering on send · D4 Stage 1 navigation
· D5 Windows floor and installer · D6 the elevated-window policy.

D5 has no step of its own: it is recorded in S0.7 and then belongs in
`project-os/Decisions.md` once Rotem answers.

## Measured, supported, assumed

Measured: nothing. This project has no code yet.

Supported by platform documentation: the capture API's Windows floor, the hotkey and
foreground restrictions around elevated windows, and the disagreement between
applications over clipboard image formats.

Assumed until Stage 0 says otherwise: that pre-creating the overlay and the editor at
startup is enough to reach the proposed latency targets, that one shared wrap function
can keep the editor and the export identical, and that a familiar stack is the right
one here.
