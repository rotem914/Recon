# Decisions.md - Archive

NOT read by default - consult only when digging into an old entry.
Moved here verbatim by project-os/rotate.ps1. Movement only: nothing is rewritten, compressed, or deleted.

## Archived decisions
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

## 2026-09-14 · The store: one folder per document in local application data, the image once, the record whole, the number its creation time

### Context

Part 5 fixed the shape, one folder per document with the image as a PNG never rewritten
and a JSON record with a schema version, atomic writes, no index. S1.8 had to choose
where the folders live, what a document's number is once numbers must survive a restart,
what the record holds, and when the page's notes reach the disk.

### Options

1. Roaming application data, beside the hotkey setting, so the documents follow the user's
   profile between machines.
2. Local application data, `%LOCALAPPDATA%\Recon\documents`, machine-local.
3. A folder the user picks, or the Pictures folder.
4. For the number: keep the per-session counter and renumber on load; or a random id; or
   the creation time in milliseconds, monotonic within a session.
5. For the notes: a host-side schema of callouts, or the page's object carried whole.

### Decision

Option 2 for the place, by the assistant: a day is tens to hundreds of megabytes (§3.8),
which a roaming profile would sync at every logon, and nothing in a document is
machine-specific, so a later move to a chosen folder is a path change. The number is the
creation time in milliseconds, bumped past the last issued when two fall in one, so the
page's stash and the disk agree on one key before and after a restart and numbers never
collide. The notes are the page's own object, carried whole and never read by the host,
which is part 5's rule that the format is not coupled to a renderer; the host adds the
schema version, the size and the source around it. Saves: debounced 800 ms while typing,
at once on blur, before navigation, before another document, before a hide, and when the
host asks at close or quit and waits up to a second for the page's answer.

### Consequences

Every capture is on disk from its first moment and a note typed is on disk within a
second of the last keystroke; a crash loses at most that. The record is rewritten whole
through a temporary file, so a document on disk is always the old one or the new one.
Cost: the image is encoded and written for every capture, on a thread, about 40 ms and
3.5 MB for a full screen; retention is the user's until Stage 2 adds deletion (§3.8). A
document read back has no file behind it (F82). Revisit the place if a user wants the
documents on another drive, and the number if two machines ever merge a store.

---

## 2026-09-14 · A deleted document goes to Recon's own trash for thirty days, by a thumbnail's × or Ctrl+Delete

### Context

§3.8 asks for one unambiguous delete and no silent deletion, and Stage 2 had to say what
happens to a deleted document and where the control sits, the first thing in Recon that
removes work.

### Options

1. Gone at once, with a confirmation dialog.
2. The system's recycle bin, which the user empties.
3. Recon's own trash beside the documents, swept after a fixed number of days.
4. The control on the thumbnail only, on a key only, or both.

### Decision

Option 3 with a thirty-day sweep and both controls, by Rotem on 2026-09-14. The key is
Ctrl+Delete, chosen by the assistant so plain Delete stays the selected note's; provisional.
The trash is Recon's rather than the recycle bin because the bin's emptying is the
system's and the user's to time, and thirty days is a promise Recon can keep itself. No
dialog: the trash is the safety net, and a dialog on every delete is what §3.8 rules out.

### Consequences

A delete is one click or one key and reversible for thirty days by the folder in the
trash, in Recon once S2.7 lands. The sweep runs at startup only, so a machine never
restarted keeps its trash longer, which errs on the safe side. Cost: the trash takes disk
until the sweep. Revisit the key if Ctrl+Delete collides with a habit, and the days if
thirty proves too many or too few.

---

## 2026-09-14 · The window under the pointer is the selection until a drag begins, listed once with the overlay, told from a drag by the system's threshold

### Context

Rotem asked for the selection to follow the window under the pointer during a capture,
the way a browser or an Explorer window is picked whole. The overlay until now knew only
a drag, and a click with no drag cancelled. Three things had to be chosen: what a click
means, where the window bounds come from, and when a press stops being a click.

### Options

1. Hover lights the window under the pointer; a click captures it; a drag draws a free
   rectangle as before.
2. A mode key to switch between window picking and free selection.
3. Window picking only, with the drag removed.

For the bounds: the window rectangle as Win32 reports it, which on Windows 10 and 11
includes an invisible resize border around every framed window; or the frame the desktop
compositor draws, which is what the eye sees. For the window list: read at every pointer
move, or once when the overlay comes up.

### Decision

Option 1, mine, on Rotem's ask. The bounds are the compositor's frame, so a picked window
does not carry a strip of its neighbour; the list is taken once, when the overlay comes up,
because the picture under the overlay is frozen at that moment and a window that moves
afterwards is not in that picture anyway. A press becomes a drag when the pointer leaves
the system's own drag threshold, so a click here means what a click means everywhere on the
machine and no number of Recon's own is invented. Cloaked windows, click-through windows,
minimised ones and Recon's own are left out of the list: none of them is a thing the user
could click on the real desktop. A window's bounds are cut to the display the pointer is
on, which is §3.2's rule for a selection.

### Consequences

A click with no drag now captures rather than cancels; the bare desktop is a window too,
so a click there captures the display. Cost: one listing of the top-level windows on the
hotkey path, a few milliseconds, not yet measured against S0.7's interval. Revisit if a
class of window is picked that should not be, or if child controls inside a window are ever
wanted, which this list does not see.

Revisited the same day: the parts are wanted, and the entry below says how.

---

## 2026-09-14 · The selection is the smallest part of the window under the pointer, the way Snagit picks a page

### Context

Rotem showed Snagit lighting a browser's page area alone, tabs and address bar left out,
and asked for that. The window pick decided that morning lit whole windows only.

### Options

1. The smallest visible child window under the pointer wins; a window with no child under
   the pointer is lit whole. Snagit's own behaviour, and what a probe of the running
   Chrome makes possible: its page area is a child window whose rectangle matched Snagit's
   marquee to the pixel.
2. A key held to switch between the whole window and its part.
3. The accessibility tree, which would also reach a browser that draws its page without
   child windows.

### Decision

Option 1, mine, on Rotem's ask and on his daily use of Snagit. The whole window is one
move away, to a spot with no part under it, a browser's tab strip for instance, so no key
is needed. The parts of a window are listed the first time the pointer rests on it, not
when the overlay comes up, because listing every child of every window would sit on the
hotkey path for windows the pointer never visits. Parts are not filtered on being
click-through, unlike top-level windows: a browser's page area is click-through by design
and is exactly the part wanted.

### Consequences

