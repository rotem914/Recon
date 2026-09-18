# Recon, the plan

One document. What Recon is, how it behaves, what was decided, and how it gets built.

Read this file and `project-os/Backlog.md` at task pickup. There is no second plan.

Status: consolidated on 2026-09-10. Approved for the Stage 0 experiment.
The stack in part 5 is a **recommended candidate, not a settled decision**: Stage 0 has
permission to reject it, and part 8 says on what evidence. The decode route and the format
decisions inside that candidate are settled (part 6a).
Built so far: S0.1 to S0.4, four islands that have not yet run as one thing. Every timing
in part 11 came from a debug build until part 11 says otherwise.

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
- A capture brings the editor to the front the way a click on its taskbar button would: a
  minimized editor is restored first, an editor behind another window comes forward
  (Rotem, 2026-09-16).

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

**Start with Windows.** The tray menu carries a "Start with Windows" tick. On, Recon writes
one value under the user's own Run key, so Windows starts it at logon, where it sits in the
tray with no window shown; off removes exactly that value. Nothing is written until the tick
is chosen, and a start by hand still opens the window. Added 2026-09-15 at Rotem's ask.

## 3.2 Capture flow

Global shortcut, freeze the desktop image, dimmed selection overlay, drag a region, open
the editor with that region.

- If Recon is visible, hide its windows before the snapshot. Its editor, dimming and
  selection decorations must never appear in the result.
- Selection happens on the frozen image, so content cannot change under the pointer
  mid-drag.
- Before a drag begins, the window under the pointer is the selection: its visible frame
  is lit as the pointer moves, and a click with no drag captures it. Where the window has
  parts, its child windows, the smallest part under the pointer is the selection instead:
  a browser's page without its tabs and address bar, a folder window's file list, the way
  Snagit picks them. A drag past the system's drag threshold draws a free region instead.
  Added 2026-09-14 at Rotem's ask, the parts the same evening. A magnifier sits beside the
  crosshair the whole time the overlay is up: a 112 px circle showing the pixels around the
  pointer large enough to tell apart, with the selection's width and height in pixels
  written under it, the lit window's or the dragged area's; the look is in
  `project-os/Design.md`. Added 2026-09-17 and 2026-09-18 at Rotem's ask.
  When the pointer moves from one window to the next, the lit area glides from the one to
  the other rather than jumping; the motion is in `project-os/Design.md`. Added 2026-09-18
  at Rotem's ask.
- A region can be selected on any connected display. A single region spanning two displays
  is out of v1: selection stays within its starting display.
- `Esc` cancels and restores the preceding context. A cancelled capture creates nothing and
  replaces nothing.
- Overlapping capture requests are ignored while a selection is already active.
- No save dialog, no format choice, no intermediate confirmation.
- The capture is on the clipboard the moment the editor opens with it, the same copy as
  `Ctrl+C`, so a paste needs no key; the HUD says copied, or NOT COPIED with the reason. A
  file opened is never copied unasked. Added 2026-09-16 at Rotem's ask.

## 3.3 One window, two entry behaviors

The same main window serves both, and which one is active must be obvious at a glance.

**A new capture opens ready for annotation.** No tool is in hand: the pointer is the
ordinary arrow, a click on the picture creates nothing until a tool is chosen, and a drag on
the picture pans it, as in viewing; every picture shown starts that way again. With a tool
in hand, Space held gives the ordinary pointer and the same pan, and letting go gives the
tool back. Rotem's calls on 2026-09-14; until then the callout tool was live and a click
started a note.

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
- **Zooming out stops at the whole picture**: its actual size when it fits the window, the
  whole of it when it does not. When the space changes, the window resized, full screen, or
  the timeline, a picture shown whole, or smaller than whole, takes the new whole view, and a
  zoomed-in view keeps its zoom. Rotem's word on 2026-09-15, both; until then it went down to
  a quarter of the fit and nothing followed the space.
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

With the callout tool active, two clicks (Rotem, 2026-09-16; one click made the bubble
and started the typing before):

1. Click the point the note refers to: the end of the line.
2. A bubble appears near it, at its automatic place, and moves with the pointer from
   there, keeping that offset, so the point it names stays visible. The line stays
   attached to it.
3. Click again where the bubble should stay. It locks there, with the text cursor inside.
4. Type. The visible text is immediately part of the composed image.
5. `Enter` or `Esc` leaves text editing and keeps the text (Enter since 2026-09-17; it broke
   the line before, which `Ctrl+Enter` does now). Between the two clicks, `Esc`, Ctrl+Z or a
   change of tool drops the unplaced bubble, and its number goes back.

Where the pointer sits in the bubble while it follows is provisional, the assistant's
choice, Rotem's to change: the bubble keeps the offset it started with, so the pointer is
beside it, not inside it.

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

- Callouts are numbered by creation order, starting at 1 for each image. Since 2026-09-17
  the number is not shown in the bubble, at Rotem's word; it is kept in the document.
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
| `Ctrl+C` while editing text | Normal text copy. It does not copy an image by surprise | 1, built at S1.7: the key is not touched while a note is being typed |
| `Ctrl+C` outside text editing | Copy the full composed image, even with an annotation selected | 1, built at S1.7 |
| `Ctrl+Shift+C` anywhere in the editor | Copy the full composed image, current text edits included | 0, built at S0.6 as the one key the clipboard operation needed to be tried by hand |
| `Ctrl+Enter` outside text editing | Copy the full composed image, then return to the previous application after success | 1, built at S1.7: the editor stays if the document changed while the copy ran; the save before the hide is S1.8's. Until 2026-09-17 it worked while a note was typed too, committing the note first |
| `Ctrl+Enter` while editing a note | Insert a newline | 1, built 2026-09-17 at Rotem's word; Shift+Enter does the same |
| `Enter` while editing a note | Leave text editing, keep the text, as a click outside the bubble does | 1, built 2026-09-17 at Rotem's word; it inserted a newline from S0.4 until then |
| `Ctrl +` (or `Ctrl =`) in the editor | One text size up, on the note being edited or the selected one, and it becomes the default for the next note | 0 |
| `Ctrl -` in the editor | One text size down, the same way | 0 |
| `Esc` during capture | Cancel the capture | 1, built at S0.2: while a window is lit and in the middle of a drag alike, nothing captured; the self test's section D presses it mid-drag. Read back on 2026-09-18, when Rotem asked for it |
| `Esc` during text editing | Leave text editing, keep the text | 1, built |
| `Esc` with an annotation selected | Clear the selection | 1, built |
| `Esc` in fullscreen | Leave fullscreen | 1, built at S1.3 |
| `Esc` in the otherwise idle editor | Save pending changes and hide the editor | 1, the hide and the focus return built at S1.1, the save at S1.8 |
| Delete outside text editing | Delete the selected annotation | 1, built; Backspace does the same |
| Undo and redo | While a note is being edited, undo works on the typing and never reaches object operations. Once editing ends, that text change takes its place in document history as one grouped step. A document deleted from the timeline is in the same history: Ctrl+Z brings it back, Ctrl+Shift+Z deletes it again, the last thing done first. The timeline's tabs too: a tab added, deleted or renamed, and a done mark on a picture, and every new action from here on (`CLAUDE.md` rule 23) | 1; the deletion at S2.9, built 2026-09-16; the tabs the same day |
| `Ctrl+O` | Open an image file | 1, built at S1.2 |
| Previous and next image, outside text editing | Walk the active navigation context, folder or Recon history, in the §3.4 order | 1, built at S1.4 for the folder and at S1.9 for history: PageDown and PageUp, Home and End for the first and last; the ends stop |
| Fit to window · actual size · zoom in · zoom out | Viewing controls, no effect on export resolution; zoom out stops at the whole picture (§3.4) | 1, built: keys, and the wheel to zoom around the pointer, with Ctrl or without, at Rotem's call on 2026-09-14; a sideways wheel pans; the zoom-out floor at Rotem's word on 2026-09-15 |
| `Space` held, outside text editing | The ordinary pointer, and a drag pans the picture whatever tool is in hand; letting go gives the tool back | 1, built at Rotem's word on 2026-09-14 |
| Fullscreen | Enter fullscreen | 1, built: F11, and the same key or Escape out |
| Annotate, and back to viewing | Switch the window between viewing, where no click creates anything, and annotation, where the tools are live and none is in hand until one is chosen (§3.3). The work stays either way | 1, built at S1.5: the `A` key and a button at the window's top right, both, Rotem's call on 2026-09-14 |
| `Ctrl+S` | Save As a PNG file, to a new file. Internal saving stays automatic | 1, built at S1.11: Windows' own Save As, the last export folder, the suggested name, an available name offered when the chosen one exists |
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
| **AVIF** | Decode, transparency preserved, through the Windows imaging stack (decided 2026-09-13, F51). It depends on the AV1 Video Extension being installed and is reported as a missing codec when it is not, the way HEIC is. An animated AVIF shows its first frame only on this route, so the animation rule does not apply to it (F53); a buildable AV1 decoder would lift that. |
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

**Revisited 2026-09-13, and sRGB stays (F60).** The main display is wide gamut, and the
capture still carries the numbers the application drew, which on an unmanaged Windows
desktop are sRGB numbers shown more saturated than meant. Treating them as sRGB keeps the
application's intent and matches what every other screenshot tool hands on; tagging them
with the display's profile would make the paste look oversaturated on everyone else's
screen. The trigger moves to Windows' own colour management: when the desktop is managed
(Windows 11 automatic colour management on, or HDR on), the frozen pixels stop being plain
sRGB numbers, and that state is what `host/src/platform.rs` now reads.

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
- The strip has tabs (Rotem, 2026-09-16, built the same day): a plus at the left of the
  picture's size band makes Main and New tab at its first press, and another New tab at
  every press after, the new one selected. Main is the whole library, pinned first and
  never deleted. Every other tab is a feed: a capture taken while it is selected lands in
  the library, so in Main, and in that tab too, so a client's remarks in a call become a
  feed of tasks. A tab is deleted by its ×, pressed twice, its captures staying in Main,
  dragged into another place after Main, and renamed by a click on its name when it is
  selected. In a tab every thumbnail carries a round done mark at its top left, shown on
  hover and kept once pressed, to say that picture is dealt with in that tab; nothing else
  follows from it. Adding, deleting and renaming a tab and the mark are in undo. The list
  is kept whole in `tabs.json` beside the documents.
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
does not edit an image beyond annotating it: no resize, no color adjustment in v1. Crop
came in at Rotem's word on 2026-09-17 (S3.6), as a cut the host makes on a copy at every
output, the source untouched. Writing back into an external file, in any format, stays out.

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
| Windows support | The API's technical minimum, the versions Recon supports and tests, and runtime packaging are three separate answers, not one. Recorded in `project-os/Decisions.md`. Answered at S0.8: the minimum is Windows 10 (WebView2's runtime; 1903 for the capture call); supported and tested is Windows 11 at 100% and 225%; Windows 10 22H2 expected, untested; packaging is Stage 1's. |
| Elevated windows | Test first. No elevated mode on an unverified assumption (S0.8). |
| Image viewing | A core capability, in one window with two entry behaviors. Recorded in `project-os/Decisions.md`. |
| Initial capture path | One copy of the whole virtual screen: one synchronous call already spans every display and every negative coordinate, so the foundation risk is retired at the lowest cost. It sits behind the capture interface, which must leave a video source possible later while adding no video infrastructure now. A second path is earned only under part 8. |
| Paste destinations | Claude and ChatGPT. Both read the clipboard the way a web application does. PNG is the implementation choice, not a proven requirement: a Chromium-based reader can convert native bitmap data into PNG for the page, so nothing here claims a bitmap-only clipboard would fail. What decides acceptance is the paste actually working in those two surfaces, recorded with the surface and its version. Accepted 2026-09-13: the host publishes `PNG`, `CF_DIBV5` and `CF_DIB` together, and a paste into claude.ai and into chatgpt.com, both the web apps in Chrome on that date, attached the composed image as a PNG in each. |
| Reference environment | Rotem's main machine at its current display scale. The environment, the font details and the web view runtime version are recorded with the reference images and in the S0.8 report. |
| The stack | A Rust host owning the pixels, a WebView2 editor laying out the text, packaged with Tauri v2. Argued in part 5 from the decode surface, from the bidirectional editing model, and from this project's own browser-driven QA gate. Recorded in `project-os/Decisions.md`. |
| Decoding | One native route for all nine formats. The web view decodes nothing, and the decoded original never crosses into it at full resolution. This flips the earlier two-provider recommendation, because a preserved image born in the renderer cannot survive a premultiplied canvas byte for byte. Recorded in `project-os/Decisions.md`. Built at S0.5 as three providers behind one interface: the `image` crate, `resvg`, and WIC for TIFF, HEIC and AVIF (F51, decided 2026-09-13). |
| TIFF, HEIC and AVIF | All three through WIC in the host: TIFF with its pages, HEIC and AVIF through whatever codec the machine has, with the named missing-codec message rather than a corrupt-file one. |
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

