# Recon, the plan

One document. What Recon is, how it behaves, what was decided, and how it gets built.

Read this file and `project-os/Backlog.md` at task pickup. There is no second plan.

Status: consolidated on 2026-09-10. Approved for the Stage 0 experiment.
The stack in part 5 is a **recommended candidate, not a settled decision**: Stage 0 has
permission to reject it, and part 8 says on what evidence. The decode route and the format
decisions inside that candidate are settled (part 6a).
Not built: nothing here exists yet, and no timing or memory figure in this document came
from running anything.

Owner: Rotem. Personal tool, intended for an open-source release.

**How to read it.** Parts 1 to 4 are the product: what it is, how it behaves, what must
never break. Parts 5 to 10 are the build: the architecture, the decisions, and the stages
with the evidence that closes each one. Part 11 says which claims here are measured, which
are documented, and which are still assumptions. Part 12 is the record of the review that
shaped it, in one table, and part 13 lists the sources behind every documented claim.

This file replaces the two documents that came before it, the product plan and the build
plan. Their whole content is here. Every revision, and what changed in it, is recorded in
`project-os/History.md`.

---

# Part 1: what Recon is

Recon is a general-purpose tool for viewing an image, taking a screenshot, adding clear
annotations, and getting the result to its destination quickly.

**One line:** open or capture an image, annotate it, and get it where it needs to go,
without losing the thread of the work you were doing.

Three core capabilities, and none of them is an add-on to the others:

- **Viewing** an image that already exists on the computer.
- **Capturing** a region of the screen.
- **Annotating** either of them.

The first version replaces Rotem's everyday region-capture and annotation workflow in
Snagit, and his everyday image viewer. Full Snagit feature parity is not a v1
requirement, and neither is a full image-management application.

The central interaction is an integrated callout: an anchor point, a connecting arrow and
an editable text bubble form one object.

Two workloads matter, and they are different:

- **Capture.** A sequence of 20 to 30 captures, with occasional returns to earlier images
  for corrections. Client feedback is the first concrete one to validate.
- **Viewing.** Opening a file from Explorer and looking at it, or walking a folder of
  images, many times a day, with no annotation at all.

Recon must also work well for a screenshot with no annotations, and for a single
annotated image copied into another application. Rogers is one output workflow, not the
definition of the product.

## Naming, and staying separate

**Recon** refers to reconnaissance: move across a screen, document what is seen, mark
points of interest, and return with a report.

It is a standalone product. Copy Ninja and its Drop Ninja module remain a separate
application, and a shared visual language can connect the tools without merging their
names, their processes, their libraries or their release cycles.

## Positioning, honestly

Describe the advantage through Recon's defaults and measured experience.

Snagit already supports global capture shortcuts and a Copy All command; see the
[official shortcut guide](https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/).
Do not build on claims that those capabilities are absent, or that an old capture library
necessarily makes an application slow.

The same honesty applies to viewing. Windows Photos exists and opens most of these
formats. The claim Recon can make is the one it can measure: opening quickly, showing the
image at a predictable size, and being one keystroke from annotating it.

---

# Part 2: design principles

1. **Fast access is a core feature.** Measure the time until a region can be selected and
   the time until an annotation can be typed. Opening a file gets the same treatment:
   measure activation to a visible image.
2. **No routine decisions during capture.** Preferences and destinations are chosen
   outside the capture loop. No format prompt, no save dialog in the normal path.
3. **Keyboard-first operation.** Repeated actions get discoverable shortcuts, and normal
   text-editing behavior survives while typing.
4. **The image is always ready.** Copying and exporting use the current visible edits.
   There is no Done or Apply step for annotations.
5. **Repeated use is the baseline.** Judge 30 captures, navigation and corrections, and a
   folder of images, not the first successful screenshot.
6. **Automation remains correctable.** Automatic callout placement is a starting position.
   The user can always move it.
7. **Work survives normal navigation and closure.** Hiding the editor, taking another
   capture, opening another file or exiting normally must not silently discard work.
8. **Viewing never changes the file.** Looking at an image does not modify it, move it,
   re-encode it or copy it into Recon's own storage. This is the promise that makes Recon
   safe to point at a folder of originals.

---

# Part 3: how it behaves

## 3.1 Application and lifecycle

- Recon runs in the background with a tray entry and a configurable global capture
  shortcut.
- The tray gives access to the editor, preferences and an explicit Quit.
- Closing the editor hides it. Quit exits the process after saving pending changes.
- Opening the editor without capturing restores the most recent managed document, capture
  or annotated file, with its editable annotations.
- Preferences live outside the normal capture flow. Starting with Windows is a preference,
  not a prerequisite.
- If a capture shortcut is unavailable, explain the conflict and allow another. Never
  silently take over another application's shortcut.
- Remember the last external application as the return target, including when another
  capture starts from inside Recon. If that application has closed, hide Recon without
  activating an unrelated one. A background completion never steals focus.

**Opening a file is a normal way in, not a special case.** All four reach the same window:

- Double-click in Explorer, through a file association.
- Open With, for a type Recon is not the default for.
- Drag and drop a file, or a selection of files, onto the window.
- `Ctrl+O` from inside Recon.

Both starting states work: Recon already running in the background, and Recon starting
because a file was activated. A second activation while it runs opens that file in the
existing window, brings that window forward, and does not start another instance. Opening
a file never discards unsaved annotation work; §3.8 says what happens to it.

**Registration and defaults.** Recon registers the file types in §3.7 that it can actually
display, and its preferences link straight into Windows' own default-apps settings so it
can be made the default viewer. Recon never silently takes an association it was not
given, and never re-takes one the user changed.

## 3.2 Capture flow

Global shortcut, freeze the desktop image, dimmed selection overlay, drag a region, open
the editor with that region.

- If Recon is visible, hide its windows before the snapshot. Its editor, dimming and
  selection decorations must never appear in the result.
- Selection happens on the frozen image, so content cannot change under the pointer
  mid-drag.
- A region can be selected on any connected display. A single region spanning two displays
  is out of v1: selection stays within its starting display.
- `Esc` cancels and restores the preceding context. A cancelled capture creates nothing and
  replaces nothing.
- Overlapping capture requests are ignored while a selection is already active.
- No save dialog, no format choice, no intermediate confirmation.

## 3.3 One window, two entry behaviors

The same main window serves both, and which one is active must be obvious at a glance.

**A new capture opens ready for annotation.** The callout tool is live and a click starts a
note.

**An existing file opens in viewing mode.** Ordinary clicking and dragging pan, select
nothing and create nothing, so no stray callout can land on someone's photograph because
the pointer moved. An explicit **Annotate** action switches that window into annotation,
arms the tools, and creates the managed document in §3.8. Leaving annotation returns to
viewing without discarding the work.

Everything else about the editor holds for both:

- One editor window holds the canvas and, as the history UI develops, a document strip at
  the bottom.
- The image fits the available workspace without changing its pixel dimensions.
- Zoom, pan and actual-size viewing are available, because a large image cannot be
  annotated accurately without them.
- There is no separate viewer or library window in v1. Older documents are reachable from
  the same editor.
- Taking a new capture saves the current document and adds another. It never overwrites the
  previous one.
- Copying leaves the editor open. Copy and Return copies the image, then hides the editor
  and returns focus to the previous application.
- Hide only after the clipboard operation succeeded and pending changes are saved. On
  failure, keep the work and show an actionable message.
- If the user starts another action, or changes the document, while Copy and Return is
  completing, report the copy result without hiding their current work.

### Opening a file that has been annotated before

Recon history holds every managed document, captures and annotated external files alike.
So an annotated file has two things that can be opened, and the answer is fixed:

**Opening the external file shows the external file.** The original, decoded from disk as
it is now. Never the saved edit behind its back.

**When a document already exists for that path, the window says so and offers it.** One
clear route, naming when it was edited. That is the "clear route to its saved edit", and
it is the only way the saved edit is reached from a file open.

**Annotate on a file that already has a document resumes that document.** It does not
start a second one. Recon never holds two managed documents for the same source path,
because two edits of one file means nobody can say which is the truth.

**Annotate, then View, then Annotate, inside an open document reuses that document.**
Toggling the mode is not a new piece of work.

**A document stands on its own preserved image.** That image is stored inside the document
and the external source is never read again for it. If the file on disk changes or
disappears afterwards, the document opens unchanged; it says the source has moved on, and
it changes nothing about the pixels it already holds.

Worked example, the one that was undefined before. Open `a.png` and annotate it: a
document is created, holding a's preserved image. Open `b.png`: a's document is saved
first, and b opens as the external file it is. Return to `a.png`: the external file is
shown, with the route to a's saved edit beside it. Take that route, or press Annotate, and
a's existing document resumes with its callouts where they were.

## 3.4 Everyday viewing

This is the part that has to be good enough to use all day with no annotation involved.

- **Fit to window** on open, **actual size** on demand, zoom in and out, and pan. Zoom
  re-encodes nothing and never changes what an export would contain.
- **Detail matches the zoom.** At 100% and beyond, the visible region shows the source's own
  pixels, not an enlarged fit-to-window preview. Fine text in a screenshot or a diagram has
  to be readable, since that is most of what gets viewed here.
- **Fullscreen**, one key in, the same key or `Esc` out.
- **The filename and the pixel dimensions are visible** without hunting for them.
- **Previous and next walk the folder** the opened file came from, across the supported
  types in it, with a position indicator such as 12 of 240.
- **The order is defined, not incidental.** A numeric-aware, case-insensitive filename
  order, so `img2` sorts before `img10` and the sequence is predictable against Explorer.
  Part 5 names the exact comparison to use.
- **The folder listing is read once per navigation context**, not rescanned on every
  keystroke. A file that has since disappeared is skipped with a quiet note, never an error
  dialog.

**Folder navigation and Recon history are two different lists.** Walking a folder never
moves through documents, and walking history never moves through the folder. The window
names the active context and its position.

Which one is active is not a guess. Four rules, and nothing else changes it:

- **Opening an external file activates folder navigation**, on that file's folder.
- **Taking a capture activates history navigation.** So does selecting a document from
  Recon history.
- **Entering or leaving annotation preserves the current context.** Annotate is a mode, not
  a move.
- **Creating a managed document never switches the context.** Annotating the third file in
  a folder leaves the user in that folder, at position three, with a document now attached
  to it. Nothing silently drops them into history.

## 3.5 Annotation

### The callout, the primary object

With the callout tool active:

1. Click the point the note refers to.
2. A bubble is created anchored near it, with the text cursor inside.
3. Type. The visible text is immediately part of the composed image.
4. `Esc` leaves text editing and keeps the text.

Clicking an existing bubble selects it rather than creating another. Double-click, or
Enter on a selected bubble, starts text editing. A bubble left empty is discarded when
editing ends and never reaches the output.

Each callout is one editable object holding its anchor, arrow, bubble, text and displayed
number. It supports selection, movement, text editing, deletion and undo from the first
usable version. The anchor can be moved on its own, and the arrow stays attached.

### Placement, and correcting it