A browser's page, a folder window's file list and a chat's message pane are each one
click. Cost: a window whose page has no child window, Firefox is the known case, lights
whole; the accessibility route stays open for it. Revisit if a part that is not what the
eye sees keeps winning, or if the key of option 2 turns out to be missed.

Rotem, the same evening: Firefox needs no support. The whole browser lit there is the
final behaviour, and the accessibility route is not planned.

---

## 2026-09-14 · The frame's dashes are light on dark, not light on nothing, drawn by hand so they can walk

### Context

Rotem asked for the white frame around the lit area to be dashed and to move, very
slowly. Two choices hid in that: what sits between the dashes, and how a moving dash is
drawn on this surface, which is plain Win32 painting with no animation of its own.

### Options

1. Light dashes with dark gaps, the classic marching frame, drawn as one-pixel runs by
   hand, with a timer stepping a phase and repainting only the frame's strips.
2. Light dashes with nothing between them, the picture showing through.
3. A dashed pen from the system, which draws dashes but has no phase, so it cannot walk.

### Decision

Option 1, mine, on Rotem's ask. A gap that shows the picture disappears on a light
picture exactly where a white dash disappears on a dark one; light on dark reads on any
background. The runs are laid along one path around the frame so a dash turns a corner
rather than restarting at each edge. Six pixels a dash and a gap, one pixel every 120 ms,
are starting values for Rotem's eye.

### Consequences

A timer ticks on each display's overlay for the length of a selection, doing nothing
unless that display has something lit, and then repainting four one-pixel strips. Cost:
the three values above are guesses until Rotem has seen them; the reply names them.
Revisit the speed and the dash length by eye, and the colours if the frame ever hides on
a mid-grey picture.

The same evening, after Rotem saw the pixel steps at 8 a second: the walk is read off the
clock at about sixty frames a second, and the one pixel at each dash edge is blended by
the fraction of a pixel walked, so the dashes glide. Two extra brushes per frame is the
whole cost. The tuned values stand at 2 pixels thick, #00B9F7 on near black, 12 on and 8
off, 12 pixels a second.

---

## 2026-09-14 · No tool is in hand when a picture opens, and a chosen one stays until the next picture

### Context

Since S3.1 a tool was always in hand, the callout by default, so a capture opened with a
crosshair over the whole window and a click on the picture started a note (§3.3). Rotem
asked for the opposite: the ordinary pointer everywhere and a click that does nothing. Two
things had to be read into that: when the default comes back, and what happens once a
tool is chosen.

### Options

1. No tool at launch only; a chosen tool stays in hand for every later picture.
2. No tool whenever a picture is shown: a capture, an opened file, a document from the
   timeline, history or the trash, a document resumed by Annotate.
3. No tool after every use: the tool drops back once its note or shape is made.

### Decision

Option 2. The default is Rotem's call; when it comes back is mine. The editor page lives
for the whole session, hidden between captures, so option 1 would bring the crosshair back
from the second capture on, which is what Rotem asked to cancel. Option 3 changes what a
chosen tool does, which was not asked. A chosen tool works as before, crosshair included,
until another picture is shown. Viewing shows the ordinary pointer as well.

### Consequences

A stray click on a capture creates nothing, the promise viewing already made for files.
Cost: a key or a button before the first note of every capture. Inside one picture, only
leaving annotation and coming back puts a tool down; whether Escape or a second click on
the lit button should do it is Rotem's to say. Revisit if daily use picks the same tool on
nearly every capture, which would argue for remembering the last one.

---

## 2026-09-14 · The wheel zooms around the pointer, with Ctrl or without, and a sideways wheel still pans

### Context

S1.3 made the wheel pan and Ctrl with the wheel zoom around the pointer. Rotem asked for
the wheel alone to zoom in and out, centred on the pointer.

### Options

1. The wheel zooms, Ctrl or not, and a sideways wheel does nothing.
2. The wheel zooms, Ctrl or not, and a sideways wheel, with no up or down in it, still pans.
3. The wheel zooms, and Shift with the wheel pans.

### Decision

Option 2. The wheel is Rotem's call; the sideways wheel is mine, since he did not mention
it and it keeps what it did. The step is the one Ctrl with the wheel already had, 25% a
notch, and the point under the pointer stays put. Option 3 adds a gesture nobody asked for.

### Consequences

The wheel no longer pans up and down. In viewing a drag still pans; in annotation the arrow
keys are what is left to pan with, since a drag on the picture belongs to the tool in hand
and does nothing with none. A touchpad's two-finger scroll up or down zooms as well, because
the page cannot tell it from a wheel. Revisit if panning a zoomed capture in annotation
proves slow; a drag with no tool in hand could pan, as it does in viewing.

The same evening, at Rotem's word: a drag with no tool in hand pans in annotation too, and
Space held pans with any tool, so the arrow keys are no longer the only pan there. The
entry on the drag and Space says how.

---

## 2026-09-14 · The window's top bar is Recon's own, not Windows' frame recoloured

### Context

Rotem said the editor had Windows' generic top bar, the plain white caption every
application gets. Two ways to make it Recon's: keep the frame and colour it, or drop the
frame and draw the bar in the page like the rest of the editor.

### Options

1. Keep Windows' frame, coloured dark through the desktop window manager: the buttons,
   the snap-layout flyout on the maximize button and the edge resizing all stay the
   system's; the bar holds nothing of Recon's.
2. No frame; the page draws a 32-pixel bar with the title, the tool buttons, and minimize,
   maximize and close on the right at the size Windows draws them. Dragging and the
   double-click are the runtime's, from one attribute; the three buttons call the runtime's
   window API, and close reaches the host's close handler like the frame's X did.

### Decision

Option 2, by Rotem on 2026-09-14, with the three window buttons named as the thing not to
forget. The tool buttons moved into the bar, which is the room option 2 promised.

### Consequences

The bar is styled like the rest of the editor and can hold what the editor needs. Cost: the
page gains four window permissions (drag, minimize, toggle maximize, close), and the
snap-layout flyout Windows 11 shows on hover over the maximize button is gone, since only
a frame of the system's own can offer it; Win+Z and dragging to an edge still snap. Edge
resizing of an undecorated window is the runtime's documented behaviour and is Rotem's
hand to confirm. Revisit if the flyout is missed, or if a caption-height change on some
Windows makes 32 look wrong.

---

## 2026-09-14 · With no tool in hand a drag pans the picture, and Space held pans it whatever tool is in hand

### Context

Once the wheel zoomed instead of panning, a capture in annotation could be panned only with
the arrow keys. Rotem said yes to a drag that pans when no tool is in hand, as viewing
already does, and asked for Space to give the ordinary pointer and the same drag while a
tool is in hand.

