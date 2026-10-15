# Code review, 2026-10-15, the whole repository after the timing log and the app folder

Asked for by Rotem with `GO CODE REVIEW`, one agent, no sub-agents. Scope: the whole tree at
commit `7f33cb1`, plus the uncommitted work in the tree (the `app/` move's records, the
heavy-folders kit task, the Mistakes row), read file by file: product code first
(`host/src/*.rs`, `host/src/capture`, `host/src/source`, `host/src/scrolling`, `host/pixels`,
`editor/index.html`, the Tauri config and capability), then the checks code (`selftest.rs`,
`measure.rs`, `bench.rs`, `fixtures.rs`, `report.rs`, `editor-checks.js`, `bench.html`), then
the process tooling: `Find-heavy-files.mjs` and `Archive-old-rows.mjs` in full, the other five
scripts and the three guards by their write and delete paths only (every write there is a
temporary file and a rename, every delete is inside `.tmp/`, a check folder of the script's own
or a partial file of its own), not line by line. Calibration: `project-os/Code_review.md`; the
worst class is a note's text or a source pixel lost quietly, and the always-check list was
walked row by row.

Every finding is pre-existing; nothing in the code changed. Counts: 1 blocking, 0 important,
4 nits. Verdicts, fix / drop / backlog, are Rotem's.

From the earlier passes, still open as written there and not repeated: R8 (an overlay panic
leaks its windows), R9 (a poisoned region mailbox) and R10 (the SVG raster cap allows a
gigabyte) from 2026-09-13; T12 (the exit on a Windows logoff) from 2026-09-16; the Visual QA
nits T4 to T8 of 2026-10-06, waiting on verdicts.

The always-check list held everywhere it was looked for: the preserved image never crosses a
canvas (`preserved_frame` decodes the PNG in the host); an external file is only read
(`source::open`, WIC with `GENERIC_READ`, the folder listing, the opened-files pointers); committed
text is read with `innerText`; every way out of a note commits it; the dirty flag clears before
the wait; every store write is flushed before its rename (`write_into`, `write_new`, the
settings file); a document's folder is written only for a listed document under the lock, and
`write_beside` everywhere else; no decode runs under the documents lock (the handle is cloned
first, three callers); the save state travels in the stash; a late region answer is dropped by
ticket and superseded in the host; the selection guard releases on drop; dimensions travel in
the body; DPI awareness is the first call in `main`; the page's ratio is measured, never from a
minimized window; the check-only commands are behind the feature, and every check writes beside
its executable or into the checks' store; the timing log carries sizes and milliseconds only.
The one row that did not hold is T1.

## Blocking

```
T1 · A capture whose picture never reached the disk is lost at the next start,   🔴 open
     while the editor says saved
Where:   host/src/editor.rs, preserve (the thread's Err arm: "document {id} image NOT
         saved, kept in memory", Preserved::Encoded and save_error), editor_save_notes,
         quit_after_saves and editor_saves_flushed; host/src/store.rs, read_one ("has no
         source.png, left alone")
Problem: A capture's record is written at once and its PNG on a thread a moment later. When
         that write fails, the thread keeps the PNG in memory as Preserved::Encoded, sets
         save_error, logs one line, and nothing reads either again: no later save retries
         the image (editor_save_notes rewrites the record alone), and save_error reaches
         neither ImageInfo nor the page, so the HUD says saved after every note typed. The
         folder on disk holds document.json with no source.png; at the next start read_one
         skips it as a folder without an image, and the capture, notes and all, is gone from
         the timeline for good, its pixels never on disk. A full disk is the plain way in:
         the record is a kilobyte and lands, the PNG is megabytes and does not, and the
         notes' saves keep landing after it. A second way in ends the same: Quit from the
         tray, or a logoff, inside the encode of a fresh capture, since quit_after_saves
         waits for the page's notes and never for the image thread, and app.exit(0) ends the
         thread mid-write. The window is the encode, a tenth of a second to a few in a debug
         build for a large capture. The screen and the HUD agree and the disk disagrees,
         which is the calibration's worst class.
Fix:     Three parts. (1) Retry the image: in editor_save_notes, when the document's preserved
         is Encoded, call store::write_source again and swap to OnDisk on success. (2) Say
         it: editor_save_notes returns the image's failure too, so the HUD reads NOT SAVED
         with the reason while the picture is not on disk, and ImageInfo carries the same for
         a document shown again. (3) Wait at quit: count encodes in flight (raised before the
         thread spawns, lowered when it ends) and let quit_after_saves exit only when the page
         has flushed and the count is zero, inside its 1.5 s cap. Rotem's to say whether a
         capture whose image can never land should be dropped from the list with a notice
         instead of kept listed.
Verify:  a new check in section 23: editor_store_break on, editor_capture_probe, wait until
         editor_history shows bytes for it (the PNG encoded and held), the HUD reads NOT
         SAVED; editor_store_break off, a note typed; within three seconds editor_store_list
         has its folder with source_png true and the note, the HUD reads saved, and
         editor_store_reload reopens it with its picture. For the quit, a unit test on the
         counter, and by hand once: a capture, Quit from the tray at once, the folder holds
         source.png. Then cargo build and --editor-check, sections 23 and 39 green.
Status:  [ ] open
```