- A small deterministic set of candidate positions near the anchor, avoiding the anchor
  itself and existing bubbles where possible.
- No promise to understand image content or to always find empty space. Content-aware
  placement is a later improvement, not a prerequisite.
- Manual movement is allowed at any time. Moving a bubble keeps its anchor and updates the
  arrow.
- After a manual move, that placement is preserved. The bubble does not jump elsewhere as
  the user types or adds another.
- Long text wraps. The bubble's chosen origin stays put while its size changes, and all of
  the text stays visible and in the output.
- If a bubble cannot fit without clipping or covering its anchor, the canvas is extended
  with a neutral annotation margin. The captured pixels are never shrunk or cropped to make
  room, and an export includes the margin.
- Manual placement may cover image content. Automatic placement must stay easy to correct.

Validate on a small crop, a dense interface, anchors near each edge, long notes and
several bubbles before improving the heuristic.

### Numbering and text

- Callouts are numbered by creation order, starting at 1 for each image.
- **Numbers are stable, gaps included.** A deleted number is not reused, undo restores the
  original number, and moving a bubble never renumbers it. The numbers in the editor, in
  the clipboard, in an exported file and in the Rogers text are the same numbers.
  Renumbering on output is rejected; the reason is in `project-os/Decisions.md`.
- Hebrew, English and mixed-direction text are supported in the first callout
  implementation. A bubble's direction is one of three explicit modes, automatic,
  left-to-right or right-to-left, and the resolved value is stored with it. Automatic
  resolves from the bubble's first strong character and applies to the whole bubble, not per
  paragraph. Alignment follows the resolved direction, and the anchor-side edge stays fixed
  while the opposite edge grows.
- **A note's text size is 20 image pixels by default, and it is the user's to change.** The
  size is stored per note and stepped along a fixed ladder from 10 to 80, and the size last
  used is what the next note gets. The bubble's padding, its number badge and its corner
  radius are proportions of that size, and a new note's width is proportional to it too, so
  a larger note is a larger note rather than a big font in a small box. An existing note
  keeps its width when its size changes, so it grows downwards.
- **A note scales with the image, and there is no minimum on-screen size.** At a
  fit-to-window view of a wide capture that means small text: 20 image pixels reads as five
  on screen at 25%. That is Rotem's answer to F46, and the size control is the answer rather
  than an on-screen floor. A floor would put the editor and the export at different sizes,
  because the export has one scale, and it would make a note cover more of the picture at
  one zoom than at another.
- The editor and the rendered output agree on wrapping, alignment, font rendering and arrow
  positions. Part 7 step S0.6 is where that is proved rather than asserted.

### Annotating something that is not a single still frame

Annotation always operates on one still raster image. Which image depends on what was open,
and the answer is fixed rather than clever:

- **An animation.** Annotation starts from the frame on screen and produces a separate
  still image. The animated original is untouched, and Recon never writes an animation back
  out.
  **The displayed frame is a real requirement, not a figure of speech.** Drawing an
  animated image element into a canvas yields that element's default frame, which is
  usually the first one, so the obvious implementation quietly annotates the wrong picture.
  The image source must expose the frame by index, and the chosen frame is stored in the
  document. S0.5 proves it end to end.
- **A multipage image.** Annotation operates on the displayed page, and the page number
  travels with the managed document so a later reader knows which one it was.
- **A vector file.** Annotation produces a raster composition, and the original vector file
  is preserved exactly. Recon never writes vector annotations into it.
  **Its pixel size is fixed once, when Annotate is pressed**, at the size then displayed
  measured in physical device pixels, so what the user was looking at is what they get.
  Those dimensions are shown at that moment and stored in the document. Later zoom never
  changes them, and neither does resizing the window or reopening the document.
  This is the one place where an export resolution comes from the view rather than from the
  source, and it is an exception on purpose: a vector has no pixel size of its own until
  someone picks one. Every other format takes its resolution from the source image.

In all three the transition is explicit and stated on screen, because the user is moving
from the file to a picture of the file.

### The secondary tools

Arrow, rectangle, text, blur and highlight, prioritized after the core loop has been used
for real work (part 10, Stage 3). Selection, movement, deletion and undo are core editing
behavior and do not wait for that stage.

## 3.6 Keyboard and output contract

Initial defaults, to be verified with Hebrew and English input layouts. Capture uses a
separately configurable global shortcut; everything below is local to Recon.

The **Stage** column says when a row becomes real. A row promising a key that does nothing
is a contract broken on day one.

| Action or context | Default behavior | Stage |
|---|---|---|
| `Ctrl+C` while editing text | Normal text copy. It does not copy an image by surprise | 1 |
| `Ctrl+C` outside text editing | Copy the full composed image, even with an annotation selected | 1 |
| `Ctrl+Shift+C` anywhere in the editor | Copy the full composed image, current text edits included | 1 |
| `Ctrl+Enter` anywhere in the editor | Copy the full composed image, then return to the previous application after success | 1 |
| `Enter` while editing a note | Insert a newline | 1 |
| `Ctrl +` (or `Ctrl =`) in the editor | One text size up, on the note being edited or the selected one, and it becomes the default for the next note | 0 |
| `Ctrl -` in the editor | One text size down, the same way | 0 |
| `Esc` during capture | Cancel the capture | 1 |
| `Esc` during text editing | Leave text editing, keep the text | 1 |
| `Esc` with an annotation selected | Clear the selection | 1 |
| `Esc` in fullscreen | Leave fullscreen | 1 |
| `Esc` in the otherwise idle editor | Save pending changes and hide the editor | 1 |
| Delete outside text editing | Delete the selected annotation | 1 |
| Undo and redo | While a note is being edited, undo works on the typing and never reaches object operations. Once editing ends, that text change takes its place in document history as one grouped step | 1 |
| `Ctrl+O` | Open an image file | 1 |
| Previous and next image, outside text editing | Walk the active navigation context, folder or Recon history, in the §3.4 order | 1 |
| Fit to window · actual size · zoom in · zoom out | Viewing controls, no effect on export resolution | 1 |
| Fullscreen | Enter fullscreen | 1 |
| `Ctrl+S` | Save As a PNG file, to a new file. Internal saving stays automatic | 1 |
| `Ctrl+Shift+Enter` | Send to Rogers, once that exists and is configured | 4 |

Ordinary typing and editing keys are never intercepted to trigger a tool while a text field
has focus. **That covers the viewing and navigation keys too:** they are inert while a note
is being edited, so typing a letter into a note never navigates away from the image it
belongs to. Copying a single annotation object is not required for v1.

Undo has one acceptance sequence, and it is the test that settles the design: edit an
existing bubble, leave text editing, move the bubble, undo restores its position, undo
again restores its previous text.

### Image output

- Clipboard and file output carry the original captured or source pixels, plus the visible
  annotations, plus any annotation margin.
- **The output covers the whole composition.** Its dimensions are the source dimensions plus
  the stored margins, and an annotation that sits entirely inside the margin, outside the
  source rectangle, is in the output like any other. Anything that sizes the output to the
  source alone would cut those annotations off.
- Copy and export snapshot the composition when invoked. Their completion never discards or
  overwrites an edit made afterwards.
- Selection outlines, editing handles, the caret and tool UI are never in the output.
- Display zoom does not change export resolution. PNG is the initial file-export default.
- A brief, non-blocking success indication appears only after the clipboard or export
  operation actually succeeded.
- `Ctrl+S` opens Save As with the last export folder and a unique suggested filename. A
  file dialog is right for an explicit export and wrong for taking a capture.
- **Save As writes a new file, always, with no exception.** The suggested name is derived
  from the source and marked as annotated. An external original is never the default
  target, and Recon overwrites nothing: if the chosen name already exists, it offers an
  available one instead of replacing what is there. There is no confirm-and-overwrite path
  in v1, because a product that forbids writing an original cannot also offer to.
- File output is a rendered snapshot. It does not replace the editable internal document,
  and later edits never silently rewrite a previous export.
- Saving into a folder that Drop Ninja already watches lets that existing workflow handle
  the file. Recon needs no Drop Ninja integration of its own.

## 3.7 Format support, stated per format

"Common images" is not a specification. Each row says what support means. The decoding
approach and its dependencies are verified in S0.5 before any of this is advertised, and a
format that cannot make the first daily-use release is named as a gap with its impact
rather than quietly dropped.

| Format | What support means |
|---|---|
| **PNG** | Decode, transparency preserved, embedded color profile honored. Annotate directly. |
| **JPG / JPEG** | Decode, EXIF orientation applied once at decode, so the frame handed to the viewer and to annotation is already upright, embedded color profile honored. Annotate directly. |
| **BMP** | Decode. Annotate directly. |
| **WebP** | Decode, transparency preserved. An animated WebP follows the animation rule in §3.5, frame access included. |
| **GIF** | Decode and play the animation, transparency included. Frame access by index, so annotation takes a still of the frame actually on screen. |
| **TIFF** | Decode, pages exposed as page navigation with the count visible. Annotation operates on the displayed page. |
| **HEIC / HEIF** | Decode, orientation applied. Annotate directly. Depends on codecs installed on the machine, so a missing codec is reported as a missing codec with the way to install it, never as a corrupt file. |
| **AVIF** | Decode, transparency preserved. An animated AVIF follows the animation rule, frame access included. |
| **SVG** | Render for viewing at any size. Annotation rasterizes once at the displayed size in physical device pixels, shows and stores those dimensions, and preserves the original vector file. |

An unsupported or unreadable file says so plainly, names the format, and leaves the file
alone.

### The decoded-image contract

Every provider hands back the same shape, so nothing downstream has to ask where an image
came from:

- **Orientation is applied exactly once, at decode.** The frame that comes out is already
  upright, and no later step rotates anything. Nothing in the document records a pending
  rotation, because there is never one to apply.
- **The dimensions are the ones after orientation.** A photograph tagged as rotated a
  quarter turn reports the swapped width and height, and that is the document's image
  space.
- **The working color space is sRGB.** An embedded profile is converted at decode, once,
  and everything after that point is sRGB: the viewer, the composer, the clipboard and the
  exported file.
- **The frame is addressable.** For an animation the provider takes a frame index; for a
  multipage file, a page index. A provider that can only hand back "the image" cannot serve
  this product.
- **What the document stores is that decoded frame**, not the original encoded bytes. This
  is what makes double-rotation impossible after a restart, and it is why a document
  survives its source file changing.

**A capture is the one case this contract cannot fully cover.** Its pixels come off the
display, not out of a file, so they carry the display's color rather than a profile. v1
treats them as sRGB, which is right on an sRGB display and an approximation on a wide-gamut
one. S0.8 records what this machine's displays actually are, and that is the trigger for
revisiting it. A designer judging UI color on a wide-gamut display is the case that would
make this matter.

S0.5 tests this through the whole cycle rather than at decode alone, because applying
orientation twice and shifting color on reopen are exactly the bugs that pass a
decode-time check.

## 3.8 Persistence, history and retention

### An external file is not Recon's file

This governs everything below it, and it is the invariant in part 4.