### Options

1. Space toggles the pan on and off with each press.
2. Space pans while it is held, and letting go gives the tool back.
3. Space held pans only from the empty picture, leaving notes and shapes to their own drags.

### Decision

Option 2, my reading of Rotem's ask, and his to change. While Space is held the notes take
no pointer, as in viewing, so a drag pans whatever it starts on; option 3 would make the
pan fail on a crowded picture. With no tool in hand a drag from the empty picture pans,
committing a note being typed and clearing a selection on the way, while a drag that starts
on a note or a shape still moves it. Space is never taken while a note is typed, where it
is a space, and the window losing the keyboard lets go of it, so a Space released in
another window cannot leave the editor panning.

### Consequences

A zoomed capture pans by hand in both modes and with any tool. Cost: one more key in the
contract, and a hold that a user expecting a toggle will not find at first. Revisit if a
toggle is wanted, or if Space is wanted for something else.

## 2026-09-14 · The timeline is a window onto the list, with no View More, and its thumbnail file is 320 wide

### Context

Rotem asked how many thumbnails the strip should show at once, when it gets heavy, whether
a View More button is wanted, and for a drag on the strip's top edge that grows the
thumbnails to 320 wide. He expects thousands of captures a month. The strip as built by
S2.1 listed every document, fetched every thumbnail at startup and kept every one, so
memory grew with the library, which §3.8 forbids.

### Options

1. A View More button, or a fold by date, that keeps the tail of the list off the page.
2. A window onto the whole list: cells placed by arithmetic, only the ones near the screen
   made, their pictures dropped when they scroll away.
3. Both: the window for load, the fold for the eye.

### Decision

Option 2, proposed by the assistant and taken by Rotem on 2026-09-14. A View More pages a
list read top to bottom; the strip is scrolled and the keys walk the same list as history,
so a hidden tail would be one the keys reach and the eye cannot. The load is solved out of
sight, and a fold by date stays available as a product choice if old work is wanted out of
the way on purpose. With it: the thumbnail file is made at 320 by 200 rather than 160 by
100, since a grown strip would otherwise show a blurred picture, and a smaller one found on
disk is remade once; the file is served as kept and scaled by the page, not resampled to
steps, because a screenful of 320 by 200 pictures is a few megabytes and a fetch burst on
each step crossed was not worth it. The list of records is still read whole at startup,
measured at 1.9 to 2.2 seconds for thirty thousand documents on a warm disk; a lazy read
by month waits on Rotem's word with that number.

### Consequences

The page holds a screenful of pictures whatever the library holds, and a startup asks for
a screenful. The thumbnail file is about four times its old size, a few percent of the
document's own image. Cost: a cell that scrolls two screens away and back is fetched
again, from a small file. Revisit the startup read when a year's library is real and its
two seconds are felt, and revisit the fold by date if Rotem wants old work out of sight.

---

## 2026-09-15 · Start with Windows is one value under the user's Run key, and a logon start shows no window

### Context

Rotem asked for a way to have Recon start with Windows. Two choices sit inside that: how
Windows is told, and what Recon shows when Windows starts it, given that a start by hand
opens the window (S2.2, 2026-09-14).

### Options

1. A value under the user's Run key, `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
2. A shortcut in the user's Startup folder.
3. A scheduled task at logon, or Tauri's autostart plugin, which writes the same Run value
   through a dependency of its own.
4. At a logon start: open the window as a start by hand does, or wait in the tray.

### Decision

Option 1, and the tray for option 4; mine, on Rotem's ask, his to veto. The Run value is
under the same hive as the type registration, written and removed by the same two helpers,
so it is exactly one value that can be removed exactly; it shows in Task Manager's Startup
apps, where Windows lets the user turn it off too. A shortcut file is a second artefact
to keep true when the executable moves; a scheduled task needs the task service and a
plugin adds a dependency for one registry value. At logon the start is Windows' act, not the
user's, and a window at every boot is the kind of thing that gets the option turned off;
the tray, with the hotkey armed, is the point of starting early. The value passes
`--startup` so the host can tell the two starts apart.

### Consequences

The tick reads the registry, so it is true after a restart and after Windows' own
removal of the value. What it cannot see: Task Manager disables a startup app by a
separate approval key and leaves the value in place, so a tick turned off there still
shows on in the tray; revisit if that confuses. The value names the executable that
wrote it, so a moved or rebuilt-elsewhere Recon needs the tick off and on again. A second
instance started with `--startup` while one runs hands over to the running one, which
shows its window, the same as any second start.

---

## 2026-09-15 · A pan drag slides the picture already painted and asks for one region at a time

### Context

Rotem found that a drag moved the picture only when the mouse was released. The page asked
the host for a region on every move, and part 5's display policy drops an answer that a
newer request has overtaken; moves come faster than a region comes back, so every answer
was dropped until the pointer stopped.

### Options

1. Stop dropping late answers during a drag.
2. Ask for a region at most once a frame.
3. Slide what is already painted with the pointer, placed by its region's own origin, and
   ask for one region at a time, the newest pan as each one lands.

### Decision

Option 3, mine, on Rotem's report. Option 1 paints a region under a pan it was not asked
for, so the picture sits away from the notes by however far the pointer moved meanwhile.
Option 2 still stalls whenever a round trip takes longer than a frame, which a large capture
does. The drop stays, and within a drag it never fires, since no newer request is made
while one is on its way.

### Consequences

The picture and the notes follow the pointer at once, and the leading edge fills in as each
region lands. While the drag lasts the canvas is placed by its region's origin rather than
snapped to a physical pixel; the drag's end paints the ordinary way, so the view at rest is
the one F74 asks for. Other input that keeps coming, a fast spin of the wheel, still asks on
every step; revisit if that shows the same stall.

## 2026-09-15 · The store is read on a thread at startup, newest first, not by month

### Context

Rotem said BUILD on the read-by-month startup list proposed under S2.8, so a year of
thousands of captures a month would not cost two seconds at every boot. Building it found
that a list read by month cannot serve the one lookup that must see every document:
Annotate on a file resumes that file's one document (§3.3), and would have to read every
month to know there is none. The storage figure in the HUD turned out to walk every folder
on every timeline refresh as well, a larger cost at that size than the startup read.

### Options

1. A lazy list read by month, with Annotate's lookup reading the rest on demand.
2. The whole list read at startup, the newest fifty records first so the latest document
   reopens at once, and the rest on a thread, newest first; the page told when it is whole.
3. An index file beside the folders, maintained on every write.

### Decision

