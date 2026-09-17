# Code review, 2026-09-17, the whole repository with the crop in

Asked for by Rotem with `GO CODE REVIEW`, one agent, no sub-agents. Scope: the whole tree
at commit `a754798`, read file by file: product code first (`host/src/*.rs`, `host/src/capture`,
`host/src/source`, `host/pixels`, `editor/index.html`, the Tauri config and capability), then the
checks code (`selftest.rs`, `measure.rs`, `bench.rs`, `fixtures.rs`, `report.rs`,
`editor-checks.js`, `bench.html`), then the process tooling (`project-os/guards/*.mjs`,
`install-hooks.mjs`, `rotate.ps1`, `backup.ps1`), which has not changed since review 3.
Calibration: `project-os/Code_review.md`; the worst class is a note's text or a source pixel
lost quietly, and the always-check list was walked row by row.

Every finding is pre-existing. Rotem's verdict, the same day: FIX ALL. T1 to T7 are fixed in the
commit that follows this document's update, each with the check its block names; the checks of
T2 and T7 needed one more checks-only command, `editor_store_remove_source`, and T3 a store
write that never makes a folder, `store::write_beside`. Counts: 1 blocking, 2 important, 4 nits.

From the earlier passes, still open as written there and not repeated: R8 (an overlay panic
leaks its windows), R9 (a poisoned region mailbox), R10 (the SVG raster cap allows a gigabyte)
from 2026-09-13, and T12 (the exit on a Windows logoff) from 2026-09-16.

The always-check list held everywhere it was looked for: the preserved image never crosses a
canvas, an external file is only read (`source::open`, WIC with `GENERIC_READ`, the folder
listing), committed text is read with `innerText`, every way out of a note commits it, a late
region answer is dropped by ticket, the selection guard releases on drop, dimensions travel
in the body, DPI awareness is the first call in `main`, the page's ratio is measured, the
check-only commands are behind the feature, every store write is flushed before its rename,
and the dirty flag clears before the wait. The one row that did not hold is T3.

## Blocking

```
T1 · A failed save is forgotten when the picture is left and returned to      🔴 fixed
Where:   editor/index.html, stashCurrent (the object put into `documents`, no save state
         in it) and loadImage ("model.save = { state: 'saved', error: '', dirty: false }")
Problem: The stash keeps a document's notes when another picture takes the screen, but not
         whether they reached the disk. Coming back, loadImage puts the stashed notes on
         screen and sets the save state to "saved", dirty false, whatever it was. Two ways
         in. One: the store refuses a save (a full disk, the folder held), the HUD says NOT
         SAVED, the user clicks another thumbnail and back: the HUD now says saved, nothing
         is dirty, and no later save carries those notes, so a quit loses them. Two: a
         note typed and Ctrl+Delete pressed inside the 800 ms debounce; the folder moves to
         the trash first, the save the switch forces is refused ("not managed"), and Ctrl+Z
         brings the picture back from the stash with the note on screen and "saved" in the
         HUD, while the record in the folder does not hold it. The screen and the HUD agree
         and the disk disagrees, which is the calibration's worst class.
Fix:     Carry the save state in the stash: stashCurrent adds `save: { ...model.save }`, and
         loadImage restores it when the stash holds one instead of resetting it; when the
         restored state is dirty, call markDirty so the timer's save retries at once. A
         document read from the host (no stash) keeps the reset, since its notes are the
         disk's.
Verify:  a new check in section 23: break the store, type a note, load another capture,
         unbreak the store, show the first document again; the HUD reads NOT SAVED or the
         note lands on disk by the timer within a second, never "saved" with the record
         missing it. Then `cargo build` and `--editor-check`, every check green.
Status:  [x] done
```

## Important

```
T2 · A capture deleted and undone while its image is still encoding loses it  🟠 fixed
Where:   host/src/editor.rs, preserve (the thread: "documents.iter_mut().find" then the
         trash branch) and trash_rejoin ("came back without its image")
Problem: Review 3's T2 sends the encoded PNG of a deleted capture into its trash folder. A
         Ctrl+Z (or Restore) inside the same window, which is the encode of the capture,
         a tenth of a second and more for a large one, moves the folder back among the
         documents before the PNG exists, finds no source.png, and refuses with "came back
         without its image"; the folder stays among the documents, unlisted. The thread
         then finds the id neither in the list nor in the trash and drops the PNG. The
         capture's only copy is gone: the trash view no longer shows it, every startup logs
         the folder as skipped, and the pixels were never on disk. Ctrl+Delete followed at
         once by Ctrl+Z is an ordinary pair of keys. The user sees NOT RESTORED, so it is
         loud, which is why it is not blocking.
Fix:     Two halves. In the thread, when the id is not listed, write the PNG wherever the
         folder is now: `store::folder(id)` when that is a directory, else the trash folder,
         else drop it with the log line. In trash_rejoin, when the record does not read or
         source.png is missing, move the folder back into the trash before returning the
         error, so the trash entry survives and the next Ctrl+Z, a moment later, succeeds.
Verify:  a new check in section 39: editor_capture_probe at 4000 by 2500, deleteDocument at
         once, undo at once, wait a second: either the picture is back, or NOT RESTORED was
         said and a second undo brings it back; editor_store_list shows no folder without
         source.png. Then the project checks and `--editor-check`.
Status:  [x] done
```

