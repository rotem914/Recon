# Recon — Product Plan and Claude Handoff

**Image viewing, screen capture and annotation for Windows.**

Owner: Rotem. Personal tool, intended for an open-source release.  
Status: revised product definition; no implementation or performance validation yet.  
Updated: 2026-09-10, revision 2. Image viewing is now a core capability.

This document replaces the original macro plan. It incorporates the product review and supplies explicit starting defaults where the original left behavior open. These defaults guide implementation; they are not claims that the interactions have already been tested. Stack selection remains conditional on Stage 0.

---

## 0. Revision note, 2026-09-10

Rotem's clarification: Recon is his primary everyday image viewer as well as his capture and annotation tool. It opens common image formats directly from the computer.

This supersedes the earlier deferral of "importing existing images". Every other settled product decision stands.

What that changed in this document:

- §1 has three core capabilities instead of two, and names the viewer it replaces.
- §2 gains principle 8: viewing never changes the file.
- §3.1 gains the file-opening entry points, file-type registration and the default-app path.
- §3.3 becomes two entry behaviors in one window: a capture opens ready to annotate, a file opens in viewing mode behind an explicit Annotate action.
- §3.4 is new: everyday viewing behavior and folder navigation.
- §4.5 is new: how annotation begins from an animation, a multipage image and a vector file.
- §5 gains the viewing keys and moves Save As into the first daily-use release.
- §5b is new: format support, stated per format instead of as "common images".
- §6.0 is new: an external file is never modified and never imported by being viewed.
- §8 drops "importing existing images" from the maybe-later list and names what viewing still excludes.
- §9 gives Stage 0 a decoding experiment, Stage 1 the viewing loop plus PNG Save As, and expands the acceptance checks.

Format tradeoffs that still need a decision are in `project-os/Plan.md` part 3, items D, E and F.

---

## 1. Product definition

Recon is a general-purpose tool for viewing an image, taking a screenshot, adding clear annotations, and getting the result to its destination quickly.

**One line:** open or capture an image, annotate it, and get it where it needs to go — without losing the thread of the work you were doing.

Three core capabilities, and none of them is an add-on to the others:

- **Viewing** an image that already exists on the computer.
- **Capturing** a region of the screen.
- **Annotating** either of them.

The first version should replace Rotem's everyday region-capture and annotation workflow in Snagit, and his everyday image viewer. Full Snagit feature parity is not a v1 requirement, and neither is a full image-management application.

The central interaction is an integrated callout: an anchor point, connecting arrow, and editable text bubble form one object. The central capture workload is a sequence of 20–30 captures, with occasional returns to earlier images for corrections. The central viewing workload is opening a file from Explorer and looking at it, or walking a folder of images, many times a day, with no annotation at all.

Client feedback is the first concrete annotation workload to validate. Recon must also work well for a screenshot without annotations, for a single annotated image copied into another application, and for an image that is only looked at. Rogers is one output workflow, not the definition of the product.

### Positioning accuracy

Describe the advantage through Recon's defaults and measured experience. Snagit already supports global capture shortcuts and a Copy All command; see the [official shortcut guide](https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/). Do not base the product on claims that these capabilities are absent or that an old capture library necessarily causes slow operation.

The same honesty applies to viewing. Windows Photos exists and opens most of these formats. The claim Recon can make is the one it can measure: opening quickly, showing the image at a predictable size, and being one keystroke from annotating it.

---

## 2. Design principles