Option 2, by the assistant, delivering what was asked for, a startup that costs the same
at thirty thousand documents as at fifty, without a list that can be partial in every path
that walks it. Only Annotate's resume waits for the thread; the timeline and the history
keys show what has arrived and refresh when the rest lands. With it, the storage figure
comes from a ledger in the store, one entry per folder, filled as folders are read and
corrected by whatever writes, moves or removes one. Option 3 was not taken because an index
can disagree with the folders, and the folders are the index (S1.8). This supersedes the
startup clause of "The timeline is a window onto the list"; the rest of that entry stands.

### Consequences

The window shows at once at any library size, and the list is whole a second or two later
on a warm disk. Cost: for those seconds the history keys stop at the loaded end, and "the
latest reopens" is the last-modified of the fifty newest by creation, so an old document
annotated last with more than fifty newer ones is not the one reopened; Rotem's to veto.
Revisit if the thread's seconds are ever felt, or if a lookup other than Annotate's turns
out to need the whole list.

---

## 2026-09-14 · A bubble is placed from a fixed list of candidates, and the margin is what the notes need, recomputed at rest

### Context

§3.5 asks for a small deterministic set of candidate positions near the anchor, avoiding
the anchor and existing bubbles where possible, and for the canvas to grow with a neutral
margin when a bubble cannot fit, never shrinking the picture. It does not say what wins when
no candidate is both inside and clear, when the margin is recomputed, or whether it ever
shrinks. Each of those changes what a user sees under their hands.

### Options

1. One fixed offset, as the first version had, and a margin set by hand.
2. Candidates in a fixed order; a clear candidate in the margin beats an overlapping one
   inside; the margin is recomputed whenever the notes come to rest, and shrinks back.
3. The same, but the margin only ever grows.
4. Content-aware placement that looks at the pixels.

### Decision

Option 2, by the assistant at S1.6. The order is below right, below left, above right,
above left, right, left, below, above, with gaps proportional to the text size, so the
first candidate is exactly the first version's one place and the S0.6 references stand.
Recomputing only at rest, never during a drag or a keystroke's layout, keeps the picture
from shifting under a bubble being moved; recomputing on every input keystroke is
allowed because typing only grows the bubble downward. Shrinking back means a note dragged
back onto the picture leaves no blank band in the output. Option 4 is what §3.5 rules
out as a prerequisite.

### Consequences

The same clicks on the same picture always give the same layout, which the checks assert
by running them twice. A crowded corner grows the canvas rather than stacking bubbles;
that half of the order is F81 and one line to reverse. The margin is never the user's
to set in v1; the checks set a floor under it. Revisit if daily use shows the canvas
growing when a user would rather have had the bubbles overlap, or if a margin that
appears and disappears as notes move reads as jumpy.

---

## 2026-09-13 · The clipboard is published by the host in three formats, PNG first

### Context

S0.6 delivers the clipboard operation, and part 6a makes acceptance the paste actually
working in Claude and ChatGPT. Tauri ships a clipboard plugin, and the web view has a
clipboard API of its own; either would have been less code than talking to Win32.

### Options

1. The web view's clipboard API: the page writes an image blob.
2. Tauri's clipboard plugin: one bitmap format, written from the host.
3. Win32 from the host: several formats in one transaction, chosen here.

### Decision

Option 3, by the assistant at S0.6. Part 5 keeps the web view off the clipboard, which
rules out option 1 on its own; and a web view can only put a canvas-encoded image on the
clipboard, which is the premultiplied round trip the preserved source must never take.
Option 2 publishes one bitmap; a Chromium page reads the registered `PNG` format first
and losslessly, and older Windows applications read `CF_DIB` only, so one format serves
one kind of destination. Three formats built before the clipboard is touched serve all of
them from the same composite.

### Consequences

`PNG` carries the exact composite, alpha included; `CF_DIBV5` carries it with alpha for
system readers; `CF_DIB` is flattened over white for readers that ignore alpha. Both
destinations took the paste as a PNG on 2026-09-13. The cost is one PNG encode plus two
bitmap copies per copy, 51 plus 22 ms in release for a 5120x1440 capture, and a Win32
module the product owns. Revisit if a destination turns out to need a fourth format, or if
the encode time ever shows on the copy path in S1.12.

---

## 2026-09-13 · Stage 0 verdict: the candidate stack is adopted, every component kept

### Context

The stack was recorded on 2026-09-10 as a recommended candidate, not a decision, with Stage 0
holding permission to reject it (part 8 of the plan). Eight steps ran, and S0.8 read every
part 8 row against their evidence.

### Options

1. Adopt the candidate as measured.
2. Replace one component the evidence accuses.
3. Reopen the stack against part 5's comparison table.

### Decision

Option 1, recommended by the assistant at S0.8 and adopted by Rotem's word to start Stage 1.
No part 8 row fired: the freeze is exact and 41 to 50 ms; the overlay is usable within 87 ms
of the hotkey and the editor within 134 ms of the selection; the export is pixel-identical to
the live editor mid-typing; twenty-eight real files decode; both paste destinations take the
clipboard. Nothing accuses a component.

### Consequences

The 2026-09-10 entry that called the stack a candidate is superseded by this one; the
architecture entries it pointed at stand. Three calls were open when this was written, none of
them a stack question: the 170 MB tray floor (F59), the wide-gamut main display against
the sRGB decision (F60, decided the same day: sRGB stays), and the hotkey against an
elevated application, which only a key press could measure (F62, measured the same day:
it fires). Replacing Tauri later would mean re-implementing the tray, the
global shortcut, the file activation and the updater, as part 8 says; that is the cost
this entry accepts. Revisit if Stage 1 use finds a cost the measurements did not.

---

## 2026-09-13 · A capture stays sRGB on a wide-gamut display; the trigger moves to a managed desktop

### Context

The sRGB decision of 2026-09-10 named one trigger for revisiting it: the record of what this
machine's displays are. S0.8 made that record, and the main display is wide gamut, 132% of
sRGB's area by its own EDID primaries. Rotem was asked whether a capture should stay sRGB or
be tagged with its display's profile, and delegated the call.

### Options

1. Keep treating a capture as sRGB, untagged, as every other screenshot tool does.
2. Tag each capture with the profile of the display it was taken on.
3. Convert each capture from the display's profile to sRGB at freeze.

### Decision

