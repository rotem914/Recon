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
- **This file rotates, the way History does.** `project-os/Archive-old-rows.mjs` (or its `.ps1` twin)
  keeps the newest 25 entries live at `Go commit` and moves older ones into
  `Decisions-archive.md`, under its own `## Archived decisions` heading. Never
  move an entry there by hand: the script creates the archive the first time
  it is needed. Those entries still BIND the project; they only aged out of the
  live read, so treat one exactly as if it were still here. The Index keeps its
  line for every one of them, so nothing becomes invisible. An archived entry
  still binds; if its Index line is in italics, only the part that line names
  as replaced no longer holds. The `## Archived decisions` heading in the
  archive says so.
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
- 2026-09-14 · _The window under the pointer is the selection until a drag begins, listed once with the overlay, told from a drag by the system's threshold._ (its clause leaving Recon's own windows out is superseded on 2026-09-19; the rest stands)
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
- 2026-09-19 · A second capture shortcut sits in its own row, Capture 2, empty until picked and cleared by its ×.
- 2026-09-18 · The Open Recon shortcut, pressed again, minimizes the window only when it is the one in front.
- 2026-09-18 · Save As on an annotated PNG or JPEG opens on the file itself and writes over it, Rotem's exception to rule 11; every other existing name is still never written over.
- 2026-09-18 · A file opened by name sits in the timeline as a pointer to where it lives, in a list of its own beside the tabs, never in the store; Rotem's call over a kept copy.
- 2026-09-18 · The mode follows what is done: a tool picked starts the annotation, a finished element puts the tool down, and the Annotate button and the A key are gone.
- 2026-09-18 · The colour picker's HEX is put on the clipboard by the host from a colour the page names, never a text; the picker starts no annotation.
- 2026-09-19 · A capture can pick Recon's own editor window like any other; only the capture overlay itself is left out, Rotem's call.
- 2026-09-19 · The wheel stretches the picture already painted and asks for one region at a time; the host says once whether a picture has a see-through pixel.
- 2026-09-19 · The magnifier is one unit in the host: the page asks for the finished panel and paints it, Rotem's choice of two.
- 2026-09-19 · Rotem's inspector is in his own builds of Recon only, behind a build switch, never in the public release.
- _2026-09-22 · The captured pointer is an element in the notes carrying its own picture, never pixels of the capture; Rotem's choice over a pointer burned in._ (superseded the same day, below)
- _2026-09-22 · The pointer is drawn into the frozen pixels after all, before the overlay, so the area is chosen on the picture that is kept; Rotem's word._ (where and when is superseded the same day, below; embedded stands)
- 2026-09-22 · The embedded pointer is the ordinary arrow, drawn where the mouse was at the selection, not where it stood at the shortcut; Rotem's word.
- 2026-09-22 · A scrolling capture is the person's own scroll, joined by numbers of rows in strips that vote; Rotem's pick over Recon scrolling by itself.
- _2026-09-22 · Whether an area scrolls is asked off the overlay's thread, by its scrollbar, by what it says, or by content taller than it; and Recon's own windows stay out of the copies by Windows' exclusion._ (its clause handing the capture the area the application names is superseded the same day, below; the rest stands)
- 2026-09-22 · A scrolling capture's area is the lit frame, whole; what the application says scrolls only decides whether the button shows.
- _2026-09-22 · Only a row that appears once is evidence of a move, standing still is a move of its own, and a panel scrolls when its children run past it together._ (its consequence that a video says "Too fast" while nobody scrolls is superseded on 2026-10-14, below; the rest stands)
- 2026-09-22 · A capture taken with Ctrl held reaches the page as any capture does, with the window left down; a copy that fails brings it up.
- 2026-09-27 · Show Recon after a screenshot, unchecked, is the Ctrl capture's path for every capture, the scrolling one included.
- 2026-09-30 · The pixel ratio is never measured from a minimized window; the one owed is taken at the window's next resize.
- 2026-09-30 · A tab's order is its own list's, and a drag moves one picture to the place under the hand.
- 2026-09-30 · Everything with the blue line is one batch, the picture on screen included; a batch deleted is one deletion of undo, its pictures under one number.
- 2026-09-30 · A Ctrl+Z or Ctrl+Shift+Z pressed while pictures come back, or are deleted again, is let go; a thing done again takes a new number, and undo picks the newest by number.
- 2026-10-01 · A Ctrl+Z pressed while a delete is on its way is let go too; the delete holds the same count the undo and redo of a deletion hold.
- 2026-10-01 · Each line of a bubble left automatic takes its own direction, by `unicode-bidi: plaintext`; an override still sets the whole bubble. (Supersedes §3.5's "applies to the whole bubble" for automatic.)
- 2026-10-06 · The commit waits for `Go commit`, the kit's new rule 22 adopted; the release build follows that commit.
- 2026-10-07 · _A capture's timing lines are kept in `%LOCALAPPDATA%\Recon\timing.log`, capped at 4 MB with one older file; never the window or the files opened._ (the file also holds the two first-open outcome lines since 2026-10-26, below, counts only; the rest stands)
- 2026-10-14 · A scrolling capture calls a page lost only once it rests out of reach; a page drawn between whole pixels is on its way, never placed and never lost.
- 2026-10-15 · _The Recon Rotem runs lives in `app/`, outside the build folder, with the release before it kept in `app/previous/`._ (its clause re-ticking Start with Windows by hand is superseded on 2026-10-19, below; the rest stands)
- 2026-10-15 · _A show refused the foreground asks again for a second and a half, never takes it by force._ (the right handed over to the Recon in the tray is superseded on 2026-10-26, below; the asks and the refusal to take it by force stand)
- 2026-10-19 · The everyday copy re-points Start with Windows to itself at every start, told by its folder's name, `app`.
- 2026-10-20 · The kit's reply rules take the numbers after Recon's own, 20 on; their numbers are mapped at every update.
- 2026-10-26 · The second instance raises the editor window itself, with the right the click gave it, and the hand-off message is Recon's own; both outcomes go to the timing log.

---

## 2026-09-19 · The magnifier is one unit in the host: the page asks for the finished panel and paints it, Rotem's choice of two

### Context
The zoom circle with the text above it stood in three places, the capture overlay, the Ruler and the Color picker, drawn by two pieces of code: the overlay's in Rust and a copy of it in the page's JavaScript. Every value Rotem tuned had to be changed twice, and the two had already drifted in their comments. He asked for one unit, updated in one place.

### Options
1. One drawing: the host draws the whole panel for all three, and the page shows the picture.
2. One list of values handed to the page, the circle still drawn twice.

### Decision
Option 1, Rotem's, of the two put to him. `host/src/magnifier.rs` holds every number, the layout, the text and the drawing. The overlay hands it the frozen pixels around the pointer and blends the panel onto its window; the page asks `region://?panel&...` with the picture's pixel, the part that may show, its scale and what stands above the circle, and paints the bytes. The answer carries the panel's size, the gap to the pointer and where the circle is, so the page places it and the checks read it without a number of their own. The text is drawn by Windows' text drawing in both, so the bundled Google Sans is unpacked to a .ttf and given to the process alone from memory; its spacing is measured from a digit's own top and the line it stands on, because this face's line is more than twice as tall as its digits.

### Consequences
A value changes once. In the editor the panel arrives with the host's answer, so it shows a moment after the pointer enters the picture, and a change of its text lands with the next answer. The placing rule, below and right by the gap and flipped at an edge, is still written twice, in `place_panel` and in the page's `placeMag`, since the page must move the panel with every pointer move without waiting; the gap itself comes from the host. The page's own colour picker click still reads its pixel by the older `mag&` request. Revisit if the moment's wait is seen, or if the placing rule ever changes.


## 2026-09-19 · Rotem's inspector is in his own builds of Recon only, behind a build switch, never in the public release

### Context
Rotem wanted his Rocket Inspector extension in his everyday Recon, not only in the demo window of the check build (`f1e3cbf`). A browser extension runs with full reach into the editor's page, and Recon is meant for a public release.

### Options
1. The product loads whatever folder an environment variable names, in every build. Anyone who can set that variable makes every copy of Recon load their code.
2. A build switch, `own-extensions`, off by default: only a build made with it has the code at all, and it still needs `RECON_PRODUCT_EXTENSIONS` to name the folder.
3. The demo window only, as landed.

### Decision
Option 2, Rotem's GO after the costs were put to him. The variable has a name of its own, not the demo's `RECON_EXTENSIONS`, so setting it for the whole user never keeps a session's demo run open. The window gets a profile of its own, `%LOCALAPPDATA%\Recon\webview-ext`: WebView2 refuses a second process on one profile whose options differ (documented, not measured here), and the check runs use the default profile while his Recon is open.

### Consequences
The release Rotem runs is built with `--features own-extensions` (`project-os/Workflow.md` step 16); a public release is built without it. With the extension loaded, the key left of 1 belongs to the inspector, so it types nothing in a note, and in Hebrew that key is the semicolon. The height of the timeline, which the page keeps in the browser's storage, starts over once in the new profile.

## 2026-09-22 · The captured pointer is an element in the notes carrying its own picture, never pixels of the capture

### Context
Rotem asked that a capture include the mouse pointer. A screen copy never holds it, and
Recon freezes the screen at the shortcut, so the pointer's place is where it stood then,
often inside the area unasked. The capture's pixels are the thing Recon promises to keep.

### Options
1. Draw the pointer into the frozen pixels: a small build, and it can never be taken off.
2. An element on top that can be moved and deleted, as Snagit does.
3. For either: in every capture, or behind a switch in Settings.

### Decision
Option 2 with the switch, Rotem's, on 2026-09-21. The rest is the assistant's, his to veto.
The element is a shape of kind `pointer` in the document's notes, its picture a PNG inside
them, so saving, the copy, the thumbnail, the crop and undo reach it with no path of their
own, and the document format stays at schema 1. The host hands the picture over once,
through the region scheme, so a return to the capture never adds a second one. The arrival
is part of the capture: the history starts with it, so Ctrl+Z never takes it off, while a
move or a delete is a step. It comes along when any part of it is inside the selection. The
switch is on by default. Its alpha comes from drawing it on black and on white, since
Windows hands a pointer over as something to draw; a pixel that inverts what is under it,
the text pointer's, is kept as solid black.

### Consequences
The overlay does not show where the pointer will land, since the frozen picture has none.
A blur drawn over the pointer blurs the capture under it and leaves the pointer sharp, as
the blur works on the source. A pointer hanging over the capture's edge is cut at the edge
in a copy, as a ruler is. Every undo step of such a document carries the picture, a few
kilobytes. Revisit if the pointer should show during the selection, if a pointer should be
addable by hand to a capture that has none, or when the size on a scaled display and an
enlarged pointer have been seen by eye.

## 2026-09-22 · The pointer is drawn into the frozen pixels after all, before the overlay

### Context
The entry above landed the pointer as an element on top of the capture, Rotem's choice of
two. Having it in hand, he said the element is not needed: a plain capture with the ordinary
pointer in it, embedded. This supersedes that entry; its reading of the pointer from
Windows, and the Settings switch, stand.

### Options
1. Draw the pointer into the capture after the area is chosen: the frozen screen shown
   while choosing has no pointer, and the capture then has one.
2. Draw it into the frozen screen at once, before the overlay goes up: the pointer is seen
   while choosing, where it will be in the capture.

### Decision
Embedded: Rotem's, on 2026-09-22. Option 2 is the assistant's, his to veto: what is seen
while choosing is what is kept, and the magnifier shows the same pixels the capture holds.
The element's code is taken out whole, page, host and checks, rather than left unused.

### Consequences
The pointer cannot be moved or taken off a capture; the switch in Settings is the only way
to a capture without one, and it holds from the next capture. The preserved image of such a
capture is the screen with the pointer, by design, and nothing of the screen under the
pointer is kept. A capture made while the element existed keeps a `pointer` shape in its
notes, which the page no longer draws, as it draws nothing for a kind it does not know; its
pixels never held the pointer, so it now shows none. The scrolling capture another session
is building reads the screen again after the area is chosen, so it holds no pointer.
Revisit if a pointer is ever wanted off one capture after the fact.

## 2026-09-22 · A scrolling capture is the person's own scroll, joined by numbers of rows in strips that vote

### Context
Rotem asked for a capture of a whole long page. A capture here was one frozen copy of the
screen, and the plan kept scrolling capture out of v1 (§3.10). Two questions had to be
settled: who scrolls the page, and how the copies are joined so that pages as they really
are, with a bar stuck to the top, a side menu, a scrollbar, a video, come out as one picture.

### Options
1. Recon scrolls the page itself, sending the wheel until the bottom. Hands free, and it
   breaks on bars that stay, pages that load as they scroll and applications that ignore a
   sent wheel.
2. The person scrolls, and Recon joins what passes. Works in anything that scrolls, and
   the person does the scrolling.
3. For the joining: compare whole frames pixel by pixel at every possible distance. Exact,
   and far too slow thirty times a second on a large area, and one moving column ruins it.
4. For the joining: reduce every row of a frame to a number, in upright strips, let each
   strip say how far the page moved, and take what most strips say.

### Decision
Rotem picked option 2 on 2026-09-22. The assistant picked option 4 for the joining, in
`host/pixels/src/stitch.rs`, where the per-pixel work is compiled fast in every profile.
Rows that stand still while the rest moves are left out of the count; the still rows at
the bottom are kept out of the middle of the picture and end it once; a column at the edge
whose pixels neither stay nor move with the page, in two placed copies or more, is a
scrollbar and is cut off with its flat track; the lowest 12 px of a copy are never kept
from any copy but the last, since Windows blends a window's round corners with what
scrolls under them. A copy that cannot be placed changes nothing, and the last placed one
stays the one to match, so scrolling back up finds the picture again.

### Consequences
A page scrolled faster than two copies share is lost until the person scrolls back, and the
bar says so. A side menu that stays shows as each copy's lower part had it, not once. A
page whose content repeats exactly, a table of identical lines, can be joined at the wrong
distance; the distance closest to the last one wins a tie. Only downward growth is kept: a
page scrolled above where the capture began adds nothing. The picture stops at 30,000 rows
or 80 million pixels, by itself. Revisit option 1 as an addition, never a replacement, if
Rotem wants hands-free capture of plain pages.

## 2026-09-22 · Whether an area scrolls is asked off the overlay's thread; Recon's own windows stay out of the copies by Windows' exclusion

### Context
Rotem's button shows only over an area that scrolls, and the overlay's thread is the one
surface that must never wait (part 5). The answer lives in another application. And once
the capture is live, Recon's dim and its bar are on the very screen being copied.

### Options
1. Show the button on every area, scrolling or not. No question to ask, and a button that
   mostly does nothing.
2. Ask on the overlay's thread as the pointer moves. Simple, and one slow application
   freezes the capture.
3. Ask on a thread of its own, once per window or part, and let the answer arrive as a
   message: the button shows a moment after the area is lit.
4. For the live part: keep every Recon window outside the copied area. Always possible
   only when the area leaves room, and a page area often reaches the screen's edges.
5. For the live part: mark Recon's windows excluded from capture, which Windows 10 2004 and
   later honours in a GDI copy of the screen.

### Decision
Options 3 and 5, the assistant's, on 2026-09-22. Three signs say an area scrolls, tried in
this order: a classic scrollbar whose range is longer than its page; an element on the way
down to the pointer that says it scrolls up and down; an element holding something taller
than itself that reaches more than 48 px past its top or bottom. Measured here: Edge 153
never says an ordinary page scrolls, and reports the content's whole height, which is why
the third sign exists; a Chromium page answers with an empty tree the first time it is
asked, so an empty answer is asked again, four times, 300 ms apart. Measured here too: a
window excluded from capture is absent from `BitBlt` with `CAPTUREBLT`, layered or not. The
area handed to the capture is what the application says scrolls, inside the lit area, and
the lit area whole when that is under 100 px either way.

### Consequences
An application that answers none of the three signs gets no button, and the ordinary
capture is what there is; which applications those are is not established. Content clipped
without being scrollable reads as scrolling, and the button there starts a capture that
never grows, which costs a press. Being asked makes a Chromium application turn its
accessibility tree on until it restarts. On a Windows older than 2004 the exclusion is
refused: the bar then goes outside the area, and the capture is refused in words when
nothing outside has room. Enter and Escape are taken as global keys while it lasts, so
the page under it cannot be sent an Enter meanwhile.

## 2026-09-22 · A scrolling capture's area is the lit frame, whole

### Context
The first build handed the capture the area its application said scrolls, cut to the lit
frame. Rotem pressed the button on a window in the right part of his screen and the live
area came up exactly half as tall as the frame he had pressed it on: the application named
a panel inside the page, and the frame on screen had promised the whole page.

### Options
1. Keep the application's area, and move the lit frame to it before the press, so the
   frame never lies. The picture holds only what scrolls, and the frame jumps about as
   answers arrive, a moment after each area is lit.
2. The lit frame, whole, always. What is framed is what is captured, and whatever stands
   still inside it, a title bar, a side panel, is left to the stitcher.

### Decision
Option 2, the assistant's, as the fix of Rotem's bug on 2026-09-22. The application's
answer now only decides whether the button shows.

### Consequences
A capture of a frame in which only a panel scrolls holds the still parts too: what is above
the panel once at the top, what is below it once at the end, and what is beside it as each
copy's lower part had it. A classic scrollbar that stands inside the frame behind a window
border is cut off only in part; measured on the check's stand-in, 10 px of 18. Revisit
option 1 if Rotem wants the picture to hold the scrolling part alone.

## 2026-09-22 · The embedded pointer is the ordinary arrow, drawn where the mouse was at the selection

### Context
The entry above drew the pointer into the frozen screen where it stood when the shortcut
was pressed. In use Rotem read it as the pointer freezing in place: he wants it where the
mouse is when the area is confirmed, the moment he named the selection, the release that
ends a drag or the click on a lit area. He was told first that after a drag that spot is
the area's corner.

### Options
1. Where the mouse was at the shortcut, as built.
2. Where the mouse is at the selection.
3. A new right click in capture mode that plants the pointer, the area chosen after it.

### Decision
Option 2, Rotem's, on 2026-09-22. Two calls ride with it, the assistant's, his to veto. The
picture is the ordinary arrow of his pointer scheme, since the pointer on screen at the
selection is the overlay's crosshair, which is Recon's and not the screen's. And the spot
travels beside the overlay's outcome, not inside it, so a selection stays a rectangle for
the self test and the scrolling capture, which read one.

### Consequences
A selection's rectangle ends one pixel before the corner a drag was released on, so after a
drag down and to the right the arrow's tip is just outside and none of it is in the
capture; after a drag up and to the left all of it is; a click on a lit window has it
whole. The frozen screen no longer shows a pointer while the area is chosen. The pointer as
Windows showed it at the shortcut is no longer read in the product; the read stays for the
tests. Revisit if the arrow should be pulled inside the area after a drag, or if option 3
is wanted after all.

## 2026-09-22 · Only a row that appears once is evidence of a move, standing still is a move of its own, and a panel scrolls when its children run past it together

### Context
Two bugs on the same evening. Rotem's design system page showed no round button: its middle
panel scrolls, and holds six sections each shorter than the panel, while the detection only
looked for one child taller than its holder. And the check's stand-in page, captured as the
whole lit frame since the morning's fix, came out too tall or too short three runs in three:
recorded copy by copy, every copy was clean, and the stitcher had taken frames that had not
moved, with only the scrollbar fading and the title bar shimmering, for scrolls of 3, 4 or
23 rows. The rows that stood still had been skipped as a stuck bar, and all that was left
to count were the page's blank rows, which line up at many distances.

### Options
1. Loosen the "same frame" threshold. Hides this case and breaks the next one: a page that
   really scrolls beside a still part wider than itself would pass for the same frame.
2. Count only rows whose number appears once in their strip, and try standing still as a
   distance of its own when no move has the strips behind it.
3. For detection: the extent of all children in the holder's width, from the highest top to
   the lowest bottom. One visually hidden link at -9999 px would make anything scroll.
4. For detection: the run of children that follows on from the ones in view, each within
   48 px of the next and reaching 64 px past the holder, both at 100% and grown with the
   display's scale.

### Decision
Options 2 and 4, the assistant's, on 2026-09-22, with three changes the review of that
evening made. Standing still is accepted only when one strip in four at most, and never
more than two, says otherwise: a scrollbar, a border, a caret; more is a page that jumped
too far beside a still part, which is lost and says so. After a stood-still the last placed
frame stays the one to match. And the two distances were first fixed physical pixels, 96
and 48, which a page's spacing outgrows on a scaled display, and 48 let a "skip to content"
link parked 40 px above a page that fits pass for scrolled content at 225%.

### Consequences
A page of identical rows, a table of the same line, gives fewer rows to vote with, and a
strip with fewer than 8 is not heard. A page standing still with a video or an animation
wider than two strips says "Too fast" while nobody scrolls. Sections more than 48 px apart
do not join a run, and a page that overflows by less than 64 px gets no button. A browser's
own zoom grows a page's spacing past both, and is not known to Recon. Revisit the gap if a
page with wide gaps between its sections shows no button.

## 2026-09-22 · A capture taken with Ctrl held reaches the page as any capture does, with the window left down

### Context
Rotem asked on 2026-09-22 that Ctrl held while clicking in capture mode copy the capture
without Recon coming up. Every capture is copied today by the editor page's own copy, after
the page has loaded it (the entry of 2026-09-16), and the host then shows the window. The
selection is either the click on a lit area or the release that ends a drag, and the focus
return target is remembered at the shortcut, before either is known.

### Options
1. The host publishes the captured pixels itself, with no page involved: the editor keeps
   what it showed, but a second copy path beside the page's, and a failure only in the log.
2. The page is told as for any capture, loads it and copies it with its own copy, and the
   host only leaves out the show; a copy that fails brings the window up with NOT COPIED.
3. Ctrl read at the button's press, or at the release that makes the selection.

### Decision
Option 2, and Ctrl read at the release, by the assistant, Rotem's to veto: `GetKeyState` in
the overlay's `WM_LBUTTONUP`, for the click and the drag alike, recorded beside the outcome
as the mouse's spot is. The capture joins the store and the timeline as every capture does.
The return target is put back as it was before the shortcut, since no editor came up to be
hidden again; left set, a later close of an editor opened from the tray would bring the
capture's application forward, which review T4 ruled out. The capture's own target is set
aside, and taken back when the editor comes up after all: a failed copy, or a capture made
before the page listens, which is brought up as any capture since only the page copies. A
target a hide spent while the overlay was up is not put back.

### Consequences
One copy path still: a change to the copy reaches this one. The page runs in a window that
is hidden, minimized or behind, as it already does at startup and in the checks; a visible
editor switches to the new capture without coming forward. A copy that fails is seen, at
the price of the window taking the focus then. The round button's scrolling capture ignores
Ctrl. Revisit if a hidden page is ever throttled into a late copy, or if Ctrl is wanted for
something else at the selection.

## 2026-09-27 · Show Recon after a screenshot, unchecked, is the Ctrl capture's path for every capture, the scrolling one included

### Context
Rotem asked on 2026-09-27 for an option in Settings, "Show Recon after a screenshot", with
a checkbox. Since 2026-09-22 a capture taken with Ctrl held is copied and the editor is left
where it was, through the page's own copy, and the round button's scrolling capture ignores
Ctrl. The checkbox names every screenshot, so the question was which captures it reaches
and what Ctrl means while it is unchecked.

### Options
1. Unchecked, every capture takes the Ctrl capture's path, the scrolling one included:
   copied, the editor left where it was; Ctrl then changes nothing.
2. Unchecked, only the click and the drag stay down, as Ctrl does; a scrolling capture
   still brings the editor up.
3. Unchecked, Ctrl held inverts it and brings the editor up for that one capture.

### Decision
Option 1, the assistant's, Rotem's to veto. The label reads as every screenshot, and a
person who unchecked it would read an editor coming up after a scrolling capture as a bug.
One path still: the page is told, loads and copies, and a copy that fails brings the editor
up with NOT COPIED, exactly as with Ctrl. The value is `show_after_capture` in the settings
file, absent meaning checked, read at every capture so it holds from the next one; Settings
stays outside Ctrl+Z, as the entry of 2026-09-18 has it.

### Consequences
Unchecking it is the way to work with Recon out of sight: every capture lands on the
clipboard and in the timeline, and the window comes up only when the copy fails or through
its own shortcut and the tray. Ctrl is a no-op while it is unchecked. The check-only probe
follows the setting, so the checks prove the path without a screen. Revisit if Ctrl should
invert the checkbox, or if a scrolling capture should be the exception after all.

## 2026-09-30 · The pixel ratio is never measured from a minimized window; the one owed is taken at the window's next resize

### Context
Rotem reported on 2026-09-30 that every picture suddenly looked pixelated. His screenshot
held each pixel of the picture twice across and down: the page's physical-to-css ratio was
0.5, the floor it is clamped to. The ratio is measured, not believed (the entry on the page's
`devicePixelRatio` in `project-os/Code_review.md`), as the window's physical width over the
page's css width, at startup, at every page resize and at fullscreen. Measured while the
window is minimized, the physical width is the taskbar stub's, 144, against a page still
1280 or 1800 wide. A restore sends the page no resize, since its own size never changed, so
nothing measured again. What sends a resize while Recon is minimized was not reproduced; a
change of the display's scale is the likely one, inferred: his machine moves between a 100%
and a 225% display, and did during this session.

### Options
1. Keep the ratio when the window is minimized and owe a
   measurement, taken when the host window next reports a size, which a restore does.
2. Take the ratio from the host's scale factor instead of measuring it.
3. Measure again on every focus of the window.
4. Refuse only an implausible measurement, a ratio far from the scale factor.

### Decision
Option 1, the assistant's, Rotem's to veto. The host adds `minimized` to the window metrics,
read in the same call as the size, so the page decides from one answer. The page keeps its
ratio and marks it owed; the host window's `onResized`, which fires on the restore, measures
it again. While it is owed, nothing is fitted or painted against the kept ratio, which may be
the old display's. Option 2 would drop the reason the ratio is measured at all; option 3 misses a restore that does not activate the window;
option 4 guesses at a threshold where the real condition can be asked.

### Consequences
A picture keeps its sharpness through a minimize, and a scale that changed while Recon was
down is taken at its return. The measurement stays the only source of the ratio. A
measurement from a window that is not minimized but still has no real size, if one ever
exists, is not covered. Revisit if a resize while minimized is ever seen to leave the ratio
from the old display after the return.

## 2026-09-30 · A tab's order is its own list's, and a drag moves one picture to the place under the hand

### Context
Rotem asked to reorder the pictures in a tab he made, by pressing one and dragging it, and
to drop the timeline's scroll by a press and a drag. A tab already saved its pictures' ids in
`tabs.json`, in the order they joined, but the strip ignored that order: it showed the
library's order, newest first, keeping the ids the tab held. A tab had no order of its own.

### Options
1. The tab's own list is its order: the strip shows it from its end, the last to join first,
   and a drag moves an id inside it.
2. A separate order field in each tab, used only once that tab has been dragged.
3. The picture dropped trades places with the one under the hand, rather than moving there
   with the ones between shifting over.

### Decision
Option 1, with the picture moving to the place under the hand, as a tab is dragged in the
bar above it. The assistant's calls, Rotem's to change. Option 2 keeps two orders for one
feed; option 3 is not what the tabs do, and a long move would scramble two places.

### Consequences
`tabs.json` keeps its shape and the host never reads it. Before any drag a tab shows its
pictures in the order they joined it, which is the library's order except in two cases: a
file opened by name again that joined the tab later now sits first, and an annotated file
whose place in the library moved keeps its place in the tab. A new capture joins a tab
first, as before. Main never reorders. The drag is one act of Ctrl+Z. The strip does not
scroll by itself when a dragged picture reaches its end; the wheel turned during the drag
scrolls under the hand. Revisit if a far move proves awkward.

## 2026-09-30 · Everything with the blue line is one batch, the picture on screen included; a batch deleted is one deletion of undo

### Context
Rotem asked for multi-selection with Ctrl in the timeline, to delete or remove from a tab
several pictures at once. His calls: a Ctrl+click only marks, the picture on screen stays;
a marked thumbnail wears the picture on screen's own blue line; and, asked what Delete takes
with a picture on screen and two marked, all three. Until now the blue line meant only the
picture on screen, and undo knew one deletion per number.

### Options
1. The batch is what wears the blue line: the picture on screen from the start, the marked
   ones with it; a Ctrl+click on the picture on screen takes it out, its line going while it
   stays on screen.
2. The batch is the Ctrl+clicked ones alone, the picture on screen never in it though it
   wears the same line.
3. For the undo: each picture of a batch a deletion of its own, one Ctrl+Z each; or the batch
   one act beside the deletions, with its own restore; or the deletions of a batch sharing
   one number, gathered by undo and put back together.

### Decision
Option 1, Rotem's, as Windows Explorer does, since option 2 deletes fewer than the screen
shows. For the undo, the deletions sharing one number, the assistant's call: the restore,
the notes kept in the stash and the refusal that says NOT RESTORED stay one path for one
picture or ten, where a separate act would be a second copy of it. The one on screen is
deleted last, so the neighbour Recon shows in its place is one that stays. The marking lives
in the page only and ends at a plain click, another picture on screen, another tab or the
trash, so no mark is ever acted on unseen.

### Consequences
A single deletion is a batch of one and behaves as before. A delete refused partway keeps
what went before it as one step of undo and says NOT DELETED for the rest. Remove from tab
takes a batch as one act, each picture put back at its old place. The blue line moved from
the picture on screen's class to a class of its own, `picked`, which the picture on screen
has until it is Ctrl+clicked; `current` still says which picture is on screen. Revisit if
Shift+click for a range is asked for, or a keyboard way to mark.

## 2026-09-30 · A Ctrl+Z pressed while pictures are on their way is let go, and a thing done again takes a new number

### Context
A review of the multi-selection found three older faults, and Rotem said fix all. Two are in
the one list of numbered acts that Ctrl+Z walks (`CLAUDE.md` rule 23). A deletion's undo waits
on the host, one call per picture, and was marked undone only when every call came back, so
a second Ctrl+Z meanwhile took the same batch again and said NOT RESTORED. And a deletion done
again by Ctrl+Shift+Z took a new number while an act or a note step done again kept its old
one, so after several redos Ctrl+Z could take the older thing first.

### Options
1. Ignore Ctrl+Z and Ctrl+Shift+Z while a deletion or its undo is on its way.
2. Claim at once: undo marks the batch undone and puts it on the redo list before the host is
   asked, and the next press is judged at once.
3. The keys in the order pressed: a press while a deletion comes back, or is deleted again,
   waits for it, and is then judged against the picture on screen.
4. For the order: a redone deletion keeps its old number; or everything done again takes a new
   one, and undo picks the newest by number rather than by its place in the list.

### Decision
Option 1, with the claim of option 2 kept, and the new numbers, the assistant's calls under
Rotem's fix all. Option 2 alone was built first, and its review found the second press judged
against the neighbour still on screen, taking back that picture's note, which the restore
then saved without it. Option 3 was built next, and its review found the held press landing
after the person had started typing, wiping the note being typed, or after another picture
was opened. Both are the worst class. A press let go loses nothing: the pictures landing are
the sign to press again. A new number for everything done again keeps the rule the list
already had for a deletion, the last thing done first. The host's delete with the editor
empty now answers that nothing is on screen, rather than failing on the missing picture after
the delete was done.

### Consequences
Two Ctrl+Z pressed within the moment pictures take to come back take one thing, and the page
logs the other as let go. A press with nothing on its way runs at once, as before, so holding
Ctrl+Z still walks back step by step. The delete itself, by the menu or Ctrl+Delete, is not
yet covered: a Ctrl+Z pressed before the host answers it is judged at once, a fault older than
this and put to Rotem. Picking by number costs a pass over the list per Ctrl+Z, which holds a
few hundred things at most. Revisit if the list ever holds thousands.

## 2026-10-01 · A Ctrl+Z pressed while a delete is on its way is let go too

### Context
The entry of 2026-09-30 left one case open: a Ctrl+Z pressed while a delete by the thumbnail's
menu or Ctrl+Delete is still on its way was judged at once. It took back a note of the picture
being deleted, and the delete, landing, dropped that note's redo, so the note was lost. A
review found it, older than the let-go rule, and Rotem said fix.

### Options
1. The delete raises the same count the undo and redo of a deletion raise, so the press is
   let go, as theirs are.
2. The press waits for the delete and is judged after it, which the review of 2026-09-30
   found unsafe for a press held back.

### Decision
Option 1, the assistant's call under Rotem's fix: one rule for every press made while pictures
are on their way. `undoBusy` became a count, raised by `deleteDocuments` for its whole run.

### Consequences
A Ctrl+Z pressed in the moment a delete takes does nothing, and the next one brings the picture
back with its notes. The 2026-09-30 entry's sentence that the delete is not yet covered no
longer holds; the rest of it stands.

## 2026-10-01 · Each line of a bubble left automatic takes its own direction, by `unicode-bidi: plaintext`; an override still sets the whole bubble

### Context
§3.5 resolved a bubble's direction once, from its first strong character, and applied it to the
whole bubble (F36). Rotem's note began in Hebrew and held English lines below it, and every
English line sat on the right. He asked that each line take the alignment of its own language.

### Options
1. `unicode-bidi: plaintext` with `text-align: start` on the text box and on the blocks the engine
   makes while a note is typed, for a bubble left automatic: the engine resolves each line, on
   screen and in the export, which inlines the computed styles.
2. Each line made an element of its own with a direction set by the page, so a line with no
   letter could follow the bubble: a second shape of the text beside the engine's editing one.
3. Keep one direction per bubble.

### Decision
Option 1, the assistant's call under Rotem's ask. F36's reason for leaving plaintext still holds
for the override, which plaintext ignores, so plaintext is applied to automatic only and the
override keeps setting the whole bubble. The text tool's notes keep one direction until Rotem
says otherwise, since he asked about the bubble.

### Consequences
A line is one broken by Ctrl+Enter; a long line that wraps keeps one direction. A line with no
letter, an empty one or one of numbers only, resolves left to right by the Unicode rule, so in a
Hebrew bubble it sits on the left, and a new empty line's caret starts there until a Hebrew
letter is typed; option 2, or a style for an empty block, would change that, and both wait for
Rotem. The stored resolved value is still the bubble's first strong character. A single-line
bubble lays out exactly as before: the six S0.6 references are identical.

## 2026-10-06 · The commit waits for Go commit, the kit's new rule 22 adopted

### Context
The ProjectOS kit update of 2026-10-06 brought a rewritten rule 22: the assistant never
commits on its own, the report's Next line says the work is uncommitted, and `Go commit`
commits everything since the last one. This project had run the kit's older rule since
2026-09-10, where the commit was the assistant's, unasked, after every task, and Rotem had
corrected a hesitation about it then (`project-os/Mistakes.md`, now Retired). The two could
not both stand.

### Options
1. Keep the per-task commit as this project's own wording, against the kit.
2. Adopt the kit's flow: tasks leave uncommitted work, `Go commit` lands it.

### Decision
Option 2, Rotem's, on 2026-10-06, in two words. The merge had already brought the kit's
text in, since the old rule was the kit's own and not this project's wording.

### Consequences
A task ends with its History row and a Next line saying the work is uncommitted; the
History row's Commit before carries `(uncommitted)`. The release build of
`project-os/Workflow.md` step 16, which is made from a commit alone, now follows the commit
`Go commit` makes, so Rotem's Recon gets a new build at `Go commit` rather than after each
task; he can call for `Go commit` whenever a build is wanted. Worth revisiting if a task's
release is wanted sooner than the next commit.

## 2026-10-07 · A capture's timing lines are kept in a file of their own, `%LOCALAPPDATA%\Recon\timing.log`, capped at 4 MB with one older file

### Context
On 2026-10-07 a capture reached the clipboard about 1.2 s after it was made, in a Recon that
had run six days, and a restart cleared it. The host prints the copy's split, and the page
its load, paint and copy, but a product run's standard output goes nowhere, so the slow
days left no record of which step was slow. The Copy Ninja session asked for a file; Rotem
said to continue.

### Options
1. The standard output only, as before: nothing is kept.
2. The whole host log in a file.
3. Only the lines that time a capture in a file, always on.
4. A file only while a setting turns it on.

### Decision
Option 3, by the assistant, Rotem's to veto. The whole log names the window a capture began
in and the files opened, which a file would keep for weeks, and the timing needs neither.
Always on, since a slow day cannot be foreseen. The lines: the S0.7 marks and the startup
marks, each image's encode, the page's two lines that time a capture, its showing and its
copy with the layer's export and the host's answer, and the host's copy split, each
stamped in milliseconds since 1970, the clock a document's number is read on, so a line
sits beside the document it times. The page's other lines and its errors stay on the
standard output, since an error can carry an opened file's path. One writer thread of the
log's own does the writing, so no capture thread and not the main one waits on the disk.
Past 4 MB the file becomes `timing.old.log`, replacing the one before; a checks build
writes `s-timing.log` beside its executable.

### Consequences
A slow day is split by reading one file: created, encoded, shown, exported, answered,
copied. Two files at most, about 8 MB. A line still waiting for the writer when Recon
quits is lost, and a line that cannot be written is dropped and counted, the file saying
how many when writing resumes. Worth revisiting before the public release, whether the
file stays on by default, and if a line ever carries what the user would not want kept.

## 2026-10-14 · A scrolling capture calls a page lost only once it rests out of reach; a page drawn between whole pixels is on its way, never placed and never lost

### Context
Rotem reported on 2026-10-14 that one notch of the wheel made the bar say "Too fast. Scroll
back up a little" at once. Recorded on his Chrome at the session's pace, on a Facebook post
dialog: every notch is drawn over about six frames in which the page sits between whole
pixels, each row a blend of two, and no row of those frames is a row of the last placed
frame; the stitcher matches rows exactly, so it called every one of them lost, and a steady
scroll keeps the page moving past the 350 ms the bar waits. The frame the page rests on
matched exactly. The same browser's main page scroll landed on whole pixels and joined live.
The self test's stand-in scrolls in whole rows and never showed it.

### Options
1. Match rows with a tolerance, and place the blended frames.
2. Track the blended frames approximately, to know how far the page went, and place only
   exact ones.
3. Keep exact matching, call a frame that cannot be placed while the page still moves "on
   its way", and wait for the frame it rests on.

### Decision
Option 3, by the assistant, inside Rotem's report. A blended frame placed into the picture
is a blurred strip in a screenshot, so option 1 trades the warning for a worse picture.
Option 2 adds a second matcher only to say "too fast" sooner. The page is at rest when a
frame is the frame before it again, by the same 97% the stitcher already uses for an
unchanged frame, so a caret or a fading scrollbar does not keep it moving. The bar's wait
starts again with every frame on its way, and only a placed or unchanged frame counts as
the page being back.

### Consequences
A slow scroll joins at every pause, however the browser draws the way there. A long flick
that never pauses joins nothing until the page rests, and is told "Too fast" only when it
rests further than a frame from the last placed one, a frame later than before. A video or
an animation wider than two strips keeps every frame different, so the bar now says nothing
there where it said "Too fast" while nobody scrolled (the 2026-09-22 entry's consequence);
nothing joins there either way. Revisit if a page is found that never rests between the
notches of a slow scroll.

## 2026-10-15 · The Recon Rotem runs lives in `app/`, outside the build folder, with the release before it kept in `app/previous/`

### Context
Every release was copied over `host/target/release/recon-host.exe`, inside cargo's build
folder, and Windows' Run value, the image types' "Open with" and Rotem's taskbar pin pointed
into that folder. On 2026-10-15 the investigation of `host\` offered for deletion found that
the assistant had then twice called `host\target\` safe to delete: it held the Recon Rotem
runs every day, and the only copy of the release, since each release overwrote the last.
Rotem approved moving it.

### Options
1. Keep it in the build folder and say so wherever the folder is described.
2. A folder of its own inside the project, `app/`, out of git, the release before it kept.
3. A folder outside the project, such as the user's programs folder.

### Decision
Option 2, Rotem's approval of the assistant's proposal. Option 1 leaves the next cleanup of
the build folder one sentence away from removing his Recon. Option 3 is outside the project,
where nothing the assistant writes may go (`CLAUDE.md` rule 12). The product keeps nothing
beside its own executable: its settings, store, trash and logs are under the user's folders,
and only check builds write beside themselves, so the move changes no data.

### Consequences
`host/target/` is a build cache again, which a clean rebuilds whole. A bad release goes back
by one copy, `app/previous/recon-host.exe`. Windows' Run value and "Open with" are written
from the executable that writes them, and the tick reads only whether a Run value exists,
whatever path it holds; so they move only in this order: every Recon quit, then
`app\recon-host.exe` started (a copy started while another Recon runs hands itself to that
one and exits, and a tick made there writes the OLD path again), then Start with Windows
unticked and ticked again, then `app\recon-host.exe --register-types`, then the taskbar pin
and any other shortcut made anew. `reg query` of the Run value and of
`HKCU\Software\Classes\Recon.Image\shell\open\command` proves it. Running a check build's
`--register-types` points "Open with" at that build, as had already happened before this
move (it pointed at `host/target/debug/`). A Go backup now carries the two executables,
about 30 MB. Revisit if Recon gets an installer of its own, or if one copy back proves too
few.

## 2026-10-15 · A show refused the foreground asks again for a second and a half, never takes it by force

### Context
The first open of the day, from the Start menu while the Recon started by Windows sits in the
tray, showed the editor behind the other windows with its taskbar button flashing (Rotem,
2026-10-15). Measured with a probe copy of the host: the launching Recon does hand its right
to the foreground over (the 2026-10-10 fix), and the Recon in the tray shows its window at
once, while the Start menu, or Search, is still the window in front; Windows refuses the
foreground to a desktop window then, whatever right the process holds, and Tauri's focus
call asks once, with its ALT-key trick, and gives up. The menu closes by itself a moment
later, and nothing asked again. A Store app in front, Settings, does not refuse.

### Options
1. After a show that was refused, ask for the foreground again on a timer, every 100 ms for
   up to 1.5 s, while the window is visible and not in front; a refused ask changes nothing.
2. Take the foreground by force: attach the window's thread to the input of the one in front
   (`AttachThreadInput`) and set the foreground through it, which the rule does not stop.
3. Create the hidden window unfocused (`focused(false)`), so the web view's focus at build
   time does not activate the hidden window; tried first, then measured not to matter: the
   hand-off comes up fine with the old window, Chrome in front.
4. Have the launching Recon, which holds the right, wait for the window and bring it forward
   itself, instead of handing the right over.

### Decision
Option 1, the assistant's, Rotem's to veto. The rule Windows applies is the one that stops a
background program stealing the window a person is typing in, and option 2 defeats it for
every show, the tray click and the capture included; it also fails against the Start menu
itself, a process its thread cannot attach to. Option 1 takes nothing a person did not ask
for: the ask succeeds only with the right the launch handed over, and that right is gone the
moment the person clicks or types elsewhere, so the window never comes forward over work that
moved on. Option 4 leaves the launching Recon alive for a second with a window to find by
title. Option 3 is reverted, and offered to Rotem apart: a Recon started by Windows takes the
foreground invisibly at logon through that focus, which is a defect of its own.

### Consequences
`editor::show` ends in `bring_forward_again`: nothing when the window is in front already,
otherwise a thread that asks on the window's own thread up to fifteen times and logs the
count, or that the window is still behind and who is in front. Every caller of `show` gets
it: the tray, the Open shortcut, the second launch, a capture, the page's own bring-up. A
process with no right to the foreground, such as a check build started from a shell, makes
fifteen refused asks and nothing else. Not measured here: that the handed-over right outlives
the Start menu's own closing, which the probe could not close without a keystroke that would
take the right away; Rotem's first open from the Start menu after the release is the proof.
Revisit if that proof fails, which would mean the shell revokes the right as the menu closes,
and option 4 or the shell's own activation becomes the route; or if a second and a half shows
as a window arriving late.

## 2026-10-19 · The everyday copy re-points Start with Windows to itself at every start, told by its folder's name, `app`

### Context
The move of the everyday Recon into `app/` (2026-10-15) left Windows' Run value on the old
path, `host\target\release\`: the value is written only at the tray's tick, and the tick was
never made again from the new copy. So every logon started the old release, the one before
the first-open fix, and the taskbar pin, re-made to name `app\`, never lit for it: Windows
lights a pin only for the window of the very program it names, and the running window got a
button of its own at the end of the bar. The 2026-10-15 entry had left the re-tick to
Rotem's hand, and a hand step after every move is the step that gets missed.

### Options
1. Rotem's hand after every move: quit, start from the pin, untick and tick.
2. Every product build re-points the value to itself at its start, when it names another copy.
3. Only a copy whose folder is named `app` re-points it; every other copy leaves it alone.

### Decision
Option 3, Rotem's pick of the code route on 2026-10-19, in his words "whenever the copy in
app\ starts"; the folder's name as the test is the assistant's. Option 1 is what failed.
Option 2 lets a release built for a check, or the copy kept in `app/previous/`, steal the
logon start by being run once. The folder's name is the one fact that tells the everyday
copy from every other: a check build's is `debug`, a release build's `release`, the kept
copy's `previous`.

### Consequences
`startup::repoint`, called in the setup of the Recon that stays, after the single-instance
hand-off, so a copy started while another runs never reaches it. It writes only when a
value exists, so the tick off stays off; only when the value names another command, so a
logon start from `app\` writes nothing; and never from a folder of another name. The first
start of a new `app\` copy after a move is still by hand, from the pin, since the logon
start names the old path until then; from that start on, logon and the pin name the same
copy and the pin lights. "Open with" is not covered: it stays `--register-types`, run by
Rotem from `app\`. Supersedes, in the 2026-10-15 entry, the clause "then Start with Windows
unticked and ticked again". Revisit if the everyday copy ever lives in a folder of another
name, an installer's for one; the test is then that installer's to carry.

## 2026-10-20 · The kit's reply rules take the numbers after Recon's own, 20 on; their numbers are mapped at every update

### Context
The kit's reply rules, `project-os/Conversations.md`, number 1 to 19 at kit commit `748f930`,
and a second file, `Conversations-2.md`, carries 20 to 25. Recon's copy already carries two
rules of its own after the kit's 18: "The plan travels with its change" and "The next step
names its model", numbered 18 and 19, both Rotem's; `CLAUDE.md` rule 13, `Workflow.md` step 16
and `Mistakes.md` point at that 19 by number. The kit's new rule 19, the wrap-up box, lands on
the same number.

### Options
1. Recon's two rules move to the end, after the kit's, and every reference to them follows.
2. The kit's rules take the numbers after Recon's: its 19 becomes 20, its second file 21 to 26,
   and every cross-reference inside the kit's text follows, at each update.
3. The kit's numbers stay, and Recon's model rule takes a letter, 19b.

### Decision
Option 2, proposed in the update report and taken with Rotem's GO on 2026-10-20. The merge
law of `Go update kit` keeps this project's own wording and numbers on every clash, and a rule
Rotem set is his; the kit's text is what bends. The unfinished run of 2026-10-16 had made the
same call.

### Consequences
Every kit update that adds a reply rule, or cites one by number, is mapped by hand: the kit's
N is Recon's N+1 from 19 on, in the headings, in "rules 21 on" in `CLAUDE.md`, and in every
"(rule N)" the kit's text carries; the compare tool shows these as clashes and never copies
them. The kit's mods plugin reads the wrap-up rule by its words, not its number, so the
renumber costs it nothing (`mods/hooks/Reply-check.ts` at `748f930`). Recon's two rules
numbered 18 stay as they are; nothing points at the second by number. Revisit if the kit ever
numbers its rules in a way that cannot be mapped, or if Rotem renumbers his own two.

## 2026-10-26 · The second instance raises the editor window itself, with the right the click gave it, and the hand-off message is Recon's own

### Context
The first open of the day still came up behind every window after the 2026-10-15 fix (Rotem,
2026-10-26). That fix handed the launching Recon's right to the foreground to the Recon in the
tray, which then asked for a second and a half; it was measured with the Start menu held open
and with a stand-in launcher, never with the real click, whose proof it left to Rotem's hand.
The product keeps no record of those lines: they go to the standard output, which a launch
from the pin has none of, so which half failed, the right revoked as the shell finished the
launch or never granted, is unknown. The only Recon shortcut on this computer is the taskbar
pin, where no Start menu is in front at all, so the refusal measured on 2026-10-15 does not
even cover it. A window hidden since logon sits under everything opened since, which is why a
refused show puts it beneath all the windows, not behind one.

### Options
1. Option 4 of the 2026-10-15 decision: the launching Recon, which holds the right Windows
   gives the program a click starts, raises the window itself and quits only then. Recon then
   sends the hand-off message in the plugin's place, since the plugin quits the moment the
   message is delivered.
2. Log first: write the two outcomes into the timing log, read the next first open, then
   choose a route on evidence.
3. Force, through `AttachThreadInput`: rejected on 2026-10-15, and still rejected.

### Decision
Option 1 with the two log lines of option 2 riding along, Rotem's `Fix` on 2026-10-26.
`handoff.rs` finds the running Recon by the plugin's own window, sends the plugin's message
(the format of version 2.4.4, tested against its receiver's reading), waits up to three
seconds for the editor window to be on screen, since a file is decoded on a thread before its
show, then asks for the front from the launching process every 100 ms up to fifteen times,
and quits. The grant through `AllowSetForegroundWindow` is gone: a launcher that can grant can
raise, and one that cannot raise could not grant. The running Recon's own asks in
`editor::show` stay. Both outcomes go into the timing log as counts, never a window's name
(the 2026-10-07 rule); the second instance writes its line straight into the file, having no
writer thread. A second instance started with `--startup` is left to the plugin: no click is
behind it.

### Consequences
Every start while Recon sits in the tray, the pin, the Start menu and a file from Explorer
alike, is raised by the process the click started. That process lives up to a second and a
half longer, without a window, and up to three seconds more while a file decodes; a file slower
than that comes up behind as before, and the log says so. The plugin's wire format is now read
in two places, its receiver and `handoff::payload`, so a plugin update is checked against the
test that pins it. The 2026-10-15 probe measurements describe the old code. NOT measured here:
the real click, the probe having run while the screen saver held the input desktop; Rotem's
first open after the release is the proof, and the two lines in the timing log say which half
failed if it fails again. Revisit then: it would mean the launching process's own right does
not survive the shell finishing the launch either, and the routes left are the shell's own
activation of the window, or force.