- **Viewing does not modify.** No re-encode, no metadata rewrite, no move, no rename, no
  thumbnail written beside it.
- **Viewing does not import.** Browsing a folder of 100 images creates zero managed
  documents. Recon holds a path, a decoded frame in memory, and nothing else.
- **Annotation is what creates a managed document**, and only for the file being annotated.
  That document holds a preserved copy of the source image as decoded, plus the annotation
  data, exactly like a capture's document.
- **Navigating away from annotated work saves it first**, in the managed document, with no
  Apply step. The external original still does not change.
- **Export writes a new file.** Deleting a managed document never touches the external
  original, and never recalls an export or a clipboard copy.

Three things, and they stay distinct: an editable internal document, a flattened exported
image, and an external file Recon is only allowed to read.

### Editable internal documents

Persist the original captured or source image and the editable annotation data separately
from any flattened output, with enough versioned metadata to reopen the document correctly
after a restart. For an annotated external file that includes its source path and the page
or frame it came from.

- New captures save automatically, and later edits save with no Apply action.
- Pending changes are saved before a normal hide, before document navigation, before
  opening another file, and before Quit.
- After a restart, the latest document reopens and every successfully saved document is
  recoverable.
- Crash recovery reaches the last successful save. Do not claim zero loss for keystrokes
  that were never persisted. Keep the autosave interval short and verify recovery.
- A disk or persistence failure is visible. Keep the recoverable work in memory and offer
  image copying as an immediate escape, plus file export once it exists. Never label it
  saved, and never discard it silently.
- Exports and clipboard copies are independent of the document. Deleting a document recalls
  nothing.

The file and database arrangement is part 5. Never couple the document format to a
rendering library's private, unversioned serialization.

### The document strip and older history

- The strip is recent document history, captures and annotated files together, and not a
  new session-management product.
- The first usable release gives previous and next through documents, and restores recent
  work after a restart.
- Stage 2 adds the thumbnail strip and a way to reach older documents inside the same
  editor, with simple date grouping if it helps.
- Changing a Rogers target never clears history.
- Full-resolution image data is loaded when needed. Thumbnails and inactive documents must
  not make memory grow with the size of the library.
- The same discipline applies to a folder walk: one decoded image at a time, plus whatever
  small look-ahead measures well, and never the whole folder.

### Retention

For v1, captures are kept until the user deletes them. No automatic expiry, no silent
deletion. External files are not retained at all, because they were never taken.

Thirty 4K captures a day is roughly 60 to 250 MB a day depending on content, so "keep
everything" is a real disk decision and the rate is stated here so it is made knowingly.

Stage 2 adds intentional deletion, visible storage usage, and a clear line between hiding a
capture from the strip and deleting its document, if both actions exist. Prefer one
unambiguous delete over inventing two concepts. Revisit age or size limits with real usage.
Disk retention and in-memory limits are separate problems.

## 3.9 Send to Rogers

### The target

- An existing project and tab are chosen before a capture sequence.
- The active destination is shown persistently in the editor, for example `Dig › ID 1`.
- The target is remembered, and its validity is re-checked before a send. A missing target
  asks for a destination and never silently picks a fallback.
- Creating a Rogers tab from Recon waits until choosing an existing one proves
  insufficient.

### Payload and result

- One capture creates one open task.
- The payload is the composed image plus the non-empty callout texts in stable numeric
  order, using the same labels the image shows.
- A capture with no callouts is still sendable, with a simple capture-time title. The exact
  field mapping is verified against Rogers' real item model, not assumed.
- The target, image and text are snapshotted at the moment of Send. Later edits and target
  changes never mutate an in-flight submission.
- On confirmed success: mark that snapshot sent, keep the local document, store the item
  reference. Hide and return only if the user is still waiting on that submission, on the
  same capture, with no edit and no other action started, and Recon still in front. Save
  pending document and status changes first.
- If the user moved to another capture, kept editing, or switched applications, update the
  submission's status without closing their current work and without taking focus. Later
  edits stay marked as changed since the sent snapshot.

### Failure, retry and sending again

- A submission identifier and its snapshot are persisted before delivery is attempted, and
  a retry reuses that identifier, so a lost response cannot produce a duplicate task.
- Send is disabled for a submission already in flight.
- A timeout is neither success nor a guarantee that nothing arrived. The submission stays
  available for reconciliation or retry under the same identifier.
- On a failure or an uncertain result, the local capture, its text, its target and its
  status stay accessible. v1 uses explicit retry; a background delivery queue is out.
- After success the created item is referenced on screen. An ordinary Send on an unchanged
  sent capture never creates a second task.
- Editing a sent capture marks it changed since that submission. An explicit **Send as new
  task** creates a new submission and a new task. Updating an existing Rogers task is out
  of v1.

### The integration boundary

Rogers is inspected before any transport or endpoint is chosen. Its API, authentication,
attachment behavior and deployment model are unverified as of this document.

Use or add one narrow, validated inbound operation that creates the item and its attachment
reliably and tolerates a safe retry. Keep it at the request and response boundary. Do not
build a plugin platform or a general integration framework, never write into undocumented
Rogers database structures, and never report success for an incomplete item or image.

## 3.10 Scope boundaries

The first daily-use release is Stage 1. Later stages extend it without changing the core
loop.

Out of scope for v1:

- macOS.
- Video or GIF capture, scrolling capture, OCR and text extraction.
- One selection spanning several displays.
- Cloud sync, hosted libraries, public sharing links.
- Templates, preset collections, stamp libraries.
- Export targets beyond the clipboard, files and Rogers.
- A separate library application or window.
- Content-aware placement as a prerequisite for callouts.
- Automatic retention cleanup.
- Creating Rogers projects or tabs, updating previously sent tasks, background delivery
  queues.
- A generalized integrations platform.

**Viewing is in scope; managing files is not.** Recon opens, shows and annotates an image.
It does not rename, move, delete, tag, rate or organize external files, does not run batch
operations over a folder, does not build a browsable thumbnail grid of the file system, and
does not edit an image beyond annotating it: no crop, no resize, no color adjustment in v1.
Writing back into an external file, in any format, stays out.

Full-window capture and further editor tools can be judged from observed use. They do not
block the initial loop.

**For the open-source release**, the README's opening description is explicit: a Windows
image viewer, screen capture and annotation tool. Name availability and discoverability
are checked before publication, and they do not block the personal-use prototype.

---

# Part 4: what must never break

`CLAUDE.md` rule 11 is the one home for this list. It currently holds one invariant, stated
by Rotem when viewing became a core capability:

> **Viewing an external file never modifies it and never imports it.** No re-encode, no
> metadata rewrite, no move or rename, no thumbnail written beside it, no copy into Recon's
> own storage, and no export that defaults to the source path.

Why it is an invariant and not a preference: Recon gets pointed at folders of originals,
client material included, and every way of breaking this is invisible until the original is
already gone.

Where this document carries it: §3.8, the Save As rules in §3.6, step S0.5's byte-for-byte
check, step S1.5's mode boundary, and step S1.11's new-file-only Save As.

---

# Part 5: the architecture

## The stack: the recommended candidate, and what still has to be shown

**Recommended: a Rust host owning the pixels, and a Chromium web view laying out the text.**
A Rust process with windows-rs for capture, decoding, the store, the clipboard and the
native overlays; a WebView2 editor for the scene, the composer and the annotation text;
Tauri v2 packaging it.

**This is a candidate pending validation, not a settled decision.** Stage 0 may reject it,
and part 8 says on what evidence. What follows is the argument for it and, just as
important, the part of that argument that is not established.

Familiarity with a stack from another project is not a reason, and rule 21 of `CLAUDE.md`
says another project is not evidence about this one.

### What actually favors it

**Decoding is the second-largest surface here, and it belongs to the host.** Nine formats,
orientation applied once, one color space, frames and pages addressable by index, and a
parse path pointed at client files, so a permanent security and format-drift surface. Rust
covers it in one toolchain with most of that parse path memory-safe: `image` for PNG, JPEG,
BMP, GIF with frame access and WebP; `resvg` for SVG, which renders `<text>`; libavif, with
a dav1d backend, for AVIF; windows-rs into WIC for multipage TIFF and for HEIC through
whatever codec the machine has.

**The text editing model comes from the engine.** Caret placement, selection rectangles,
caret movement across a direction boundary, word boundaries and the platform's editing keys
are Chromium's. That is a real saving, and §3.5's contract is what has to be met, not the
saving.

**The project's existing verification tooling runs on it.** `project-os/Workflow.md` step 9
drives the running app with a browser tool and reads its console and document tree, so the
editor, which is the surface that changes for years, stays inside that apparatus.

### What is NOT established, and must not be argued as if it were

**Other candidates do have editing models.** Qt's `QGraphicsTextItem` is editable inside a
graphics scene, with a text document and a cursor behind it. That does not show it meets
§3.5's export contract; it does refute any claim that a bidirectional editing model has to
be built from scratch elsewhere. An earlier revision of this plan made that claim, and it
was wrong.

**Native UI is not condemned to manual verification.** WPF exposes automation peers for its
standard controls and Qt exposes accessibility interfaces for its widgets. Custom-drawn
annotations need their own semantic exposure in any of these candidates, including this one:
a canvas-drawn callout is no more accessible in a web view than in Direct2D. The work to
price is that specific exposure, not the whole native UI.

**A workflow written around browser inspection is a tooling constraint, not a product
requirement.** It is a genuine cost to price and adapt. It is not evidence that a browser
editor is the better product.

**The SVG renderer is not tied to the host language.** `resvg` ships a C interface, so any
of these candidates can use it. What remains true is narrower: Direct2D's own SVG document
supports a documented subset with no `<text>`, and Qt's SVG module lacks `clipPath`, though
it has gained extensions since 6.7. Neither fact eliminates a candidate that pairs a
different renderer with a different host.

**One decode route versus two is an engineering judgment, not a measured fact.** The reason
for one route is correctness (see the boundary rule below), and that reason stands on its
own. Its relative cost has not been measured.

### The comparison, on concrete implementations

Documentation-first, as it should be: this table is what each candidate would actually use.
Prototype only the consequential uncertainties in the strongest alternatives, and never
build several complete applications to decide this.

| Candidate | Text editing | SVG | Decoders | Automation route | Deployment | Standing update duty |
|---|---|---|---|---|---|---|
| **Rust host + WebView2 editor** | editable DOM, engine caret and selection | `resvg` crate | `image`, `resvg`, libavif with dav1d, WIC | document tree, plus this project's existing browser tooling; custom exposure still needed for canvas-drawn parts | WebView2 runtime, present on current Windows | the web view's own cadence, four decode crates, Tauri and wry |
| **C# on .NET + WinUI 3** | a rich text control, or one DirectWrite layout per bubble | `resvg` through its C interface, or Direct2D's subset which drops `<text>` | WIC covers all nine, or the same crates through interop | automation peers for standard controls, custom peers for the scene | .NET runtime and WinUI packaging | .NET yearly, WinUI, plus any interop it writes |
| **C++ + Direct2D/DirectWrite** | DirectWrite layout per bubble, caret and selection hand-written | `resvg` C interface, or Direct2D's subset | WIC plus libavif | UI Automation, hand-written for custom drawing | nothing beyond the OS | libavif, `resvg`, and everything it writes itself |
| **Qt 6** | `QGraphicsTextItem` with a text document, editing included | QtSvg with the 6.7 extensions, or `resvg` | Qt image plugins plus libavif; frame seek in GIF and animated WebP is unproven and is F24's failure returning | `QAccessible` for widgets, custom for the scene | Qt runtime bundled | Qt feature releases twice a year, plus licensing |
| **Electron** | editable DOM, same as the first row | via a native addon, or the engine | engine plus addons | document tree | a pinned Chromium bundled | Chromium majors, six or seven a year |