Option 1, by the assistant at Rotem's delegation. On a Windows desktop that is not colour
managed, an application draws sRGB numbers and a wide-gamut panel shows them more saturated
than meant; the freeze copies those numbers. Treating them as sRGB keeps the application's
intent, which is what a designer annotating a UI is judging, and what the people who receive
the paste see on their own screens. Option 2 would make the paste reproduce the panel's
exaggeration everywhere else. Option 3 would change the very numbers the application drew,
which is the rule 11 discipline broken at the freeze.

### Consequences

Nothing changes in the pipeline. The revisit trigger changes: a desktop that Windows colour
manages, with automatic colour management or HDR on, stops handing plain sRGB numbers to
GDI, and `host/src/platform.rs` reads that state per display. When that state is seen on
the machine, or a client's colours are questioned, this entry is the one to supersede.

---

## 2026-09-14 · The web view's security posture: a strict policy, and the region scheme answers the page's origin only

### Context

Stage 0 ran with no content security policy and with the region scheme, which serves the
screen's pixels, answering any origin (review T12). Fine while the only page is Recon's own
and nothing remote loads; not the setting an open-source release ships with. S1.1 is where
the product path first builds the window, so the plan put the decision here.

### Options

1. Leave both open and rely on the page never loading anything remote.
2. A policy that names what the page uses and nothing else, and a region scheme that
   answers the page's own origin only.
3. A policy plus a nonce per load and a token on every region request.

### Decision

Option 2, by the assistant at S1.1. The policy allows the page's own origin for scripts
and styles, inline styles (the export writes computed styles inline), images from data and
blob URLs (the export layer travels as a data URL), and connections to the region scheme
and Tauri's IPC origin; objects, frames, form posts and other bases are refused. The region
scheme's CORS header names the page's origin. Option 3 buys nothing while the page is the
only code in the web view and costs a token on the hot path.

### Consequences

Every editor check passes under the policy, on a 100% and a 225% display. The S0.3 bench's
local socket route is refused by it (F76); that step is closed and the bench runs with the
policy off if it is ever needed. Anything the page loads in future, a web font or a remote
resource, must be added to the policy deliberately, which is the point. Revisit if a second
page or remote content ever enters the web view.

---

## 2026-09-14 · Previous captures are kept in memory, encoded and capped, until the store exists

_Superseded on 2026-09-14 by the store entry below: the cap and the in-memory list are gone._

### Context

§3.3 says a new capture never overwrites the previous one, and the store that keeps
documents on disk is S1.8. Stage 0 simply dropped the previous capture. S1.1 had to keep it
somewhere, and thirty raw captures a day would be a gigabyte of memory, which §3.8 forbids.

### Options

1. Keep dropping the previous capture until S1.8.
2. Keep every previous capture raw in memory.
3. Keep previous captures PNG-encoded, on a thread off the capture path, capped in number.
4. Bring S1.8's disk store forward.

### Decision

Option 3, by the assistant at S1.1, as a stopgap this entry names as one. A full-screen
capture encodes to about 3.5 MB in 50 ms in release, on its own thread; fifty of them is
under 200 MB, and the fifty-first drops the oldest with a log line. The page keeps each
document's notes by the host's document number. Option 4 would have pulled the schema and
the save discipline of S1.8 into a step about the lifecycle.

### Consequences

Nothing captured in a session is overwritten by the next capture, and S1.9's navigation has
something to navigate. What is lost: a capture past the cap, and everything at quit, until
S1.8. That entry supersedes this one when the store lands; the cap and the in-memory list go
with it. An opened file is never retained here, because an external file is never taken.

---

## 2026-09-10 · Why the required format set is affordable, superseding one paragraph

### Context

The entry above that names the required format set justified it with "all seven decode in the
web view, so requiring them costs little". The decode route then changed to one native path,
which makes that reason false while leaving the set itself right.

### Options

1. Leave the reason standing, since the membership did not change.
2. Supersede the reason explicitly, so nobody later defends the set with an argument that no
   longer holds.

### Decision

Option 2. The membership of the required set is unchanged: PNG, JPG, GIF, WebP, BMP, AVIF and
SVG required, HEIC and HEIF required to attempt, TIFF deferrable.

### Consequences

The set is affordable because `image` covers five of the seven and `resvg` and libavif cover
the other two, all inside the host's own toolchain, not because a browser was going to do it
for free. The price is four decode dependencies and their security watch, which part 11 of
the plan now records as a standing cost rather than an absence. Nothing about the membership
moves, and dropping a required format is still a scope decision that comes back to Rotem.

---

## 2026-09-10 · The stack is a recommended candidate, not a settled decision

Supersedes the stack entry above, on its status and on part of its argument. The
recommendation itself is unchanged.

### Context

The earlier entry recorded the stack as chosen. Its argument contained three claims that
were not established: that the alternatives would need a bidirectional editing model written
from scratch, that a native UI leaves this project on manual verification forever, and that
the SVG renderer follows from the host language. Qt's own documentation shows an editable
text item inside its graphics scene. WPF and Qt both expose automation interfaces for their
standard controls. `resvg` ships a C interface, so any host can use it. And the third
argument, this project's browser-driven QA gate, is a tooling habit to price and adapt, not
a product requirement.

### Options

1. Keep it recorded as decided, and let Stage 0 confirm it.
2. Record it as a recommended candidate pending validation, correct the comparison, and give
   Stage 0 explicit permission to reject it.

### Decision

Option 2, on Rotem's direction.

### Consequences

The recommendation does not move, and the reasons that survive are real: the decode surface
belongs to the host, the engine supplies the editing model rather than the plan having to,
and the existing verification tooling runs on a document tree. What changes is their weight,
and the plan's honesty about them. Part 5 now names what is not established and carries a
comparison of concrete implementations, documentation first, prototyping only the
consequential uncertainties in the strongest alternative. Part 8 gains a row that reopens the
editor architecture or the stack on combined implementation and maintenance cost, whether or
not any single gate failed, and notes that replacing Tauri means re-implementing the tray,
the global shortcut, the Explorer argument forwarding, the association bundle and the
updater.

One more correction rides along: the cost of one decode route against two was recorded as an
answered fact and is not one. One route stays, because exact equality requires it, and that
reason needs no cost claim.

Revisit at the S0.8 report, which is now allowed to recommend against this stack.

---

## 2026-09-11 · A note's text size is the user's, with no minimum on-screen size

### Context

Looking at the editor for the first time found a note that cannot be read at a
fit-to-window view of a wide capture (F46). Annotations are laid out in image pixels and
one transform scales them, which is exactly what keeps zoom from re-wrapping a line, and
the same property shrinks a note along with the image. The editor had one hard-coded size
of 15 pixels and no way to change it.

### Options

