# Recon — Decisions

Why non-obvious choices were made.

`project-os/History.md` records **what** changed. This file records **why** a direction was
chosen, so nobody re-argues it in six months and nobody quietly undoes it.

## How you maintain this file

- Add an entry when a choice was non-obvious and a reasonable person would have
  picked differently. Routine work needs no entry.
- **Append only.** Never rewrite or delete a past entry, even a wrong one. The
  wrong ones are the record of what this project already tried.
- **A changed decision is superseded, not edited.** Write a new entry naming the
  one it replaces, and italicize the old line in the Index so nobody follows a
  rule that has moved.
- **A fully replaced entry may move to an archive.** When superseded entries pile
  up, create `Decisions-archive.md` beside this file — the first time you need
  it, not before — and move the entry verbatim: never rewritten, never
  summarized. Its Index line stays here, marked superseded, so the trail
  survives.
- **This file also rotates, the way History does.** `project-os/rotate.ps1`
  keeps the newest 25 entries live at `Go commit` and moves older ones into the
  same `Decisions-archive.md`, under its own `## Archived decisions` heading.
  Those entries still BIND the project; they only aged out of the live read, so
  treat one exactly as if it were still here. The Index keeps its line for every
  one of them, so nothing becomes invisible. Two kinds of entry therefore share
  that archive — superseded (dead) and rotated (alive) — and its header says so.
- Every new entry also gets a line in the Index, in the same change. The Index is
  the part people read; an entry missing from it is an entry nobody opens.
- Use the required format below. All four parts, every time — an entry without
  Consequences is a note, not a decision.
- Write it so a stranger can follow it without the conversation that produced it.

## Required format

```md
## YYYY-MM-DD · Decision title

### Context
What problem or constraint forced a choice.

### Options
1. Option one.
2. Option two.
3. Option three.

### Decision
What was chosen, and by whom.

### Consequences
What this enables, what it costs, what future work must not break, and what
would make it worth revisiting.
```

## Index

_Older entries archived -> see `Decisions-archive.md` (moved by project-os/rotate.ps1, not rewritten)._

Every decision below, oldest first. Read this list; open only the entries your
task touches. A line in _italics_ means part of that entry no longer holds.

- 2026-09-10 · Callout numbers never change, gaps included.
- 2026-09-10 · The capture API minimum is not the product support baseline.
- 2026-09-10 · Image viewing is a core capability, not a later addition.
- 2026-09-10 · One document per source, resumed rather than duplicated.
- 2026-09-10 · Save As never overwrites anything.
- 2026-09-10 · An SVG raster size is fixed when annotation begins.
- 2026-09-10 · sRGB is the working color space for v1.
- 2026-09-10 · _The minimum format set for the daily-use release._ (membership stands, its stated reason is superseded)
- 2026-09-10 · _The stack: a Rust host owning the pixels, a web view laying out the text._ (recorded as decided; superseded below, it is a candidate)
- 2026-09-10 · One native decode route, and the original never enters the web view.
- 2026-09-10 · The annotation text layer is DOM, and the export is that same DOM.
- 2026-09-10 · _Why the required format set is affordable, superseding one paragraph._ (the libavif clause is superseded below; the membership stands)
- 2026-09-10 · _The stack is a recommended candidate, not a settled decision._ (superseded by the Stage 0 verdict below)
- 2026-09-11 · A note's text size is the user's, with no minimum on-screen size.
- 2026-09-13 · The display proxy is served from a pyramid of halves, by one worker.
- 2026-09-13 · AVIF decodes through the Windows imaging stack, not a bundled libavif.
- 2026-09-13 · The clipboard is published by the host in three formats, PNG first.
- 2026-09-13 · Stage 0 verdict: the candidate stack is adopted, every component kept.
- 2026-09-13 · A capture stays sRGB on a wide-gamut display; the trigger moves to a managed desktop.
- 2026-09-14 · The web view's security posture: a strict policy, and the region scheme answers the page's origin only.
- 2026-09-14 · Previous captures are kept in memory, encoded and capped, until the store exists.
- 2026-09-14 · A bubble is placed from a fixed list of candidates, and the margin is what the notes need, recomputed at rest.
- 2026-09-14 · _Previous captures are kept in memory, encoded and capped, until the store exists._ (superseded by the store below)
- 2026-09-14 · The store: one folder per document in local application data, the image once, the record whole, the number its creation time.
- 2026-09-14 · A deleted document goes to Recon's own trash for thirty days, by a thumbnail's × or Ctrl+Delete.
- 2026-09-14 · The window under the pointer is the selection until a drag begins, listed once with the overlay, told from a drag by the system's threshold.
- 2026-09-14 · The selection is the smallest part of the window under the pointer, the way Snagit picks a page.
- 2026-09-14 · The frame's dashes are light on dark, not light on nothing, drawn by hand so they can walk.
- _2026-09-14 · No tool is in hand when a picture opens, and a chosen one stays until the next picture._ The tool staying is superseded on 2026-09-18.
- 2026-09-14 · The wheel zooms around the pointer, with Ctrl or without, and a sideways wheel still pans.
- 2026-09-14 · The window's top bar is Recon's own, not Windows' frame recoloured.
- 2026-09-14 · _The timeline is a window onto the list, with no View More, and its thumbnail file is 320 wide._ (its startup clause is superseded below; the rest stands)
- 2026-09-15 · Start with Windows is one value under the user's Run key, and a logon start shows no window.
- 2026-09-14 · With no tool in hand a drag pans the picture, and Space held pans it whatever tool is in hand.
- 2026-09-15 · A pan drag slides the picture already painted and asks for one region at a time.
- 2026-09-15 · The store is read on a thread at startup, newest first, not by month.
- 2026-09-15 · Under a pan drag lies a small copy of the whole picture, taken whenever the whole picture is painted.
- 2026-09-15 · The design system lives in project-os/Design.md, read before a visible change.
- 2026-09-15 · Zooming out stops at the whole picture as the space is now, and a whole view follows the space.
- 2026-09-16 · The focus return target is used once: a hide with no capture since activates nothing.
- 2026-09-16 · A deleted document is undone by the same Ctrl+Z as a note, the last thing done first.
- 2026-09-16 · Google Sans is bundled with the page, the medium Latin subset, for the image size only.
- 2026-09-16 · A new capture is copied by the page's own copy, after it loads, not by the host at present.
- 2026-09-16 · The timeline's tabs are the page's own file beside the documents, Main implicit, a feed a list of ids.
- 2026-09-16 · Between a callout's two clicks the bubble keeps its automatic offset from the pointer, and a drop gives its number back.
- 2026-09-16 · The ruler is a box in the scene with its size in image pixels beside the corner the drag ended at, and its × is counter-scaled.
- 2026-09-16 · The thumbnail with the notes is composed by the host from a layer the page draws at thumbnail scale, at every save.
- 2026-09-17 · The crop is one rect beside the notes, cut by the host on a copy at every output, and the notes keep their image coordinates.
- 2026-09-17 · An empty bubble gives its number back.
- 2026-09-17 · A restore waits for an image still being encoded.
- 2026-09-17 · The one row's scroller band replaces the 8 px under the thumbnails.
- 2026-09-18 · The capture drag's size bubble is built pixel by pixel and blended on whole, not a GDI round rectangle.
- 2026-09-18 · The magnifier is the frozen slice stretched pixel for pixel, up from the first pointer, the size under it: the picture's reading, not the words'.
- 2026-09-18 · The lit window glides to the next one edge by edge on the frame's own clock, inside one display, and jumps when Windows does not animate.
- 2026-09-18 · Settings changes a global shortcut by trying the new one first; the shortcut that opens Recon is none until picked; the two are let go while one is being pressed.
- 2026-09-18 · A change made in Settings stays outside Ctrl+Z, Rotem's call.
- 2026-09-18 · The Open Recon shortcut, pressed again, minimizes the window only when it is the one in front.
- 2026-09-18 · Save As on an annotated PNG or JPEG opens on the file itself and writes over it, Rotem's exception to rule 11; every other existing name is still never written over.
- 2026-09-18 · A file opened by name sits in the timeline as a pointer to where it lives, in a list of its own beside the tabs, never in the store; Rotem's call over a kept copy.
- 2026-09-18 · The mode follows what is done: a tool picked starts the annotation, a finished element puts the tool down, and the Annotate button and the A key are gone.
- 2026-09-18 · The colour picker's HEX is put on the clipboard by the host from a colour the page names, never a text; the picker starts no annotation.