**Required in Stage 1:** PNG, JPG, GIF, WebP, BMP, AVIF and SVG. AVIF decodes through the
Windows imaging stack (decided 2026-09-13, F51), so it degrades the way HEIC does: the
named-codec message is a pass, and an animated AVIF shows its first frame.

The reason this set is affordable has changed, and it is worth re-reading before it is
treated as settled. It used to be "the web view decodes all seven for free". With one native
decode route that is void: the seven are affordable because `image` covers five of them,
`resvg` covers SVG, and the Windows imaging stack covers AVIF with nothing added to the
build. The membership does not change. The price does: three decode dependencies and their
security watch, forever, as part 11 now says, plus one installed codec for AVIF and HEIC.

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
**[x] S0.4b · One thing, end to end, and the numbers made real**

Model: Fable 5.1. It changes the product path and it decides what part 11 means.

Added on 2026-09-13 from the review in `notes/2026-09-13-review.md`, and it runs before
S0.5. The four steps above are islands: the hotkey path ends by writing a PNG to disk, and
the editor is built only by its own diagnostic runtime. This step joins them and then
re-measures everything on the build the product would ship.

Delivers: the editor created hidden at startup on the product path, a capture handed to it
after the selection and shown at fit; the fit view served from a pyramid built once per
image, so opening a 4K image is not a one-second wait (F43); the check-only and demo-only
commands behind a cargo feature so the shipping binary has no command that lets the page
capture the screen or write a file; a region worker that serves only the newest request;
and every figure in part 11 re-measured with `--release` and recorded beside its debug
number.

Evidence: hotkey, drag, and the editor opens with that region on the canvas. The fit view
of the 3840x2160 probe under 50 ms in release. The 33 editor checks and the self test
passing on both profiles. Part 11 carrying two columns.

**Closed 2026-09-13.** Hotkey, drag, editor: the capture reaches the canvas, and on the
product path. Run with `--capture-demo`, which fires the capture and drives the overlay with
synthesized input, the editor is shown **13 ms** after the selection in release (44 ms in
debug) and the page paints the capture 38 ms after hearing of it. The window itself is
created hidden at startup in about 220 ms, before any hotkey can fire, and closing it hides
it.

**The fit view is 47 ms, not 1.1 s, and the second was never the resampler's.** A generic
function is compiled in the crate that calls it, at that crate's optimisation level, so
optimising the `image` crate in the dev profile did nothing for the resample. The per-pixel
work now lives in `host/pixels`, a crate optimised in every profile, and the image gets a
pyramid of halves built on demand by the one region worker: level 1 of the 3840x2160 probe
takes 67 to 82 ms once, and every fit view after it 44 to 51 ms, in debug and release alike.
The 1:1 path is untouched and still crisp: the checkerboard check passes on both profiles.

