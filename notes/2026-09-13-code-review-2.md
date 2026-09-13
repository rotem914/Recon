# Code review, 2026-09-13, the whole repository after Stage 0

Asked for by Rotem after S0.8 closed: a review of every stage so far, and every problem fixed
until it is stable. Scope: the whole tree at commit `c45587e`, read file by file, product code
first (`host/src/*.rs`, `host/src/capture`, `host/src/source`, `host/pixels`,
`editor/index.html`, the Tauri config and capability), then the checks code
(`selftest.rs`, `measure.rs`, `bench.rs`, `fixtures.rs`, `report.rs`, `editor-checks.js`).
Calibration: `project-os/Code_review.md`; the worst class is a note's text or a source pixel
lost quietly.

Every check that could run ran on this tree before and after the fixes: fmt, clippy with
`-D warnings` on every target and feature, both builds, 40 unit tests, the 74 editor checks,
the decode report on the 28 real files (every file hashed the same, nothing written), the
self test with the machine idle, and a five-run measurement in release (5 of 6 attempts
clean, overlay usable at most 80.1 ms, editor usable at most 135.0 ms, floor 167 MB).

Counts: 1 blocking, 3 important, 8 nits. Introduced by Stage 0: all of them; nothing here
predates this repository. Fixed in this pass: R1 to R5, R11 and R12. Reported and left: R6
to R10, each with its home.

## Blocking

```
R1 · Switching notes mid-typing lost the first note's text          🔴 fixed
Where:   editor/index.html, startEditing
Problem: Clicking from a note being typed straight into another note set model.editing to
         the second without committing the first. layoutScene then rewrote the first note's
         element from the stale model text, and everything typed into it was gone. The
         worst class in the calibration, and it needed no unusual input.
Fix:     startEditing commits any other note still being edited before it takes over.
Verify:  editor check 9, "switching notes mid-typing keeps the first note's text": passes.
Status:  [x] done
```

## Important

```
R2 · A superseded region answer was raised as an error               🟠 fixed
Where:   editor/index.html, paintRegion
Problem: The host answers a replaced region request with 409 on purpose (the design of
         2026-09-13); the page treated any non-OK status as a thrown error, so a held arrow
         key produced an unhandled rejection per keypress, logged as an error.
Fix:     A 409 returns quietly; anything else is still thrown.
Verify:  no "unhandled rejection" line in a checks run that steps zoom and pan (0 now).
Status:  [x] done

R3 · Ctrl+Shift+C did not fire under a Hebrew keyboard layout         🟠 fixed
Where:   editor/index.html, the keydown handler
Problem: The shortcut matched event.key against "C"; under a Hebrew layout the same key
         reports a Hebrew letter, so the one copy key Stage 0 wired was dead for the
         product's own user. §3.6 asks for both layouts to be verified; S1.7 does the rest.
Fix:     Match event.code === "KeyC", the physical key.
Verify:  press the key under the Hebrew layout; the HUD shows "copied ... to the clipboard".
         Not machine-verifiable here; the editor checks cover the handler's path.
Status:  [x] done

R4 · An opened still was held twice in memory                          🟠 fixed
Where:   host/src/source/raster.rs, Still
Problem: On the first ask the provider handed the frame over and kept a full clone of the
         pixels for a second ask nothing makes. Every opened still therefore cost twice its
         size in the host, 192 MB twice for the 48-megapixel PNG, hidden from S0.7's live
         image number, which counts the editor's copy only.
Fix:     The provider keeps the file's bytes and decodes again on a second ask.
Verify:  cargo test; the decode report on the 28 real files, every line passing. The S0.7
         walk re-measured in release: the host's private bytes peak at 60.5 MB during the
         walk against 92.9 MB before the fix, on the same 4608x1976 phone JPEG; the whole
         process peaks at 304 MB against 361 MB.
Status:  [x] done
```

## Nits