1. **Fast access is a core feature.** Measure time until the user can select a region and time until they can type an annotation. Recon should feel readily available from the background. Opening a file gets the same treatment: measure time from activation to a visible image.
2. **No routine decisions during capture.** Choose preferences and destinations outside the capture loop. Do not interrupt normal capture with format prompts or save dialogs.
3. **Keyboard-first operation.** Repeated actions need discoverable shortcuts. Preserve normal text-editing behavior while typing.
4. **The image is always ready.** Copying and exporting use current visible edits. There is no Done or Apply step for annotations.
5. **Repeated use is the baseline.** Evaluate 30 captures, navigation, and corrections, not only the first successful screenshot. Evaluate a folder of images the same way.
6. **Automation remains correctable.** Automatic callout placement is a starting position. The user can always move it.
7. **Work survives normal navigation and closure.** Hiding the editor, taking another capture, opening another file, or exiting normally must not silently discard completed work.
8. **Viewing never changes the file.** Looking at an image does not modify it, move it, re-encode it, or copy it into Recon's own storage. This one is not a preference; it is the promise that makes Recon safe to point at a folder of originals.

---

## 3. Capture, viewing and editor lifecycle

### 3.1 Application behavior

- Recon runs in the background with a tray entry and a configurable global capture shortcut.
- The tray provides access to the editor, preferences, and an explicit Quit action.
- Closing the editor hides it; Quit exits the process after saving pending changes.
- Opening the editor without capturing restores the latest capture and its editable annotations.
- Preferences are accessed outside the normal capture flow. Startup with Windows is a preference, not a prerequisite for use.
- If a capture shortcut is unavailable, explain the conflict and allow another shortcut. Do not silently take over another application's shortcut.
- Remember the last external application as the return target, including when another capture starts from inside Recon. If that application has closed, hide Recon without activating an unrelated application. Background completions must not steal focus.

**Opening a file is a normal way in, not a special case.** All four of these reach the same window:

- Double-click in Explorer, through a file association.
- Open With, for a type Recon is not the default for.
- Drag and drop a file, or a selection of files, onto the window.
- `Ctrl+O` from inside Recon.

Both starting states must work: Recon already running in the background, and Recon starting because a file was activated. A second activation while it is running opens that file in the existing window rather than starting another instance, and brings that window forward. Opening a file never discards unsaved annotation work; §6.0 says what happens to it.

**Registration and defaults.** Recon registers the file types in §5b that it can actually display, and its preferences link straight to Windows' own default-apps settings so Rotem can make it the default viewer. Recon never silently takes a file association it was not given, and never re-takes one the user changed.

### 3.2 Capture flow

Global shortcut → freeze the desktop image → dimmed selection overlay → drag a region → open the editor with that region.

- If Recon is visible, hide its windows before taking the snapshot. Recon's editor, dimming, and selection decorations must not appear in the result.
- Selection uses the frozen image, rather than content changing underneath the user while dragging.
- Support selecting a region on any connected display. A single region spanning multiple displays is deferred from v1; keep selection within its starting display.
- `Esc` cancels selection and restores the preceding context. Cancellation creates no capture and does not replace the previous one.
- Ignore overlapping capture requests while a selection is already active.
- No routine save dialog, format choice, or intermediate confirmation.

Stage 0 must verify the hide/snapshot/focus behavior with real applications, including an open menu or changing content. Document any limitation instead of implying universal capture support.

### 3.3 One window, two entry behaviors

The same main window serves both, and which one it is must be obvious at a glance.

**A new capture opens ready for annotation.** Behavior as it has always been: the callout tool is live and a click starts a note.

**An existing file opens in viewing mode.** Ordinary clicking and dragging pan, select nothing, and create nothing. No stray callout can appear on someone's photograph because the pointer moved. An explicit **Annotate** action switches that window into annotation, arms the tools, and creates the managed document described in §6.0. Leaving annotation returns to viewing without discarding the work.

Everything else about the editor holds for both:

- One main editor contains the canvas and, as the history UI develops, a capture strip at the bottom.
- The image fits the available workspace without changing its underlying pixel dimensions.
- Provide zoom, pan, and access to actual-size viewing as needed to annotate large images accurately.
- There is no separate viewer or library window in v1. Older captures remain accessible from the same editor.
- Taking a new capture saves the current document and adds another; it does not overwrite the previous capture.
- Copying leaves the editor open. Copy and Return copies the image, then hides the editor and returns focus to the preceding application.
- Hide only after the clipboard operation succeeds and pending document changes are saved. On failure, retain the work and show an actionable message.
- If the user starts another action or changes the document while Copy and Return is completing, report the copy result without hiding their current work.

