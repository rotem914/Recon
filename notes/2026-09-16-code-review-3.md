# Code review, 2026-09-16, the whole repository with Stage 3 in

Asked for by Rotem with `GO CODE REVIEW`, one agent, no sub-agents. Scope: the whole tree
at commit `965b53b`, plus the uncommitted image-size band in the working tree, read file by
file: product code first (`host/src/*.rs`, `host/src/capture`, `host/src/source`,
`host/pixels`, `editor/index.html`, the Tauri config and capability), then the checks code
(`selftest.rs`, `measure.rs`, `bench.rs`, `fixtures.rs`, `report.rs`, `editor-checks.js`,
`bench.html`), then the process tooling (`project-os/guards/*.mjs`, `install-hooks.mjs`,
`rotate.ps1`, `backup.ps1`). Calibration: `project-os/Code_review.md`; the worst class is a
note's text or a source pixel lost quietly, and the always-check list was walked row by row.

Nothing was changed and no check was run: every finding waits on a verdict, fix, drop or
backlog. Every finding is pre-existing; the step that brought it in is named where it helps.

Counts: 1 blocking, 3 important, 8 nits. From the 2026-09-13 pass: R6 (a capture arriving
mid-typing) is closed by S1.1's stash, which commits the note first; R7 is in the Backlog as
F68; R8 (an overlay panic leaks its windows), R9 (a poisoned region mailbox) and R10 (the
SVG raster cap allows a gigabyte) are still open as written there and are not repeated.

The always-check list held everywhere it was looked for: the preserved image never crosses
a canvas (`Managed::frame` decodes the PNG in the host), an external file is only read
(`source::open`, WIC with `GENERIC_READ`, the folder listing), committed text is read with
`innerText`, every way out of a note commits it (a click, a mode change, a document switch,
Escape, the timeline), a late region answer is dropped by ticket, the selection guard
releases on drop, dimensions travel in the body, DPI awareness is the first call in `main`,
the page's ratio is measured, and the check-only commands are behind the feature.

## Blocking

```
T1 · A keystroke during an in-flight save can be left off the disk        🔴 open
Where:   editor/index.html, saveNow (the lines "if (saving) await saving;" then
         "save.dirty = false;")
Problem: saveNow copies the notes, then waits for a save already on its way, and only
         after that wait clears the dirty flag. A keystroke typed during the wait marks
         the document dirty and arms the 800 ms timer; the wait ends, the flag is cleared
         over it, the OLD notes go to disk, and the timer's save finds nothing dirty and
         returns. The HUD says "saved". The text is in the page's memory only: Escape
         hides the editor with "saved", and a quit or a restart then loses those
         characters. The window is the length of one write, a few milliseconds, and it
         needs a second save (blur, Escape, Ctrl+Enter, the timer) to be called inside
         it; rare, but the class is the calibration's worst one, so it is rated by the
         class and not by the odds. Rotem's to lower.
Fix:     Clear the flag before the wait, not after: move "save.dirty = false" to just
         after the notes are copied (before "if (saving) await saving"), so a keystroke
         during the wait re-dirties the document and the timer's save carries it. Leave
         the failure branch as it is, it sets dirty back to true.
Verify:  a new editor check in section 23: start a save, type one more character while
         it is on its way (dispatch an input event before awaiting it), wait for the
         timer, read the record with editor_store_read: the last character is on disk.
         Then `cargo build` and `--editor-check`, every check green.
Status:  [ ] open
```

## Important

```
T2 · A capture deleted while its image is still being encoded loses the image   🟠 open
Where:   host/src/editor.rs, preserve (the thread) and editor_delete_document;
         host/src/store.rs, write_source
Problem: A capture's record is written at once and its PNG on a thread a moment later.
         Ctrl+Delete, or the thumbnail's ×, in that moment moves the folder to the trash
         with document.json only; the thread then calls write_source, which finds no
         source.png, recreates documents/<id>/ and writes the PNG there. The trash holds
         a record with no image, so Restore is refused ("came back without its image"),
         and the recreated folder has no record, so every startup logs it as skipped.
         The pixels survive on disk, but the capture is gone from every list and cannot
         come back. The window is the encode of a full capture, about 100 ms and more in
         a debug build; deleting a capture the instant it lands is an ordinary gesture.
Fix:     In the preserve thread, after the encode, take the DOCUMENTS lock and check the
         id is still listed before writing. If it is gone, write the PNG into
         trash_root()/<id>/source.png when that folder exists (the document was trashed),
         else drop it. Never recreate a folder under the store root for an id that is
         not in the list.
Verify:  a new editor check in section 29: editor_capture_probe at a large size, then
         deleteDocument at once; wait a second; editor_store_list shows no folder for
         the id, editor_trash_list shows it, and Restore brings it back with its picture.
         Then the project checks.
Status:  [ ] open
```