```
R5 · The product build warned about unused fields and one function     🟡 fixed
Where:   host/src/source/mod.rs Notes; host/src/platform.rs this_process_is_elevated
Problem: F57, plus one more from S0.8: read only by the feature-gated report.
Fix:     cfg attributes.
Verify:  cargo build with no features prints no warning.
Status:  [x] done

R6 · A capture arriving mid-typing discards the note being typed       🟡 open, S1.1
Where:   editor/index.html, loadImage
Problem: A new capture resets the callouts and the editing state without a commit; the
         text typed so far is gone with the previous capture. This is the capture lifecycle
         S1.1 defines ("no overwriting of a previous capture"), so it is that step's, and it
         is named there.
Fix:     S1.1: commit, then keep or hand over the previous document per the plan.
Verify:  S1.1's checks.
Status:  [ ] open

R7 · The margin colour lives in two places                             🟡 open, backlog
Where:   host/src/compose.rs MAT; editor/index.html #mat
Problem: The host composites the margin in E9EAEC and the page paints the same value by
         hand; a change to one leaves the screen and the output disagreeing.
Fix:     Hand the colour to the page with the image info, or read it from one file.
Verify:  the S0.6 margin reference stays identical after the change.
Status:  [ ] open

R8 · An overlay panic leaks its windows and GDI objects                🟡 open
Where:   host/src/overlay.rs, select_region
Problem: The selection guard releases on a panic, but the thread-local STATE keeps the
         surfaces; the next capture replaces it without teardown. Functional, and a leak
         per panic, which has not happened.
Fix:     Tear down a leftover STATE at the start of select_region.
Verify:  a self-test case that panics inside the overlay thread.
Status:  [ ] open

R9 · A poisoned region mailbox would stall the page for good           🟡 open
Where:   host/src/editor.rs, region_worker
Problem: If the worker thread ever panicked with the slot locked, later fetches would
         never be answered and every view would stop painting. Nothing in the worker
         panics today.
Fix:     Recover from a poisoned lock and answer the pending request with an error.
Verify:  unit test with a forced poison.
Status:  [ ] open

R10 · The SVG raster cap allows a one-gigabyte pixmap                    🟡 open
Where:   host/src/source/svg.rs, render_size
Problem: A 16384x16384 cap is 1 GB of RGBA; a hostile viewBox would allocate it.
Fix:     Cap the area, not the side, or fail the open with a message.
Verify:  unit test on render_size.
Status:  [ ] open
```

```
R11 · The self test's "leaked overlay" verdict was unsound                🟡 fixed
Where:   host/src/selftest.rs, drag_test
Problem: After the drag it compared the crop from the frozen frame against the screen and
         called any still difference a leaked overlay. The window under the rectangle
         repaints when the overlay hands it the foreground back (a File Explorer row did,
         14% of the bytes), so the test failed on a machine in use while the product was
         right; and the crop is cut from a frame frozen before the overlay existed, so it
         cannot hold an overlay pixel by construction.
Fix:     The leak check asks Windows whether an overlay window is still alive; the pixel
         comparison stays informative, and the pointer is moved off the rectangle first.
Verify:  --selftest: "no overlay window is left on the desktop", every section passing.
Status:  [x] done
```

```
R12 · The HDR query sat inside the hotkey-to-overlay interval                🟡 fixed
Where:   host/src/main.rs, begin_capture
Problem: S0.8 put a DisplayConfig query at every freeze, on the capture thread, so it ran
         inside the interval S0.7 measures. The five-run re-measurement in release put the
         overlay-usable median at 79.6 ms against 74.0 ms at S0.7, which is that query.
Fix:     The query runs on its own thread and logs its line when it has it.
Verify:  a later thirty-run measurement; the target is 250 ms, so this was never a miss.
Status:  [x] done
```

## Reach

R1 to R3 change the page's editing and key handling, which every editor check exercises;
the 74 checks pass. R4 changes the raster provider only; every other provider and the
decode report are unchanged and pass. R5 is attributes. Impact: local.

## Not found, and looked for

Every file write in the host, listed: the checks' evidence beside the executable, the
references folder, the fixture folder, and a measurement's CSVs. The product itself writes
nothing (rule 11). The region scheme answers with `Access-Control-Allow-Origin: *` and the
content security policy is unset; both are S1.1's decisions by the plan's own text, and the
page loads nothing remote today.