---

## 2026-09-15 · Under a pan drag lies a small copy of the whole picture, taken whenever the whole picture is painted

### Context

With the drag sliding what is already painted, Rotem found the edges it uncovers empty for
a moment, filling in as each region came back from the host. A region holds only what was
on screen, and on his wide display each one is large, so the empty strip was plain to see.

### Options

1. Ask for a sharp margin around the view, so a slide uncovers pixels already held.
2. Keep a small copy of the whole picture under the sharp one, shown while a drag lasts, so
   an uncovered edge shows the picture softly at once and sharpens when its region lands.
3. Paint only the strips a drag uncovers, as small regions of their own, into a larger
   canvas.

### Decision

Option 2, mine, on Rotem's report. Option 1 multiplies every region on a wide display and
slows each one, and a fast drag still outruns it. Option 3 is a tile system in all but
name. The copy is taken from the canvas itself whenever the whole picture is painted,
which every picture's first view is, and kept at two megapixels at most, so it costs the
host nothing and the page a few megabytes.

### Consequences

A drag never shows an empty edge; for a moment the edge is softer, then sharp. The copy
shows only while a pan drag lasts and hides when an ordinary paint lands, so the view at
rest and every screen comparison are untouched. Cost: a picture shown already zoomed in,
such as another frame of a file stepped at a high zoom, has no fresh copy until its whole
picture is next painted, and its drag shows the old empty edge until then. Revisit if that
case is met in use, or if the softness itself is noticed.

## 2026-09-15 · The design system lives in `project-os/Design.md`, read before a visible change

### Context

Rotem asked for the hover fade of the new sidebar to be saved in a design system file,
`Design.md`, as the token A1. `CLAUDE.md` rule 13 sends a free-standing document with no
home to `notes/`; `project-os/` holds the docs a session reads at pickup.

### Options

1. `notes/Design.md`, by rule 13's letter: a free-standing document.
2. `project-os/Design.md`, beside the calibration files, with a line in the pickup reading
   list that loads it for any visible change.
3. Tokens as comments in `editor/index.html` only.

### Decision

Option 2, mine, Rotem's to veto. A token nobody reads is a value the next session guesses
from a neighbour, which is what the file exists to stop; `notes/` is read once and never
again. Option 3 has no place for a value the page does not use yet.

### Consequences

Every value Rotem states lands in `project-os/Design.md` in the change that first uses it,
and a visible change looks values up there first. Cost: one more file on the pickup list,
loaded only for visible work. Revisit if the file grows past what a session can read at
pickup, when it would split into tokens and components.

---

## 2026-09-15 · Zooming out stops at the whole picture as the space is now, and a whole view follows the space

### Context

Rotem asked that zooming out stop at the picture's original size. A picture opens at the
fit view, capped at actual size: a picture that fits the window opens at its actual size, a
larger one opens whole. Until then the wheel and the - key went down to a quarter of the
fit. The review found that the space a picture has changes after it opens: the window is
maximized or restored, full screen comes and goes, the timeline appears, hides or is
dragged. The page kept the fit measured at opening, did not refresh it when the timeline
appeared, and moved no zoom when the space changed.

### Options

1. The floor is the size the picture opened at; when the space grows, the picture stays that
   size.
2. The floor is the whole picture as the space is now, capped at actual size; when the space
   changes, a picture shown whole, or smaller than whole, takes the new whole view.
3. The floor is actual size for every picture.

### Decision