### Tauri, and what replacing it would cost

Tauri is not merely an installer. It supplies the tray, the global shortcut, single-instance
argument forwarding from Explorer, the web view host, the file-association bundle and the
updater. Replacing it means re-implementing that list, so it is a smaller decision than the
stack and a larger one than packaging.

The alternatives, and the evidence that would switch to each, are in part 8.

## The split, and why

The obvious instinct is to judge that one stack for the whole application. Two surfaces here
fail for entirely different reasons, and only one of them is in doubt, so they are split
before the experiment starts: native code owns the screen, the clipboard and the disk; the
web view owns the editor.

| Layer | Runs as | Owns |
|---|---|---|
| Host | native process | tray, preferences, global hotkey, window lifecycle, the return-focus target |
| Capture | native | freezing the desktop, the captured region, physical desktop coordinates, all behind one interface |
| Image source | native, one route for every format | opening a path, decoding to the contract in §3.7, orientation applied once, sRGB, frame and page access by index, behind one interface. The web view decodes nothing. |
| Overlay | native, one borderless window per display | the dim, the selection rectangle, cancel, handing the region to the host |
| Store | native | one folder per document, the preserved source, the versioned annotation data, atomic writes |
| Clipboard | native | several image formats in one operation, success reported before any hide |
| Editor shell | web view | toolbar, document strip, destination indicator, keyboard routing, the viewing controls |
| Scene and composer | web view | the one composer: display at the current zoom, export at original scale |
| Text editing | web view, an input layer above the canvas | caret, keyboard input, bidirectional typing, sharing the composer's wrapping |

## The boundaries that hold it together

Each one exists to stop a specific failure.

**The web view never touches the screen or the clipboard.** So a web view limitation can
never break capture or output.

**The host never renders an annotation.** So there is exactly one composer, and export is
that composer at original scale. This is what keeps the editor and the file identical.

**The document never stores a screen coordinate.** So it cannot be invalidated by moving a
monitor.

**Capture is reached through one small interface**, freeze and region in, image out, so the
implementation behind it can be replaced, or joined by a second one, without the editor
noticing.

**An opened file arrives through the same shape**, one image source interface, path in,
decoded frame and metadata out, with one native implementation behind it for every format.
Past that boundary the editor cannot tell a capture from a file, which is what lets one
window serve both.

Its contract is the decoded-image contract in §3.7, frame access included. One route, in the
host, for all nine formats.

Two rules, and they are separate things. The first is a correctness boundary and is pinned.
The second is a transport policy and is a measurement.

**Pinned: the preserved source is authoritative, it lives in the host, and it never
round-trips a canvas.**

- The host holds the decoded source, and that copy is what an export is made from.
- Export emits **only the annotation layer**, over a transparent background, covering the
  **full composition**: the source dimensions plus any stored annotation margin, and it
  crosses **encoded**, not raw. S0.3 measured that same layer at 31.6 MB and 359 ms raw
  against 162 KB and 3.5 ms as a PNG the web view encoded in 78 ms.
- **The host composites** that layer over the untouched source at the stored margin offset,
  then writes the file or the clipboard.

The reason is exact equality. A canvas is premultiplied, so a straight-alpha pixel at low
alpha does not survive the round trip; a preserved image born in the renderer cannot be byte
identical for any image with transparency. An opaque capture round-trips perfectly, which is
exactly why a check written against a capture cannot catch it.

**A policy, decided by measurement: what representation the editor gets to display.** The
boundary above does not say the editor may never receive full resolution. Showing a copy
cannot corrupt a source that stays authoritative in the host. So S0.3 measures the crossing
and the representation is chosen from that, not asserted here.

What the policy must satisfy either way, because Recon is replacing an everyday viewer:

- **The visible region always has enough detail for the current zoom.** A preview generated
  for fit-to-window cannot be enlarged to serve 100% or 200%: fine text turns to mush. Zoom,
  pan and window resize each fetch what the visible region needs, as an updated preview or as
  tiles.
- **The cache is bounded**, and a stale response that arrives after the view moved is
  discarded rather than painted.
- **Annotation coordinates stay in image space**, so whatever the display representation is,
  it never leaks into the document.

S0.4 tests it on the case that exposes it: a large image with fine text, at fit, at 100%, at
an enlarged view, and again after panning.

## The annotation text layer, and how it exports

The text is not drawn into a canvas. It is DOM, and the export is that same DOM.

- Each bubble is an absolutely positioned element, laid out in **image-space CSS pixels**.
- **Direction is three explicit modes on the bubble: automatic, left-to-right, and
  right-to-left**, and the resolved value is stored in the document.
  An earlier revision said `unicode-bidi: plaintext` plus `direction` would do this. It would
  not: under the CSS specification `plaintext` takes each bidi paragraph's base direction from
  the Unicode heuristics and ignores the element's own `direction`, so the manual override did
  not exist and the contract was per paragraph rather than per bubble.
  So: automatic resolves the bubble's base direction from its first strong character, once,
  and applies it to the whole bubble; an override sets an explicit direction configuration
  instead. Alignment follows the resolved direction, and the export carries the same resolved
  value rather than re-detecting anything.
- **Display zoom is a CSS transform on the container**, so layout is computed before the
  transform exists and zoom cannot re-wrap anything. That is R6's worst failure removed by
  construction rather than by discipline.
- Editing happens in a real editable element, so the caret, the selection rectangles, caret
  movement across a Hebrew and English boundary, word boundaries and the platform's editing
  keys are the engine's.
- **Export serializes that subtree** into an SVG `foreignObject` as a data URI, at the
  original pixel dimensions, and draws it into a canvas sized to the original image. Only
  that annotation layer crosses back to the host.

Three requirements come with the export path, and each one is a silent failure if it is
skipped:

- **Fonts are inlined**, as base64 `@font-face` inside the serialization. An SVG loaded as an
  image loads no external resource, so an un-inlined font substitutes silently and the
  export stops matching the editor.
- **Every bubble style is inlined**, never inherited from a stylesheet the serialization does
  not carry.
- **Caret and selection decorations must not affect layout.** Outline and background, never
  border or padding.

The bubble style vocabulary stays deliberately small, because `foreignObject` rasterization
has a long tail. And this path deliberately does not depend on the newer canvas text
metrics APIs, which are not settled across engines.

## Coordinate spaces, and the transforms between them

Four spaces, named, with the conversion written down at each boundary. One conversion at
the edges is not a guarantee against drift; naming the spaces is what makes a wrong
conversion findable.

| Space | Units | Used by |
|---|---|---|
| Desktop | physical pixels across all displays, negative positions included | the freeze, the overlay, the selected rectangle |
| Image | pixels local to the source image, captured or opened, origin at its top-left, original scale, orientation already applied | the stored document: anchors, bubbles, arrows, text boxes |
| Canvas | image space plus the annotation margin, as an offset record | the composer, the clipboard, the exported file |
| View | canvas space through zoom, pan and display scale | what the editor draws and what the pointer hits |

The display proxy lives in view space: it is the image resampled to what the screen can
show, and it is never the thing an export is made from. Annotation coordinates stay in image
space, so the proxy's resolution never leaks into the document.

Desktop to image happens once, at capture, and is then discarded. Image to canvas is the
margin offset, stored as four edge values so a growing margin never rewrites a single
annotation coordinate. Canvas to view is the display transform and is never persisted.
Nothing in a document refers to a display, a scale factor or a monitor arrangement, so
yesterday's document opens the same way after the monitors move.

## The document on disk, as a starting position

One folder per document, holding the source image as a PNG that is never rewritten, plus a
JSON document with a schema version. An index database, if there is one, holds pointers and
never the images; SQLite is a candidate for that index in a later stage, not a requirement
for Stage 0. Every write goes to a temporary file and is renamed into place, per
`project-os/QA.md` §5. Autosave is debounced while typing and forced on blur, navigation,
opening another file, hide and quit. The schema itself is settled at step S1.8.

## The filename order for folder navigation

Use the Win32 logical string comparison, `StrCmpLogicalW`, or an equivalent natural sort
where that call is not available. It is numeric-aware and case-insensitive, so `img2`
precedes `img10`.

Never inherit whatever order the file system happens to return. Whether the result matches
Explorer closely enough to feel predictable is checked in S1.4, not assumed here.

---

# Part 6: the decisions

## 6a. Settled

| Decision | Answer |
|---|---|
| Text direction | Detected from the first strong character, with a manual override (§3.5). |
| Undo | Specified as observable behavior, not a stack count, with the acceptance sequence in §3.6. |
| Numbering | Stable numbers and gaps in every output. Renumbering rejected. Recorded in `project-os/Decisions.md`. |
| Stage 1 navigation | A position indicator, plus shortcuts to the first and last (S1.4, S1.9). |
| Windows support | The API's technical minimum, the versions Recon supports and tests, and runtime packaging are three separate answers, not one. Recorded in `project-os/Decisions.md`. |
| Elevated windows | Test first. No elevated mode on an unverified assumption (S0.8). |
| Image viewing | A core capability, in one window with two entry behaviors. Recorded in `project-os/Decisions.md`. |
| Initial capture path | One copy of the whole virtual screen: one synchronous call already spans every display and every negative coordinate, so the foundation risk is retired at the lowest cost. It sits behind the capture interface, which must leave a video source possible later while adding no video infrastructure now. A second path is earned only under part 8. |
| Paste destinations | Claude and ChatGPT. Both read the clipboard the way a web application does. PNG is the implementation choice, not a proven requirement: a Chromium-based reader can convert native bitmap data into PNG for the page, so nothing here claims a bitmap-only clipboard would fail. What decides acceptance is the paste actually working in those two surfaces, recorded with the surface and its version. |
| Reference environment | Rotem's main machine at its current display scale. The environment, the font details and the web view runtime version are recorded with the reference images and in the S0.8 report. |
| The stack | A Rust host owning the pixels, a WebView2 editor laying out the text, packaged with Tauri v2. Argued in part 5 from the decode surface, from the bidirectional editing model, and from this project's own browser-driven QA gate. Recorded in `project-os/Decisions.md`. |
| Decoding | One native route for all nine formats. The web view decodes nothing, and the decoded original never crosses into it at full resolution. This flips the earlier two-provider recommendation, because a preserved image born in the renderer cannot survive a premultiplied canvas byte for byte. Recorded in `project-os/Decisions.md`. |
| TIFF and HEIC | Both through WIC in the host: TIFF with its pages, HEIC through whatever codec the machine has, with the named missing-codec message rather than a corrupt-file one. |
| The annotation text layer | DOM in image-space pixels with `unicode-bidi: plaintext`, zoom as a CSS transform so layout precedes it, and export through a serialized `foreignObject` with fonts and styles inlined. Recorded in `project-os/Decisions.md`. |
| Note text size | 20 image pixels by default, stored per note, stepped with `Ctrl +` and `Ctrl -`, and the size last used becomes the next note's default. A note scales with the image and there is no minimum on-screen size. Rotem's call on F46, recorded in `project-os/Decisions.md`. |