### 3.4 Everyday viewing

This is the part that has to be good enough to use all day, with no annotation involved.

- **Fit to window** on open, **actual size** on demand, zoom in and out, and pan. Zoom never re-encodes anything and never changes what an export would contain.
- **Fullscreen**, with one key in and the same key or `Esc` out.
- **The filename and the pixel dimensions are visible**, without hunting for them.
- **Previous and next walk the folder** the opened file came from, across the supported types in that folder, with a position indicator (for example 12 of 240).
- **The order is defined, not incidental**: a numeric-aware, case-insensitive filename order, so `img2` sorts before `img10` and the sequence matches what Explorer shows closely enough to be predictable. State the exact comparison in the technical plan rather than inheriting whatever the file system returns.
- **The folder listing is read once per navigation context** and is not rescanned on every keystroke. A file that has since disappeared is skipped with a quiet note, not an error dialog.

**Folder navigation and capture history are two different lists.** Walking a folder never moves through captures, and walking captures never moves through the folder. The window says which context is active and what position it is at. If both exist at once, the active one is the one the last navigation used, and it is named on screen.

---

## 4. Annotation behavior

### 4.1 Primary object: callout bubble

With the callout tool active:

1. Click the point on the screenshot that the note refers to.
2. Create an anchored bubble nearby and place the text cursor inside it.
3. Type the note. The visible text is immediately part of the composed image.
4. Press `Esc` to leave text editing while retaining the text.

Clicking an existing bubble selects it instead of creating another. Double-click or Enter on a selected bubble starts text editing. A newly created bubble left empty is discarded when text editing ends and is excluded from image output.

Each callout is one editable object containing its anchor, arrow, bubble, text, and displayed number. It supports selection, movement, text editing, deletion, and Undo/Redo from the first usable version. The anchor can be repositioned independently while the arrow remains attached.

### 4.2 Automatic placement and fallback

- Use a small deterministic set of candidate positions near the anchor initially. Avoid covering the anchor and existing bubbles where possible.
- Do not promise to recognize all meaningful image content or always find empty space. Content-aware placement is an improvement to evaluate later.
- Permit manual movement at any time. Moving a bubble preserves its anchor and updates the arrow.
- After manual movement, preserve that placement. Do not jump the bubble elsewhere as the user types or adds another bubble.
- Wrap long text. Keep the bubble's chosen origin stable while its size changes, and keep all text accessible and included in output.
- If the bubble cannot fit without clipping or covering its anchor, extend the canvas with a neutral annotation margin. Do not shrink or crop the captured pixels to make space. Export includes any added margin.
- Manual placement may cover image content; automatic positioning must remain easy to correct.

Validate this behavior on a small crop, a dense interface, anchors near each edge, long notes, and multiple bubbles before improving placement heuristics.

### 4.3 Numbering and text

- Number callouts by creation order, starting at 1 for each capture.
- Numbers remain stable after deletion; gaps are allowed and deleted numbers are not reused for new callouts. Undo restores the original number. Moving a bubble does not renumber it.
- The numbers exported as text to Rogers match the numbers visible in the image.
- Support Hebrew, English, and mixed-direction text in the initial callout implementation.
- The editor and rendered output must agree on wrapping, alignment, font rendering, and arrow positions.

### 4.4 Secondary tools

The planned secondary set is arrow, rectangle, text, blur, and highlight. Prioritize these after using the core loop for real work. Selection, movement, deletion, and Undo/Redo are core editing behavior and must not wait for this stage.

### 4.5 Annotating something that is not a single still frame

Annotation always operates on one still raster image. What that image is depends on what was open, and the answer is fixed rather than clever:

- **An animation.** Annotation starts from the frame on screen and produces a separate still image. The animated original is untouched, and Recon does not attempt to annotate every frame or to write an animation back out.
- **A multipage image.** Annotation operates on the page currently displayed, and the page number travels with the managed document so a later reader knows which one it was.
- **A vector file.** Annotation produces a raster composition at the size then displayed, and the original vector file is preserved exactly as it was. Recon does not write vector annotations back into it.

In all three cases the transition is explicit and stated on screen, because the user is moving from "the file" to "a picture of the file".

---

## 5. Keyboard and output contract

These are initial editor shortcut defaults. Verify them with Hebrew and English input layouts. Capture uses a separately configurable global shortcut; image-copy shortcuts below are local to Recon.

The **Stage** column says when the row is real. A row promising a key that does nothing yet is a contract Recon has broken on its first day.

| Action or context | Default behavior | Stage |
|---|---|---|
| `Ctrl+C` while editing text | Normal text-copy behavior; it does not unexpectedly copy an image | 1 |
| `Ctrl+C` outside text editing | Copy the full composed image, even when an annotation is selected | 1 |
| `Ctrl+Shift+C` anywhere in the editor | Copy the full composed image, including current text edits | 1 |
| `Ctrl+Enter` anywhere in the editor | Copy the full composed image and return to the previous application after success | 1 |
| `Enter` while editing a note | Insert a newline | 1 |
| `Esc` during capture | Cancel capture | 1 |
| `Esc` during text editing | Leave text editing and retain the text | 1 |
| `Esc` with an annotation selected | Clear the selection | 1 |
| `Esc` in fullscreen | Leave fullscreen | 1 |
| `Esc` in the otherwise idle editor | Save pending changes and hide the editor | 1 |
| Delete outside text editing | Delete the selected annotation | 1 |
| Undo/Redo | Typing undoes typing while a note is being edited; once editing ends that text change takes its place in document history as one grouped step | 1 |
| `Ctrl+O` | Open an image file | 1 |
| Previous / next image, outside text editing | Walk the current navigation context, folder or captures, in the order §3.4 defines | 1 |
| Fit to window · actual size · zoom in · zoom out | Viewing controls, no effect on export resolution | 1 |
| Fullscreen | Enter fullscreen | 1 |
| `Ctrl+S` | Save As a PNG file, to a new file; internal saving stays automatic | 1 |
| `Ctrl+Shift+Enter` | Send to Rogers, when that feature is available and configured | 4 |

Do not intercept ordinary typing or editing keys to trigger tools while a text field has focus. **This now covers folder navigation too:** the previous and next keys, and the viewing keys, are inert while a note is being edited, and typing a letter into a note never navigates away from the image it belongs to. Copying one annotation object is not required for v1.

### Image output

- Clipboard and file output contain the original screenshot or source pixels plus the visible annotations and any annotation margin.
- Copy and export snapshot the composition when invoked. Their completion must not discard or overwrite edits made afterward.
- Selection outlines, editing handles, caret, and tool UI are not output.
- Display zoom does not change export resolution. Use PNG as the initial file-export default.
- Provide a brief, nonblocking success indication only after the clipboard or export operation succeeds.
- Ctrl+S opens Save As with the last export folder and a unique suggested filename. A file dialog is appropriate for an explicit file export, not for taking a capture.
- **Save As writes a new file, always.** The suggested name is derived from the source and marked as annotated, and an existing file is never overwritten without the user choosing that file by name in the dialog. An external original is never the default target.
- File output is a rendered snapshot. It does not replace the editable internal document. Subsequent edits do not silently rewrite a previous export.
- Saving into a folder already watched by Drop Ninja lets that existing workflow handle the file. Recon needs no custom Drop Ninja integration; this depends on the user's configured watched folder.

## 5b. Format support, stated per format

"Common images" is not a specification. Each row below says what support means, and the technical plan verifies the decoding approach and its dependencies before any of it is advertised. A format that cannot make the first daily-use release is named as a gap with its impact, never quietly dropped.

