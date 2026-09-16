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
- 2026-09-14 · No tool is in hand when a picture opens, and a chosen one stays until the next picture.
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