## 6b. What used to be open here

Three format decisions lived in this section: how decoding splits between providers, TIFF
with its pages, and HEIC. All three are answered in 6a, and they closed together, because
the stack decision answered them at once: one native decode route, with WIC for TIFF and
HEIC.

The alternatives are not gone, they moved. Deferring TIFF is the scope decision in 6c.
Bundling an AVIF or HEIC decoder is a cost line in part 11. Moving the composer in-process
is the trigger in part 8.

Nothing in this plan now waits on a decision. What waits is measurement.

## 6c. The minimum format set for the daily-use release

"Ship the shorter list" is bounded. It is not a licence to drop a format because it turned
out to be inconvenient.

**Required in Stage 1:** PNG, JPG, GIF, WebP, BMP, AVIF and SVG.

The reason this set is affordable has changed, and it is worth re-reading before it is
treated as settled. It used to be "the web view decodes all seven for free". With one native
decode route that is void: the seven are affordable because `image` covers five of them and
`resvg` and libavif cover the other two, all in the host's own toolchain. The membership does
not change. The price does: four decode dependencies and their security watch, forever, as
part 11 now says.

**Required to attempt, allowed to degrade:** HEIC and HEIF. On a machine without the codec,
the message that names it and says how to install it is a pass, not a gap.

**The one format a Stage 0 result may defer:** TIFF, with its pages. HEIC needs the host
provider too, so that is not what separates them: TIFF is the one host-provider format with
no acceptable degraded state. A missing HEIC codec has a defined message that counts as a
pass, while TIFF has nothing partial to ship, since it brings a page-navigation surface
with it. Deferring it is a scope decision Rotem makes explicitly and
`project-os/Decisions.md` records. S0.5 can report that TIFF is expensive; it cannot decide
to ship without it.

A required format that will not decode is not a shorter list. It is a blocked stage, and it
comes back to Rotem as one.

Details that wait for their own step: the document schema (S1.8), the placement candidate
set (S1.6), the storage-rate note (Stage 2), the Rogers submission record (Stage 4).

---

# Part 7: Stage 0, the smallest experiment that settles the foundations

Purpose: retire the risks that would change the architecture, on the smallest build that
can produce evidence. Eight steps. Anything not on this list is not Stage 0, including
packaging, a second capture path, full undo, and anything that manages files.

Each step carries a checkbox, a suggested model with a short reason, what it delivers, and
the evidence that closes it. The model is a suggestion for whoever picks the step up, not a
setting. A step is done when its evidence exists, not when its code runs.

----
**[x] S0.1 · Host shell: tray, hotkey, quit**

Model: Sonnet 5. Mechanical, documented APIs.

Delivers: a background process with a tray entry, a configurable global hotkey, and a quit
that leaves nothing running.

Evidence: the hotkey fires while the process has never shown a window; a hotkey already
taken by another application is reported as a conflict rather than silently lost; quit
leaves no process behind.

**Closed 2026-09-10.** All three hold. The third was closed by Rotem clicking Quit himself,
after three automation routes failed to hold the hidden-icons flyout open long enough; the
logs and the two gaps that run exposed are in `project-os/History.md`.
One number to carry into S0.7: the tray-only process sat at 20 MB with no window, which is
the floor before any web view exists.

----
**[x] S0.2 · Freeze, overlay, region selection**

Model: Opus 5. DPI, native APIs, irreversible shape.

Delivers: the capture interface with the chosen path as its one implementation, one overlay
window per display, drag to select, cancel, and the region handed back in desktop
coordinates and converted once into image space.

Evidence: two displays at 100% and at 150% or 200%, with the secondary placed left of the
primary so coordinates go negative; the selected rectangle and the resulting pixels agree
exactly; the overlay and the editor never appear in the output; cancel restores the previous
context and creates no capture; a second hotkey press during selection is ignored; an open
menu is either captured or the limitation is written down.

**Closed 2026-09-11, with one condition deferred rather than met.**

What held: all four corners of the 3840x2160 display at 225% crop identical to a fresh
screen copy, byte for byte, so the conversion is right at both extremes and a scale error is
ruled out along with an origin error. A synthesized drag returns exactly the rectangle
dragged. The captured pixels equal the screen with the overlay gone, which is the
overlay-never-in-the-output check. Escape captures nothing. A second hotkey during a
selection is ignored. A context menu, shadow included, is in the freeze.

What is deferred, and why it is not a blocker: the two-display case, mixed scaling and a
display arranged left of the primary so desktop coordinates go negative. Rotem has one
display, so the machine cannot produce either condition, and hardware is not being bought
for a Stage 0 gate. What is actually untested is Windows reporting a negative origin, which
is documented behaviour rather than code written here; the code's own handling of it is
covered by seven unit tests in `capture/coords.rs` built on synthetic layouts, including a
display to the left, a display above, and a rectangle that would wrap a u32 if the
arithmetic were not done in i64. The drag clamp added by the review is unreachable here for
the same reason, since a single-display cursor cannot leave its own display.

It moves to part 11 as a watch item: the first time a second display is ever attached, run
`--selftest` and read section A.

----
**[x] S0.3 · The boundary, measured before anything is built on it**

Model: Opus 5. It decides whether the pinned design was ever optional.

Delivers: the process boundary measured in isolation, with no editor code involved.

Evidence: four crossings timed, in milliseconds per megabyte, in both directions. A
full-resolution 4K frame from the host to the web view. The same frame as a display proxy.
A mostly transparent 4K annotation layer from the web view back to the host. And the small
proxy on the folder-walk path, which §3.4 says happens many times a day. Compare the plain
message route against the custom-protocol and local-loopback routes.

Part 5's design is adopted either way, because exact equality and the clipboard need it
regardless. What this step settles is whether the naive full-resolution design was ever
available, and it replaces the estimate this design was reasoned against, a maintainer's own
explicitly unscientific figure, with a number from this machine.

**Closed 2026-09-11.** Measured by the web view's own clock, five repetitions per row,
median reported, with the empty round trip measured separately at about 2 ms so protocol and
transfer can be told apart. Full numbers in part 11.

**The naive design was never available, but not for the reason the plan assumed.** A whole
31.6 MB frame crosses in 290 ms through the message channel and the custom protocol, or
140 ms through a local socket. On top of a 166 ms freeze that is most of the budget spent
before the editor has done anything. The proxy the design pins costs 40 to 74 ms for the
same view, and an encoded proxy 6 to 16 ms.

**The figure the design was reasoned against was two to four times pessimistic.** It said
roughly 200 ms per 10 MB; the measurement says 90 ms per 10 MB through the message channel
and 44 ms through a socket. The design does not change, because exact equality is what pins
it, but the plan should stop quoting a number that was wrong.

**The route matters more than the plan expected.** The message channel and the custom
protocol are indistinguishable, at about 9 ms per MB in both directions. A local socket is
roughly twice as fast for large payloads, 4.4 ms per MB in and 5.7 ms out. So if a large
crossing is ever genuinely needed, there is a route for it, and part 8's boundary trigger did
NOT fire.

**And the layer should cross encoded, which the plan did not say.** The same 4K annotation
layer is 31.6 MB raw at 359 ms, or 162 KB at 3.5 ms once the web view encodes it to PNG,
which takes 78 ms there. Encoded wins by two orders of magnitude, and part 5 now says so.

----
**[x] S0.4 · Editor window, scene, one callout**

Model: Opus 5. The composition contract is the product's spine.

Delivers: the editor window pre-created and hidden at startup, one image on the canvas, and
one callout that can be created, typed into, committed, selected, moved and deleted. The
anchor moves independently and the arrow follows.

Evidence: every one of those operations exercised by hand; an empty bubble discarded when
editing ends and absent from the output.

**Plus the hidden-window check, run in the configuration this product would ship.** This
failure looks like latency and is actually rendering suspension, which is the easiest
misdiagnosis in the design. Show the pre-created hidden window and confirm the first painted
frame is current rather than blank or stale, and confirm a file drop works on a window that
was created hidden.

Disabling the browser's occlusion calculation is a diagnostic, not part of the gate. The
vendor cautions against relying on those flags in production, because they can change or
disappear. So: run the gate without it. If the design only passes with the flag, that is an
unresolved dependency, recorded as one in the S0.8 report, and a supported route has to be
found before this design is called fit for years of use.

**Plus the display-detail check**, because §3.4 promises the viewer's detail matches the
zoom: open a large image containing fine text, and read that text at fit, at 100%, at an
enlarged view, and again after panning. An enlarged fit-to-window preview fails this.

Full undo and redo are Stage 1 (S1.7), not a Stage 0 gate.

**Closed 2026-09-11.** Twenty-six checks, all passing, run with `--editor-check` and written
to a report beside the executable. Two of them are the ones worth having:

**Zoom cannot re-wrap a line, and now that is measured rather than argued.** The same
wrapped bubble, mixed Hebrew and English, lays out at exactly 140 px at 17%, 33%, 100% and
200%. That is part 5's claim holding by construction: the text is laid out in image pixels
and one CSS transform scales it, so there is no zoom at which the layout is recomputed.

**The detail matches the zoom, so F35 is closed by pixels.** The probe image's left half is
a one-pixel checkerboard. Reading one strip of the canvas: 199 alternations out of 200 at
actual size, 0 at fit-to-window, 199 again on the way back, and 199 after a pan. An
enlarged fit-preview would have read 0 at every step.

The rest: a callout created by a click, typed into, committed, selected, moved, its anchor
moved on its own with the arrow following, and deleted; an empty bubble discarded when
editing ends, whitespace included; numbering that keeps its gaps, where deleting 2 leaves
the next one 4; three direction modes resolving as §3.5 says and reaching the element.

**The hidden window shows a current first frame, without the flag.** The page painted while
the window was still hidden, the host showed it and then looked at that area of the SCREEN:
100% of the sampled pixels were the colour the page painted, 0% black, 40 ms after a show
that took 12 to 22 ms. Asking the page what it drew would have proved nothing, because a
suspended surface has a perfectly correct document behind it. The occlusion flag the review
struck from this gate was not used, so this is the shipping configuration.

**And then it was looked at, which found two more.** Twenty-six passing assertions did not
notice that the image sat in the top-left corner of the window with two thirds of it empty,
or that a note is unreadable at fit zoom on a wide capture. Those are F45, fixed, and F46,
which is Rotem's to decide. Workflow step 9 asks for the look on anything visible, and this
is why.