```
T3 · The strip's thumbnail writer can recreate a deleted document's folder    🟠 fixed
Where:   host/src/editor.rs, thumbnail ("if dir.is_dir() { ... write_atomic(&dir.join
         ("thumb.png") ..."); host/src/store.rs, write_atomic ("create_dir_all(parent)")
Problem: The thumbnail a strip cell asks for is made on a thread of its own and written into
         `documents/<id>/` outside the list's lock, after a bare `is_dir` test; write_atomic
         creates the parent folder when it is missing. A delete on the main thread that
         moves the folder between that test and the write recreates `documents/<id>/` with
         only thumb.png in it. From then on `store::restore` refuses that document for good
         ("already among the documents"), so its Restore and its Ctrl+Z never work, the
         30-day sweep removes the real folder from the trash, and the stray one is logged
         as skipped at every startup. The window is the microseconds between `is_dir` and
         `File::create`, so the odds are small; the always-check list's rule, a document's
         folder is written only for a listed document under the list's lock, is broken
         either way. Rotem's to lower.
Fix:     Take the DOCUMENTS lock around the `is_dir` test and the write, and write only when
         the id is still listed; or give the store a `write_beside` that refuses to create
         the folder (no create_dir_all) and use it for thumb.png. The second is smaller and
         also protects any later writer of a document's folder.
Verify:  a store unit test: a thumb.png write into a folder that is not there returns an
         error and leaves no folder; then the project checks and `--editor-check`.
Status:  [x] done
```

## Nits

```
T4 · After the last picture is deleted the empty editor still shows its shapes  🟡 fixed
Where:   editor/index.html, showEmpty (resets callouts, not shapes, crop or margin)
Problem: Ctrl+Delete on the only document clears the notes but leaves model.shapes,
         model.crop and model.margin as they were; layoutScene then draws the arrows,
         rectangles, highlights, rulers and blur boxes of the deleted picture on the empty
         stage until another picture opens. Cosmetic, and only on the last document.
Fix:     In showEmpty reset shapes, nextShape, selectedShape, crop, cropping, margin and
         marginFloor the way loadImage does.
Verify:  section 29's "last document deleted" check: draw a rectangle first, and assert
         `#arrows` holds no `[data-shape]` and no `.ruler` or `.blur` element remains.
Status:  [x] done
```

```
T5 · An empty bubble discarded after the second click burns its number        🟡 fixed
Where:   editor/index.html, commitEditing (the empty-text branch) against cancelPlacing
Problem: Between the two clicks Escape gives the number back (cancelPlacing). After the
         second click, Escape with nothing typed discards the bubble through commitEditing,
         which keeps nextNumber, so the next note is 2 with no 1 on the picture. Numbers
         never change for a deleted note by decision; an empty bubble was never a note.
Fix:     In commitEditing, when the discarded callout is the newest one (its number is
         nextNumber minus one), give the number back as cancelPlacing does. Or keep the
         gap and say so; Rotem's call.
Verify:  section 41: two clicks, Escape with nothing typed, then two clicks and a word: the
         note is number 1.
Status:  [x] done
```

```
T6 · Two more decodes under the documents lock                                  🟡 fixed
Where:   host/src/editor.rs, thumbnail ("document.frame(\"thumbnail\")?" inside the lock
         block) and editor_annotate ("d.frame(name)" inside the lock block)
Problem: Review 3's T7 moved show_document's decode outside DOCUMENTS; these two still
         decode the preserved PNG under it. A thumbnail made for a document read from disk,
         or Annotate resuming one, holds every timeline refresh and every save for the
         decode, a tenth of a second on a large picture. Not wrong, a stall.
Fix:     preserved_handle under the lock, preserved_frame after it, as show_document does.
Verify:  the project checks; no check measures the stall.
Status:  [x] done
```

```
T7 · A delete whose neighbour cannot be shown says NOT DELETED for a gone document  🟡 fixed
Where:   host/src/editor.rs, editor_delete_document ("show_document(next)?"); editor/
         index.html, deleteDocument (the catch that says NOT DELETED)
Problem: The folder is in the trash and the document off the list before the neighbour is
         shown; if that show fails (its PNG unreadable), the command returns the error, the
         page says NOT DELETED, keeps the deleted picture on screen, records no deletion for
         Ctrl+Z, and the timeline is stale until its next refresh. Needs a corrupt neighbour.
Fix:     After the move, a failed neighbour show falls back to the empty state and returns
         Ok(None), with the neighbour's error on the log; the page then shows the empty
         editor and the deletion is recorded.
Verify:  a check with the neighbour's source.png removed from the checks' store before
         Ctrl+Delete: the editor goes empty, the trash holds the deleted one.
Status:  [x] done
```

## Reach

No change was made, so nothing propagated. Of the fixes proposed: T1 is in the page's stash
and load path, which every switch of picture goes through, and its check has to cover a
capture arriving mid-typing (S1.1) and a resume through Annotate as well as the timeline
click. T2 and T3 are in the store's write path and the trash's move path, shared by every
document; each fix should be followed by the full editor checks, sections 23, 29, 37 and 39
in particular, not only its own. T4 to T7 are local.

## Not found, and looked for

An external file written or copied on any viewing path; a source pixel changed by the
composer or the crop outside the layer; a note losing a line break on commit; a way out of a
note that skips the commit; a keystroke lost during a save; a store write not flushed before
its rename; an unescaped file or tab name reaching the page's markup; a check-only command
reachable in the product build; a region answered to another origin; a temp file left after
a write; the tabs file written out of order; the crop reaching the document's pixels; the
invariants in `CLAUDE.md` rule 11 in every path that opens, shows, annotates, crops or
exports a file.

## The working tree

The one uncommitted file, `host/references/s06/environment.txt`, is a check run's rewrite of
the reference environment, two lines of dates and window numbers, another session's; it was
read and raises nothing.