Option 2, Rotem's call on 2026-09-15 between the first two. Option 1 was built first, and
its review found a zoom that could not be zoomed back out after the window shrank and grew
again. Option 3 puts a large capture's opening view below the floor.

### Consequences

The wheel and the - key stop at the whole picture, read from the stage at each zoom, and
never enlarge a picture that is below it for the moment the page takes to measure a new
space. The kept fit is measured again, and a view at it or below it follows, when the
timeline refreshes, the trash shows, the window resizes, full screen changes, or the margin
changes, so 0, the HUD's "fit" and the zoom-out stop are one number. A zoomed-in view keeps
its zoom, unless the whole picture grows past it while the space changes, when it is a whole
view from then on. A stage with no area, a timeline dragged to the top of a short window,
moves nothing, and zooming out there stops at the fit kept from before. Side effect: a picture that opens just as the timeline first appears fits above it
now, where it used to open with its bottom under the timeline. Revisit if a large capture
should open at actual size.

## 2026-09-16 · A deleted document is undone by the same Ctrl+Z as a note, the last thing done first

### Context

Undo (S1.7) walked one picture's notes; a picture deleted from the timeline could only come
back through the Trash chip. Rotem asked for Ctrl+Z to bring it back. A deletion is an
action across pictures, the notes' undo is per picture, and one key has to serve both.

### Options

1. One order across both: every note step and every deletion takes a number from one
   counter, and Ctrl+Z takes whichever is newest.
2. Notes first: Ctrl+Z walks the picture's notes to their start, and only then reaches a
   deletion.
3. A separate key or button for undoing a deletion.

### Decision

Option 1, Rotem's on 2026-09-16 when the two were put to him as scenes ("A"). Redo is
Ctrl+Shift+Z alone and deletes the picture again, no Ctrl+Y named for it; nothing is said
on screen when the picture comes back. Redo takes the most recently undone thing first,
and a step of a picture no longer on screen is passed over, since redoing it would change a
picture the user is not looking at.

### Consequences

Ctrl+Z after a delete always brings the picture back, whatever the picture on screen holds.
The deletion list and the counter live in the page's memory, like the notes' undo, so a
quit ends them and the Trash chip is the way back after one. A picture's own note history
keeps its per-picture redo tail, reachable when the global list has nothing newer. A refused
restore leaves the undo path rather than blocking it. Revisit if undo ever persists across
a restart, or if a second cross-picture action joins the history.

---

## 2026-09-16 · The focus return target is used once: a hide with no capture since activates nothing

### Context

Review 3 (T4) found that the application remembered at the hotkey was never forgotten, so
every later hide of the editor, opened from the tray or for a file from Explorer, brought
that application to the front. §3.1 wants the focus back where the capture began and never
on an unrelated window; hours later, the last capture's application is that window.

### Options

1. Take the target on use: one return per capture, then nothing until the next hotkey.
2. Clear the target on every route that shows the editor without a capture: the tray, a
   file opened, a second instance.
3. Leave it, and call the jump the chain-of-captures behaviour.

### Decision

Option 1, by the assistant under Rotem's FIX ALL, his to change: one line, and it keeps
"a chain of captures still returns", since each capture remembers afresh. Option 2 does the
same in three places and misses the next route that shows the window. A hide with nothing
remembered lets Windows pick, which is what §3.1 asks for a closed application too.

### Consequences

The first Escape or close after a capture returns the focus; a second hide of the same
editor, with no capture between, activates nothing. Copy and Return behaves the same. The
self test's section G and the S1.10 checks assert it. Revisit if Rotem wants a tray-opened
editor to return anywhere in particular.

---

---

## 2026-09-16 · Google Sans is bundled with the page, the medium Latin subset, for the image size only

### Context

Rotem asked for the image size's text in Google Sans, medium, from Google Fonts. The web view's
security policy in `host/tauri.conf.json` takes stylesheets and fonts from the page's own origin
only (`style-src 'self' 'unsafe-inline'`, `font-src 'self'`), and Recon has to work without a
network. Google Fonts serves the weight as 25 files, one per script; the family is under the SIL
Open Font License, whose text Google Fonts' own file list for the family carries.

### Options

1. Link Google Fonts' stylesheet from the page at runtime: the policy refuses it, and it needs a
   network.
2. Bundle every file Google Fonts serves for the weight, all 25 scripts.
3. Bundle only the file for the script the text is written in, Latin, which holds the digits
   and the x.
4. Inline the font in the page as base64: the policy allows no data: fonts.

### Decision

Option 3, with the family's `OFL.txt`, in `editor/fonts/`, 23 KB. The subset is the assistant's
call, Rotem's to veto.

### Consequences

A character outside Latin in that container falls back to the system font, character by
character. Another weight or script is one more file from the same stylesheet. The notes keep
the system font, so the export needs no inlined font; putting Google Sans on the notes would
bring part 5's inlining back, and whether the policy lets the serialized layer load it is not
established here. Revisit when the font is wanted on the notes or on text in another script.

---

## 2026-09-16 · A new capture is copied by the page's own copy, after it loads, not by the host at present

### Context

Rotem asked on 2026-09-16 for every capture to land on the clipboard by itself, with no key
pressed. The clipboard is the host's (part 5), and the host holds the captured pixels before
the page hears of them, so the copy could be made in either place.

### Options

1. The host publishes the raw frame at `present`, before the window shows: no page involved,
   but no notice on screen, and a failure only in the log.
2. The host publishes the frame and tells the page in an event, which shows the notice: a
   second copy path beside `editor_copy`, and a new event.
3. The page calls its own `copyComposed` once the capture has loaded, the same call as
   `Ctrl+C`: one copy path, the notice and the NOT COPIED line for free.

### Decision

Option 3, by the assistant, Rotem's to veto: a gate in the page's `capture-ready` listener on
`source === 'capture'` and no file, so a file opened or a frame stepped is never copied unasked.

### Consequences

Every copy goes through one path, so a change to the copy reaches the automatic one. The cost
is an empty layer export before the copy, and the copy lands after the window shows rather
than before; a check's capture probe copies too, since it announces itself as a capture. Revisit
if the copy ever shows on the capture-to-usable marks, or if the pixels are wanted on the
clipboard before the window is up.