**Four things this step found in total**, none of them in its own evidence list: F43 to F46
in part 12.

**Text size, added 2026-09-11.** F46 asked whether a note needs a minimum on-screen size,
and the answer is no: 20 image pixels by default, and the size is the user's, stepped with
`Ctrl +` and `Ctrl -`. Five more checks, thirty-three in total. The one that matters reports
instead of asserting, because the number is what the decision is about: a 20-pixel note is
72 px tall in the image and 24 px on screen at a 33% fit view, so its text reads as about
6.7 px. Looking at it on a 5120x1440 capture says the same in one picture: the 20-pixel note
is unreadable at 25%, and a 40-pixel one is comfortable.

----
**[ ] S0.5 · Open and decode an existing image**

Model: Opus 5. Format behavior, and a decision gate for the product surface.

One decode route, in the host, for all nine formats (part 6a). The web view decodes nothing.

Delivers: the image source interface, path in, decoded frame plus metadata out, implemented
once in the host for all nine formats; and the same canvas showing an opened file instead of
a capture.

The route under test is the chosen one, not a comparison: `image` for PNG, JPEG, BMP, GIF and
WebP, `resvg` for SVG, libavif with a dav1d backend for AVIF, and WIC for multipage TIFF and
for HEIC. Name the chain and its update duty in the report: dav1d is a codec backend inside
libavif, not an interchangeable whole-file AVIF library.

Evidence, one line per format in the §3.7 list: did it decode, how long did the open take,
and what came out. Transparency preserved where the format has it, EXIF orientation applied,
an embedded color profile honored, a very large image opened without stalling, an animation
playing, a multipage file exposing its pages.
Every format that did not work is named with what it would take, and no format is reported
as supported on the strength of another format working.
The external-file boundary is proved here too: after opening, viewing and navigating, every
source file is byte for byte what it was, and no managed document was created.

**The animation frame test, end to end.** Select a frame that is not the first one,
annotate it, save the document internally, restart Recon, then export. The same frame is
what appears at every one of those points. A first-frame substitution anywhere, including
after the restart, fails this step. This is the test that decides whether the chosen
provider really exposes frame access, and the plan assumes nothing about it.

**The orientation and color cycle test.** Take an EXIF-rotated JPEG and an image carrying an
embedded color profile through the whole cycle: viewing, Annotate, an internal save, a
restart, and an export. Orientation is applied exactly once, so the export is not rotated
twice and not left upright-in-the-viewer-only. The color does not shift between viewing and
annotation, and does not shift after reopening. A decode-time check alone does not close
this: applying orientation twice and shifting color on reopen are precisely the bugs that
pass one.

**The SVG raster size test.** Rasterize at annotate time, note the dimensions the document
recorded, then zoom, resize the window, reopen the document, and export. The exported pixel
dimensions are the ones recorded at annotate time, every time.

**SVG is compared on content, not on dimensions.** The size test above passes a render that
came out silently mangled, and Recon's core case is annotating diagrams. So: a Figma export
using masks, an Illustrator export whose fills live in a CSS `<style>` block, and a diagram
whose labels are `<text>`. Each one is compared against how it looks in a browser, and any
element the renderer dropped is named.

**Animated WebP and animated AVIF get their own frame-by-index confirmation**, separately
from GIF. Neither is assumed from the other: the WebP animation path and the AVIF sequence
path are different code, and animated AVIF is the likeliest row to need libavif directly.

This step is the gate for the format claims in §3.7, inside the limits in part 6c. It may
report that a format is expensive or unavailable; it may not decide to ship without a
required one.

----
**[ ] S0.6 · Output fidelity and the clipboard**

Model: Opus 5. Fidelity, and the one place work can be lost.

Delivers: the composer used for both display and export, and a native clipboard operation
publishing several image formats.

Three separate checks, because one comparison cannot answer all three questions:

Original pixels survive. Export a capture with no annotations, decode it, and compare its
image area against the captured source at the margin offset. Encoding is lossless, so this
one is exact equality, not a tolerance.
**And it needs a transparent case, or it cannot fail.** A capture is opaque, and at full
alpha the premultiplied and straight encodings are identical, so the check as first written
could not catch the one thing it exists to catch. So: annotate a PNG that has
semi-transparent regions, then compare every source pixel no annotation covers, for exact
equality.

Annotated output is correct. Compare six documents against reviewed reference images
covering Hebrew, English, mixed direction, a long wrapped note, an anchor against each edge,
and a document that added a margin. State the reference environment and the comparison
tolerance, since font rendering is what the tolerance is for.

The live editor and the output agree while typing. Copy with the caret still active, mid
note, in each of the three text cases, and compare what the EDITOR IS SHOWING against the
copied image: the text itself, the wrapping, the alignment and the position.
Not the composed output against the clipboard. Those two come out of the same renderer, so
they can agree with each other while both differ from the layer being typed into, which is
the failure this check exists to catch.
Exclude the editing decorations, the caret and the selection handles, from the comparison.
The product forbids an Apply step, so this is the case that matters, and committed text is
the easy half.

**Apply the three remedies before reading the result**, or the result is about the remedies
rather than about the design: fonts inlined as base64 in the serialization, every bubble
style inlined rather than inherited, and caret and selection decorations that cannot affect
layout.

**Then read it against the trigger in part 8, with the two clauses kept apart.** A different
wrap point is a trigger at any tolerance, because that is the requirement failing. A
different alignment, or a bubble box whose position or size differs by more than the stated
tolerance, is a trigger. Glyph antialiasing inside tolerance is not. Nobody widens a
tolerance past a wrap failure.

**Record whether the export route is origin-clean** in the web view version actually
shipping: whether reading the pixels back from the serialized layer throws, and whether a
same-engine fallback route exists if it ever does. This behavior is universally implemented
and unspecified, so it is a watch item rather than a one-time answer.

**The margin case.** Put a callout entirely inside the annotation margin, outside the source
rectangle, and export. It survives, at the right offset, in an output whose dimensions are
the source plus the stored margins.

Plus: a paste verified in Claude and in ChatGPT, with the surface and version recorded
beside each result.

----
**[ ] S0.7 · Instrumentation and the thirty-run measurement**

Model: Sonnet 5. Mechanical, known shape.

Delivers: six marks per run and two reported intervals, a CSV per run set, and a memory
sample that separates the live document from retained history.

Marks: hotkey received · overlay displayed · overlay accepting input · selection completed ·
editor displayed · editor accepting annotation input.

Intervals: hotkey received to overlay usable, and selection completed to editor usable.
Everything else is diagnostic, and the user's drag time is inside neither interval.

**Usable means both halves, on one timing basis:** the correct content is on screen AND
input is accepted. Visible is not interactive, and interactive is not visible either: input
handlers can be live while the window still shows blank or stale pixels, and an interval that
ends at input readiness alone would score that as ready.

Evidence: thirty runs at 4K with the process already in the background, reported as raw runs
plus the maximum, against the proposed targets of 250 ms to a usable selection and 500 ms to
a usable editor. State the percentile method explicitly: with thirty samples the second-worst
value is the nearest-rank 95th percentile, and other methods give other answers, so the label
means nothing without the method beside it. The raw list carries more than either.
Process startup is measured and reported separately.

**The memory sample is read as both a floor and a slope.** Growth with the size of the
library is a failure of R11. The floor is a separate question and it is not exempt: this
process sits in the tray all day, so report the floor as a number to be judged on its own,
alongside the delta across thirty captures and across a folder walk, and state the explicit
release discipline that produced it.

----
**[ ] S0.8 · The limits, and the go or no-go report**

Model: Opus 5. Judgment on evidence.

Delivers: what the platform actually did on this machine, and one short report.

Evidence: the hotkey tested against a real application running as administrator on the normal
desktop, with the result recorded either way and no product decision attached to a guess;
best-effort return-focus tested against that same window and against an application that has
since closed; the API's own minimum Windows version stated separately from the versions Recon
supports and tests; HDR behavior determined, with "not supported in v1, detected rather than
silently wrong" an acceptable answer; each connected display color gamut recorded, sRGB or
wide, because that record is the trigger named in §3.7 and in the sRGB decision; the web
view runtime version recorded, because every S0.6 reference image is meaningful only against
a named engine version and that engine updates on a fortnightly cadence; the reference
environment recorded.

The consent prompt's own secure desktop is out of scope by design: nothing on the user
desktop reaches it, and no product decision follows from it.

The report separates measured facts, documented behavior and assumptions, and recommends
keeping or replacing a named component rather than the whole stack.

---

# Part 8: what Stage 0 is allowed to conclude

Each failure has a next move, and a failure is diagnosed before anything is replaced. A
failure that is not resolved is reported as a limitation rather than absorbed.

**Stage 0 has permission to reject the recommended stack.** Most single failures accuse one
component, and the rows below name which. But the last row exists because a gate that cannot
return a verdict against the design is not a gate.

| If this fails | Then |
|---|---|
| The chosen freeze path is wrong on DPI, misses window content, or is the diagnosed cause of a missed latency target | Locate the cause first, then implement the other capture path and compare. These are the only conditions that earn a second path, and a demonstrated performance failure traced to the capture implementation is one of them. |
| Activation is slower than the target | Find where the time goes first: process wake, window show, the freeze, the first paint, or input readiness. Replace the component the measurement accuses, not the one that is easiest to blame. If it accuses the freeze itself, that is the row above. |
| The clipboard cannot satisfy the two destinations | Diagnose it: which format, which application, which failure. Fix it, or report an explicit limitation with the applications named. It does not pass the gate on the grounds that the clipboard is native by design. |
| The export drifts from the display and shared wrapping cannot close it | Moving the composer to native code is a candidate, not a remedy: it must be revalidated for editing and output together, including the active-caret case, before it counts. |
| A target format will not decode, or needs a component that is not on the machine | Name the format, the cause and the impact. If it is a deferrable format under part 6c, ship the shorter list with the gap stated. If it is a required one, this is a blocked stage and a scope decision for Rotem, not a shorter list. Either way it does not fail the stack. |
| The export route is unavailable, or drifts from the editor, and the three remedies do not close it | Move the composer in-process: a Direct2D and DirectWrite scene inside the same Rust host, through windows-rs, keeping every other component. One text layout object per bubble then serves the caret geometry, the screen draw and the export draw, and the requirement stops being a discipline. Not a different stack, and not a different host language: the decode decision was made on its own grounds and stands. |
| The boundary is the measured cause of a missed editor-ready target, and the proxy design does not close it | Same move as the row above. That accuses the crossing, which is what an in-process composer removes. **Measured at S0.3 and it did not fire:** the pinned proxy costs 40 to 74 ms raw and 6 to 16 ms encoded, against 290 ms for the whole frame. A local socket is the escape route if a large crossing is ever needed, at roughly half the cost of the message channel. |
| Editing is unusable at 4K | Tile the canvas, or reconsider the editor surface, with the measurement in hand. |
| The combined implementation and maintenance cost of this candidate turns out to be unacceptable, whether or not any single gate failed | Reopen the editor architecture, or the stack, against the comparison table in part 5. Start from documentation, prototype only the consequential uncertainties in the strongest alternative, and record the verdict in `project-os/Decisions.md` as a superseding entry. Replacing Tauri also means re-implementing the tray, the global shortcut, the Explorer argument forwarding, the file-association bundle and the updater. |