| Format | What support means |
|---|---|
| **PNG** | Decode, transparency preserved, embedded color profile honored. Annotate directly. |
| **JPG / JPEG** | Decode, EXIF orientation applied on display and carried into annotation, embedded color profile honored. Annotate directly. |
| **BMP** | Decode. Annotate directly. |
| **WebP** | Decode, transparency preserved. An animated WebP follows the animation rules in §4.5. |
| **GIF** | Decode and play the animation, with transparency. Annotation follows §4.5: a still from the displayed frame. |
| **TIFF** | Decode, with multiple pages exposed as page navigation and the page count visible. Annotation operates on the displayed page (§4.5). |
| **HEIC / HEIF** | Decode, orientation applied. Annotate directly. Depends on codecs installed on the machine, so a missing codec is reported as a missing codec, with the way to install it, and never as a corrupt file. |
| **AVIF** | Decode, transparency preserved. An animated AVIF follows §4.5. |
| **SVG** | Render for viewing at the displayed size. Annotation produces a raster composition and preserves the original vector file (§4.5). |

Across every row: orientation and color handling are applied consistently in the viewer and in any annotated output, so an image never rotates or shifts color when the user presses Annotate. An unsupported or unreadable file says so plainly, names the format, and leaves the file alone.

---

## 6. History, persistence, and retention

### 6.0 An external file is not Recon's file

This section governs everything below it.

- **Viewing does not modify.** No re-encode, no metadata rewrite, no move, no rename, no thumbnail written beside it.
- **Viewing does not import.** Browsing a folder of 100 images creates zero managed documents. Recon holds a path, a decoded frame in memory, and nothing else.
- **Annotation is what creates a managed document**, and only for the file being annotated. That document holds a preserved copy of the source image as it was decoded, plus the annotation data, exactly like a capture's document.
- **Navigating away from annotated work saves it first**, in the managed document, with no Apply step. The external original still does not change.
- **Export writes a new file** (§5). Deleting a managed document does not touch the external original, and does not recall an export or a clipboard copy.

The distinction that already existed, between an editable internal document and a flattened exported image, is unchanged. This section adds a third thing that is neither: an external file Recon is only allowed to read.

### 6.1 Editable internal documents

Persist the original captured or source image and editable annotation data separately from flattened output. Include sufficient versioned document metadata to reopen the capture correctly after restarting, including, for an annotated external file, its source path and the page or frame it came from.

- Save new captures automatically and save subsequent edits without an Apply action.
- Save pending changes before normal hide, document navigation, opening another file, or Quit.
- Reopen the latest capture and recover all successfully saved documents after restart.
- Crash recovery is to the last successful save; do not claim zero loss for unpersisted keystrokes. Keep the autosave interval short and verify recovery behavior.
- A disk or persistence failure must be visible. Keep recoverable work in memory and provide image copying as an immediate recovery path, plus file export once it exists; do not label it saved or silently discard it.
- Image exports and clipboard copies are independent of the internal document. Deleting a local document does not recall copies, exported files, or a Rogers item.

The exact file/database arrangement belongs to technical planning. Avoid coupling the editable document format to one rendering library's private, unversioned serialization.

### 6.2 Capture strip and older history

- Treat the strip as recent capture history, not a new session-management product.
- The first usable stage provides basic previous/next navigation and restores recent work after restart.
- Stage 2 adds the full thumbnail strip and a way to reach older captures within the editor, with simple date grouping if useful.
- Changing a Rogers target does not clear the capture history.
- Load full-resolution image data when needed. Keep thumbnails and inactive captures from causing memory usage to grow with the entire library.
- The same discipline applies to a folder walk: one decoded image at a time, plus whatever small look-ahead is measured to help, and never the folder.

### 6.3 Retention default

For v1, retain captures until the user explicitly deletes them. No automatic expiry or silent deletion. External files are not retained at all, because they were never taken.