---

## 2026-09-16 · The timeline's tabs are the page's own file beside the documents, Main implicit, a feed a list of ids

### Context

Rotem asked on 2026-09-16 for tabs on the timeline: a plus makes Main and New tab, a capture
taken on a tab lands in Main and in that tab, a tab can be deleted but never Main, and the
tabs after Main can be reordered. A tab is a list of tasks from a client, so it has to
survive a restart, and the host owns every file Recon keeps.

### Options

1. A tab recorded on each document: a `tab` field in `document.json`, the strip grouping
   by it.
2. One list the page owns, `tabs.json` beside the documents, each tab naming the ids of its
   captures, the host writing it whole and never reading it, like a document's notes.
3. The page's own browser storage, like the timeline's remembered height.

### Decision

Option 2, by the assistant. Main is not stored: it is the whole library, as the timeline
always was, so no document changes when a tab is made, deleted or reordered, and the
document format stays at schema 1. Option 1 would rewrite a document's record for a tab
change and could put a capture in one tab only, where Rotem's feeds are lists a capture may
join. Option 3 is cleared with the web view's data and cannot be backed up with the store.
The plus and the tabs sit in the picture's size band, at its left, where Rotem placed the
plus; the tabs' look is provisional until he states one (`project-os/Design.md`).

### Consequences

A feed is a list of document ids, so a document deleted to the trash leaves its tab when the
library does and returns with a restore or Ctrl+Z; a document swept from the trash leaves a
dangling id that lists nothing. The file is written through a temporary file and a rename
like every record, one write at a time from the page, and a file that does not parse is set
aside under a dated name rather than written over. The plus lives in the band, so no tab can
be made before the first picture is on screen. Revisit if a tab needs a name of its own, a
place for a file opened rather than captured, or more tabs than the band's left half holds.

---

## 2026-09-16 · Between a callout's two clicks the bubble keeps its automatic offset from the pointer, and a drop gives its number back

### Context

Rotem asked on 2026-09-16 for the callout in two clicks: the first marks the end of the
line, the bubble moves with the mouse, the second locks it and starts the typing. He did
not say where the pointer sits in the bubble while it follows, nor what happens to a bubble
abandoned between the clicks.

### Options

1. The pointer at the bubble's centre: the bubble jumps under the pointer at the first
   move, covering the point just clicked, and the second click lands on the bubble.
2. The bubble keeps the offset it started with, from its automatic place to the anchor:
   no jump, the anchor stays visible beside the pointer, the second click lands on the
   picture.
3. A drag from the anchor to the bubble, as Snagit draws a callout: one press, not two
   clicks, which is not what was asked.

For the abandoned bubble: keep its number as a gap, as an empty bubble discarded at
commit does; or give the number back, as a drawn shape shorter than four pixels does.

### Decision

Option 2, by the assistant, provisional and Rotem's to change (`project-os/Plan.md` §3.5
says so). The number goes back: nothing of the bubble was ever shown as a note, so there
is no gap for the numbering rule to keep, and the next callout takes the number the
abandoned one would have. Escape, Ctrl+Z, a change of tool, leaving annotation and another
picture arriving each drop it.

### Consequences

The first move never jumps the bubble, and the automatic place is a starting position as
§3.5 promises. The pointer is beside the bubble, not inside it, so locking it exactly on a
spot means aiming its corner, which is the cost. Ctrl+Z between the clicks takes back the
first click and leaves nothing to redo. Revisit if Rotem wants the pointer inside the
bubble, or a drag instead of the second click.

---

## 2026-09-16 · The thumbnail with the notes is composed by the host from a layer the page draws at thumbnail scale, at every save

### Context

Rotem asked on 2026-09-16 that the notes on a picture show on its thumbnail in the timeline
too. The thumbnail was made once by the host from the document's own image and kept as
`thumb.png`; the notes live in the page's scene, and only the page can draw them, since the
host never renders an annotation (`project-os/Map.md`).

### Options

1. The page draws the notes over the thumbnail in the strip itself, from its scene: works
   for the picture on screen only, and after a restart every other thumbnail loses its notes,
   unless the page rebuilds a scene for each document from the saved notes.
2. The page sends its full-size layer to the host at every save, as a copy does, and the
   host composes and resamples it into `thumb.png`: a full-size rasterize and PNG encode on
   the page's thread after every pause in typing, a few hundred milliseconds on a 4K capture.
3. The page draws the layer at the thumbnail's own scale, the composition fitted into the
   box, and the host resamples the picture to that scale, composes the small layer over it
   with the margin around, and writes `thumb.png` anew. Chosen.

### Decision

Option 3, by the assistant. The cost on the page is a thumbnail-sized canvas and PNG, the
host's work is one resample and a small compose, and the result is on disk, so a restart and
every other picture in the strip keep their notes. The blur rects are applied to a copy at
full size before the resample, exactly as a copy applies them. The refresh runs after each
save of the notes, not awaited by the save, one at a time with the newest scene queued.

### Consequences

`thumb.png` is no longer made once: it follows the notes, so the timeline shows what an
export would, margin and all, and a thumbnail of a composition with a margin is the whole
composition fitted into the box. Only the picture on screen is thumbnailed this way; a
document's notes changed while it is not on screen do not happen today. The page's box
size, 320 by 200, is repeated in the page (`THUMB_BOX`) and must match the host's. Revisit
if the notes should also show on a picture's thumbnail before its first save, or if the
store gains a way to render notes without the page.

---

## 2026-09-16 · The ruler is a box in the scene with its size in image pixels beside the corner the drag ended at, and its × is counter-scaled

### Context

Rotem asked for a ruler: mark an area, and its size in pixels, width and height, shows the
whole time in a bubble beside the pointer; he chose the reading where the box and its size
stay on the picture and in every copy, and asked for a round 16 px × with a 12 px X on hover
to delete it. Three things had to be placed: where the size goes once the pointer is gone,
what scale the label and the × live at, and how the ruler is drawn.