**Every figure in part 11 now has a release column**, and two findings closed on it: the
host's PNG encoder takes 12 ms in release, not 633 (F41), and the freeze is 41 to 50 ms, not
145 (part 8's freeze row watches the release number from here on).

**The product build has no command that touches the screen or the disk.** The eight
check-only commands, the probe, the self test, the bench and the demo runs are behind the
`stage0-checks` cargo feature; the product registers four commands: the image's size, show,
the window metrics and the log. The page's `listen` needed a capability file, which is the
one place the page's permissions are now written down.

**What a check found while this was built.** The self test's overlay-in-output comparison
was inconclusive on this desk because a video was playing under the test area; it now
samples a candidate area twice and drags somewhere still, and the comparison is exact
again. And 36 editor checks pass, three of them new: a typed Enter yields a two-line note,
an engine-made paragraph break reads as a line break, and the committed note is laid out
on two lines (54 px against 27 px for one).

----
**[x] S0.5 · Open and decode an existing image**

Model: Fable 5.1. Format behavior, and a decision gate for the product surface.

**What closes here, and what is carried.** Three of the gates below run through an internal
save, a restart and an export, which are S1.8 and S0.6 and do not exist when this step runs.
So S0.5 closes on: decode, display, the frame or page reaching the model by index, the
byte-for-byte external-file check, and the per-format evidence line. The export leg of the
animation, orientation, colour and SVG tests is a carried gate in S0.6; the save-and-restart
leg is a carried gate in S1.8. Neither is ticked here on partial evidence. And S0.6's export
spike runs before this step's format work, because it is a day against a week and it decides
the composer.

**Built 2026-09-13, and open on two inputs.** The image source exists: `host/src/source`,
path in, decoded frame and metadata out, one interface, three providers behind it. The
`image` crate for PNG, JPEG, BMP, GIF and WebP; `resvg` for SVG; the Windows Imaging Component
for TIFF, HEIC and AVIF. The editor shows an opened file on the same canvas a capture uses
(`--open <file>`, and `[` `]` step frames or pages), and a file's frame or page is asked for by
index and comes back as itself.

The evidence, from `--decode-report` on the generated fixtures, one line per file, in debug and
release: PNG with a swapped-primaries profile came out converted (stored red reads as green,
so the profile was honoured and not just noticed); the transparent half stayed transparent;
a JPEG tagged orientation 6 came out 200x300 with the red corner at the top right and the
note saying 6 was applied; BMP as written; GIF three frames red, green, blue, 80 ms each,
frame 3 refused; a hand-wrapped animated WebP the same, and a lossless still with its alpha;
a three-page TIFF through WIC, page three blue; an 8000x6000 PNG opened in 100 ms and
handed over in 26 ms in release (326 and 25 in debug); three SVGs rendered, and rendered
the same as this web view within 0.66% of pixels on text, 0.01% on a CSS style block and
0.08% on a mask with a clip, measured in the editor checks by drawing the same file in
Chromium. A text file was refused by name of what it is. And the invariant: every one of
the twelve files hashed the same before and after being opened, viewed and stepped
through, nothing appeared beside them, and nothing appeared in Recon's data folder.

**What has no evidence yet, and why the step stays open.** HEIC and AVIF have no fixture
this toolchain can generate, so their rows are pending a real file each: a HEIC from a
phone, an AVIF saved from a browser. The decoder for both is in place, and this machine
has the HEIF, HEVC and AV1 extensions installed, so the expected result is a decode; the
expected result is not evidence. The SVG content test ran on hand-written files shaped
like a Figma mask export and an Illustrator style block; a real export of each is worth
one more line. Also worth a real file: a phone JPEG with both an orientation and a
profile, and a real animated WebP.

**The AVIF chain is not the one the plan named, and that is Rotem's call (F51).** libavif
with dav1d needs a C toolchain (cmake, nasm, meson) this machine does not have, and the
three pure-Rust AV1 ports on crates.io do not compile on Rust 1.98. AVIF therefore goes
through WIC here, which means it depends on the AV1 Video Extension being installed, the
same way HEIC depends on its extension. Part 6c lists AVIF as required with no degraded
state; the options are to accept the codec dependency for AVIF with the named-codec
message, or to add the C toolchain and build the chain the plan named. Decided 2026-09-13,
at Rotem's delegation: the Windows imaging stack, recorded in `project-os/Decisions.md`.

**Two carried gates, restated.** The export leg of the animation, orientation, colour and
SVG tests runs at S0.6; the save-and-restart leg at S1.8. And one number for the record:
the first fit view of the 48-megapixel PNG waits 442 ms for pyramid level 1 in release,
which is F52.

**Closed 2026-09-13, on twenty-eight real files fetched from public test sets.** Rotem asked
for the files to be found rather than supplied, so they came from the AOM AV1-AVIF test
suite (Link-U, Microsoft and Netflix folders), the Nokia HEIF sample set, libheif's
examples, the recurser EXIF-orientation set, the ianare EXIF samples, Pillow's test images,
tlnagy's example TIFFs, Google's WebP gallery, Mathias Bynens's animated WebP, an
Illustrator export in a public repository, and two Figma exports quoted in public issues.
They live in the scratch folder, not the repository; part 13 names the sources. Every one
of the twenty-eight decoded, every one hashed the same afterwards, and nothing was written
anywhere. What they showed, beyond the generated fixtures:

HEIC decodes on this machine through its installed extension, three files, 1280x854 to
1440x960, 13 to 51 ms to open. AVIF decodes the same way, eight files including 4K, 10-bit
and alpha. A rotated AVIF from the Microsoft set came out upright and portrait, so WIC
applies the file's rotation itself; looked at, it is a photograph of Ronda standing the
right way up. A real animated WebP steps through its twelve frames, and the twelfth is the
one on screen. A JPEG carrying a 400 KB profile was converted; a real EXIF-rotated
landscape and portrait came out 1800x1200 and 1200x1800 with 6 and 8 applied. Multipage
TIFFs of 27, 35 and 2 pages expose their pages. The two Figma exports, one with an alpha
mask and one with a mask plus a filter and a gradient, and the Illustrator export with its
style block all render within 0.7% of pixels of this web view.

**Two things a real file found that a generated one could not.** First, WIC's converter
does not apply an embedded colour profile: a profile-tagged TIFF came back unconverted
until the frame's colour context was read and put through the same sRGB conversion the
other providers use, which is done now and shows as "converted from a profile of 3144
bytes" on that file (F55, resolved). Second, an animated AVIF through WIC is one frame: the
two Netflix image sequences report a single frame, so frame access by index for AVIF
sequences does not exist on this route (F53). That sharpens F51: the codec dependency is
not the only cost of the WIC route for AVIF; the animation rule in §3.5 is unmet for it.

**One small thing.** The `image` crate does not read EXIF orientation out of a WebP, so
a WebP tagged with one comes out as stored (F54). §3.7 promises orientation for JPEG and
HEIC, not WebP, and a phone does not write WebP, so it is a nit and it is recorded.

**Still carried.** The export leg to S0.6 and the save-and-restart leg to S1.8, unchanged.

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
**[x] S0.6 · Output fidelity and the clipboard**

Model: Fable 5.1. Fidelity, and the one place work can be lost.

**The export spike comes first, before S0.5.** Serialize the demo scene with fonts inlined,
draw it, read it back, encode it, send it to the host, composite it over the capture at the
margin offset, write the file, and record the WebView2 version beside the result. One
composited PNG whose source area equals the capture byte for byte. If reading back throws,
part 8's in-process composer row fires now, before nine formats are wired to a route that
does not exist.

**Spike run 2026-09-13, and the route holds.** On WebView2 152.0.4191.66, the demo scene
(two notes, 20 and 40 px, English and Hebrew, on a 5120x1440 capture) serialized to 68 KB
of SVG, rasterized in 3 ms, and the canvas read back cleanly: origin-clean is true on the
shipping engine, and it stays a watch item per update. The web view encoded the layer in
38 ms (193 KB); the host decoded it in 7 ms, composited it over the untouched capture in 6
ms and wrote the file in 14 ms, in release. Every one of the 7.2 million pixels the layer
did not touch is byte for byte the source, and 183,493 pixels carry the notes. Looked at:
the exported file shows both notes at image scale with the same four line breaks as the
window screenshot, the Hebrew note right-aligned with its number on the right. Not yet
done here, and still S0.6's: the semi-transparent source case, the six reference images
with a stated tolerance, the live-typing comparison, the margin, and the clipboard. The
fonts are the system's, so nothing was inlined; a web font, if one is ever chosen, brings
that remedy back.

**Carried from S0.5:** the export leg of the animation frame, orientation, colour and SVG
raster size tests. The frame or page chosen at S0.5 is what the export shows.

Delivers: the composer used for both display and export, and a native clipboard operation
publishing several image formats.

**Built 2026-09-13.** The composer is one host function (`host/src/compose.rs`): the source
dropped into the output byte for byte at the margin offset, alpha included, the margin around
it in one neutral colour, and the page's layer over both in straight alpha. The page sends
that layer in canvas space, decorations stripped by a colours-only "plain" state while the
styles are read, and nothing is committed or blurred to make it: a note being typed is in
the copy as typed (§3.6). The clipboard is the host's (`host/src/clipboard.rs`): `PNG`,
`CF_DIBV5` and `CF_DIB` in one transaction, every format built before the clipboard is
touched. `Ctrl+Shift+C` is wired so the operation can be tried by hand; the other copy keys
stay S1.10. The margin colour is E9EAEC, a value picked to have one, and Rotem's to change.

**Check 1, original pixels survive: passed, on a source that could fail it.** A 320x200 PNG
whose alpha runs 0 to 255 with a band of odd values in the middle: no notes, the output is
the source byte for byte; two notes, every uncovered pixel exact; a margin of 30, 0, 120 and
20, the source exact at its offset in a 470x220 output. The same count is zero on the 5120
x1440 demo capture (183,493 pixels covered) and on every one of the six documents below.

**Check 2, annotated output is correct: six references, committed.** `host/references/s06/`
holds ref-hebrew, ref-english, ref-mixed, ref-long, ref-edges and ref-margin, laid on a
640x400 synthetic screen, plus `environment.txt`: WebView2 152.0.4191.66, the window at
100% on a 5120x1440 display, the note font ui-sans-serif resolving to Segoe UI. The
tolerance is stated there and in the check: a channel difference above 48, premultiplied,
counts a pixel; a document passes at 0.5% or fewer; a different wrap or a moved box fails at
any tolerance. The references were looked at (numbers right, arrows to the right edges,
Hebrew right-aligned with its number on the right, the margin note inside the margin), and
a second run against them was identical on every one. They are valid against that
environment and that web view version only, which is why both are written beside them.

**Check 3, the live editor and the output agree while typing: zero pixels differ.** In each
of the three text cases a note was typed, the caret moved to the middle and more typed
there, and the copy taken with the caret live. The text box on screen, at zoom 1 with the
decorations off, against the same rectangle of the composite: identical, 0 of 22,680
pixels, the same four line bands to the pixel, in Hebrew, English and mixed direction. The
decorations are colours only, and the check asserts the text box does not move by a pixel
when they are switched off. Editing is not ended by the copy, and focus stays.

**Origin-clean on WebView2 152.0.4191.66:** true, on every run; still a watch item per
update, and the message says so if it ever fails.

**The margin case: passed.** A note entirely inside a 240-pixel right margin exports in an
880x400 output, at its offset, and reaches the clipboard the same size.

**The clipboard:** read back the way a destination reads it, all three formats present and
the PNG the composite byte for byte. Then pasted, by hand through the browser, into
claude.ai and chatgpt.com: both attached it as an image (part 6a records the surfaces).

**Carried from S0.5, closed:** a stepped GIF exports frame 3, not frame 1; the rotated JPEG
exports upright at 200x300 with its mark top right; the profiled PNG exports converted,
its stored red green, its transparent half exact; the SVG exports at the raster size fixed
at open. The save-and-restart leg stays with S1.8.

**Not done here, on purpose.** The margin is set explicitly; growing it when a bubble
cannot fit is S1.2's placement work. Panning cannot reach the far edge of a margin at a
zoom above fit (F56, nit). The 225% display case of check 3 was not run, because this
machine's display is at 100% today (part 10); the reference environment says so.
**Run at 225% on 2026-09-14**, when the display was scaled: the wrap and the box were the
same to the pixel, and 12.5% of the text box's pixels differed until the canvas offset was
snapped to a physical pixel (F74), 6.6% after, which is glyph antialiasing through a
fractional transform, the clause the tolerance is for; the check's bar is 8% on a scaled
display and 3% at 100%, where the figure is still 0 (F75).

**Reviewed (rule 17, high):** this task's own diff. Two findings on it, both fixed: the
clipboard's comment claimed a failed publish leaves the old contents, which is only true
before the clipboard is emptied, and a needless lint allowance. One pre-existing, reported
not fixed: the product build warns that three fields of the decode notes are read only by
the report (F57, nit). Reach: the export layer's two consumers, the demo and the checks, run
green; the view offset and the file load, which every view uses, are covered by the seventy
-three editor checks and by looking at the demo.

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
**[x] S0.7 · Instrumentation and the thirty-run measurement**

Model: Opus 5. Mechanical, known shape. Release build only.

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

**Measured 2026-09-13, release build, and both intervals are well inside their targets.**
Thirty captures of this machine's one display, 5120x1440 at 100%, with the process already in
the tray and the editor hidden. That is 89% of a 4K frame's pixels; the plan's 4K runs need a
4K display this machine no longer has. The run drives the product path with the real hotkey
and a synthesized drag of the whole display; the drag is in neither interval.

**Hotkey received to overlay usable:** 74.0 59.3 56.6 63.3 74.9 71.1 72.3 70.1 75.7 80.5 75.6
73.9 75.5 74.4 76.7 76.4 72.5 74.5 72.6 86.7 74.7 73.2 74.7 73.3 70.2 73.9 76.4 74.0 69.8 74.0
ms. Maximum 86.7 ms against 250. The freeze is about 49 ms of it.

**Selection completed to editor usable:** 113.1 116.6 121.8 125.9 125.5 129.6 124.4 124.7 125.1
121.1 125.7 124.9 130.4 124.3 124.8 125.5 130.0 134.1 120.6 124.8 125.1 112.4 122.1 124.6 122.0
120.5 125.3 124.8 123.0 121.2 ms. Maximum 134.1 ms against 500. The show returns about 22 ms
after the selection; the page's region, paint and focus take about 100 ms more.

**The percentile method:** nearest rank, the value at rank ceil(p/100 x n) in ascending order.
With thirty samples the 95th percentile is the 29th value, the second-worst: 80.5 ms for the
overlay and 130.4 ms for the editor. Medians by the same method: 74.0 and 124.7 ms.

**How "usable" was read, on one clock.** Every mark is taken in the host, in `host/src/marks.rs`;
the page's two are stamped when the host hears of them, so they are late by one crossing,
never early. The overlay is usable at the later of "displayed", every display's overlay
painted once, and "accepting input", a message it posts to itself being dispatched by its
own loop. The editor is usable at the latest of the show call returning, the page's second
animation frame after the new pixels, and the page holding keyboard focus with the image
loaded. What this cannot see: the compositor's final present, up to one display refresh
after "displayed". After each run the host looked at the editor on the screen: 4.4 to 4.9%
black and 286 to 359 coarse colours in every run, so a picture, never a blank surface.

**Process startup, measured separately,** from the start of `main` over seven launches: ready
255 to 328 ms, the editor page booted 18 to 46 ms after that. From launch, including the
operating system's time before `main`: 268 to 304 ms warm, 646 ms for the first launch.

**Memory, as a floor and a slope.** Private bytes of the host plus the six web view processes
it starts; working sets are in the CSV.

The floor, tray and hidden editor, nothing captured: 168 to 172 MB over seven launches. The
host is 6 to 7 MB of it and the web view 162 to 165 MB. That number is judged on its own: a
process that sits in the tray all day costs 170 MB, and nearly all of it is the web view that
the stack decision chose (part 5). Part 8 has no row for it; it is F59.

Across thirty captures: 220 MB after them, 52 MB above the floor. 35 MB of that is the live
document, the last capture and its pyramid, kept on purpose while the editor is hidden. No
slope: the host is flat after the first capture, and the web view climbs about 3.6 MB per
capture and falls back when it collects, at captures 16, 23 and 29, to 170 to 178 MB each time.

Across a folder walk of the twenty-eight real files from S0.5: a peak of 361 MB while a 4K
AVIF and the animated files were open, and 200 MB after, 20 MB below where the walk started,
with the live image down to 0.1 MB. Memory does not grow with the number of images seen.

**The release discipline that produced it.** The host holds one image: the current capture
or the opened frame, plus the pyramid levels built from it, replaced and never accumulated
on each capture or open. Nothing is retained across images, because the store is S1.8; today
retained history is zero by construction, and the live document is the number above. An
opened animation keeps its decoded frames while it is the current file. The page holds one
viewport-sized canvas, and each region's buffer is garbage once painted. The editor is hidden,
never destroyed, so its web view is part of the floor.

**Not measured here, and why.** A 4K display, as above. The 225% scaling case, because this
display is at 100% today. The product build itself: the measurement runs on the checks build,
with the two checks-only costs on the timed path switched off while it measures, the probe
image at boot and the PNG written for every capture. The first attempt kept that PNG write
and was also disturbed: five runs lost their marks because someone was using the machine.
Its editor median was 154 ms, and the harness now waits for quiet input and counts only clean
runs. That attempt is in the History row.

----
**[x] S0.8 · The limits, and the go or no-go report**

Model: Fable 5.1. Judgment on evidence.

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

**The report, 2026-09-13.** Written from the evidence of S0.1 to S0.7 and the platform
facts read on this machine that day: Windows 11 Home 25H2, build 26200, one active display.

**Measured on this machine.**

The display is a Samsung LS49AG95 at 5120x1440, 100% scale, on `\\.\DISPLAY1`. Windows
reports HDR supported and OFF, 8 bits per channel, RGB, SDR white 80 nits. A Sony TV is
connected but is not an active display path today. Detection is built (`host/src/platform.rs`):
every display's state is logged at startup, and a freeze taken while any display has HDR on
logs that the frozen pixels are the SDR view Windows gives GDI. That is "not supported in v1,
detected rather than silently wrong"; the user-facing message for it is Stage 1 (F61).

Colour gamut, from each monitor's own EDID primaries. The Samsung: red 0.691, 0.293; green
0.273, 0.656; blue 0.148, 0.055. Its triangle is 132% of sRGB's area and 98% of DCI-P3's:
a wide-gamut display. The Sony: red 0.625, 0.340; green 0.277, 0.594; blue 0.152, 0.070,
96% of sRGB: an sRGB-class display. The record the sRGB decision named as its trigger now
says the main display is wide, so that trigger has fired (F60), and it is Rotem's call.

Return of focus, through the self test's section F, against a window in a process of its
own. After an Escape the foreground goes back to the window that was in front, and the log
names it with its process and elevation. With that window closed while the overlay was up,
Escape still cancels cleanly, nothing is restored, and Windows chooses what is in front; no
product decision rests on which window it chooses.

The web view is WebView2 152.0.4191.66, recorded with the S0.6 references in
`host/references/s06/environment.txt`; every reference image is meaningful against that
version only. The codecs installed here: HEIF Image Extension 1.2.48, HEVC Video Extension
2.5.33, AV1 Video Extension 2.0.30, WebP Image Extension 1.2.31, Raw Image Extension 2.5.35.

Elevation: this process runs unelevated. The hotkey against an elevated application in the
foreground cannot be measured by a synthesized key press, which Windows refuses when an
elevated window is in front, so Rotem pressed it: with Task Manager in front, the release
product logged the fire, the overlay came up, and a 396x334 selection reached the editor 2 ms
after it. The hotkey works against an elevated foreground on this machine (F62, closed). That
run's build predates the overlay's foreground line, so the window's elevation is Rotem's word
and Task Manager's nature, not a log line; every later build logs it.

**Documented behavior, not measured here.**

WebView2's Evergreen runtime is documented for Windows 10 and Windows 11, is part of Windows 11,
and is on the vast majority of Windows 10 devices; it takes the same updates as the Microsoft
Edge Stable channel, which is why the runtime version is recorded beside every reference. Tauri
v2 documents Windows 7 and later with WebView2 preinstalled from Windows 10 version 1803. The
per-display capture call's own minimum is Windows 10 version 1903 (the decision of 2026-09-10).
The HEIF and AV1 codecs are Microsoft Store extensions, present on this machine without being
installed by anyone.

So the three answers part 6a asked for: the API minimum is Windows 10 (WebView2's runtime, and
1903 for the capture call); the versions Recon supports and tests are Windows 11, 100% and
225% scaling, which is what every Stage 0 figure was measured on; Windows 10 22H2 is expected
to work and is untested. Packaging is Stage 1's.

**Assumptions still open.** A capture on a display with HDR on
looks as Windows' SDR rendering looks, which is documented and not seen here. A desktop with
two displays at different scales, covered by the conversion's unit tests and not by a machine.
Whether a session logoff is refused by the host's exit guard (the NOT VERIFIED note in
`host/src/main.rs`).