Stage 2 provides intentional deletion, visible storage usage, and a clear distinction between hiding a capture from the recent strip and deleting its stored document if both actions exist. Prefer one unambiguous delete action over adding unnecessary concepts.

Revisit optional age or size limits after real usage. Disk retention and active memory/cache limits must be handled separately.

---

## 7. Send to Rogers

### 7.1 Target setup

- Choose an existing project and tab before the capture sequence.
- Show the active destination persistently in the editor, for example `Dig › ID 1`.
- Remember the target, but verify that it remains valid before sending. A missing target requires selecting a destination; never silently choose a fallback.
- Creating a Rogers tab from Recon is deferred until choosing an existing tab proves insufficient.

### 7.2 Payload and result

- One capture creates one unchecked/open task in Rogers.
- Include the composed image and nonempty callout texts in stable numeric order, using the same labels as the image.
- A capture without callouts remains sendable. Use a simple capture-time title when no note supplies a title; verify the exact field mapping against Rogers' existing item model.
- Snapshot the target, image, and text at the moment of Send. Later edits or target changes must not mutate an in-flight submission.
- On confirmed success, mark the submitted snapshot as sent, retain the local editable document, and store the resulting item reference. Hide and return only if the user is still awaiting that submission on the same capture, has not edited or started another action, and Recon is still foreground; save any pending document/status changes first.
- If the user has moved to another capture, continued editing, or switched applications, update the original submission's status without closing current work or taking focus. Later edits remain marked as changed since the submitted snapshot.

### 7.3 Failure, retry, and sending again

- Persist a submission identifier and snapshot before attempting delivery. Use the same identifier for retries of that submission so a lost response cannot produce a duplicate task.
- Disable repeated Send for a submission already in progress.
- Do not mark a timeout as successful or assume that Rogers received nothing. Keep the submission available for reconciliation or retry with the same identifier.
- On a reported failure or uncertain result, keep the local capture, text, target, and status accessible. The first version uses explicit retry; a background delivery queue is deferred.
- Show a reference to the created Rogers item after success. Ordinary Send on an already-sent unchanged capture must not create another task.
- Editing a capture after sending marks it as changed since that submission. An explicit **Send as new task** action creates a new submission and task. Updating existing Rogers tasks is deferred from v1.

### 7.4 Integration boundary

Inspect Rogers before choosing the transport or designing new endpoints. Its current API, authentication, attachment behavior, and deployment model have not been verified by this plan.

Use or add a narrow, validated inbound operation in Rogers that creates the item and attachment reliably and supports safe retries. Keep it reusable at the request/response boundary; do not build a plugin platform or general integration framework. Recon must not write directly into undocumented Rogers database structures or report success for an incomplete item/image operation.

---

## 8. Scope boundaries

The first daily-use release is Stage 1. Later stages extend it without changing the core loop.

Out of scope for v1:

- macOS.
- Video or GIF capture, scrolling capture, and OCR/text extraction.
- A single selection spanning multiple monitors.
- Cloud sync, hosted libraries, and public sharing links.
- Templates, preset collections, and stamp libraries.
- Export targets beyond clipboard, files, and Rogers.
- A separate library application or window.
- AI-based semantic placement as a prerequisite for callouts.
- Automatic retention cleanup.
- Creating Rogers projects/tabs, updating previously sent tasks, or background delivery queues.
- A generalized integrations platform.

**Viewing is in scope; managing files is not.** Recon opens, shows and annotates an image. It does not rename, move, delete, tag, rate or organize external files, does not run batch operations over a folder, does not build a browsable thumbnail grid of the file system, and does not edit an image beyond annotating it: no crop, no resize, no color adjustment in v1. Writing back into an external file, in any format, stays out (§6.0).

Full-window capture and further editor tools can be evaluated from observed use; they do not block the initial loop.

---

## 9. Build order and acceptance criteria