### Options

1. The size in screen pixels, always readable, and some other size in the copy.
2. The size in image pixels like a note, beside the corner the drag ended at; the × alone
   counter-scaled to 16 screen pixels.
3. The ruler drawn in the shapes' SVG with a `<text>` label, measured for its bubble.

### Decision

Option 2, by the assistant. The label goes into the copy, and the copy has one scale, so
the label is laid out in image pixels like every note (the 2026-09-11 decision: no minimum
on-screen size). It sits beside b, the corner the drag ends at, which is where the pointer
is while the drag lasts, so "beside the pointer" and "stays where it was" are the same
place. The × is editing UI, never in a copy, so it is the one part sized in screen pixels:
the scene's scale is undone on it through a CSS variable the scene's transform sets. The
ruler is an HTML box in the scene like the blur, not SVG, so the label's bubble sizes
itself and the hover × is plain CSS; the export takes the × out by its `data-ui` mark.

### Consequences

At a fit view of a wide capture the size reads small, as a note does; the note's size
ladder does not reach it. The × stays 16 px at every zoom, so it can cover a ruler smaller
than 16 screen pixels. A shape with `kind: "ruler"` in a saved document is ignored by a
page before this one, which draws nothing for a kind it does not know. Revisit if Rotem
states a look for the label, or wants the size readable at every zoom on screen.

## 2026-09-17 · The crop is one rect beside the notes, cut by the host on a copy at every output, and the notes keep their image coordinates

### Context

Rotem asked for a crop in the editor: a crop button in the sidebar, and with it in hand
handles on the picture's edges that are dragged inward, and the picture is cut. The plan's
v1 scope line kept crop out, so his word overrides it. Three things had to be placed: what
a crop does to the pixels the document keeps, what it does to the coordinates every note
already holds, and how the on-screen picture, the copy and the thumbnail agree.

### Options

1. The crop rewrites the document: the preserved image is cut on disk and every note is
   shifted by the crop's origin; undo keeps a copy of the pixels.
2. The crop is one rect in image pixels saved with the notes; the page shows, sizes and
   pans the picture as that rect, the notes keep their coordinates, and the host cuts a
   copy of the source at every output, as the blur is applied, the source never touched.
3. The crop is a negative margin, so the composer and the page's margin arithmetic carry it.

### Decision

Option 2, by the assistant. It keeps rule 11's shape for the document as well as for the
file: the preserved image is written once and never rewritten, so Ctrl+Z gives every pixel
back by dropping the rect, and a document saved before this change is unchanged. The notes
never move, so the crop is a cut and not a change to what the notes are placed on, and the
export shifts the layer by the crop's origin in one place. Option 3 was rejected because
the margin is recomputed from the notes at rest, and a crop folded into it would grow back
around a note outside the crop. Inward only, a side no smaller than 8 px, the release as
the moment of the cut, the dimmed outside and the key K are the assistant's provisional
calls, Rotem's to change.

### Consequences

Every output path takes one more header, `crop`, and the composer cuts a copy: a copy of a
large capture costs one more copy of its pixels. A note placed outside the crop takes
margin around the crop, as one past the picture's edge does, so a cropped picture can grow
a margin where the cut pixels were. The pan's centred value is the crop's origin rather
than zero, so every place that reads the picture's edges reads `pictureRect()` and never
the image's size. A saved crop that does not fit its picture is dropped at load. A crop in
a saved document is ignored by a page before this one. Revisit if Rotem wants a crop
dragged outward to bring pixels back, a crop at any size, or a look of his own.

## 2026-09-17 · An empty bubble gives its number back

### Context

Numbers never change and gaps stay (2026-09-10): a deleted note leaves its number unused
so every output agrees with every other. The callout in two clicks gives the number back
when the bubble is dropped between the clicks. After the second click, Escape with nothing
typed discarded the bubble through the commit, which kept the number, so the next note was
2 with no 1 on the picture (review 4, T5).

### Options

1. Keep the gap: an empty bubble counts as a note that was deleted.
2. Give the number back when the discarded bubble is the newest, as between the clicks.
3. Give it back always, renumbering the notes above it.

### Decision

Option 2, at Rotem's FIX ALL on review 4. An empty bubble was never a note: nothing was
typed, nothing was copied, nothing was saved with it, so there is no output for a gap to
agree with. Option 3 would renumber real notes, which the 2026-09-10 decision forbids.

### Consequences

`commitEditing` gives the number back only when the bubble's number is the newest one; a
bubble emptied later, after other notes were made, keeps its gap, since a note may have
been copied with it. A text note has no number and gives nothing back. The discard adds no
undo step either way.

## 2026-09-17 · A restore waits for an image still being encoded

### Context

A capture's record is written at once and its PNG on a thread a moment later. Review 3's
T2 sent the PNG of a capture deleted in that moment into its trash folder. Review 4's T2:
Ctrl+Z inside the same moment moved the folder back before the PNG existed, found no image,
refused, and left the folder among the documents unlisted; the thread then found the id
nowhere and dropped the PNG, the capture's only copy.

### Options

1. Refuse the restore at once and let the thread follow the folder, so a second Restore
   from the trash view works a moment later.
2. Let the restore wait up to a second and a half for the image, the thread writing it
   wherever the folder is now, and put the folder back into the trash when it cannot rejoin.
3. Make the delete itself wait for the encode before moving the folder.

### Decision

Option 2, by the assistant under Rotem's FIX ALL. Ctrl+Delete then Ctrl+Z is one gesture,
and the picture coming back is what the gesture means; option 1 makes it a NOT RESTORED and
a trip to the trash view. Option 3 makes every delete of a fresh capture wait, and the
delete is the common case.

### Consequences

`trash_rejoin` polls for `source.png` up to `REJOIN_WAIT`, 1.5 s, on the thread the command
runs on, only when the image is missing; a trashed folder with no image at all costs that
wait once and then says NOT RESTORED, with the folder back in the trash under a fresh stamp,
so its thirty days start again. The image thread writes beside the folder's files wherever
the folder is, never into a folder made for it. Revisit if a capture's encode ever takes
longer than the wait on a slow machine: the check in section 39 is the one that will say so.