**The verdict, component by component.** Every part 8 row was read against the evidence, and
none fired. The GDI freeze: 41 to 50 ms for the whole 5120x1440 screen, exact against the
screen, keep. The Win32 overlay: usable 87 ms after the hotkey at worst, keep. The Tauri and
WebView2 editor: usable 134 ms after the selection at worst, its export pixel-identical to
the live editor mid-typing, keep. The decode route, `image`, `resvg` and WIC: twenty-eight real
files, keep, with AVIF on WIC as decided. The host composer and the Win32 clipboard: every
untouched pixel exact and both destinations took the paste, keep. The cost row: 170 MB in the
tray, nearly all the hidden web view (F59), is the one number that argues, and it argues for
an idle policy, not for another stack. **Recommendation: go to Stage 1 on this stack.** F60 and F62 were closed the same
day; F59, the tray floor, stays open as a number Rotem judges. Recorded in `project-os/Decisions.md`.

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

- [x] S1.1 Capture lifecycle: repeated captures, hide and show, the remembered return
      target, no overwriting of a previous capture, and the empty-state line naming the
      hotkey. Model: Opus 5, documented behaviour wired as documented; plus the content
      security policy and the region scheme's allowed origin, decided here where the
      product path first builds the window.
      **Built 2026-09-14.** Every capture and every opened file is a numbered document, and
      the page keeps each document's notes by that number: a new capture commits the note
      being typed, stashes the previous document's notes, and starts empty (F67 closed). The
      previous capture's pixels are kept by the host, PNG-encoded on their own thread (12 ms
      for a probe, about 3.5 MB for a full screen), capped at fifty until S1.8 moves them to
      disk; an opened file is never retained (§3.8). The return target is read at the hotkey,
      before the overlay, and kept across a capture started from inside Recon; the editor
      hides on close and on Escape when idle, and the focus goes back to that window, or to
      nothing when it has closed (self test G, both cases). The tray gained "Open Recon",
      and the empty state says "Press Ctrl+Shift+4 to capture" with the configured key. The
      content security policy is set (self, the region scheme, the IPC origin, data and blob
      images, inline styles for the export) and the region scheme answers the page's own
      origin only; recorded in `project-os/Decisions.md`. Run on a 225% display, which
      found the offset F74 and the antialiasing figure F75. Three product-path captures on
      the 3840x2160 display at 225%: overlay usable within 88 ms, editor usable within 275 ms,
      each previous capture retained in under 40 ms (part 11). The save half of Escape and of
      a new capture is S1.8's; until then the notes live in the page's memory.
- [x] S1.2 File opening and activation: the file association, Open With, drag and drop and
      `Ctrl+O`, both when Recon is already running and when it starts because a file was
      activated, one window rather than a second instance, plus the type registration and
      the link into the Windows default-apps settings. Model: Fable 5.1, it is the entry point.
      **Built 2026-09-14, closed the same day by Rotem's hand: the registration run, a
      double-click in Explorer opening the image in Recon, a drop replacing it.** Every way in reaches
      the one window: a bare path on the command line, which is what Explorer, "Open with"
      and a file association hand over, opens at startup (29 ms to the editor shown); a
      second instance hands its path to the running one and exits, 49 ms, the running window
      opening it in 32 ms; a file dropped on the window opens, several drop the first; and
      `Ctrl+O` opens Windows' own picker, owned by the editor, filtered to the twelve
      extensions, on a thread of its own. Registration is written as data first
      (`host/src/registration.rs`, tested): Recon in every type's "Open with" list and in
      Windows' Default apps through a Capabilities key, under the current user, and no
      extension's default is ever written or re-taken; `--register-types` writes it,
      `--unregister-types` removes exactly that, and the tray's "Default apps settings"
      opens Windows' own page. What only a hand can do: run the registration on this
      machine, then double-click and "Open with" in Explorer, and drop a file on the window.
      Those three are the asks; the picker is machine-checked to open and to close on Escape.
- [x] S1.3 The viewing surface: fit to window on open, actual size, zoom, pan, fullscreen,
      and the filename and pixel dimensions on screen. Model: Opus 5.
      **Built 2026-09-14.** Fit on open, actual size and the key zoom were S0.4's; this step
      adds the wheel, which pans, and Ctrl with the wheel, which zooms around the pointer so
      the point under it stays put (checked to within a pixel), F11 in and out of fullscreen
      with Escape leaving it before anything else Escape does, and the file's name in the
      window title beside the dimensions the HUD already shows. Zoom changes nothing an
      export contains: the composer reads the image, never the view (S0.6). Dragging to pan
      with the pointer is S1.5's, where viewing mode and annotation mode part ways. Since
      2026-09-14, at Rotem's word, the wheel zooms around the pointer with Ctrl or without,
      and only a sideways wheel pans.
- [x] S1.4 Folder navigation: previous and next across the supported types in the opened
      file's folder, the defined numeric-aware order, the position indicator, and the named
      active context so it can never be confused with Recon history. Model: Opus 5.
      **Built 2026-09-14.** The folder is listed once, when a file is opened from a folder
      that is not the current context, and never on a keystroke; the supported types are
      the twelve extensions of §3.7, by name. The order is Windows' own logical comparison,
      so a.gif, b.jpg, IMG1.png, img2.png, img10.png, checked in a folder built for it. The
      image info carries the position, the count and the context's name, and the HUD says
      "4 of 4 in folder". PageDown and PageUp walk, Home and End jump, the ends stop. A
      file that has vanished since the listing is skipped with a log line and the count
      follows. History is a second list and S1.9's; the context name is what keeps the two
      apart on screen.
