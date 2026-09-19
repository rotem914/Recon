# Recon — Bug Atlas

The map of this project's recurring bug classes.

`project-os/History.md` records that a bug was fixed. This file records the
PATTERN, so the next session recognizes it in minutes instead of rediscovering
it in hours. Read the matching row before writing a fix.

## When a bug earns a row

Add or update a row when:

- the same bug pattern appears a second time,
- a fix took several attempts because the real root cause was hidden,
- a library or API behaves in a known dangerous way,
- a future session is likely to reach for the same WRONG fix again.

A one-off typo earns nothing. The atlas is for classes, not incidents.

## How to use it

1. Name the symptom you see.
2. Search this table for a matching row.
3. If a row matches, follow its fix and checklist before debugging anew.
4. If nothing matches, debug normally.
5. If the issue turns out to be a pattern, add the row in the same task.

## Rotation

`project-os/rotate.ps1` keeps the newest 30 Atlas rows live and moves
older ones verbatim into `BugAtlas-archive.md` at `Go commit`. Rows are
relocated, never edited, renumbered or deleted, so an archived row still
answers a search.

## Atlas

| # | Symptom | Root cause | The fix that holds | Times bitten | Where recorded |
|---|---|---|---|---|---|
| 1 | A drag, or any input that keeps coming, moves the picture only when it stops, or uncovers edges that stay empty until the picture comes back | Every move asks the host for a region, and each newer request makes the answer still on its way stale, so the page drops every answer until the input rests; and a region holds only what was on screen, so a slid picture has nothing beyond its own edges | While the input continues, slide what is already painted by its region's own origin, ask for one region at a time, the newest as each lands, and show a small copy of the whole picture under it, so an uncovered edge is never empty. Never remove the stale-answer drop instead: a region painted under a pan it was not asked for sits in the wrong place. The wheel is the same case with a zoom in it: stretch the painted region to the new zoom (`stretchCanvas`) and ask for one at a time | 3 | History 2026-09-15, "The drag paints while it moves" and "The drag's edges are never empty"; Decisions, the same date; History and Decisions 2026-09-19, the wheel in a fast spin |
| 2 | A change to how a picture loads or paints turns a check on a capture probe red: a note kept twice, or the drag's copy of the whole picture missing; the product looks fine | `editor_capture_probe` announces `capture-ready`, and the page's listener loads the same capture while the check loads it too, so two loads run at once; every extra paint a load issues supersedes the other load's paint, and the last region to land can be one that is not the whole picture | Keep a load to one paint, and make any follow-up paint conditional on a change that really happened (a zoom that moved); never add a second paint to `loadImage` to wait out a superseded one. When a probe check flips after such a change, suspect the double load before the code under test | 2 | History 2026-09-14, "The top bar" and "No tool in hand by default" (section 16's notes); History 2026-09-15, "The zoom-out floor" (section 32's copy) |
| 3 | The picture disappears for good after a zoom key and stays gone until another document opens; the canvas is hidden | A stage with no area, a timeline at its full height over a short window, measures a whole-picture fit of 0; code that copies that fit into the zoom leaves a zoom of 0, the next zoom key divides by it, and the pan becomes NaN, which nothing clears | Never write a fit measured on a stage under 2 px: skip the measurement and keep the fit from before (`stageHasArea` in `editor/index.html`), wherever the fit is measured, the follow when the space changes and the opening of a picture | 2 | History 2026-09-15, "The zoom-out floor" (the follow) and "A picture opened with no room" (the opening) |
| 4 | A delete, or any move of a document's folder, is refused with "Access is denied" now and then, most often right after the timeline refreshed | Windows refuses to move a folder while a file in it is open, and a thumbnail being read or made on its thread holds one for a few milliseconds; before review 3 the refusal was logged and the document was dropped from the list anyway, to come back at the next startup | Retry the rename for up to a second on a permission error (`store::rename_retry`, under the trash and the restore alike), and never take the document off the list before the move succeeded; a refusal that outlasts the second is returned to the page, which says NOT DELETED | 2 | History 2026-09-16, "Review 3 fixed, T1 to T11"; the review document, T2 and T10; review 4, T2 |
| 5 | The timeline, a save or a thumbnail waits a tenth of a second now and then, on a large picture, right after a document from disk is shown, thumbnailed or resumed | A preserved PNG is read and decoded while the documents lock is held, so every command that touches the list waits for the decode | Clone the handle under the lock (`Managed::preserved_handle`) and decode after it (`preserved_frame`); never call a decode inside a `DOCUMENTS.lock()` block | 3 | Review 3, T7 (`show_document`); review 4, T6 (`thumbnail`, `editor_annotate`) |
| 6 | A deleted document cannot be restored, "already among the documents", and a startup logs a folder with no record as skipped; or a capture deleted and undone at once comes back without its image | A thread writes into `documents/<id>/` after a bare `is_dir` test, and the write makes the folder again when a delete moved it meanwhile; or a restore finds the record before the image thread has written it and gives up | Write beside a document's files with `store::write_beside`, which never makes the folder; write the image wherever the folder is now; and let the restore wait for an image still encoding and put the folder back into the trash when it cannot rejoin | 2 | Review 3, T2; review 4, T2 and T3 |