## Nits

```
T2 · Each capture's image is written and flushed to disk under the documents lock   🟡 open
Where:   host/src/editor.rs, preserve (the thread: DOCUMENTS.lock() then store::write_source);
         host/src/store.rs, write_into (write_all, sync_all, rename)
Problem: The thread takes the documents lock and, under it, writes the PNG through
         write_atomic, which flushes the file to the disk before the rename: 5 to 30 MB
         synced while every editor_documents (the timeline refresh the capture itself
         triggers), editor_save_notes, editor_image_info and editor_delete_document wait.
         Tens of milliseconds on an SSD, hundreds on a slow or busy disk, once per capture,
         right as the editor comes up. The lock was taken on purpose (review 3, T2) so a
         delete cannot move the folder between the check and the write; the stall is the
         price, and it is BugAtlas row 5's class, work on the disk under the documents lock.
Fix:     Write the temporary file into the document's folder with no lock held, then take the
         lock for the listed-or-not check and the rename alone, which is microseconds. The
         folder exists once the record was written; a delete meanwhile moves the whole folder
         with the temporary inside it, the rename then fails, and the trash branch writes
         afresh as it does today. store::write_source grows a two-step form, or write_into
         is split into "prepare" and "place".
Verify:  cargo test; --editor-check sections 23 and 39 (a capture deleted while it encodes,
         and undone, still comes back whole). No check measures the stall.
Status:  [ ] open
```

```
T3 · The app background has drifted between its homes                            🟡 open
Where:   host/src/magnifier.rs:35, FILL_RGB (0x0D, 0x0E, 0x12); editor/index.html:9,
         --paper #0C0C0E; host/src/selftest.rs:707, the ground expected as (13, 14, 18);
         project-os/Design.md, Colours, "App background" and the "second home" line under
         Capture size bubble
Problem: Rotem moved the app background from #0D0E12 to #0C0C0E on 2026-09-17. The page
         followed; the host's constant did not, so the capture's magnifier panel, the
         editor's ruler and colour picker circle (drawn by the host since 2026-09-19) and the
         scrolling capture's bar stand on the old colour, one shade off the page they sit on.
         Design.md names the host's constant as a second home of this token, and Backlog F68
         is the same shape for the margin colour. The self test's sections C and H check the
         panel's ground against the old value, so a fix moves them together.
Fix:     FILL_RGB to (0x0C, 0x0C, 0x0E) and selftest.rs lines 707 to 709 to the same;
         Design.md's second-home line to name magnifier.rs, where the constant moved on
         2026-09-19. A one-home fix is F68's: not here.
Verify:  --selftest sections C and H pass with the new ground; a capture's panel beside the
         editor's size band, one colour by eye; --editor-check sections 44 and 45 unchanged.
Status:  [ ] open
```