1. Leave it: one fixed size, scaling with the image.
2. Give a note a minimum on-screen size, so it stops shrinking past a floor.
3. Give the size to the user: a default of 20 image pixels, adjustable per note.

### Decision

Option 3, Rotem's call on 2026-09-11. The default is 20 image pixels and the size steps
along a fixed ladder with `Ctrl +` and `Ctrl -`, on the note being edited or the selected
one. The size last used becomes the next note's default.

### Consequences

The architecture keeps the property it depends on: layout is computed in image pixels
before the transform exists, so zoom still cannot re-wrap a line, and the export is that
same layer at that same size. Option 2 was the one to reject for a reason beyond taste:
an on-screen floor applies at a display zoom, the export has one scale, so the two would
disagree about what a note looks like, and a note would cover more of the picture at one
zoom than at another.

The cost is state. A note's size has to survive an internal save, a reopen and an export,
so the S1.8 schema carries it per note, and the export takes the size from the note rather
than from a stylesheet. Everything about the bubble that used to be a fixed pixel value,
its padding, its number badge and its corner radius, is now a proportion of the text size;
at 20 those proportions are the values the first version hard-coded, so nothing moved at
the default.

What this does not solve: at a fit view of a 5120-pixel-wide capture, 20 image pixels is
five pixels on screen, so the default alone does not make a note readable there. The
answer today is the size control and the zoom. Revisit if real use in S1.12 shows the size
being raised on nearly every note, which would argue for a default derived from the
image's own dimensions rather than an absolute one.

---

## 2026-09-13 · The display proxy is served from a pyramid of halves, by one worker

### Context

Part 5 leaves the display representation to measurement, with one policy: the visible
region always carries the detail its zoom implies, and a stale answer is dropped. S0.4
served every request by resampling the full image, and a fit view of a 4K image cost 1.1 s
(F43). The review also found that every request spawned its own thread, so a held key did
the work ten times over (T8).

### Options

1. Keep resampling the full image per request, and only fix the build profile.
2. Tiles: cut the image into fixed tiles per zoom level and serve the visible ones.
3. A pyramid of halves, built on demand, with one region worker that serves only the
   newest request from the nearest level at or above the requested scale.

### Decision

Option 3. Chosen here, as the smallest change that meets the policy, and open to revisiting
at S0.7. The profile fix rode along because it was the real cause of the second: a generic
resample is compiled in the calling crate, so the per-pixel work moved to `host/pixels`,
a crate optimised in every profile.

### Consequences