No Stage 0 result justifies building the document library or the Rogers integration to
compensate for an unproven capture or decoding path.

---

# Part 9: Stage 1, the daily-use release

Dependency order, not dates. Each item stays usable on its own, and the stage ends with real
work rather than a feature count.

- [ ] S1.1 Capture lifecycle: repeated captures, hide and show, the remembered return
      target, no overwriting of a previous capture, and the empty-state line naming the
      hotkey.
- [ ] S1.2 File opening and activation: the file association, Open With, drag and drop and
      `Ctrl+O`, both when Recon is already running and when it starts because a file was
      activated, one window rather than a second instance, plus the type registration and
      the link into the Windows default-apps settings. Model: Opus 5, it is the entry point.
- [ ] S1.3 The viewing surface: fit to window on open, actual size, zoom, pan, fullscreen,
      and the filename and pixel dimensions on screen.
- [ ] S1.4 Folder navigation: previous and next across the supported types in the opened
      file's folder, the defined numeric-aware order, the position indicator, and the named
      active context so it can never be confused with Recon history.
- [ ] S1.5 The transition into annotation: viewing mode arms no tool and no click can create
      a callout, an explicit Annotate action switches modes, and leaving annotation keeps the
      work. Annotate creates a managed document with its preserved decoded image, or resumes
      the one that already exists for that source, never a second one. Opening a file that
      has a document shows the file with the route to its saved edit (§3.3).
      Model: Opus 5, it is the boundary that protects the originals.
- [ ] S1.6 Callout completion: the deterministic candidate positions, the neutral margin when
      a bubble cannot fit, and stable numbering with gaps, identical in the editor and in
      every output. Model: Opus 5.
- [ ] S1.7 The keyboard contract, routed by context, with the stage column honored, the
      viewing and navigation keys inert while a note is being edited, and undo and redo in
      full against the acceptance sequence in §3.6.
- [ ] S1.8 The document store: the schema from part 5, plus the source path and the page or
      frame for an annotated file, debounced autosave, forced saves on blur, navigation,
      opening another file, hide and quit, reopen after restart, and a visible failure that
      never claims to have saved. Model: Opus 5, it touches stored work.
- [ ] S1.9 History navigation: previous and next through documents, captures and annotated
      files alike, the position indicator, shortcuts to the first and last, and the context
      activation rules in §3.4.
- [ ] S1.10 Copy and Return, including every failure path: a failed clipboard, a failed save,
      a refused activation, a closed target application, and a user who moved on
      mid-operation.
- [ ] S1.11 PNG Save As: a new file every time, a suggested name derived from the source and
      marked as annotated, no path by which an external original is the default target, and
      an available name offered when the chosen one exists. There is no overwrite path.
- [ ] S1.12 The trial: thirty captures of real client feedback and a week of using Recon as
      the everyday viewer, with the friction recorded in `project-os/History.md` and anything
      deferred sent to `project-os/Backlog.md`.

Stage 1 is the release that replaces both current tools. It ships without the thumbnail
strip, without export options beyond PNG, and without Rogers.

Stage 1 prepares nothing for Rogers. It persists editable documents, and that is all the
foundation Stage 4 is entitled to assume.

**Acceptance for the stage as a whole**, beyond each item's own:

- Capture and copy with zero annotations; annotate with one bubble; use several; correct an
  earlier capture.
- Thirty captures in sequence with no lost work and no Done step anywhere.
- Open a file from Explorer by double-click and by Open With, with Recon closed and with
  Recon already running; the second case uses the existing window.
- Drag and drop a file onto the window, and open one with `Ctrl+O`.
- Walk a folder of mixed supported types, in the defined order, with the position indicator
  correct at both ends, and confirm folder and history navigation never move each other.
- Move from viewing into annotation and back, and confirm no click in viewing mode ever
  created a callout.
- Confirm every source file is byte for byte unchanged after viewing, navigating and
  annotating, and that Save As wrote a new file.
- Open `a.png`, annotate it, open `b.png`, return to `a.png`: the external file is shown, the
  route to its saved edit is there, and taking it resumes the same document with its
  callouts, rather than creating a second one.
- Change `a.png` on disk after annotating it, then reopen its document: the document is
  unchanged, and it says the source has moved on.
- Save As onto a name that already exists: an available name is offered, and nothing is
  overwritten.
- Annotate a non-first animation frame, restart, and export: the same frame throughout.
- Annotate the third file in a folder and confirm the navigation context is still that
  folder at position three.
- Representative files: transparency, EXIF orientation, an embedded color profile, very large
  dimensions, an animation, and a multipage file where supported.
- Small crops, dense images, each edge, long text, mixed Hebrew and English, and a bubble
  moved manually before its text changes.
- Undo and redo, delete, copy while text is being edited, copy with an object selected, and
  a repeated capture while Recon is open.
- Restart normally and recover editable work. Simulate an interrupted run and verify recovery
  to the last successful save. Exercise clipboard and save failures with no false success and
  no discarded work.
- Compare equivalent capture, annotation and viewing tasks against the current tools. Record
  concrete friction, not a feature count.

---

# Part 10: the later stages

A dependency outline only. Each one is planned properly when it arrives.

**Stage 2, the document library.** The thumbnail strip, older documents inside the same
editor, captures and annotated files together, intentional deletion, visible storage use with
the daily rate stated, and the export options beyond the PNG default. Depends on S1.8.
Acceptance: retrieve and edit yesterday's capture and yesterday's annotated file after a
restart; exported files match the visible composition at original resolution; existing exports
are never silently changed; old history is not all loaded as full-resolution images.

**Stage 3, the secondary tools.** Arrow, rectangle, text, blur, highlight, ordered by what
daily use actually demanded. Depends on the scene and composer being stable.
Acceptance: each tool takes part in selection, editing, undo, persistence and identical
clipboard and file rendering, and its arrival disturbs nothing in the callout flow.

**Stage 4, Rogers delivery.** Inspect Rogers first, then design the submission record against
what is actually there. The work is destination validation, item and attachment behavior,
what counts as success, duplicate prevention, retry under a reused identifier, and error
handling. Calling any of that transport understates it. Explicit retry only, no background
queue in v1.
Acceptance: successful delivery, an invalid target, Rogers unavailable, a rapid double send, a
response lost after the item was created, a retry after restarting Recon, the active target
changed during a request, and editing and re-sending a sent capture. Continuing to edit,
moving to another capture, or switching applications during delivery must not let a late
success close work or take focus.

---

# Part 11: the evidence status of every claim here

**Measured, on this machine, 2026-09-11:** the display is one 3840x2160 panel at 225%
scaling, at the desktop origin. Freezing the whole virtual screen with the chosen path costs
**166 to 176 ms** across five runs, which is most of the 250 ms budget from the hotkey to a
usable overlay and is the number part 8's freeze row now watches. The tray-only process sits
at **20 MB** with no window, which is the floor before any web view exists. Two per-pixel
passes are on that path and unoptimised: the freeze converts to RGBA, and the overlay
converts back to the byte order Windows wants.

**The boundary, measured at S0.3 on 2026-09-11**, by the web view's own clock, five
repetitions per row, median, with an empty round trip of about 2 ms measured separately:

| Crossing | Message channel | Custom protocol | Local socket |
|---|---|---|---|
| 31.6 MB frame in | 290 ms | 290 ms | 140 ms |
| 7.9 MB proxy in | 74 ms | 82 ms | 40 ms |
| 2.2 MB encoded proxy in | 15.5 ms | 15.4 ms | 5.8 ms |
| 3.5 MB folder-walk proxy in | 37 ms | 37 ms | 31 ms |
| 31.6 MB layer out | 359 ms | not applicable | 181 ms |
| 162 KB encoded layer out | 3.5 ms | not applicable | 3.9 ms |

About 9 ms per MB through the message channel and the custom protocol, which measure the
same; about 4.4 ms per MB in and 5.7 ms out through a local socket. The web view encodes a
4K layer to PNG in 78 ms, producing 162 KB.

**From S0.4, on the same machine:** showing the pre-created hidden window takes 12 to 22
ms, and its first painted frame is already current. A 1:1 region costs 1 ms, because nothing
is resampled. A fit-to-window region of the 3840x2160 probe costs **1.1 to 1.15 s**, which
is the resampler and is F43: the fit view is the first thing anyone sees when they open an
image, so a second of it is the viewer's core promise broken.

**A caution about every number above:** this machine's display arrangement changed three
times during Stage 0, between 5120x1440 and 3840x2160 at 225%. Every measurement therefore
carries the display it was taken on, and a figure quoted without one means nothing here.

**And the window is not DPI-scaled at all**, which is F44. The monitor's effective DPI is
216, 225%, but the editor window reports 96, its scale factor is 1, its physical and CSS
sizes are both 1280x800, and a band just outside that rectangle is not part of the window.
So the image path is exactly right, one image pixel to one physical pixel at actual size,
and the interface around it renders at a third of the size the display asks for.

**One number that is a problem for later:** the host's own PNG encoder, at its default
settings, took **633 ms** to encode a 1920x1080 image, while the web view encoded a larger
one in 78 ms. That sits on the Save As path in S1.11, not on the capture path, and it needs a
faster setting or a different encoder before it ships.

**Also found by running it:** the process is DPI-unaware unless it says otherwise, and an
unaware process is told this display is 1707x960. Every coordinate, blit and comparison is
then wrong while still looking plausible, so the host now declares per-monitor awareness as
its first act and the self test asserts it.

Everything else below is still unmeasured.

**Documented platform behavior:** the per-display capture call's own minimum Windows version;
that injected input is subject to privilege restrictions; that a request to bring a window to
the foreground can be refused; which image formats a Chromium-based view decodes on its own,
and which ones the Windows imaging stack decodes without an extra component. Sources in part
13.

**Deferred until the hardware exists:** whether Windows reports a negative desktop origin
and a mixed-scale layout the way the coordinate arithmetic expects, and whether the drag
clamp in the overlay behaves when a drag crosses onto another display. One display is
attached, so neither can be produced. The arithmetic is unit-tested against synthetic
layouts; the platform half runs the first time a second display appears, with `--selftest`
section A.

**Behavior to verify, currently unknown:** whether a registered global hotkey is delivered
while an application running as administrator holds the foreground (S0.8); whether best-effort
activation succeeds against such a window and against a closed one (S0.8); which clipboard
formats Claude and ChatGPT actually accept (S0.6); what the freeze and the first paint really
cost on this machine (S0.7); what each target format actually does here, decode or not, how
fast, and whether orientation and color survive the whole cycle rather than just the decode
(S0.5); whether the host's decoders give frame-accurate access to a chosen animation frame
here, confirmed per format rather than inferred from GIF (S0.5); whether the HEIC codec is even present on this machine
(S0.5).