Stage 0 is a feasibility experiment. Each subsequent stage must remain usable on its own. Use the result for real work and record friction before proceeding to the next stage. Do not implement the entire roadmap in one pass.

### Stage 0 — prove the capture, viewing and editor foundations

**Build only:** background hotkey → clean region capture → minimal canvas → editable, manually movable callout → image clipboard, plus a small file-opening and decoding experiment on that same canvas.

Start by evaluating Tauri + React + Rust, given the owner's experience with that stack in Copy Ninja. A familiar stack is a candidate, not proof of suitability. SQLite is a storage candidate for later stages, not a requirement for the experiment.

**Acceptance evidence:**

- Capture on two displays with different scale factors, including 100% and 150%/200% and a display positioned left of the primary one. Selected bounds and resulting pixels agree.
- Verify that the editor and capture overlay do not appear in the result; cancellation and focus restoration work.
- Check a small image and a 4K image, original-resolution output, and Hebrew/English mixed text. Paste into the two actual destination applications.
- **Open representative files on the same surface** and report, per format, whether it decoded, how long it took, and what it looked like: transparency, EXIF orientation, an embedded color profile, very large dimensions, an animation, and a multipage file. Name every format that did not work and what it would take.
- Measure process startup separately from capture activation while Recon is already running in the background. For background activation, initial working targets on the test machine are p95 ≤250 ms from the hotkey to usable selection and p95 ≤500 ms from completed selection to an editor ready for annotation input. These are proposed targets to evaluate, not validated promises; they do not apply to process startup.
- Run 30 captures with navigation/replacement of the experimental image and report latency and memory behavior. Separate live document memory from retained history; identify uncontrolled growth or accumulating delay.
- State the tested Windows version, hardware, display setup, and limitations. Explicitly determine HDR behavior before claiming support for it.

**Exit:** a short evidence-based recommendation to retain the stack or change the specific failing component, and a per-format verdict on decoding. Do not build the library or Rogers integration to compensate for an unproven capture or decoding path.

### Stage 1 — complete daily-use loop, viewing included

**Build:** reliable capture lifecycle, file opening and activation, the everyday viewing surface, folder navigation, the transition into annotation, full basic callout editing, deterministic placement with manual correction, context-aware keyboard actions, Copy and Return, PNG Save As, basic previous/next through captures, and editable autosave/reopen.

**Acceptance:**

- Capture and copy with zero annotations; annotate with one bubble; use multiple bubbles; correct an earlier capture.
- Complete a 30-capture sequence without losing previous work or requiring a Done step.
- **Open a file from Explorer by double-click and by Open With**, with Recon closed and with Recon already running; the second case uses the existing window and does not start a second instance.
- **Drag and drop a file onto the window**, and open one with `Ctrl+O`.
- **Walk a folder** of mixed supported types, in the defined order, with the position indicator correct at both ends; confirm folder navigation and capture navigation never move each other.
- **Move from viewing into annotation and back**, and confirm that no click in viewing mode ever created a callout.
- **Confirm every source file is byte-for-byte unchanged** after viewing, navigating and annotating, and that Save As wrote a new file.
- Include representative files: transparency, EXIF orientation, an embedded color profile, very large dimensions, an animation, and a multipage file where supported.
- Test small crops, dense images, each edge, long text, mixed Hebrew/English, and a bubble moved manually before its text changes.
- Verify undo/redo, delete, copy while text is being edited, copy with an object selected, and repeated capture while Recon is open.
- Restart normally and recover editable work. Simulate an interrupted run and verify recovery to the last successful save. Exercise clipboard and save failures without false success or discarded work.
- Compare equivalent capture/annotation/return tasks, and equivalent viewing tasks, with the owner's existing tools. Record concrete friction and results; do not substitute feature counts for usability evidence.

### Stage 2 — full capture library and file output

**Build:** thumbnail strip, access to older captures in the same editor, intentional deletion, storage visibility, and the export options beyond the Stage 1 PNG default.