```
T3 · The store's writes are never flushed before the rename                    🟠 open
Where:   host/src/store.rs, write_atomic; host/src/export.rs, write_new
Problem: write_atomic writes the temporary file with std::fs::write and renames it into
         place with no sync_all. On a power cut or a crash of the machine in the seconds
         after, NTFS can commit the rename before the data, and the file that survives is
         document.json or source.png with zero bytes: a note whose text vanished, or a
         document whose image is gone, with no half file to warn anyone. QA §5 asks for
         a write that is whole or absent; this one is whole or absent only while the
         machine stays up. write_new for Save As has the same gap: flush() on a File
         flushes nothing.
Fix:     In write_atomic open the temporary with File::create, write_all, then sync_all
         before the rename; in write_new call sync_all before returning Written::New.
         A few milliseconds per write.
Verify:  the store unit test and the export unit test still pass; a save in the running
         app still shows "saved" within the debounce. There is no test for a power cut;
         the change is the documented remedy.
Status:  [ ] open
```

```
T4 · Closing the editor brings the last capture's application to the front,
     however the editor was opened                                              🟠 open
Where:   host/src/focus.rs, TARGET and return_to_target; host/src/editor.rs, hide
Problem: The return target is remembered at the hotkey and never cleared. Every hide
         (the close button, Escape, Copy and Return) activates it. So after one capture
         from Chrome, opening a file from Explorer and closing Recon brings Chrome
         forward instead of leaving Explorer where it was; opening the editor from the
         tray hours later and closing it does the same. §3.1 says the focus goes back to
         where the capture began, and never to an unrelated window; this is that window,
         one session later.
Fix:     Two options. (a) Clear the target once it has been used: return_to_target takes
         the slot, so the second hide with no capture between activates nothing and
         Windows picks. (b) Clear it whenever the editor is shown by a route that is not
         a capture: editor::show from the tray, open_path, a second instance. (a) is one
         line and matches "a chain of captures still returns", since each capture
         remembers afresh. Rotem's to say whether a tray-opened editor should return
         anywhere at all.
Verify:  the S1.10 check "the target is there" still passes; a new line after it: hide
         again with no capture between, editor_last_return says "no application to
         return to". Then the self test's section G.
Status:  [ ] open
```

## Nits

```
T5 · The Save As dialog calls itself a PNG dialog while offering JPEG          🟡 open
Where:   host/src/dialog.rs, save_png, the title "Save As a PNG, a new file"
Problem: Since S2.6 the type list offers JPEG beside PNG and the write encodes by the
         chosen name; the title still says PNG, and the function is still named for it.
Fix:     A title without the format: "Save As, a new file". Rename the function if the
         diff is touched anyway.
Verify:  the S1.11 check still passes; the title read by eye once.
Status:  [ ] open
```

```
T6 · The export blur is a box blur with no running sum                         🟡 open
Where:   host/src/compose.rs, blur_rects
Problem: Every output pixel sums up to 81 neighbours per pass, two passes, both axes:
         a blur over a 3840 by 2160 region at the 40 px radius is about three billion
         additions inside the Copy command, on the thread the page waits on. Seconds of
         a frozen editor for a large blur. Small blurs are fine.
Fix:     A sliding window per row and per column: add the entering pixel, subtract the
         leaving one, divide once. Same output, linear in the region.
Verify:  the S3.4 check "exporting again gives the same blur" still passes, and a blur
         over the whole of a 4K probe copies in under 200 ms in release.
Status:  [ ] open
```

```
T7 · A document is decoded from its PNG under the documents lock                🟡 open
Where:   host/src/editor.rs, show_document (document.frame inside the lock)
Problem: Reading and decoding source.png of a large document, 100 ms and more, holds
         DOCUMENTS; the timeline's editor_documents, a thumbnail's outgrown check and a
         save in flight all wait for it. Not a wrong result, a stall at every click on
         the strip.
Fix:     Clone the Preserved handle (the path, or the Arc) under the lock, release it,
         then decode.
Verify:  the S2.1 check still passes; the stall is not measured by any check today.
Status:  [ ] open
```

