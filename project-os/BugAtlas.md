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
| 1 | A drag, or any input that keeps coming, moves the picture only when it stops, or uncovers edges that stay empty until the picture comes back | Every move asks the host for a region, and each newer request makes the answer still on its way stale, so the page drops every answer until the input rests; and a region holds only what was on screen, so a slid picture has nothing beyond its own edges | While the input continues, slide what is already painted by its region's own origin, ask for one region at a time, the newest as each lands, and show a small copy of the whole picture under it, so an uncovered edge is never empty. Never remove the stale-answer drop instead: a region painted under a pan it was not asked for sits in the wrong place | 2 | History 2026-09-15, "The drag paints while it moves" and "The drag's edges are never empty"; Decisions, the same date |