- [x] S1.5 The transition into annotation: viewing mode arms no tool and no click can create
      a callout, an explicit Annotate action switches modes, and leaving annotation keeps the
      work. Annotate creates a managed document with its preserved decoded image, or resumes
      the one that already exists for that source, never a second one. Opening a file that
      has a document shows the file with the route to its saved edit (§3.3).
      Model: Fable 5.1, it is the boundary that protects the originals.
      **Built 2026-09-14.** A file opens in viewing: the scene takes no pointer, a click
      selects nothing, a drag pans, and the checks press the same click in both modes to
      see one create nothing and the other create a note. A capture opens in annotation.
      The `A` key switches, and so does a button at the top right that names the other
      mode, both at Rotem's call, with the mode named in the HUD. Annotate on a
      file asks the host for the document: created once for that path and frame, its
      decoded image preserved at once and PNG-encoded on a thread, in the same in-memory
      list the previous captures sit in until S1.8; or resumed when one exists, never a
      second. Leaving annotation keeps the notes and the document. Reopening the file
      shows the file as it is on disk, with the route in the HUD ("annotated before, 2
      min ago; A resumes it"), and A resumes the one document on its own preserved
      pixels: the check overwrites the file with a different picture of a different size
      in between, sees the new file on reopen and the preserved image on resume, with the
      line that the file has moved on. The folder position is the same before, during and
      after. Frame stepping inside an annotated animation is F79; the cap is F80.
- [x] S1.6 Callout completion: the deterministic candidate positions, the neutral margin when
      a bubble cannot fit, and stable numbering with gaps, identical in the editor and in
      every output. Model: Fable 5.1.
      **Built 2026-09-14.** Eight candidates around the anchor in a fixed order, below
      right first, which is where the first version always put a note, the gaps a
      proportion of the text size; the first one inside the composition and clear of the
      other bubbles and their anchors wins, else the first clear one and the margin grows
      to hold it, else the first. The margin is what the notes need over a floor of zero,
      recomputed when a note is created, typed into, committed, resized, removed or dropped
      after a drag, never during one, and it shrinks back when the notes no longer need it;
      the fit view follows when the view was at fit. Numbers were already stable with gaps;
      the check now reads them out of the exported layer's markup. The pan runs over the
      whole composition when it does not fit the window, so a note in the margin is
      reachable at any zoom, which closes F56. Checked on the 640x400 scene: the same
      anchors place the same way twice, four notes near one anchor overlap nothing, each
      edge and the corner send the bubble to the open side, a 120x80 crop takes a 236 by
      37 margin and exports as 356x117 with every source pixel exact, a long note typed
      near the bottom takes 247 below and gives it back when dragged up, a moved bubble
      stays put when another arrives, and the S0.6 references are unchanged. One rule of
      the order is Rotem's to reverse, F81. Recorded in Decisions.
- [x] S1.7 The keyboard contract, routed by context, with the stage column honored, the
      viewing and navigation keys inert while a note is being edited, and undo and redo in
      full against the acceptance sequence in §3.6. Model: Opus 5, the contract is written.
      **Built 2026-09-14, on Fable at Rotem's word to keep running.** Undo and redo are
      snapshots of the notes, one per completed operation: a note committed with its text,
      moved, its anchor moved, resized, removed; an operation that changed nothing, an empty
      bubble discarded at commit for one, adds no step. Ctrl+Z, Ctrl+Y and Ctrl+Shift+Z
      outside typing; while a note is typed the engine's own undo has the key and the
      object history is not reached. The acceptance sequence runs in the checks: edit an
      existing bubble, leave editing, drag it, undo restores the position, undo again
      restores the previous text, redo and Ctrl+Shift+Z bring both back; a deleted note
      comes back on undo with its own number and the next note after a redo is 4. Ctrl+C
      copies the composed image outside typing and is the text copy inside it; Ctrl+Enter
      commits the note being typed, copies, and hides only when the document did not
      change meanwhile. Ctrl+S and Ctrl+Shift+Enter are taken and do nothing, the stage
      column honoured, which also keeps the web view's save dialog away. F11 joined the
      viewing keys that are inert while typing; the checks press Ctrl+Z, Ctrl+C,
      Backspace, Delete, 1, A, F11 and PageDown into a note being typed and none is taken.
      The undo table's own rule in §3.6 is the record; nothing here adds a key it does not
      name.
- [x] S1.8 The document store: the schema from part 5, plus the source path and the page or
      frame for an annotated file, debounced autosave, forced saves on blur, navigation,
      opening another file, hide and quit, reopen after restart, and a visible failure that
      never claims to have saved. Model: Fable 5.1, it touches stored work.
      **Carried from S0.5:** the save-and-restart leg of the animation frame, orientation,
      colour and SVG raster size tests.
      **Built 2026-09-14.** One folder per document under the user's local application
      data, `Recon\documents\<number>`: `source.png`, the preserved image, written once
      and never rewritten, and `document.json` with a schema version, the size, the
      source (a capture, or the file's path and frame with its stamp) and the page's notes
      carried whole. Every write is a temporary file renamed into place. A document's
      number is its creation time in milliseconds, so numbers read back from disk never
      collide with new ones. A capture is a document from its first moment: the record
      at once, the image from a thread. The page saves its notes debounced while typing
      and at once on blur, before navigation, before another document, before a hide and
      when the host asks at close or quit; the HUD says saved only after the host did,
      and a failed save says NOT SAVED with the reason and keeps the work until the next
      save lands. At startup the store is read and the latest document reopens, hidden.
      Checked: the folder and both files after a capture, no temporary file left, the
      typed note on disk on its own, the hide carrying the change first, the image's
      bytes unchanged after every save, the reload with the notes from disk and the page
      holding none, a failing store shown and recovered from. The S0.5 leg: the GIF's
      third frame, the JPEG upright at 200x300, the profiled PNG converted and the SVG at
      its raster size are each, after the reload, byte for byte the frame decoded afresh
      from the file. A real process restart was Rotem's hand on 2026-09-14: a capture,
      Annotate, a note, Copy and Return, Save As, then Recon restarted and opened from the
      tray with the document there. Frame stepping inside a document stays F79.
- [x] S1.9 History navigation: previous and next through documents, captures and annotated
      files alike, the position indicator, shortcuts to the first and last, and the context
      activation rules in §3.4. Model: Opus 5.
      **Built 2026-09-14, on Fable at Rotem's word to keep running.** Recon history is the
      documents in creation order, oldest first, captures and annotated files alike; the
      same PageUp, PageDown, Home and End walk it, the ends stop, and the HUD says "2 of
      5 in history" for a capture as it says "3 of 4 in folder" for a file. The four
      activation rules of §3.4 are one flag in the host: a capture sets history, a
      document shown from history sets history, opening a file sets the folder, and
      Annotate touches nothing, so the checks see the folder stay active at the same
      position after Annotate, and a capture after that at 5 of 5 with the annotated file
      at 4. A document walked to from history stands on its own preserved image, named
      after its file, with no file open behind it (F82).
- [x] S1.10 Copy and Return, including every failure path: a failed clipboard, a failed save,
      a refused activation, a closed target application, and a user who moved on
      mid-operation. Model: Fable 5.1, every branch is a way to lose work.
      **Built 2026-09-14.** The order is fixed: the note being typed is committed, the
      image is copied, the notes are saved, and only then does the editor hide; the hide
      reports what the focus return did. Every way out is on screen, as a line under the
      HUD, never a dialog, and the work stays: a clipboard another application holds
      gives NOT COPIED with the reason and nothing hides; a store that refuses the save
      gives "copied, but NOT SAVED" and the editor stays; a change made while the copy
      ran gives "copied; the editor stays, since you moved on"; a target that has closed
      hides with nothing activated; no target hides with nothing to return to; a target
      that is there gets the focus, or the system refuses and the line says which. The
      idle Escape follows the same rule: a failing save keeps the editor and says so.
      Checked with the clipboard held by a second process of our own through a window,
      since an open with no window does not keep another process out on this machine;
      the store pointed at a file; a note moved during the copy; a stand-in application
      opened and closed as the target. The return with the stand-in there came back as
      "focus returned to the application the capture began in". A change that grows the margin during the copy makes the copy itself
      fail, since the composition moved under it, and that path reads NOT COPIED, F83.
- [x] S1.11 PNG Save As: a new file every time, a suggested name derived from the source and
      marked as annotated, no path by which an external original is the default target, and
      an available name offered when the chosen one exists. There is no overwrite path.
      Model: Fable 5.1, it writes files next to originals.
      **Built 2026-09-14.** Ctrl+S takes the composition as it is, as the copy does, and
      opens Windows' own Save As on its own thread, on the folder last exported to (the
      user's Pictures folder until then) with the name filled in: the source file's stem
      marked "annotated", or "capture <date> <time> annotated" for a capture, never the
      file's own name. The dialog's own overwrite prompt is off, because the question it
      asks is never asked here: a chosen name that exists is not written, and the dialog
      comes back with the first free "name (n)" filled in, until a free one is chosen or
      the user cancels. The file is created new, never truncated, and a write that fails
      midway is removed. The outcome is a line under the HUD: saved, nothing saved, NOT
      SAVED with the reason, or which name was offered. Checked: the suggested names for
      a file and a capture, the write of a new file, the same name refused with "(2)"
      offered and the first file's bytes untouched, the offered name writing, the next
      plan opening on the folder just exported to, Ctrl+S opening the dialog; Escape
      closing it is sent only when our window is in front. On the way, the HUD's notice
      became state the HUD draws, so the autosave's redraw no longer wipes it.
- [ ] S1.12 The trial: thirty captures of real client feedback and a week of using Recon as
      the everyday viewer, with the friction recorded in `project-os/History.md` and anything
      deferred sent to `project-os/Backlog.md`. Model: Opus 5, it records.
      **Started 2026-09-14, and stopped at once:** Rotem's first day said the environment is
      not ready for daily use without three things, which are Stage 2's first steps pulled
      forward, below. The trial resumes when they are in.

**Stage 2, pulled forward.** The three things daily use could not start without, at
Rotem's word on 2026-09-14. The rest of Stage 2 stays in part 10.

- [x] S2.1 The timeline: a strip of thumbnails along the bottom of the window, every
      document, captures and annotated files alike, newest last, the current one marked,
      a click showing that document with its notes, the keys walking it as they walk
      history, the strip out of the way in fullscreen. Thumbnails are made once per
      document on a thread, kept beside the document, and never the full image, so old
      history costs no memory (part 10's acceptance). Model: Fable 5.1, it is a design.
      **Built 2026-09-14.** A 96-pixel strip along the bottom, shown whenever there is a
      document, the stage ending above it; a thumbnail per document at most 160 by 100,
      the shape of its picture, oldest first so the newest is at the right, the current
      one outlined and scrolled into view, the file's name or "capture" with the time on
      hover. The host makes each thumbnail once, on a thread of its own per request,
      from the document's own image, and keeps it as `thumb.png` beside the document;
      the strip fetches it through the region scheme. Since 2026-09-16, at Rotem's word,
      the thumbnail carries the notes: after every save of them the page draws its layer at
      the thumbnail's scale and the host writes `thumb.png` anew with it composed over the
      picture, margin and all, and the strip's cell takes the new picture. A click shows that document with
      its notes and history active; the keys walk the same list. Fullscreen puts the
      strip away and the stage takes the whole window. Checked: three captures listed
      oldest first with the third current, three thumbnails at 133x100, 75x100 and
      160x50, kept on disk, a click on the first showing it at 1 of 3 with its note, the
      mark following, fullscreen and back. The look is provisional and Rotem's to change.
- [x] S2.2 The icon opens the window: a click on the tray icon shows the editor with the
      latest document and the timeline; the menu stays on the right button; starting
      Recon again, from the Start menu or the taskbar, does the same through the single
      instance. Model: Opus 5, wired as documented.
      **And the first start too, at Rotem's word on 2026-09-14:** launching Recon opens its
      window like any application, with the latest document and the timeline, and the
      window sits in the taskbar while it is shown. The tray stays as it was: closing the
      window hides it and Recon stays alive there, and only the tray's Quit ends it.
      **Built 2026-09-14, on Fable at Rotem's word.** The left button on the tray icon
      shows the editor as it is, the latest document reopened at startup and the timeline
      under it; the menu is the right button's. A second start with no file was already
      showing the editor since S1.2. The click itself is Rotem's to try; nothing here can
      press a tray icon.
      **Since 2026-09-15, at Rotem's ask:** a start by Windows at logon, through the tray's
      "Start with Windows" tick (§3.1), shows no window; Recon waits in the tray.
- [x] S2.3 Copy and Save As on an annotated image: what Rotem met on the first day; the
      cause decides the step, a fix if the keys failed on an annotated file, visible
      controls if the keys were not the way in. Model: Fable 5.1 once the cause is known.
      **Built 2026-09-14, the cause found on the way.** The window Rotem was testing was
      the release build from midnight, before annotation, the store, Copy and Return and
      Save As existed, so the keys were not in it. Both were made buttons as well, Copy
      and Save As beside the mode at the top right, since that is how Annotate was
      settled and a hand reaches a button before it finds a key; each does exactly what
      its key does and gives the keys back at once. Ctrl+C while a note is being typed
      stays the text copy, as §3.6 says; the button copies the image either way. The
      release is rebuilt at the same path once the old window closed. Provisional look,
      Rotem's to change; the keys on the new build are his to try.
- [x] S2.4 Visible storage use (§3.8): the HUD's first line says how many documents the
      store holds and how many megabytes they take, read from the file sizes and never
      from the images, refreshed with the timeline. The daily rate stays stated in §3.8.
      Model: Opus 5, wired as documented; built on Fable at Rotem's word, 2026-09-14.
- [x] S2.5 Intentional deletion (§3.8): one unambiguous delete of a document, its folder
      gone from the store and its thumbnail from the timeline, never the external original
      and never an export, with a way back for a slip. Rotem's calls on 2026-09-14: the
      control on the thumbnail and on a key, both; a deleted document goes to a trash and
      is gone for good after thirty days. Model: Fable 5.1, it deletes.
      **Built 2026-09-14.** A small × on a thumbnail when the pointer is over it, and
      Ctrl+Delete for the document on screen (a provisional key, so plain Delete stays the
      note's). One click, no dialog: the document leaves the list and its folder moves,
      whole, into Recon's own trash beside the documents, stamped with the time; the
      notice says so. The trash is Recon's, not the system's recycle bin, whose emptying
      is not ours to time; at every startup whatever has been in it thirty days is
      removed for good. If the deleted document was on screen the newer neighbour shows,
      else the older, else the editor is empty with the timeline and the controls away,
      and the next capture fills it again. An external file's document deleted leaves the
      file where it was. The way back for a slip is the folder in the trash; a restore
      inside Recon is S2.7. Checked: a thumbnail's delete on a document not on screen,
      Ctrl+Delete on the one on screen showing its neighbour, the last one emptying the
      editor, the file untouched, the sweep removing a folder backdated thirty-one days
      and keeping the others, a capture after the empty state.
- [x] S2.7 Restore from the trash: a way inside Recon to bring a trashed document back
      within its thirty days. Model: Opus 5, the folder moves back the way it went.
      **Built 2026-09-14, on Fable at Rotem's word.** When the trash holds anything, the
      timeline ends with a "Trash · n" chip; it opens the trash in the same strip, each
      deleted document as its thumbnail with how many days ago and a Restore button, and a
      Back chip returns. Restore moves the folder back the way it went, the document
      rejoins the list and shows with its notes. The strip stays for the chip even when no
      document is left. Checked: no chip with an empty trash, the chip and its count after
      a delete, the trash view with the thumbnail, Restore bringing the document back on
      screen with its note, into the list and out of the trash. Provisional look, Rotem's
      to change.

- [ ] S2.8 The timeline at scale, and a strip that grows: how many thumbnails the strip
      holds at once, and a drag on its top edge that enlarges them. Asked by Rotem on
      2026-09-14; a spec, nothing built. Model: Fable 5.1, it is a design and it changes
      what is kept on disk.
      **What is true today.** The strip lists every document and, the first time it is
      shown, asks the host for every thumbnail at once, one thread each; every thumbnail
      fetched is then held by the page for the life of the window, as its bytes and as an
      image element. The host holds nothing per document, since a stored document's image
      stays on disk until it is shown (S1.8). So the strip is the one place where memory
      grows with the library, which §3.8 forbids, and the startup burst grows with it. At
      the rate §3.8 states, a few hundred documents is one to two weeks of use, and a few
      hundred is where a startup begins to stutter; nothing is heavy at today's count.
      **No View More button.** A button pages a list read top to bottom; the strip is
      scrolled, and the keys walk the same list as history (S1.9), so a hidden tail would
      be one the keys reach and the eye cannot. The load is solved out of sight instead:
      the strip keeps a box for every document, sized from the width and height the host
      already lists, so the scroll length is right from the first frame; only the boxes on
      screen, plus one screen's worth on either side, fetch and hold their picture, and a
      box that scrolls two screens away lets its picture go. At most eight fetches are in
      flight at once. The page then holds a fixed number of pictures whatever the library
      holds, and a startup asks for a screenful, not a library. A View More, or a fold by
      date, earns its place only if Rotem wants old work out of sight on purpose, which is
      a product call and not a load one; the date grouping §3.8 names is the form it would
      take.
      **The strip grows by a drag.** The strip's top edge is a handle, six pixels tall,
      under the vertical resize cursor. Dragging it up makes the strip taller and the
      thumbnails larger, live; dragging it down makes them smaller, down to today's 96
      pixels, which stays the minimum, and a double-click on the edge returns to the
      default. The default was 96 too; at Rotem's word on 2026-09-16 it is 128, which
      makes the thumbnails 112 tall. A thumbnail keeps its picture's shape and grows with the row, from 124
      wide today to 320 wide at most. A strip taller than one row of 320 stops growing them
      and wraps instead: a second row, a third, as many as the height holds, newest first
      in reading order, and the strip then scrolls vertically rather than sideways. The
      strip can take at most 96% of the window's height (60% as first specified, 96% at
      Rotem's word on 2026-09-15), so the picture always keeps the
      rest. The height is remembered by the page across restarts, one number; fullscreen
      puts the strip away as it does and brings it back at that height. The trash view
      uses the same rows. The current document stays scrolled into view through a resize,
      and the picture above refits or keeps its pan exactly as it does when the window is
      resized, through the same path.
      **The thumbnail file grows with it, once.** A thumbnail kept at 160 by 100 is blurry
      at 320, so the host makes `thumb.png` at 320 by 200 at most from then on, and remakes
      a smaller one it finds the first time a larger one is asked for; the file is roughly
      four times today's and still a few percent of the document's own image. The page asks
      for the size it shows, in three fixed steps (160, 240 and 320 wide) so a drag does
      not refetch on every pixel, and the host serves the kept file resampled down to that
      step, so a small strip pays a small picture's memory and a 320 strip pays a 320 one:
      a wall of three 320 rows across a 5120-wide window, margins included, is a few tens
      of megabytes, and a 96-pixel strip is a few. The page's pixel is the screen's pixel
      today (the window is not DPI-scaled, S0.4), so 320 means 320 on the screen; the day
      the window scales, the file is made at 320 times that scale and nothing else moves.
      **Rotem's calls, proposed here and open:** the ceiling, 60% as proposed and 96% at Rotem's word on 2026-09-15; the 96 minimum, which
      keeps the drag one-directional; and whether the drag needs a keyboard twin, which
      QA §6 flags on any pointer-only control.
      **Folded in at Rotem's word on 2026-09-14, for thousands of captures a month:** the
      strip must not build even an empty box per picture, and the list of records should
      be read by month, newest first, rather than whole at startup.
      **Built 2026-09-14, on the three proposed calls as they stand.** The strip is placed
      by arithmetic: every document is a cell of one size, the cells that exist are the
      screen's and one screen's worth on either side, and the scroll length is a block the
      size of the whole list, so there is no element per picture at any count; thirty
      documents in a 1280-wide strip make nineteen elements, and a scroll to the end drops
      the newest and makes the oldest. The top edge is the handle, 96 the floor and the
      default, 216 the one row of 320 by 200, then rows entering as room appears (a snap to whole rows was built first and jumped the handle away from the pointer; gone at Rotem's word on 2026-09-15), 96% of the window the
      ceiling, a double-click the reset, the height remembered by the page. The host makes
      `thumb.png` at 320 by 200 and remakes a 160 by 100 one it finds, once, under a rule
      unit-tested on sizes. Two things differ from the spec above. The kept file is served
      as it is and the page scales it, not resampled to a step: a screenful of 320 by 200
      pictures is a few megabytes, and a fetch burst on every step crossed was not worth
      the saving. And the list is still read whole at startup, because it was measured
      first: thirty thousand documents, a year at thousands a month, scan in 1.9 to 2.2
      seconds on a warm disk (`store::tests::thirty_thousand_documents_scan_in`, run by
      hand), paid once per boot since Recon lives in the tray. Reading by month is a
      lazy document list through every path that walks history, delete and restore, so
      it waits on Rotem's word with the number in hand rather than being built on a guess.
      **The startup read, built 2026-09-15 at Rotem's word ("BUILD").** Not by month, and
      here is why: Annotate on a file must find that file's one document (§3.3), so the
      list has to be whole before that lookup, and a list read by month would have to
      read every month for it. What was asked for is the effect, a startup that costs the
      same at thirty thousand documents as at fifty, and that is what is built: the folder
      names are listed once, the newest fifty records are read before the window shows and
      the last-modified of them reopens, and every older record is read on a thread,
      newest first, in chunks; the page is told with `store-loaded` when the list is whole,
      refreshes the timeline and reads "n of m" again. The one lookup that must see
      everything, Annotate's resume, waits for the thread; the timeline and the history
      keys show what has arrived. Checked: sixty documents, the latest reopened 8 ms after
      the reload with fifty listed, the rest arriving on the thread, the page told, sixty
      cells and "60 of 60". Found on the way, and fixed in the same step: the storage
      figure in the HUD walked every folder and every file on every timeline refresh,
      which at thirty thousand documents is seconds per refresh; it now comes from a
      ledger in the store, one entry per folder, filled as folders are read and corrected
      by the write, move or removal that changed one, and checked byte for byte against a
      walk. One behaviour differs, Rotem's to veto: "the latest reopens" is now the
      last-modified of the fifty newest by creation, so an old document annotated last,
      with more than fifty newer ones, would not be the one reopened.
- [x] S2.9 Undo of a deletion: Ctrl+Z brings back the picture just deleted from the
      timeline. Asked by Rotem on 2026-09-16, his pick over undo buttons in the sidebar
      and over undo that survives a restart; a spec, nothing built. Model: Fable 5.1, it
      moves a document's folder and it changes what undo means.
      **What is true today.** Undo and redo (S1.7) walk one document's notes and shapes,
      a snapshot per completed operation, kept in the page's memory per document until
      Recon quits. A delete (S2.5) moves the document's folder whole into Recon's trash
      and drops the page's copy of its notes and their undo; the way back is the Trash
      chip and its Restore button (S2.7), which always puts the restored document on
      screen. Ctrl+Z after a delete undoes the neighbour's last note operation instead,
      or nothing.
      **The rule: the last thing done is what Ctrl+Z undoes.** Every note operation and
      every deletion takes a number from one counter as it happens. Ctrl+Z compares the
      newest deletion still undoable with the current document's newest note step and
      takes the newer; Ctrl+Shift+Z takes the oldest undone one the same way. So delete,
      then Ctrl+Z, brings the picture back; add a note to one picture, delete another,
      then Ctrl+Z, brings the deleted picture back first, the note still there; two
      deletes and two Ctrl+Z bring both back, the later one first. Rotem's call on
      2026-09-16, "A", over undoing the notes first.
      **What coming back means.** The folder moves back out of the trash the way it went,
      through the S2.7 move, never rewritten, and the document rejoins the list in its
      place by number. If it was on screen when deleted, it shows again; if it was not,
      the picture on screen stays and only the thumbnail returns, which needs a restore
      that rejoins the list without showing, a host change. The page keeps the deleted
      document's notes and their undo history in memory instead of dropping them, so a
      note's undo is where it was after the round trip. The Trash chip's count follows.
      No notice: the picture coming back is the whole message (Rotem, 2026-09-16).
      **Redo deletes it again**, by Ctrl+Shift+Z, into the trash by the same delete path,
      so a Ctrl+Z pressed by mistake is itself undone. Rotem's word on 2026-09-16: no
      Ctrl+Y for it, Ctrl+Shift+Z is the redo.
      **The empty state.** After the last document is deleted the editor is empty and
      the keys today guard on a picture; Ctrl+Z must work there and bring the document
      back on screen.
      **What it does not cover.** A restart: the deletion list lives in the page's
      memory like the notes' undo, so after a quit the Trash chip is the way back, as it
      is now. A document swept from the trash cannot come back, but the sweep runs only
      at startup, so no undoable deletion is ever swept mid-session. A restore refused,
      the folder gone or already back, says "NOT RESTORED: <why>" and leaves the undo
      path, so the next Ctrl+Z reaches what was done before it; the trash view is still
      there to try.
      **Files.** `editor/index.html`: the counter, the deletion list, `undo` and `redo`
      choosing by number, `deleteDocument` keeping the stash, the empty-state key path;
      `host/src/editor.rs`: the restore that rejoins the list without showing, beside
      `editor_trash_restore`; `host/capabilities/default.json` for the new command;
      `editor/editor-checks.js`: a section. Not touched: `host/src/store.rs`, whose
      `trash` and `restore` are the moves and stay as they are; the note undo's
      snapshots; the trash view. The §3.6 undo row gets its clause when this is built.
      **Rotem's calls, 2026-09-16, all three settled above:** the last thing done is what
      Ctrl+Z takes; Ctrl+Shift+Z deletes it again; no notice when it comes back.
      **Checks, in the editor checks.** Delete the document on screen, Ctrl+Z: the same
      document is on screen with its notes, and its note undo still walks. Delete one
      not on screen, Ctrl+Z: the picture on screen unchanged, the thumbnail back in its
      place. Delete the last one, Ctrl+Z from the empty state: it is back. Add a note,
      delete another picture, Ctrl+Z brings the picture back with the note still there,
      Ctrl+Z removes the note, Ctrl+Shift+Z puts the note back, Ctrl+Shift+Z deletes the
      picture again. Two deletes, two Ctrl+Z: both back, the later first. No notice
      after a restore, and the failure line when one is refused. `source.png` and `document.json` hash the same before the
      delete and after the undo, the trash folder is empty after it and the chip gone. A
      restore made to fail shows its line and leaves the undo path. Risk high: it moves
      folders in the store.
      **Built 2026-09-16.** As specified: one counter across note steps and deletions, undo
      taking the newer of the current picture's step and the newest deletion, redo taking the
      most recently undone thing first and passing over a step of a picture no longer on
      screen; the deletion's stash kept, so the notes and their undo come back; the host's
      `editor_trash_rejoin` beside `editor_trash_restore`, the same move without the show,
      and the Restore button now built on the same half. Two things the review of the diff
      caught and the build fixed: a refused restore kept its entry, so every later Ctrl+Z hit
      the same refusal and never reached the notes, and now leaves the undo path; and the
      empty path stashed with a save, which could only fail into a folder already in the
      trash, and now stashes without one. Checked in section 39 of the editor checks, twenty-
      one lines, every scene of the spec: back on screen with its notes and its own undo
      still walking, the image and the record byte for byte unchanged, the thumbnail alone
      back for a picture not on screen, the last thing done first both ways, two deletes and
      two Ctrl+Z, the empty state, a fresh action ending the redo, a refused restore. Not
      looked at by eye in the release; the checks drove the real page.

**Stage 3, the secondary tools, begun at Rotem's word on 2026-09-14** before the trial
said which one daily use wanted first; the plan's own order is taken. Every tool joins the
same selection, undo, save and export the callouts have (part 10's acceptance), lives
beside them and never in their numbering, and is chosen by a key and a button both, as
Annotate and Copy were settled.

- [x] S3.1 The tools and the arrow: a tool in hand, callout by default, chosen by C and L
      or by the buttons at the top right; the arrow drawn by a drag from tail to head, its
      head a polygon rather than a marker so the serialized export renders it the same;
      selected by a click with handles at both ends, moved by a drag, removed by Delete,
      undone and redone, saved with the notes and back after a restart, in the export
      layer as it is on screen. A drag shorter than four pixels makes nothing. Model:
      Fable 5.1, the tool contract is new. **Built 2026-09-14**, checked in section 32 of
      the editor checks; the callout flow's checks all still pass. Since 2026-09-14, at
      Rotem's word, no tool is in hand when a picture opens: the pointer is the ordinary
      one and a click on the picture creates nothing until a tool is chosen. The same
      evening, also his: a drag on the picture with no tool pans it, and Space held gives
      any tool the ordinary pointer and the same pan.
- [x] S3.2 Rectangle and highlight: the same drag, a rectangle outlined in the arrow's
      colour, a highlight as a translucent fill; R and H. Model: Opus 5, the mechanics exist.
      **Built 2026-09-14, on Fable at Rotem's word.** A drag in any direction makes the
      rectangle it crossed; the rectangle is a rounded outline in the arrow's red, the
      highlight a translucent yellow fill; both are selected by a click, moved whole,
      undone, saved and exported as the arrow is. Checked in section 33 of the editor
      checks.
- [x] S3.3 Text: a note without the number, the bubble or the arrow, typed where it is
      clicked; T. Model: Opus 5.
      **Built 2026-09-14, on Fable at Rotem's word.** A text note is a callout in every
      way that matters to editing, typed, resized, moved, selected, undone, saved and
      exported through the same paths, with its bubble transparent, its number gone and no
      anchor or arrow made for it; red with a white halo so it reads on a picture. It
      takes no callout number, so the notes' numbering stays theirs. Checked in section
      34 of the editor checks.
- [x] S3.4 Blur: a region the composer blurs in the source at export, since the page never
      holds the pixels (part 5); the layer carries the region and the host does the work,
      on screen through a blurred region from the region service. Model: Fable 5.1, it
      changes the composer.
      **Built 2026-09-14.** B, then a drag, makes a blur region, a shape like the others:
      selected, moved, deleted, undone, saved. On screen it is a box in the scene that
      blurs what is behind it, which is the canvas, so no pixels cross to the page. In the
      output the page sends the regions as a header beside the layer, and the composer
      blurs a copy of the source inside them, a box blur two passes each way with a radius
      of an eighth of the shorter side between six and forty pixels, the same number the
      screen uses; the document's own pixels are never touched, so exporting twice gives
      the same blur and deleting the blur gives every source pixel back, which the checks
      see. Section 35 of the editor checks.
- [x] S3.5 Ruler, at Rotem's word on 2026-09-16: a tool that marks an area and writes its
      size in pixels, width and height, in a bubble beside the pointer the whole time the
      drag lasts; released, the box and its size stay on the picture and in every copy,
      like the rectangle (his pick, option 1 of two, over a measurement that leaves
      nothing). Hovering it shows a round 16 px × with a 12 px X that deletes it (his
      spec). M and the button after Blur. Model: Opus 5, the shape mechanics exist.
      **Built 2026-09-16.** A box in the scene like the blur's, its size written beside
      the corner the drag ends at, which is where the pointer is, and it stays there;
      selected, moved whole, deleted by Delete or its ×, undone, redone, saved and
      exported with the other shapes; the × is 16 screen pixels at every zoom and never
      in a copy. The look, a 2 px line in the callout line's blue and a dark 14 px label,
      is provisional, in `project-os/Design.md`. Section 43 of the editor checks.
- [x] S3.6 Crop, at Rotem's word on 2026-09-17, over the v1 scope line that kept crop out:
      a crop button in the sidebar, and with it in hand handles on the picture's edges that
      are dragged inward to cut the picture. Model: Fable 5.1, it reaches the output path.
      **Built 2026-09-17.** The crop is one rect in image pixels beside the notes: the
      picture on screen, the size band, every copy and the thumbnail are the crop, the notes
      keep their image coordinates, the host cuts a copy of the source at every output and
      the document's own pixels are never touched, so Ctrl+Z gives every pixel back. Eight
      handles, corners and sides, inward only, a side no smaller than 8 px; a click on the
      picture with the tool in hand makes nothing and a drag pans. K and the button after
      Ruler, the key provisional. Saved with the notes, one step of undo, back after a
      restart. The look is provisional, in `project-os/Design.md`. Section 44 of the editor
      checks.
- [x] S2.6 Export beyond PNG: Save As offering JPEG beside PNG, the same never-overwrite
      path, the quality fixed rather than asked. Model: Opus 5, the path exists.
      **Built 2026-09-14, on Fable at Rotem's word.** The Save As dialog offers JPEG beside
      PNG, PNG first; the name's extension follows the type chosen, and the write encodes
      by the extension: JPEG at quality 90, flattened over white since JPEG has no alpha,
      PNG otherwise, through the same never-overwrite path. Checked: a .jpg name writes a
      JPEG of the composition's size, a .png name a PNG, and the same JPEG name again is
      offered "(2)".

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

**Stage 4, Rogers delivery: in the Backlog at Rotem's word on 2026-09-14, whole.** Nothing of
it is planned or built until he takes it out; §3.9 stays as the record of what it is, and
`Ctrl+Shift+Enter` stays a key that is taken and does nothing.
Inspect Rogers first, then design the submission record against
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

**Every figure dated 2026-09-11 in this part was measured from a debug build**, and the
plan did not say so until the review of 2026-09-13 (F47). The same runs were repeated with
`--release` on 2026-09-13, on this machine's current arrangement, one 5120x1440 display at
100%, and the two are recorded side by side here. A design rests on the release number.

| Figure | Debug | Release |
|---|---|---|
| Freeze of the whole 5120x1440 screen | 143 to 145 ms | 41 to 50 ms |
| Host PNG encode of a 1920x1080 image | 633 ms | 12 ms |
| Editor window created hidden at startup | 223 ms | 216 ms |
| Selection to editor shown, on the product path | 44 ms | 13 ms |
| Capture painted after the page hears of it | 25 ms | 38 ms |
| Fit view of the 3840x2160 probe, after its level exists | 49 to 51 ms | 44 to 51 ms |
| Building pyramid level 1 of that probe, once | 80 to 82 ms | 67 to 74 ms |
| 1:1 region, 1280x800, request to paint | 25 ms | 27 to 32 ms |
| Whole 31.6 MB frame in: message channel, custom protocol, local socket | 290, 290, 140 ms | 194, 192, 47 ms |
| 7.9 MB proxy in | 74, 82, 40 ms | 50, 49, 23 ms |
| 2.2 MB encoded proxy in | 15.5, 15.4, 5.8 ms | 15, 16, 6 ms |
| 3.5 MB folder-walk proxy in | 37, 37, 31 ms | 23, 24, 17 ms |
| 31.6 MB layer out: message channel, local socket | 359, 181 ms | 290, 152 ms |
| 162 KB encoded layer out | 3.5, 3.9 ms | 4.8, 2.6 ms |
| Web view PNG encode of the 4K layer | 78 ms | 81 ms |
| Export spike, 5120x1440 layer: serialize and rasterize in the web view | 3 ms | 3 ms |
| Export spike: web view PNG encode of that layer, 193 KB | 41 ms | 38 ms |
| Export spike: host decode, composite, encode to file | 50, 51, 269 ms | 7, 6, 14 ms |
| S0.6 copy of the 5120x1440 demo: web view PNG encode of the layer, host decode plus compose, PNG encode plus two DIBs, clipboard publish | not run | 38, 18, 51, 22 ms |
| S0.6 copy of an 880x400 document: compose, encode, publish | 10, 32, 1 ms | 0, 1, 0 ms |
| S0.6 check 3, text box on screen against the composite, three cases | 0 pixels differ | 0 pixels differ |
| S0.7 hotkey received to overlay usable, 30 runs at 5120x1440: max, 95th by nearest rank, median | not run | 86.7, 80.5, 74.0 ms |
| S0.7 selection completed to editor usable, 30 runs: max, 95th by nearest rank, median | not run | 134.1, 130.4, 124.7 ms |
| S0.7 process start to ready, and to the editor page booted | not run | 255 to 328, 276 to 374 ms |
| S0.7 memory floor, tray and hidden editor, private bytes, host plus web view | not run | 168 to 172 MB (host 6 to 7) |
| S0.7 memory after 30 captures, and after a 28-file walk | not run | 220 MB (35 MB the live document), 200 MB |
| S0.8 display colour: HDR state, bits, SDR white, on the active display | HDR supported and off, 8 bits RGB, 80 nits | same |
| S0.8 display gamut from EDID: main display, second display | 132% of sRGB (wide), 96% of sRGB | same |
| S0.8 focus after Escape: to the window in front; with it gone | restored; cancelled cleanly, Windows chose | same |
| S1.1 on the 3840x2160 display at 225%, three product-path captures: hotkey to overlay usable, selection to editor usable | not run | 67 to 88 ms; 266 to 275 ms |
| S1.1 the previous capture retained, PNG-encoded on its own thread, 3840x2160 | not run | 34 to 37 ms, 4.3 MB |
| Open an 8000x6000 PNG (192 MB on disk): read and decode, then hand over the frame | 326, 25 ms | 100, 26 ms |
| That file's first fit view, waiting on pyramid level 1 (4000x3000) | 640 ms (514 for the level) | 575 ms (442 for the level) |
| Open a 120x90 GIF, three frames, and show it | 3 ms to shown | 3 ms to shown |
| First SVG open, which loads the system fonts once | 165 ms | 17 ms |
| Every other fixture: open and first frame | 0 to 8 ms | 0 to 8 ms |
| Real HEIC, 1440x960 and 1280x854, through WIC: open, then frame 0 | 13 to 51 ms, 8 to 26 ms | not re-run |
| Real AVIF, 800x533 to 3840x2160, through WIC: open, then frame 0 | 6 to 128 ms, 3 to 23 ms | not re-run |
| Real 4608x1976 phone JPEG: open, then frame 0 | 43 ms, 4 ms | not re-run |
| Real animated WebP, 400x400, 12 frames: open | 27 ms | not re-run |
| Real 35-page TIFF, 439x167: open, then a page | 3 ms, 0 ms | not re-run |

The fit view's debug number is the same as its release number because the resample now
runs in `host/pixels`, which is optimised in every profile; the 1.1 s recorded below was the
generic resample compiled unoptimised in the host crate, not the algorithm.

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
is resampled. A fit-to-window region of the 3840x2160 probe cost **1.1 to 1.15 s** in that
build, which was F43; S0.4b brought it to 47 ms with a pyramid and an optimised pixel crate,
and the table above carries both.

**A caution about every number above:** this machine's display arrangement changed three
times during Stage 0, between 5120x1440 and 3840x2160 at 225%. Every measurement therefore
carries the display it was taken on, and a figure quoted without one means nothing here.

**And the window is not DPI-scaled at all**, which is F44. The monitor's effective DPI is
216, 225%, but the editor window reports 96, its scale factor is 1, its physical and CSS
sizes are both 1280x800, and a band just outside that rectangle is not part of the window.
So the image path is exactly right, one image pixel to one physical pixel at actual size,
and the interface around it renders at a third of the size the display asks for.

**One number that looked like a problem and was the build profile:** the host's own PNG
encoder took **633 ms** to encode a 1920x1080 image in debug, and takes **12 ms** in
release. F41 is closed on that, and the encoder stays as it is.

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
origin-clean across web view updates (it is, on 152.0.4191.66, measured 2026-09-13); the four decode dependencies and their security watch;
the web view's own update cadence, which makes every reference image dependent on a recorded
engine version; and any reliance on a browser flag, which the vendor does not support for
production use and which therefore cannot be part of a gate.

**Not inspected:** Copy Ninja and Rogers. Both are Rotem's to point at, in a task that names
them.

---

# Part 12: the review trail

Seventy-eight findings were raised against the plan and folded into the parts above. This table
is the record; the fixes themselves live where the table points. Severity is how the finding
was rated when it was raised.

| # | Finding | Sev | Where it lives now | Status |
|---|---|---|---|---|
| F1 | Nothing guaranteed the exported image matched the editor | 🔴 | Part 5 boundaries, S0.6 three checks | Verified at S0.6: the live text box and the output are pixel-identical mid-typing |
| F2 | The elevated-window claim was an assumption presented as documentation | 🟠 | S0.8, part 11 | Claim withdrawn, behavior to verify |
| F3 | Two latency targets, and the timestamps could not produce one of them | 🔴 | S0.7 six marks, two intervals | Resolved in plan, verification pending |
| F4 | Clipboard formats unspecified, so a passing paste test proved little | 🟠 | Part 6a destinations, S0.6 | Accepted at S0.6: PNG, CF_DIBV5 and CF_DIB published, both destinations took the paste |
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
| F31 | The exact-equality check used an opaque capture, so it could not fail on the only input that can | 🔴 | S0.6 check 1 | Verified at S0.6 on a PNG whose alpha runs 0 to 255 |
| F32 | The stack was justified by familiarity with another project, which rule 21 forbids as evidence | 🟠 | Part 5, `project-os/Mistakes.md` | Resolved: argued from this product's own requirements |
| F33 | The replacement argument overstated what the alternatives cannot do: Qt has an editing model, native UI has automation peers, resvg has a C interface, Qt SVG has extensions since 6.7 | 🟠 | Part 5, its not-established section and comparison table | Resolved: downgraded to a recommended candidate with the overstatements named |
| F34 | A browser flag the vendor does not support for production sat inside a readiness gate | 🟠 | S0.4, part 11 watch items | Resolved: the gate runs in the shipping configuration |
| F35 | A fit-to-window display proxy cannot serve 100% or 200% zoom, so the everyday viewer would show mush | 🔴 | §3.4, part 5 display policy, S0.4 detail check | Resolved: detail follows the zoom, transport decided by S0.3 |
| F36 | The stated direction override did not exist: `unicode-bidi: plaintext` ignores `direction` and resolves per paragraph, not per bubble | 🔴 | §3.5, part 5 text layer | Resolved: three explicit modes, resolved value stored |
| F37 | The annotation export was sized to the source, so a callout inside the added margin would be cut off | 🔴 | §3.6, part 5 boundary rule, S0.6 margin case | Verified at S0.6: a note inside a 240-pixel margin is in an 880x400 output |
| F38 | Revision leftovers: two decode providers in two places, a web view decoder question, F18 still marked open, and dav1d described as a whole-file library | 🟠 | Part 5, S0.5, part 11, this table | Resolved in place |
| F39 | Readiness ended at input acceptance, so a blank or stale window could score as ready | 🟠 | S0.7 | Resolved: content and input, on one timing basis |
| F40 | The boundary figure the design was argued against was two to four times pessimistic, and it came from a discussion thread | 🟡 | Part 11's measured table, S0.3 | Resolved: measured here, and the plan quotes the measurement |
| F41 | The host's PNG encoder takes 633 ms for a 1920x1080 image at default settings, and it sits on the Save As path | 🟠 | Part 11, S1.11 | Resolved: 12 ms in a release build. The 633 was the debug profile |
| F42 | The plan said the annotation layer crosses back, without saying it crosses encoded, which is two orders of magnitude cheaper | 🟡 | Part 5 boundary rule, S0.3 | Resolved: encoded, and the numbers are in part 11 |
| F43 | The fit-to-window view costs 1.1 s, because it resamples the whole image, and it is the first thing anyone sees when opening one | 🟠 | Part 11, S0.4b, and S1.3's viewing surface | Resolved: 47 ms, from a pyramid of halves and a pixel crate optimised in every profile |
| F44 | The editor window is not DPI-scaled: the monitor is at 225% and the window reports 96 dpi, so the interface renders at a third of the size the display asks for | 🟠 | Part 11, S0.4's closing note | Open: the image path is correct, the interface is not, and the cause is not yet established |
| F45 | An image smaller than the window sat in its top-left corner instead of the middle, which twenty-six passing assertions did not notice | 🟠 | S0.4, the editor's own paint path | Resolved: centred, and the callout layer carries the same offset |
| F46 | A note is unreadable at fit zoom on a wide capture, because annotations live in image space and scale with the image | 🟡 | §3.5, S0.4, S1.6 | Resolved: no on-screen floor. The default note text is 20 image pixels and the size is the user's, which is Rotem's call |
| F47 | Every figure in part 11 came from a debug build, and the plan did not say so, while decisions were being drawn from them | 🟠 | Part 11, S0.4b | Resolved: part 11 carries debug and release side by side |
| F48 | S0.5 could not close as written: three of its gates need the export and the store, which are later steps | 🟠 | S0.5, S0.6, S1.8 | Resolved: the legs are carried gates in S0.6 and S1.8, and the export spike runs first |
| F49 | The four Stage 0 pieces had never run as one thing, so the second latency interval had nothing to measure | 🟠 | S0.4b | Resolved: hotkey to editor on the product path, 13 ms from selection to shown |
| F50 | The check-only commands let the page ask the host to capture the screen and write files, in the runtime the product editor will use | 🟠 | Part 5 boundaries, S0.4b | Resolved: behind the stage0-checks feature, and the page's permissions are in one capability file |
| F51 | The AVIF chain the plan named, libavif with dav1d, cannot be built on this machine without a C toolchain, and the pure-Rust AV1 ports do not compile on this Rust; AVIF decodes through WIC and so depends on an installed codec, which part 6c does not allow for a required format | 🟠 | §3.7, part 6a, part 6c, S0.5 | Decided 2026-09-13 at Rotem's delegation: the Windows imaging stack, with the named-codec message as the degraded state, like HEIC; part 6c says so. Revisit if a pure-Rust AV1 decoder compiles on this toolchain, or daily use (S1.12) meets an AVIF this route cannot open |
| F52 | The first fit view of a 48-megapixel image waits 442 ms for pyramid level 1 in release | 🟡 | Part 11, S0.5, S1.3 | Open: a 2x2 box average would build the level in a fraction of the time; revisit if S1.12 shows files that large in daily use |
| F53 | An animated AVIF through WIC is one frame: the frame-by-index rule in §3.5 is unmet for AVIF sequences on this route | 🟠 | §3.7, S0.5, F51 | Accepted 2026-09-13 with F51: first frame only, stated in §3.7 and part 6c; a buildable AV1 decoder would close it, on F51's revisit triggers |
| F54 | The image crate does not read EXIF orientation out of a WebP, so a tagged WebP comes out as stored | 🟡 | §3.7, S0.5 | Open: §3.7 promises orientation for JPEG and HEIC only; recorded, not scheduled |
| F55 | WIC's converter does not apply an embedded colour profile, so a tagged TIFF, HEIC or AVIF would have been shown and exported as if sRGB | 🟠 | §3.7 contract, S0.5 | Resolved: the frame's colour context is read and converted through the same sRGB step as the other providers |
| F56 | Panning is clamped to the image, so the far edge of a margin cannot be reached at a zoom above fit | 🟡 | S0.6, S1.6 | Closed at S1.6: the pan runs over the composition when it does not fit the window, and the check reaches the margin's far edge at zoom 8 |
| F57 | The product build warns that three fields of the decode notes are read only by the feature-gated report | 🟡 | S0.5, `host/src/source/mod.rs` | Resolved in the review of 2026-09-13 (R5): the fields and one platform function carry a cfg attribute for the product build |
| F58 | S0.7 says memory growth with the library "is a failure of R11", and no part of the plan defines R11 | 🟡 | S0.7, §3.8 | Open: read at S0.7 as §3.8's rule that memory must not grow with the size of the library or a folder, which is what was measured; the label needs a home or a rewrite |
| F59 | The tray process's memory floor is about 170 MB, and 162 to 165 MB of it is the hidden editor's web view | 🟡 | S0.7, part 5, part 8 | Open: judged on its own, as S0.7 asks; part 8 has no row that fires on it. A web view destroyed when idle and recreated on the hotkey would lower it and cost the editor interval its warm start |
| F60 | The main display is wide gamut (132% of sRGB by its EDID primaries), so the sRGB decision's own revisit trigger has fired: a capture taken there is treated as sRGB when it is not | 🟠 | §3.7, sRGB decision, S0.8 | Decided 2026-09-13 at Rotem's delegation: sRGB stays for v1, because the captured numbers are what the application drew and a display tag would oversaturate them everywhere else; the revisit trigger is now a colour-managed desktop (automatic colour management or HDR on), which the product detects. Recorded in Decisions |
| F61 | HDR is detected and logged, and nothing tells the user that a capture on an HDR display is the SDR rendering | 🟡 | S0.8, S1 | Open: a one-line notice in the editor when the frozen display had HDR on; Stage 1 |
| F62 | The hotkey against an elevated application in the foreground is not measured, and cannot be by a synthesized key | 🟠 | S0.8, S0.1 | Measured 2026-09-13: Rotem pressed the key with Task Manager, an elevated window, in front; the release product logged "HOTKEY FIRED", the overlay came up, and a selection reached the editor. The hotkey works against an elevated foreground on this machine. Closed |
| F63 | Clicking from a note being typed straight into another note lost the first note's text: the switch never committed it, and the layout rewrote its element from the stale model | 🔴 | §3.5, S0.4, S0.6 | Fixed in the review of 2026-09-13 (R1): a switch commits the note being left; editor check 9 covers it |
| F64 | A superseded region answer, the host's 409 by design, was raised by the page as an error on every held key | 🟠 | Part 5 display policy, S0.4b | Fixed (R2): a 409 returns quietly |
| F65 | Ctrl+Shift+C matched the key's character, so under a Hebrew layout the copy key was dead | 🟠 | §3.6, S0.6, S1.7 | Fixed (R3): matched by physical key; the rest of the contract's layout check is S1.7's |
| F66 | An opened still was held twice in the host, a full clone kept for a second ask nothing makes | 🟠 | §3.8, S0.5, S0.7 | Fixed (R4): the provider keeps the file's bytes and decodes again if asked twice |
| F67 | A capture arriving while a note is being typed discards that note with the previous capture | 🟡 | S1.1 | Resolved at S1.1: a new document commits the note being typed and stashes the previous document's notes by its number before it loads |
| F68 | The margin colour is written in two places, the host's composer and the page's mat | 🟡 | S0.6, Backlog | Open: hand the colour to the page with the image info; a backlog item |
| F69 | An overlay panic would leak its windows and GDI objects, replaced without teardown on the next capture | 🟡 | S0.2 | Open: tear down a leftover state at the start of a selection; has not happened |
| F70 | A poisoned region mailbox would stop every later view from painting | 🟡 | S0.4b | Open: recover from the poisoned lock and answer with an error; nothing in the worker panics today |
| F71 | The SVG raster cap of 16384 a side allows a one-gigabyte pixmap from a hostile viewBox | 🟡 | §3.7, S0.5 | Open: cap the area, or refuse the open with a message |
| F72 | The self test called any still difference after a drag a leaked overlay, and a window repainting as it took the foreground back failed it while the product was right | 🟡 | S0.2 self test | Fixed in the review of 2026-09-13 (R11): the leak check asks whether an overlay window is still alive; the crop predates the overlay by construction |
| F73 | S0.8's HDR query ran on the capture thread at every freeze, inside the hotkey-to-overlay interval, about 5 ms on the median | 🟡 | S0.7, S0.8 | Fixed in the review of 2026-09-13 (R12): the query runs on its own thread |
| F74 | On a scaled display the canvas and the notes sat at a whole CSS offset, a fraction of a physical pixel, so the screen was resampled and 12.5% of a note's pixels differed from the copy | 🟠 | Part 5, S0.6 check 3, S1.1 | Fixed at S1.1: the offset lands on a physical pixel |
| F75 | On a scaled display the screen draws a note through a fractional transform and 6.6% of its pixels antialias differently from the copy, with the wrap and the box identical | 🟡 | S0.6 check 3, S1.1 | Recorded: the antialiasing clause; the live check's bar is 8% on a scaled display, 3% at 100% |
| F76 | The content security policy refuses the S0.3 bench's local socket route, so `--bench` needs the policy off to run its third column | 🟡 | S0.3, S1.1 | Open: a closed step's tool; run it with the policy set to null if it is ever needed again |
| F77 | Registration runs only by the user's hand: the code is tested as data, and whether Explorer's double-click and "Open with" reach Recon on this machine is unverified until Rotem runs it | 🟠 | S1.2, §3.1 | Closed 2026-09-14: Rotem ran the registration, double-clicked an image in Explorer and it opened in Recon, then dropped another on the window and it replaced the first |
| F78 | Several files dropped at once open the first only; the folder they came from is S1.4's navigation context | 🟡 | S1.2, S1.4 | Open: S1.4 |
| F79 | A managed document is keyed by path and frame, but stepping frames inside it keeps the document's number while its preserved image is the one frame, so a note placed on frame 3 of a resumed frame-0 document sits on pixels the document does not hold | 🟠 | §3.8, S1.5, S1.8 | Backlog, Rotem's call on 2026-09-14 |
| F80 | The in-memory list of documents is capped at fifty, and an annotated file's document dropped at the cap loses its notes with no store to fall back on | 🟠 | S1.1 decision, S1.5, S1.8 | Closed at S1.8: the cap is gone, every document is on disk from its first moment, and the in-memory list holds paths |
| F82 | A document read from disk stands without a file behind it, so its frames or pages cannot be stepped and its folder context is none until the file is opened again | 🟡 | §3.3, S1.8, S1.9 | Backlog, Rotem's call on 2026-09-14; the document names its file, and Ctrl+O or the folder walk reaches the file itself |
| F83 | A note added while Copy and Return is composing can grow the margin, and the host then refuses the layer as the wrong size, so that "moved on" reads as NOT COPIED rather than as copied and staying | 🟡 | §3.3, S1.10 | Backlog, Rotem's call on 2026-09-14; today the notice is right that nothing was copied, and a second Ctrl+Enter copies |
| F81 | When no candidate is both inside the picture and clear of the other bubbles, placement takes a clear one in the margin over an overlapping one inside, so a crowded corner grows the canvas rather than stacking bubbles | 🟡 | §3.5, S1.6 | Backlog, Rotem's call on 2026-09-14; one line in `placeCallout` when it comes |

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
- [AOM AV1-AVIF test files](https://github.com/AOMediaCodec/av1-avif/tree/master/testFiles) · the AVIF samples S0.5 ran on: Link-U fox, Microsoft kids, bbb alpha and Ronda rotate90, Netflix image sequences.
- [Nokia HEIF samples](https://github.com/nokiatech/heif/tree/gh-pages/content/images) and [libheif examples](https://github.com/strukturag/libheif/tree/master/examples) · the HEIC samples.
- [EXIF orientation examples](https://github.com/recurser/exif-orientation-examples), [EXIF samples](https://github.com/ianare/exif-samples) and [Pillow's test images](https://github.com/python-pillow/Pillow/tree/main/Tests/images) · the rotated and profile-tagged JPEGs, the tagged WebP and TIFF.
- [Example TIFFs](https://github.com/tlnagy/exampletiffs) · the multipage TIFFs.
- [Google WebP gallery](https://developers.google.com/speed/webp/gallery) and [an animated WebP](https://mathiasbynens.be/demo/animated-webp) · the WebP samples.
- [An Illustrator export](https://github.com/rogerpence/using-svgs-in-css), [a Figma export with an alpha mask](https://github.com/dnfield/flutter_svg/issues/988) and [one with a mask and a filter](https://github.com/gregberge/svgr/issues/336) · the real SVG exports compared against the web view.