**Acceptance:** retrieve and edit yesterday's capture after restart; exported files match the visible composition at original resolution; existing exports are not silently changed; old history is not all loaded as full-resolution images into memory.

### Stage 3 — secondary annotation tools

**Build:** arrow, rectangle, text, blur, and highlight, ordered by actual need discovered in daily use.

**Acceptance:** each tool participates in selection, editing, Undo/Redo, persistence, and identical clipboard/file rendering. Its addition must not disrupt the primary callout flow.

### Stage 4 — Rogers delivery

**Build:** choose and display an existing target, create one task per capture, send-and-return, submission status, item reference, and safe manual retry.

**Acceptance:** test successful delivery, invalid target, unavailable Rogers, rapid double-send, a response lost after item creation, retry after restarting Recon, changing the active target during a request, and editing/re-sending a previously sent capture. Verify the image/text destination and that retries do not create duplicates. Continuing to edit the same capture, moving to another capture, or switching applications during delivery must not cause a late success to close work or take focus.

---

## 10. Technical planning handoff for Claude

Use this document as the product baseline. The build plan is `project-os/Plan.md`; where the two differ, an approved revision there wins, and each one is listed in its revision blocks.

1. Read repository instructions and inspect the current workspace first. Do not assume code or infrastructure already exists.
2. Identify the smallest native capture path, image decoding path and editor rendering approach that can satisfy Stage 0. Inspect relevant existing Copy Ninja code only if it is available; reuse proven pieces selectively, without coupling Recon to that application's runtime.
3. Explain the boundaries between Windows capture/window management, file activation and decoding, the editor, editable documents, and image output. Specify the minimum supported Windows version based on the chosen native APIs. Keep package and database choices proportional to the first stage.
4. Describe how a single document model drives editing, autosave, and rendering. Call out coordinate conversion, text direction, callout sizing, original-resolution export, orientation and color handling as explicit risks to test.
5. Produce a bounded Stage 0 work plan, validation method, and go/no-go evidence for the stack and for each format. Distinguish measured facts, documentation support, and assumptions.
6. Identify material conflicts or unsupported requirements. Resolve ordinary implementation details using this plan; seek owner direction only for a real product tradeoff that changes the agreed behavior or scope.
7. Keep later stages as a short dependency outline. Inspect Rogers when its stage approaches; do not invent its current API or build integration infrastructure during the capture experiment.

No runtime code was reviewed or benchmarked in preparing this plan. The source references below confirm available building blocks, not the complete Recon experience.

---

## 11. Naming and release identity

**Recon** refers to reconnaissance: move across a screen, document what is seen, mark points of interest, and return with a report.

Recon is a standalone product. Copy Ninja and its Drop Ninja module remain a separate application. A shared visual language can connect the tools without merging their names, processes, libraries, or release cycles.

For open-source release, make the README's opening description explicit: a Windows image viewer, screen capture and annotation tool. Name availability and discoverability can be checked before publication; they do not block the personal-use prototype.

---

## 12. Reference material

- [Snagit Hotkeys Guide](https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/) — documents global capture and Copy All shortcuts; use accurate comparisons.
- [Tauri Webview Versions](https://tauri.app/reference/webview-versions/) — Windows WebView2 foundation; not a latency guarantee.
- [Tauri Clipboard Manager API](https://v2.tauri.app/reference/javascript/clipboard-manager/) — image clipboard operations; destination compatibility still needs testing.
- [Windows CreateForMonitor](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createformonitor) — one available native desktop capture building block, not a mandated implementation.
- [High DPI Desktop Application Development on Windows](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows) — DPI contexts, coordinate handling, and mixed-display testing.
- [Native WIC codecs](https://learn.microsoft.com/en-us/windows/win32/wic/native-wic-codecs) — which formats the Windows imaging stack decodes without an extra component, and where page access comes from.
- [Image file type and format guide](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types) — which formats a Chromium-based view decodes on its own.