A fit view is 47 ms and a level is built once per image, 67 to 82 ms for level 1 of a 4K
image, on the worker rather than on the capture path. Nothing is ever enlarged: the level
chosen is always at or above the requested scale, and the 1:1 path is a plain crop, so the
checkerboard check still holds. One worker means one resample in flight and no pile-up;
a replaced request is answered as superseded and the page already drops it by ticket.
Cost: memory for the levels, a third of the image again at most, held for the life of the
image; and a first fit view that pays for level 1. Tiles would cap memory per view and
allow partial repaints, which is what would make them worth it: revisit if S0.7's memory
slope or a very large image (S0.5's stall test) argues for them.

---

## 2026-09-13 · AVIF decodes through the Windows imaging stack, not a bundled libavif

### Context

The plan named libavif with a dav1d backend for AVIF, so that a required format would not
depend on a codec installed on the machine (part 6c). S0.5 could not build that chain: it
needs a C toolchain (cmake, nasm, meson) this machine does not have, and the three pure-Rust
AV1 decoders on crates.io do not compile on Rust 1.98. AVIF was wired through the Windows
imaging stack instead, which needs the AV1 Video Extension, and on real files it decoded
every still, 4K, 10-bit and alpha included, but returned one frame of an animated AVIF
(F51, F53). Rotem was asked on 2026-09-13 and delegated the call.

### Options

1. Accept the Windows imaging stack for AVIF: no build dependency, an installed codec at
   run time, stills complete, animation as a first frame, the named-codec message when the
   codec is missing.
2. Install the C toolchain and build libavif with dav1d: the format decodes with nothing
   installed, animation included, and every contributor of the open-source release needs the
   same toolchain to build the host.
3. Wait for a pure-Rust AV1 decoder that compiles here, and ship without AVIF until then.

### Decision

Option 1, by the assistant at Rotem's delegation. HEIC already takes exactly this route with
the same degraded state, and part 6c already counts the named-codec message as a pass for
it; AVIF joins that clause rather than opening a new one. The AV1 Video Extension ships with
current Windows 11, and this machine had it without anyone installing it. An animated AVIF
has not been seen in the daily use the product is for: screenshots, client material and
phone photos. Option 2 buys that rare case at the price of a C toolchain on every machine
that builds Recon, forever. Option 3 drops a required format for an unknown wait.

### Consequences

AVIF stays required; its degraded state is the message that names the codec and the store,
and an animated AVIF shows its first frame, said in §3.7 and part 6c. The build stays pure
Rust, with three decode dependencies instead of four. The provider sits behind the one image
source interface, so moving AVIF to another decoder later touches one file and no caller.
Revisit when a pure-Rust AV1 decoder compiles on the project's toolchain, or when daily use
(S1.12) meets an AVIF this route cannot open; either one reopens F51 and F53 together.

---

## 2026-09-10 · The minimum format set for the daily-use release

### Context

Stage 0 is allowed to come back with a shorter format list than the target. As written, that
let one experiment quietly drop any format, including one the daily viewer cannot do
without, and call it evidence.

### Options

1. Leave the list open and judge each gap when it appears.
2. Name the formats the release requires, so a gap in one of them is a blocked stage rather
   than a shorter list.

### Decision

Option 2, on Rotem's direction. The set: PNG, JPG, GIF, WebP, BMP, AVIF and SVG are
required; HEIC and HEIF are required to attempt and allowed to degrade with a named missing
codec; TIFF, with its pages, is the one row a Stage 0 result may defer.

### Consequences

S0.4 can report that a format is expensive or unavailable, and it cannot decide to ship
without a required one; that comes back to Rotem as a scope decision and lands here. The
seven required formats were chosen because the web view decodes all of them, so requiring
them costs little beyond the decoded-image contract. TIFF is deferrable not because it is the only
host-provider format, since HEIC is one too, but because it is the only one with no
degraded pass: a missing HEIC codec has a defined message, and TIFF has nothing partial to
ship. Cost: if the web view turns out to decode less than
expected on this machine, the blocked-stage path is more expensive than a shorter list would
have been, which is the point.
Revisit when S0.4 reports, or when real use shows a format nobody opens.

---

## 2026-09-10 · The stack: a Rust host owning the pixels, a web view laying out the text

### Context

The plan named Tauri, React and Rust and justified them with the owner having used that
stack on another project. That is not a reason, and rule 21 of CLAUDE.md forbids treating
another project as evidence about this one. What the choice has to answer is fit for this
product and years of maintenance by one person.

### Options

Five were evaluated against the plan's twelve requirements, each one attacked afterwards by
a second reviewer:

1. A Rust host with windows-rs plus a WebView2 editor, packaged with Tauri v2.
2. C# on .NET with WinUI 3. WPF was struck: no text-layout surface of the kind needed, no
   SVG.
3. C++ with Direct2D and DirectWrite.
4. Qt 6, standing for the cross-platform toolkits. Avalonia and Flutter were excluded on
   format coverage and on per-display window support.
5. Electron with native addons.

### Decision

Option 1. Chosen here, on the argument in part 5 of the plan, and open to Rotem's veto.

### Consequences

Three requirements decided it. Decoding nine formats with orientation and color applied once
is the second-largest surface in the product and it is a host problem: Rust answers it in one
toolchain with most of the parse path memory-safe, while every alternative writes interop for
four native libraries, and two of them render SVG wrongly and silently, which matters because
annotating diagrams is a core case. Mixed Hebrew and English needs an editing model, not just
a layout engine, and only a browser gives the caret, the selection geometry and the direction
handling without hand-writing them. And this project's own QA doctrine, the browser-driven
gate in Workflow.md step 9 and the accessibility checks in QA.md section 6, only functions on
a document tree; on a canvas toolkit every visible change would fall to a manual checklist
forever.

Costs, all real: a web view memory floor before the image is even loaded; a runtime that
updates fortnightly, so reference images need re-baselining and re-baselining is where
tolerances get quietly widened; the packaging layer's own taxes, including file associations
that need hand-written installer hooks rather than a config key; and four decode dependencies
to watch for security fixes.

Tauri is the least load-bearing part: replacing it later with a direct web view host is a
packaging decision, not a stack decision. The alternative, and the exact trigger that
switches to it, is one row in part 8 of the plan: move the composer in-process to Direct2D
and DirectWrite inside the same Rust host, keeping every other component. Revisit at the S0.6
result, or if the boundary measurement in S0.3 accuses the crossing.

---

## 2026-09-10 · One native decode route, and the original never enters the web view

### Context

The plan's first answer was to let the web view decode the seven formats it can and give the
host only TIFF and HEIC. That cannot coexist with the rest of the plan: the store is native,
the document keeps the decoded frame, and an export has to carry the original pixels with
exact equality rather than a tolerance.

### Options

1. Two providers: the web view decodes what it can, the host decodes the rest.
2. One native route for all nine formats, with the web view decoding nothing.

### Decision

Option 2, which flips the plan's earlier recommendation.

### Consequences

A canvas is premultiplied. A straight-alpha pixel at low alpha does not survive the round
trip, so a preserved image created inside the renderer cannot be byte-identical for any image
with transparency. An opaque capture round-trips perfectly, which is exactly why the check
written against a capture could not have caught it; a semi-transparent case is now part of
S0.6. Beyond correctness: orientation-once and sRGB-once get exactly one place they can be
wrong, S0.5's matrix narrows to one route, and exact equality becomes structural, because the
host copies the untouched source and blends the annotation layer over it.

The rule this pins, from day one and before any editor code exists: the decoded original
never enters the web view at full resolution and never round-trips a canvas. The web view
gets a display-resolution proxy; the export emits only the annotation layer; the host
composites. Retrofitting that would rewrite the composer's relationship to the image.
Cost: four decode dependencies in the host and their security watch, forever, which is the
cheapest available version of that cost rather than its absence.
Revisit only if the boundary measurement in S0.3 shows the proxy path cannot serve accurate
annotation.

---

## 2026-09-10 · The annotation text layer is DOM, and the export is that same DOM

### Context

The requirement that the editor and the exported image agree exactly on wrapping, alignment
and geometry, in mixed Hebrew and English, is the plan's hardest. Two render paths for the
same text is how that requirement fails, and the usual shortcut, a text input floating over a
canvas that draws the text separately, is exactly the failure.

### Options

1. Draw the text into the canvas and implement the caret, selection and direction handling.
2. Keep the text as DOM for editing, and serialize that same DOM for the export.

### Decision

Option 2. Bubbles are absolutely positioned in image-space pixels, base direction comes from
`unicode-bidi: plaintext` with a per-bubble override, display zoom is a CSS transform on the
container, and the export serializes the subtree into an SVG `foreignObject` at the original
dimensions.

### Consequences

Layout is computed before the zoom transform exists, so zoom cannot re-wrap text: the worst
version of the failure is removed by construction rather than by discipline. The editing
model, caret placement, selection rectangles, movement across a direction boundary and the
platform's editing keys, comes from the engine instead of being written by hand, which is the
largest single work item in every alternative.

Three things become mandatory, and each is a silent failure if skipped: fonts inlined as
base64 in the serialization, because an SVG loaded as an image fetches nothing external and
substitutes silently; every bubble style inlined rather than inherited; and caret and
selection decorations that cannot affect layout. The bubble style vocabulary stays small,
because rasterizing embedded HTML has a long tail. This path deliberately avoids the newer
canvas text-metrics APIs, which are not settled across engines.
Revisit at the trigger in part 8, or if the export route stops being origin-clean and no
same-engine replacement works.

---


## 2026-09-10 · Callout numbers never change, gaps included

### Context

Callouts are numbered by creation order, and deleting one leaves a gap, so a feedback
list can read 1, 2, 4, 7. The plan review proposed renumbering at export or at send so
the client would see a clean sequence.

### Options

1. Renumber automatically at export or at send.
2. An explicit Renumber action, triggered by the user.
3. Keep stable numbers, gaps included, in every output.

### Decision

Option 3, chosen by Rotem.

### Consequences

A number, once shown, means the same thing in the editor, in the clipboard, in an
exported file and in a Rogers task, so a reference made in a chat message or a task
comment stays valid for good.
Cost: a list can look untidy after deletions, and that is accepted.
Future work must never derive a displayed number from a position in a list, and must
never renumber on any output path.
Option 2 stays a backlog idea if real use asks for it.
Revisit only if a client actually misreads a gap.

---

## 2026-09-10 · The capture API minimum is not the product support baseline

### Context

The per-display capture call documents Windows 10 version 1903 as its own minimum, and
the first build plan treated that one number as the versions Recon supports, the
versions it tests, and the installer strategy all at once.

### Options

1. Adopt the API minimum as the product baseline and let it settle packaging too.
2. Keep three separate answers: the API constraint, the supported and tested versions,
   and how the web view runtime reaches a user.

### Decision

Option 2, chosen by Rotem.

### Consequences

The API minimum is stated where that API is chosen and constrains that component only.
The supported and tested versions become a product decision made with Stage 0 evidence,
and runtime packaging is a distribution decision that cannot block the experiment.
Cost: three answers to track instead of one.
Future work must not quote an API requirement as a support promise.
Revisit when the capture path is chosen for good.

---

## 2026-09-10 · Image viewing is a core capability, not a later addition

### Context

The product plan deferred opening existing images, listing it among the things to
evaluate from observed use. Rotem then clarified that Recon is his primary everyday image
viewer as well as his capture and annotation tool, and has to open common formats
straight from disk.

### Options

1. Keep viewing out of v1 and revisit once the capture loop is in daily use.
2. Ship a separate viewer application beside Recon.
3. One window with two entry behaviors: a capture opens ready to annotate, a file opens
   in viewing mode behind an explicit Annotate action.

### Decision

Option 3, chosen by Rotem.

### Consequences

Format support becomes a product surface. Each format is stated per row, and one that
cannot make the first release is named as a gap with its impact instead of being covered
by a claim about common images. Two of the target formats have no browser decoder, so
decoding needs two providers behind one interface and three decisions are open in
`project-os/Plan.md` part 6b, items D to F.

External files gain the invariant in CLAUDE.md rule 11: viewing never modifies and never
imports, so a folder of originals is safe to point Recon at.

Stage 0 gains a decoding experiment, and Stage 1 grows from seven items to twelve,
including PNG Save As, because an annotated external image has to be saveable on day one.

Cost: the first daily-use release is larger than it was, and the viewer has to be good
enough to replace an existing one rather than merely present.

Revisit only if the decoding evidence in S0.4 shows the viewer is impractical on this
stack.

---

## 2026-09-10 · One document per source, resumed rather than duplicated

### Context

Open `a.png`, annotate it, open `b.png`, come back to `a.png`. The plan did not say whether
that shows the original file, the saved edit, or starts a second edit. All three were
reachable readings, and two of them lose work or hide it.

### Options

1. Reopening the file shows the saved edit, since that is the newer work.
2. Reopening shows the original, and annotating it starts a second document.
3. Reopening shows the original, with a route to the saved edit, and annotating resumes the
   one document that already exists for that path.

### Decision

Option 3, on Rotem's direction that history holds every managed document and that an
external open shows the original with a clear route to its saved edit. The
no-second-document half is the reading taken here; say so if a second edit of one file
should be possible.

### Consequences

Recon never holds two managed documents for one source path, so there is never a question
about which edit is the truth. A document stands on its own preserved decoded image and
never re-reads the source, so a file that changes or disappears on disk cannot alter or
break the saved work; the document says the source has moved on and changes nothing.
Cost: a user who wants two different annotated versions of one file has to export the
first, and v1 gives them no other route. Future work must not add a second document for
the same path without revisiting this.
Revisit if a real workload wants two edits of one original.

---

## 2026-09-10 · Save As never overwrites anything

### Context

The keyboard and output contract promised that Save As always writes a new file, and in the
same section allowed overwriting a file the user picked by name in the dialog. Those cannot
both hold, and the second one is a path to writing over an external original, which rule 11
forbids.

### Options

1. Keep the exception: the user chose that name, so honor it.
2. Keep the exception but block it when the chosen name is the source file.
3. Remove the exception. Save As writes a new file, and an existing name gets an available
   one offered instead.

### Decision

Option 3, on Rotem's direction, for the current v1 scope.

### Consequences

The product contract and the acceptance tests agree, and there is no code path that can
write over a file Recon did not create. Option 2 was rejected because a rule with one
exception needs the exception tested, and "is this the source file" is exactly the check
that gets subtly wrong. Cost: a user who genuinely wants to replace an earlier export does
it in Explorer. Future work must not reintroduce a confirm-and-overwrite dialog.
Revisit only if replacing an export becomes a real friction in daily use.

---

## 2026-09-10 · An SVG raster size is fixed when annotation begins

### Context

Annotating a vector file has to produce pixels, and the plan said "at the displayed size".
That made the annotation resolution depend on the window size and the zoom at that instant,
which contradicts the rule that display zoom never changes export resolution.

### Options

1. Rasterize at a fixed nominal size, ignoring the view.
2. Rasterize at the displayed size, recomputed whenever the view changes.
3. Rasterize once, at the displayed size in physical device pixels at the moment Annotate is
   pressed, then show and store those dimensions and never change them.

### Decision

Option 3, on Rotem's direction to define the dimensions at annotation time, expose them and
persist them.

### Consequences

What the user was looking at is what they get, and a later zoom, window resize or reopen
cannot change the export. Physical device pixels rather than CSS pixels, so a 150% display
gives the pixels that display was actually showing. This is the one place where an export
resolution comes from the view instead of the source, and it is documented as an exception
because a vector has no pixel size of its own. Cost: annotating the same SVG twice at
different window sizes gives two different resolutions, which is honest but has to be
visible, hence showing the dimensions at that moment. Future work must not recompute a
document's raster size after it was stored.
Revisit if a chosen output size is wanted instead.

---

## 2026-09-10 · sRGB is the working color space for v1

### Context

Images arrive carrying embedded profiles, and captures arrive carrying whatever the display
was. Without one working space named, color shifts silently between viewing, annotation and
export, and the same file looks different after a reopen.

### Options

1. Carry each image's own profile through the viewer, the composer, the clipboard and the
   export, converting only at the very end.
2. Convert everything to sRGB once, at decode, and work in sRGB everywhere after that.

Option 1 is the correct answer for a color-managed image editor. It also needs color
management in the canvas, in the clipboard formats and in every export path, and most
destinations Recon feeds, including a browser-based chat, will treat what they receive as
sRGB regardless.

### Decision

Option 2, sRGB, converted once at decode. Chosen here, not by Rotem, as the ordinary
implementation reading of his instruction to name an explicit working color space.

### Consequences

Color is predictable and identical in the viewer, in an annotated export and after a
restart, which is what the S0.4 cycle test checks. Cost: a wide-gamut source is flattened
to sRGB, and a capture taken on a wide-gamut display is treated as sRGB when it is not,
which can shift the very UI colors a designer is reviewing. S0.7 records what this
machine's displays are, and that record is the trigger for revisiting this.
Revisit if the test machine turns out to be wide gamut and the shift is visible.

---