## 2026-09-17 · The one row's scroller band replaces the 8 px under the thumbnails

### Context

Rotem's timeline was 128 tall: 112 px thumbnails, 8 px above and below. His sideways
scroller is a 12 px band under the row: 4 px clear, a 4 px thumb, 4 px clear of the
window's bottom. The band takes its 12 px out of the strip's height, and the cells were laid
out from the strip's full height, so every thumbnail's bottom 4 px sat under the band; the
selected thumbnail's blue border showed it. The three numbers cannot all hold in 128.

### Options

1. The timeline grows and the thumbnails keep their 112: the band below the cells.
2. The timeline stays 128 and the thumbnails shrink to 100.
3. Both stay and the band shrinks to 8 px, the thumb 2 px clear instead of 4.

### Decision

Option 1, Rotem's pick on 2026-09-17. The band replaces the 8 px below the cells rather
than adding to them: its own 4 px clear above the thumb is the gap under the thumbnail,
which is his scroller spec word for word, and the timeline is 133 tall: the 1 px top line,
8 px, 112 px, and the band. The reply offered 132, having left the line out; 133 keeps the
112 and the 8 whole, and the line is the one pixel nobody measures.

### Consequences

In one row a cell's height is the strip's less 21 px at every height, the 96 floor included
(120 by 75 cells there, from 128 by 80). The rows the strip wraps into keep their 8 px
below, since their scroller stands at the side. A height remembered on a machine from
before stays until the strip's edge is double-clicked. Revisit if Rotem wants the 8 px
below back as well, which makes the default 141.

---

## 2026-09-18 · The capture drag's size bubble is built pixel by pixel and blended on whole, not a GDI round rectangle

### Context