```
T4 · A file opened by a relative path, or on a drive that is away, is forgotten   🟡 open
Where:   host/src/editor.rs, open_path_as (links.push(Link { path: path.to_path_buf() ..}))
         and editor_documents (links.retain(|l| l.path.is_file())); host/src/export.rs,
         same_file; host/src/main.rs, file_argument and the single-instance closure, whose
         cwd is _cwd, unused
Problem: A pointer stores the path as it was handed over. Opened by a relative path
         (recon-host.exe pic.png from a terminal, which file_argument accepts after is_file()
         against that process's own folder), the pointer holds "pic.png"; every later
         is_file() runs against the running Recon's folder, so after a restart, or at once
         when a second instance hands the path to a Recon started from the tray at logon,
         the pointer is dropped at the next timeline refresh, with no notice and no undo. The
         hand-off also never reads the second instance's own folder, so a relative path from
         another folder opens nothing. The same pruning forgets a pointer whose drive is away
         for a moment, a USB stick or a network share, for good. And same_file, a string
         comparison, cannot match a relative stored path against the dialog's absolute one,
         so Save As over a JPEG opened that way is offered a new name instead of writing over,
         the safe side. Explorer, the picker and Open with hand over absolute paths, so the
         everyday path is fine; the terminal and the unplugged drive are the cases.
Fix:     Make the path absolute at open_path_as (std::path::absolute), and resolve the second
         instance's path against its cwd before file_argument. For the pruning, drop a
         pointer only when its folder is there and the file is not, and keep one whose folder
         cannot be read; whether the thumbnail should say "away" is Rotem's call.
Verify:  cargo test: a relative path is stored absolute; the "an opened file in the timeline"
         check still passes; by hand: recon-host.exe pic.png from another folder with Recon
         in the tray opens it; a file on a USB stick opened, the stick out, the timeline
         refreshed, the stick back: the pointer is there.
Status:  [ ] open
```

```
T5 · A viewed TIFF, HEIC or AVIF stays open on disk while it is on screen         🟡 open
Where:   host/src/source/wic.rs, open (CreateDecoderFromFilename with GENERIC_READ, the
         decoder kept in Wic for the life of the Opened, the _bytes parameter unused)
Problem: source::open has already read the whole file into bytes; WIC then opens it again by
         name, and the decoder, which Opened holds in state().document until another picture
         replaces it, keeps that handle. While a TIFF, HEIC or AVIF is on screen, Explorer's
         rename, move or delete of the file is refused as in use, and so is another
         program's save over it; a PNG or JPEG, decoded from the bytes, has no such hold.
         Inferred from WIC's contract (a decoder made from a file name reads the file on
         demand), not measured here. Viewing never modifies the file, so rule 11 holds; it
         holds the file instead.
Fix:     Decode from the bytes already in memory: an in-memory stream over them
         (SHCreateMemStream, or CreateStreamOnHGlobal) and CreateDecoderFromStream, the
         stream owned by Wic. The second read of the file goes away with the handle.
Verify:  --decode-report on the fixtures passes, the TIFF's three pages and the hash check
         included; by hand: a TIFF open on screen, renamed in Explorer, the rename goes.
Status:  [ ] open
```

## Reach

No change was made, so nothing propagated. Of the fixes proposed: T1 sits in the preserve
thread, the save path and the quit, which every capture, every save and the tray's Quit go
through, so it is shared, and its check has to cover a capture deleted during its encode
(section 39) as well as the quit. T2 is the same thread's write path, shared with T1 and best
done with it. T3 is one constant read by the capture overlay, the editor's magnifier and the
scrolling bar, and by the self test. T4 reaches the opened-files list, the hand-off and Save
As, three paths. T5 is local to the WIC provider.

## Not found, and looked for

An external file written or copied on any viewing path, the opened-files pointers included; a
source pixel changed by the composer, the crop or the blur outside the layer; a note losing a
line break on commit; a way out of a note that skips the commit; a keystroke lost during a
save; a store write not flushed before its rename; a document's folder made for an unlisted
document; a decode under the documents lock; a save state reset over stashed notes; an
unescaped file or tab name reaching the page's markup; a check-only command in the product's
command list; a check writing into a real folder; a region answered to another origin; a temp
file left after a write; the tabs file written out of order; the crop reaching the document's
pixels; the timing log carrying a name or a path; the scrolling capture's bar in a copy of the
screen; the stitcher growing past its cap; the pointer drawn outside its box; the registration
taking a default; the invariants in `CLAUDE.md` rule 11 in every path that opens, shows,
annotates, crops, exports or deletes a file.

## The working tree

The uncommitted changes (`.gitignore`, `Decisions.md`, `History.md`, `Map.md`, `Mistakes.md`,
`Workflow.md`, the new `notes/2026-10-15-projectos-heavy-folders-task.md`) are the records of
the `app/` move and of the heavy-folders kit task, another session's; they were read as they
stand and raise nothing about the code.