```
T8 · Two thumbnail makers for one document share one temporary file name       🟡 open
Where:   host/src/store.rs, write_atomic ("{name}.{pid}.tmp"); host/src/editor.rs,
         the thumb= branch (a thread per request)
Problem: The temporary name is the target plus the process id, so two threads making
         thumb.png for the same id at once, which a fast scroll of the strip can ask for
         before the first has cached its URL, write the same temporary file and one of
         the renames fails, logged as "thumbnail not kept". Harmless today, since both
         hold the same bytes, and a trap for the next writer that shares a target.
Fix:     Add a per-process counter to the temporary name, or hold a per-id lock around
         the make.
Verify:  the store unit test still passes; a scroll back and forth over thirty
         thumbnails logs no "not kept" line.
Status:  [ ] open
```

```
T9 · Ctrl+S twice opens two Save As dialogs                                     🟡 open
Where:   editor/index.html, saveAs; host/src/editor.rs, editor_save_as
Problem: Nothing refuses a second Save As while one is open: a second press composes
         again and starts a second dialog thread over the first, and the outcome slot
         then reports whichever closes last.
Fix:     In the page, ignore Save As while the last outcome's state is "open" (the
         save-as-done event clears it); or in the host, refuse when SAVE_AS_OUTCOME is
         open.
Verify:  press Ctrl+S twice with the dialog up: one dialog; the S1.11 check still passes.
Status:  [ ] open
```

```
T10 · A delete whose move to the trash fails drops the document from the list    🟡 open
Where:   host/src/editor.rs, editor_delete_document
Problem: The document is removed from the list before the folder is moved; if the move
         fails (a file held open, a full disk) the folder stays among the documents and
         the document is gone from the timeline until the next startup reads it back.
         Logged, never shown.
Fix:     Move first, then remove from the list on success; on failure keep it listed
         and return the error so the page shows it as a notice.
Verify:  a check with the store broken (editor_store_break on) deletes a document: the
         notice says NOT DELETED and the timeline still shows it.
Status:  [ ] open
```

```
T11 · The decode report watches the wrong Recon folder                          🟡 open
Where:   host/src/source/report.rs, recon_data_dir (APPDATA\Recon)
Problem: The report's "nothing was written to Recon's own data folder" hashes
         %APPDATA%\Recon, which holds the hotkey config; since S1.8 the documents live in
         %LOCALAPPDATA%\Recon. The check can no longer see a document written by mistake
         during a decode.
Fix:     Snapshot both folders, or the store root and the trash beside it.
Verify:  `--decode-report` on the fixtures folder still passes, with the second line
         naming the documents folder.
Status:  [ ] open
```

```
T12 · The exit on a Windows logoff is still unverified                          🟡 open
Where:   host/src/main.rs, the ExitRequested handler ("NOT VERIFIED" in the comment)
Problem: An exit request with no code is refused, so the only way out is the tray. The
         comment from S0.8 says a session logoff was never tried; if it arrives as such a
         request, Recon holds up the shutdown until Windows kills it, and the page's
         debounced save has no chance to land. Still not in the S0.8 record.
Fix:     None until measured: log off with Recon running and a note typed in the last
         second; see whether Windows shows Recon as blocking, and whether the note is on
         disk afterwards. Then either accept the request with no code, or save on
         WM_ENDSESSION.
Verify:  the logoff itself, by hand, once; the result into History.
Status:  [ ] open
```

## Reach

No change was made, so nothing propagated. Of the fixes proposed: T1 is one line in the
page's save path, which every save goes through; T2 and T3 are in the store's write path,
which every document, thumbnail and export goes through; T4 is in the one place the focus
is returned, which every hide goes through. Each is shared, and each fix should be followed
by the full editor checks, not only its own section.

## Not found, and looked for

An external file written or copied on any viewing path; a source pixel changed by the
composer outside the layer; a note losing a line break on commit; a way out of a note
that skips the commit; an unescaped file name reaching the page's markup (every name goes
through textContent or a title attribute); a check-only command reachable in the product
build; a region answered to another origin; a temp file left after a write; the invariants
in `CLAUDE.md` rule 11 in every path that opens, shows, annotates or exports a file.

## The working tree

The uncommitted image-size band (`editor/index.html`, `editor-checks.js`,
`project-os/Design.md`, `host/references/s06/environment.txt`) was read as it stands. It
raises nothing on its own; it is another session's work in progress with no History row
yet, and its checks already expect the 128 px default that `f49447f` committed.