Rotem asked for the dragged area's size beside the crosshair while a capture drag lasts:
14 px text in a bubble of the app's background with 8 px corners. The overlay is plain
Win32 painting, whose own rounded rectangle has stepped corners, and whose text call clears
the alpha of every pixel it touches, on a window whose pixels the compositor shows by their
alpha (the dim's note in `host/src/overlay.rs`).

### Options

1. GDI's own round rectangle and text call, straight onto the window: two calls, stepped
   corners, and the text's pixels left with no alpha.
2. The bubble in a bitmap of its own: the fill written pixel by pixel, each corner pixel
   covered by the fraction of it inside the arc, the text drawn into that, its alpha put
   back, and one blend onto the window.
3. A layered window of the bubble's own, moved with the pointer.

### Decision

Option 2, mine. The corners come out as smooth as the page's, the window never holds a
pixel without its full alpha, and the cost is a bitmap of a few thousand pixels per paint,
on the drag's own moves. A third window would be one more thing to place, focus and tear
down on the latency path.

### Consequences

The bubble is the paint's fourth step, drawn wherever the dirty region touches it, so a
tick's strip through it redraws that strip; a move invalidates its old and new places.
Every number scales with the display's own scale, as the cursor does: 14 px text is 14 on
Rotem's display and 32 on one at 225%. Cost: the app background is now written in the host
too, beside the page's `--paper`, the second home Backlog F68 already names for the margin
colour; a change to the token has to reach both until the page hands the host its value.
Revisit if the text should be the page's Google Sans, which the host would have to load
from the page's own file, or when the gap from the pointer or the text's colour is stated.

---

## 2026-09-18 · The magnifier is the frozen slice stretched pixel for pixel, up from the first pointer, the size under it: the picture's reading, not the words'

### Context

Rotem asked for a small zoom window like the one in a picture of another tool: a 112 px
circle in which single pixels can be told apart. His words said the circle; the picture
said more. It shows the tool while a window is lit under the pointer, not mid-drag, with
the circle in a dark panel below and left of the pointer and the lit window's size written
under the circle, where the day before the size had been asked for beside the crosshair
during a drag. `CLAUDE.md` rule 1 says that where the words and a picture differ, the
picture is built or the question asked first, never the words alone.

### Options

1. The picture's reading: the panel up the whole time the overlay is, hovering and
   dragging, below and left of the pointer, with the selection's size moved under the
   circle; the pixels stretched straight from the display's frozen bitmap with no
   smoothing, drawn into the panel and blended on as the size bubble was.
2. The words' reading: the circle during a drag only, the size bubble left where it was
   beside the pointer, the circle placed clear of it.
3. A layered window of the magnifier's own, moved with the pointer.

### Decision

Option 1, mine, on the picture. A magnifier earns its place before the press, placing the
drag's first corner, and the picture shows it there; a size beside the pointer and a circle
under it would have fought for the same 8 px. The stretch reads the surface's own bitmap,
so the circle shows exactly the frozen picture the selection is made on, and a third window
would be one more thing on the latency path.

### Consequences

Every move repaints a panel of about 120 by 143 px at 100%, hovering included, where before
a hover repainted only a changed lit window; a stretch and two small bitmaps per move. The
count of pixels across the circle is odd so the pointer's own sits in the middle, and past
the display's edge the panel's fill shows. Everything read from the picture is provisional
in `project-os/Design.md`: the side of the pointer, the gap, the hover, the zoom, the grid,
the ring and the lines through the centre are each one number or one flag, and his word
moves any of them. Revisit if the hover's constant repaint is ever felt, or if the size
should not be shown while hovering after all.

---

## 2026-09-18 · The lit window glides to the next one edge by edge on the frame's own clock, inside one display, and jumps when Windows does not animate

### Context

Rotem asked for the lit area of the capture overlay, the dashed frame and the undimmed
window inside it, to move from one window to the next instead of jumping there when the
pointer crosses. The overlay is plain Win32 painting with no animation of its own; the
marching frame already ticks about sixty times a second and reads its walk off the clock.

### Options

1. Glide the rectangle: each of its four edges moves from the old window's to the new
   one's over a fixed time, eased, the dim and the frame following it, on the frame's own
   ticks; a pointer that moves on mid-glide sets out again from wherever the area is.
2. Cross-fade: the old area dims out while the new one lights, both rectangles fixed.
3. Glide across displays too, painting the in-between rectangles on both surfaces.
4. For the timing: a token of the overlay's own, or the design system's A1.

### Decision

Option 1, mine, on Rotem's ask: an animation of the area's movement is a move, and a
move is what a picker of windows does. The glide is kept to one display, since a selection
never spans two (§3.2) and each surface would otherwise have to paint a rectangle that is
not its own; a move to a window on another display jumps as before. The timing is A1's,
144 ms and ease-out, the design system's one motion token, provisional until Rotem states
a move token. A click mid-glide picks the window under the pointer, never the rectangle
on its way. With Windows' "Animation effects" off, the glide is off and the area jumps,
which is QA §7's reduced-motion gate at this surface; a setting that cannot be read counts
as on.

### Consequences

Each tick that moves the area repaints the union of where it was and where it is, the
whole area and not only the frame's strips, for the 144 ms a glide lasts; at rest the
cost is what it was. The self test reads the frame's band mid-glide and after it, so a
jump or a glide that never arrives turns the overlay row red. Cost: the duration and the
easing are guesses until Rotem has seen them. Revisit them by eye, and revisit the
one-display rule if a glide between displays is ever wanted.

Later the same night, Rotem's values: 256 ms and ease-in, cubic, slow to set out and quick
to arrive, in place of A1's 144 ms and ease-out. The self test's looks moved to 200 and 450
ms after the move to suit them.

Then again, the same night: 164 ms and ease-out, cubic, quick to set out and slow to
arrive; the looks at 80 and 300 ms after the move.

Then 192 ms, the ease-out kept; the look past 140 ms inconclusive.

## 2026-09-18 · Settings changes a global shortcut by trying the new one first

### Context
Rotem asked for a Settings modal, opened from three dots left of minimize, where the
shortcut that opens Recon and the shortcut that starts a capture are chosen. Until now the
capture shortcut was read from `%APPDATA%\Recon\recon.json` at startup and nothing wrote
that file. The plan's §3.1 already binds the behaviour: a taken shortcut is explained and
another allowed, and Recon never takes a shortcut over silently.

### Options
1. Drop the old shortcut, then register the new one, and put the old one back on failure.
2. Register the new one first, and let the old one go only once Windows has given the new.
3. Save the choice and apply it at the next start.

### Decision
Option 2, the assistant's, Rotem's to veto. With option 1 a failure to put the old one back
leaves Recon with no capture shortcut at all; with option 3 a taken shortcut is only found
out after a restart. Three more choices ride with it. The shortcut that opens Recon is
NONE until Rotem picks one: a second global shortcut is a second thing that can collide
with another application, so it is never taken by default, and it is the only one that can
be cleared. While a field waits for a shortcut, both global shortcuts are let go, so
pressing the current one picks it instead of firing a capture under the modal; they come
back the moment the waiting ends, by a key, by Escape, by closing, or by the window losing
focus. And the two can never be the same shortcut, however it is spelled. The file is
written after Windows agrees and flushed before its rename; if the write fails the change
is put back and the field says NOT SAVED.

### Consequences
A shortcut is a string as Rotem sees it, "Ctrl+Shift+4", built from the physical key so it
is the same under a Hebrew layout; "Win" is translated for the parser. A check run has no
shortcut plugin and a settings file of its own beside its executable, so a check never
takes a global shortcut and never touches Rotem's file; the real swap against Windows is
therefore proven by hand on the release, not by the checks. The lock on the two shortcuts
is never held across a registration, since the plugin finishes one on the main thread,
where a fired shortcut asks for the same lock. Open: whether a settings change joins
Ctrl+Z (`CLAUDE.md` rule 23); it does not yet, asked of Rotem on 2026-09-18.

## 2026-09-18 · A change made in Settings stays outside Ctrl+Z

### Context
`CLAUDE.md` rule 23 puts every action that changes something a person sees or keeps into
undo and redo. Settings landed the same day with two shortcuts a person picks and keeps, so
the rule as written reached it, and the entry above left the question open.

### Options
1. Ctrl+Z in the editor also takes back a shortcut change.
2. Settings stays outside undo, as in most applications.

### Decision
Option 2, Rotem's, on 2026-09-18.

### Consequences
Rule 23 carries the exception in its own text. The undo list stays a list of acts on
pictures, notes, tabs and the timeline; a Ctrl+Z pressed after closing Settings undoes the
last of those, never a shortcut. A later setting follows this entry unless Rotem says
otherwise for it. This closes the open question in the entry above.

## 2026-09-18 · The Open Recon shortcut minimizes only the window in front

### Context
Rotem asked that a second press of the Open Recon shortcut minimize the window: press, it
opens; press again, it minimizes. A press can also arrive while Recon's window is open
behind another application, where "again" is not what the person means.

### Options
1. Minimize whenever the window is open, wherever it sits.
2. Minimize only when Recon is the window in front; in every other state bring it forward.

### Decision
Option 2, the assistant's, Rotem's to veto. With option 1 a press meant to reach Recon from
another application would make it vanish instead of arrive, and a second press would be
needed every time. In the tray, minimized, or behind another window, the press shows it.

### Consequences
It minimizes to the taskbar, as the minimize button does, never hides to the tray: Rotem's
word was minimize. The rule is one pure function, `settings::open_press`, under a unit
test. The tray icon's click and a capture still only ever show the window.

## 2026-09-18 · A file opened by name sits in the timeline as a pointer, in a list of its own, never in the store

### Context
Rotem asked that an opened file, an SVG or any other picture, join the timeline "as if it
were a screenshot". A screenshot is there because Recon keeps its own copy, and rule 11
says an external file is never copied into Recon's storage, so the ask and the invariant
met head on.

### Options
1. A pointer: the thumbnail names the file where it lives, nothing is copied, and it leaves
   the timeline when the file is moved or deleted.
2. A kept copy like a capture's, which stays after the original is gone and lifts rule 11.
3. For either: every file stepped onto in a folder joins, or only the file opened by name.

### Decision
Option 1, and only the file opened by name: Rotem's, on 2026-09-18. "If we update it, it
saves the original" is read as what Annotate already does, the image preserved once a note
is wanted. Built as a list of its own, `opened.json` beside `tabs.json`, rather than as a
kind of record in the document store: the store's every path assumes a `source.png`, and
its trash, restore and ledger are the project's most bitten code.

### Consequences
A pointer holds a path, a size and a number, never pixels; its thumbnail is made from the
file when asked for and lives in memory only. It shares the documents' numbers, so tabs,
done marks, the keys and Ctrl+Z treat it as any thumbnail, and Annotate turns it into the
file's document under the same number. Its x takes it off the list and keeps nothing in the
trash; Ctrl+Z puts it back within the session only. The picker, "Open with" and the command
line join; previous and next in a folder never do. The storage figure does not count it.
Every timeline refresh asks whether each pointed file is still there, which a dead network
path could make slow: worth a cap or a lazy test if it ever bites.

## 2026-09-18 · Save As on an annotated PNG or JPEG writes over the file it came from

### Context
Rotem annotated a picture he had opened and pressed Save As: Recon offered a new name
beside the original, "img annotated.png", as rule 11 and S1.11 had it, and he wanted the
file he opened saved, not a second version of it. Rule 11 is his own invariant, so the ask
was put back to him as the scene it is: the clean original gone from disk.

### Options
1. Save As writes over the opened file, for the types that can be written back as
   themselves, PNG and JPEG.
2. Keep a new file every time, as built.
3. For the other types under option 1: refuse, or keep the new annotated PNG.

### Decision
Option 1, and the new annotated PNG for the other types: Rotem's, on 2026-09-18. Built as
the narrowest door: only a managed document's own source file, only when that path is the
one chosen in the dialog, which opens on it; the write goes through a temporary file, a
flush and a rename, so a failure leaves the original whole. The Windows overwrite prompt
stays off, since pressing Save on the file's own name is the answer to its question.

### Consequences
`CLAUDE.md` rule 11 carries the exception in its own text. A file only viewed, any other
existing name, and an SVG, GIF, WebP, BMP, TIFF, HEIC or AVIF keep the never-overwrite
path. The document keeps its clean preserved picture and its notes, so a second Save As
composes from the clean picture and never burns notes in twice, and the clean picture can
still be had from Recon while the document lives. A note in the margin makes the saved
file larger than the original was. The document's record of its file's size and time
follows the save, so it does not report its own save as a change on disk.

---

## 2026-09-18 · The mode follows what is done: a tool picked starts the annotation, a finished element puts the tool down, and the Annotate button and the A key are gone

### Context

A file opened in viewing, and nothing could be added to it until the Annotate button or
the A key was pressed; a tool picked there lit up and did nothing. Rotem said on
2026-09-18 that jumping between the two modes by hand is the problem and the switch has to
happen by itself: a callout made and its text finished, and the picture is back to viewing;
a click on that callout, and it is edited. He chose, of the scenes put to him, a tool
picked again for every note, and the button and the key removed rather than kept as an
override. This replaces the second half of 2026-09-14, "a chosen one stays until the next
picture".

### Options

1. Flip the page's mode to viewing after every element, and back at a click on one. Viewing
   takes the pointer off the notes, so the click would have to be hit-tested by hand, and
   entering annotation reloads the picture from the host, in the middle of a click.
2. Leave the mode to mean only "this picture has a document", and make the state Rotem calls
   viewing the one that already exists inside annotation: no tool in hand, the ordinary
   pointer, a drag pans, a click on a note edits it.

### Decision

Option 2. A finished note (`commitEditing`) and a drawn shape (the release of a draw that
made one) put the tool down; a press that draws nothing keeps it, and the crop tool, which
makes no element, stays in hand. A tool picked over a file only viewed asks the host for
the document first (`pickTool`), once however many picks arrive, and arms the tool only if
that picture is still the one shown. The tool keys work in viewing for that reason.

### Consequences

One key or button before every element, which is what Rotem chose. Any of the eight tool
keys pressed over a viewed file now makes its managed document, where only A did; the file
itself is never touched (rule 11), and nothing is made until a tool is picked. A file
annotated before still opens as the plain file, and its notes appear when a tool is
picked, since the route the A key gave is the tool's now. `setMode('view')` is left in the
page for the checks only; nothing in the product calls it. The tool in hand was never part
of undo, so Ctrl+Z is unchanged: it takes back the note or the shape, not the tool.

## 2026-09-18 · The colour picker's HEX is put on the clipboard by the host from a colour the page names, and the picker starts no annotation

### Context
Rotem asked for a colour picker in the left sidebar: a click on the picture copies the HEX
under it. The page may not touch the clipboard (part 5, and the capability file says so),
and since the same day a tool picked over a file only viewed starts its annotation.

### Options
1. The page writes the text itself through the browser's clipboard.
2. A host command that publishes any text the page sends.
3. A host command that takes three numbers, a colour, and writes the HEX itself.

### Decision
Option 3, the assistant's, Rotem's to veto. The page reads the pixel through the magnifier's
own route, so the HEX copied is the HEX its circle shows, and sends red, green and blue;
the host formats `#RRGGBB` and publishes it as text. The picker is the one tool that starts
no annotation: it changes nothing, so a file only viewed stays viewed and no document is
made for it (rule 11 untouched). Put down after the click and the zoom circle are Rotem's
two calls.

### Consequences
The page still cannot publish anything of its choosing: seven characters of a colour is all
this adds. The pick is no act, so Ctrl+Z has nothing to take back (rule 23), as with Copy.
Over an SVG being viewed the colour comes from the fixed raster the magnifier reads, not the
sharp redraw on screen, so on an edge zoomed far in the two can differ by a shade. A second
text the host should publish would want its own narrow command, not a widening of this one.