**Assumed until Stage 0 says otherwise:** that pre-creating the overlay and the editor at
startup is enough to approach the proposed latency targets; that shared wrapping plus the
three S0.6 checks are enough to keep the editor and the export in agreement; that a
display-resolution proxy is enough for accurate annotation at every zoom level; that the
stack in part 5 is right, on the argument made there rather than on anyone's familiarity
with it.

**A judgment, not a measured fact:** what one native decode route costs against two
providers. One route is the design because of exact equality, and that reason stands on its
own; the relative cost has not been measured and must not be quoted as if it had been.

**Watch items rather than measurements:** whether the serialized export route stays
origin-clean across web view updates; the four decode dependencies and their security watch;
the web view's own update cadence, which makes every reference image dependent on a recorded
engine version; and any reliance on a browser flag, which the vendor does not support for
production use and which therefore cannot be part of a gate.

**Not inspected:** Copy Ninja and Rogers. Both are Rotem's to point at, in a task that names
them.

---

# Part 12: the review trail

Forty-six findings were raised against the plan and folded into the parts above. This table
is the record; the fixes themselves live where the table points. Severity is how the finding
was rated when it was raised.

| # | Finding | Sev | Where it lives now | Status |
|---|---|---|---|---|
| F1 | Nothing guaranteed the exported image matched the editor | 🔴 | Part 5 boundaries, S0.6 three checks | Resolved in plan, verification pending |
| F2 | The elevated-window claim was an assumption presented as documentation | 🟠 | S0.8, part 11 | Claim withdrawn, behavior to verify |
| F3 | Two latency targets, and the timestamps could not produce one of them | 🔴 | S0.7 six marks, two intervals | Resolved in plan, verification pending |
| F4 | Clipboard formats unspecified, so a passing paste test proved little | 🟠 | Part 6a destinations, S0.6 | Resolved in plan, acceptance at S0.6 |
| F5 | The selection overlay should not be a web view | 🟠 | Part 5, the whole split | Resolved in plan |
| F6 | Text direction was named but not defined | 🟠 | §3.5, S0.6 | Decided |
| F7 | Undo was specified by context, which the user cannot see | 🟠 | §3.6 and its acceptance sequence, S1.7 | Decided |
| F8 | The document format was deferred while three requirements depended on it | 🟠 | Part 5 document on disk, S1.8 | Resolved in plan, schema at S1.8 |
| F9 | `Ctrl+S` sat in the contract while export was a later stage | 🟠 | §3.6 stage column, S1.11 | Resolved, and Save As is real in Stage 1 |
| F10 | Return-focus needed a named mechanism and a failure path | 🟠 | §3.1, S0.8, S1.10 | Resolved in plan, verification pending |
| F11 | Numbering gaps reaching the client, and the renumbering idea | 🟡 | §3.5, `project-os/Decisions.md` | Decided: gaps stay, renumbering rejected |
| F12 | "Keep forever" had an unstated cost | 🟡 | §3.8 retention, Stage 2 | Resolved in plan |
| F13 | Capture 1 was twenty-nine keypresses away | 🟡 | S1.9 | Decided |
| F14 | An API minimum, a support baseline and an installer were treated as one question | 🟡 | Part 6a, S0.8, `project-os/Decisions.md` | Decided |
| F15 | First run and the empty editor were missing | 🟡 | S0.1, S1.1 | Resolved in plan |
| F16 | HDR was left open | 🟡 | S0.8 | Resolved in plan |
| F17 | Viewing must never modify or import the source file | 🔴 | Part 4, §3.8, S0.5, S1.5, S1.11 | Stated as an invariant, verification pending |
| F18 | Two target formats have no browser decoder, so common images cannot be promised | 🟠 | §3.7, part 6a, part 6c, S0.5 | Closed by the one native decode route; the required set is in part 6c |
| F19 | One window, two modes, and the pointer did not know which | 🟠 | §3.3, S1.5 | Resolved in plan, verification pending |
| F20 | Two navigation lists in one window | 🟡 | §3.4, S1.4, S1.9 | Resolved in plan |
| F21 | A folder walk could grow memory without a bound | 🟡 | §3.8, S0.7 | Resolved in plan, measured at S0.7 |
| F22 | Reopening an annotated external file was undefined: original, saved edit, or a second document | 🟠 | §3.3 opening a file annotated before, S1.5 | Resolved in plan, verification at S1.5 |
| F23 | Save As promised a new file and permitted overwriting a chosen one, which contradicted the invariant | 🔴 | §3.6, S1.11 | Resolved: no overwrite path exists |
| F24 | "Annotate the displayed frame" was not achievable as written, since an animated element yields its default frame | 🔴 | §3.5, part 5 boundaries, S0.5 frame test | Resolved in plan, proof pending at S0.5 |
| F25 | Rasterizing an SVG at displayed size made export resolution depend on the window | 🟠 | §3.5, §3.7, S0.5 size test | Resolved: fixed and stored at annotate time |
| F26 | Orientation and color were checked at decode, not through the persistence cycle | 🟠 | §3.7 decoded-image contract, S0.5 cycle test | Resolved in plan, proof pending at S0.5 |
| F27 | Which navigation context was active was left to the last navigation | 🟡 | §3.4 four rules, S1.9 | Resolved in plan |
| F28 | The plan claimed a bitmap-only clipboard would fail in the two destinations | 🟠 | Part 6a | Claim removed; the paste itself is the criterion |
| F29 | "Ship the shorter list" let S0.5 drop any format without a scope decision | 🟠 | Part 6c, part 8, S0.5 | Resolved: a required format is a blocked stage |
| F30 | The two-provider decode split would have created the preserved image inside the renderer, where a premultiplied canvas cannot return a straight-alpha pixel unchanged | 🔴 | Part 5 boundary rule, part 6a, S0.6 check 1 | Resolved: one native route, and the original never enters the web view |
| F31 | The exact-equality check used an opaque capture, so it could not fail on the only input that can | 🔴 | S0.6 check 1 | Resolved: a semi-transparent PNG case added |
| F32 | The stack was justified by familiarity with another project, which rule 21 forbids as evidence | 🟠 | Part 5, `project-os/Mistakes.md` | Resolved: argued from this product's own requirements |
| F33 | The replacement argument overstated what the alternatives cannot do: Qt has an editing model, native UI has automation peers, resvg has a C interface, Qt SVG has extensions since 6.7 | 🟠 | Part 5, its not-established section and comparison table | Resolved: downgraded to a recommended candidate with the overstatements named |
| F34 | A browser flag the vendor does not support for production sat inside a readiness gate | 🟠 | S0.4, part 11 watch items | Resolved: the gate runs in the shipping configuration |
| F35 | A fit-to-window display proxy cannot serve 100% or 200% zoom, so the everyday viewer would show mush | 🔴 | §3.4, part 5 display policy, S0.4 detail check | Resolved: detail follows the zoom, transport decided by S0.3 |
| F36 | The stated direction override did not exist: `unicode-bidi: plaintext` ignores `direction` and resolves per paragraph, not per bubble | 🔴 | §3.5, part 5 text layer | Resolved: three explicit modes, resolved value stored |
| F37 | The annotation export was sized to the source, so a callout inside the added margin would be cut off | 🔴 | §3.6, part 5 boundary rule, S0.6 margin case | Resolved: source plus stored margins, composited at the offset |
| F38 | Revision leftovers: two decode providers in two places, a web view decoder question, F18 still marked open, and dav1d described as a whole-file library | 🟠 | Part 5, S0.5, part 11, this table | Resolved in place |
| F39 | Readiness ended at input acceptance, so a blank or stale window could score as ready | 🟠 | S0.7 | Resolved: content and input, on one timing basis |
| F40 | The boundary figure the design was argued against was two to four times pessimistic, and it came from a discussion thread | 🟡 | Part 11's measured table, S0.3 | Resolved: measured here, and the plan quotes the measurement |
| F41 | The host's PNG encoder takes 633 ms for a 1920x1080 image at default settings, and it sits on the Save As path | 🟠 | Part 11, S1.11 | Open: needs a faster setting or another encoder before S1.11 ships |
| F42 | The plan said the annotation layer crosses back, without saying it crosses encoded, which is two orders of magnitude cheaper | 🟡 | Part 5 boundary rule, S0.3 | Resolved: encoded, and the numbers are in part 11 |
| F43 | The fit-to-window view costs 1.1 s, because it resamples the whole image, and it is the first thing anyone sees when opening one | 🟠 | Part 11, S0.4, and S1.3's viewing surface | Open: the fit view needs a cheap downscale, not a good one |
| F44 | The editor window is not DPI-scaled: the monitor is at 225% and the window reports 96 dpi, so the interface renders at a third of the size the display asks for | 🟠 | Part 11, S0.4's closing note | Open: the image path is correct, the interface is not, and the cause is not yet established |
| F45 | An image smaller than the window sat in its top-left corner instead of the middle, which twenty-six passing assertions did not notice | 🟠 | S0.4, the editor's own paint path | Resolved: centred, and the callout layer carries the same offset |
| F46 | A note is unreadable at fit zoom on a wide capture, because annotations live in image space and scale with the image | 🟡 | §3.5, S0.4, S1.6 | Resolved: no on-screen floor. The default note text is 20 image pixels and the size is the user's, which is Rotem's call |

Two rules earned during those passes, and they hold for the build too: a check must name the
two things it compares and the failure that would turn it red (`project-os/QA.md` §12), and a
comparison between two outputs of the same code proves only that the code is deterministic.

---

# Part 13: sources

- [Snagit hotkeys guide](https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/) · the global capture and Copy All shortcuts that already exist, so comparisons stay accurate.
- [Tauri webview versions](https://tauri.app/reference/webview-versions/) · the Windows web view foundation, not a latency guarantee.
- [Tauri clipboard manager](https://v2.tauri.app/reference/javascript/clipboard-manager/) · image clipboard operations; destination compatibility is still to be tested.
- [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey) · what hotkey registration does and does not promise.
- [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) · the documented privilege restriction, which is on injection.
- [SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) · when activation is allowed and when it is refused.
- [CreateForMonitor](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createformonitor) · the per-display capture call and its Windows 10 1903 minimum.
- [High DPI development on Windows](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows) · DPI contexts and coordinate handling.
- [Native WIC codecs](https://learn.microsoft.com/en-us/windows/win32/wic/native-wic-codecs) · what the Windows imaging stack decodes with nothing added, and where page access comes from.
- [Image file type and format guide](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types) · what a Chromium-based view decodes on its own, which is where the TIFF and HEIC gaps come from.
- [Canvas image sources](https://html.spec.whatwg.org/multipage/canvas.html#image-sources-for-2d-rendering-contexts) · why drawing an animated image element gives its default frame, which is what F24 is about.
- [Chromium clipboard on Windows](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/base/clipboard/clipboard_win.cc) · the implementation that can hand native bitmap data to a page as PNG, which is why the F28 claim was withdrawn.
