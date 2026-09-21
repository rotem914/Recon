# Recon — Design

The design system: the tokens and the components the editor's look is built from.
Every value here is Rotem's, stated by him; the assistant records and applies it, never
invents one. A new value or component lands here in the same change that first uses it,
so the page and this file never disagree.

Read it before any change to something a person can see. A value the page already carries
is looked up here, not guessed from a neighbour.

## Motion

| Token | Name | Duration | Easing | Where it is used |
|---|---|---|---|---|
| A1 | Fade in | 144 ms | ease-out | The hover fill of an icon button (`editor/index.html`, `#controls button`); the tick of a done mark as it is pressed (Rotem, 2026-09-16); the icon's line turning the icon line hover colour (Rotem, 2026-09-19); the hover border of a timeline thumbnail (Rotem, 2026-09-19). |

A motion respects the reduced-motion preference: gated at its own surface, the final
state reachable with no animation at all (`project-os/QA.md` §7).

## Colours

| Token | Value | Where it is used |
|---|---|---|
| App background | `#0C0C0E` | The page ground behind the picture and the panels (Rotem, 2026-09-17, from #0D0E12 earlier that day; #111217 was his on 2026-09-14). |
| Icon button hover | `#21222C` | The fill of an icon button's square under the pointer (Rotem, 2026-09-15). |
| Icon line | `#C3C6CA` | The line of an icon button's icon in the controls sidebar (Rotem, 2026-09-16). |
| Icon line hover | `#E7E8E9` | The line of an icon button's icon under the pointer, fading in by A1 (Rotem, 2026-09-19, from white). |
| Scroller | `#21222C` | The thumb of the timeline's scroller, sideways in one row and vertical once its rows wrap (Rotem, 2026-09-15). |

## Fonts

| Font | Bundled | Where it is used |
|---|---|---|
| Google Sans | medium (500), the Latin subset, from Google Fonts under the SIL Open Font License, in `editor/fonts/` with its `OFL.txt` | The image size (Rotem, 2026-09-16). |

## Components

### Icon button

Stated by Rotem on 2026-09-15 for the controls sidebar; the sizes restated on 2026-09-16.

| Part | Value |
|---|---|
| Container | 48 by 48, 13 px corners, no fill of its own (Rotem, 2026-09-16, from 32 by 32; the corners 2026-09-17, from square) |
| Icon | 32 by 32, centred, a 2 px stroke in the icon line colour (Rotem, 2026-09-16, from 20 by 20 with a 1.5 px stroke in the button's text colour) |
| Between buttons | 8 px |
| Hover | the container fills with the icon button hover colour, and the icon's line turns the icon line hover colour, both fading in by A1 (the line, Rotem, 2026-09-19) |
| Name | every icon button carries its name and its key as a title, so it has a name without its text |

Used by: the controls sidebar down the left edge of the editor, the eight tools. Highlight, Copy
and Save As left it on 2026-09-18 (Rotem); Copy and Save As stay on their keys. The colour
picker joined it the same day.

### Button group

Stated by Rotem on 2026-09-15 for the controls sidebar.

| Part | Value |
|---|---|
| Container | one container around all the sidebar's buttons, no look of its own |
| Placement | centred in the sidebar's height, between the top bar and the image size above the timeline, and across its width |
| Sidebar | 64 px wide, 8 px of air around the 48 px buttons, the picture starting at its right edge (Rotem, 2026-09-16, option 2 of two: from 48 wide) |

### Tooltip

Stated by Rotem on 2026-09-15 for the icon buttons.

| Part | Value |
|---|---|
| Placement | on the right of the button, centred on it |
| Opens | on hover over the button |
| Text | the action's name only |

The look is not stated yet. Provisional in the page until it is: the icon button hover
colour as its fill, 12 px text, a 4 px radius, 8 px from the button, fading in by A1.

### Scroller

Stated by Rotem on 2026-09-15 for the timeline along the bottom.

| Part | Value |
|---|---|
| Thumb | 4 px thick, fully rounded, the scroller colour |
| Track | no fill |
| Sideways, one row | 4 px clear of the screen's bottom edge and 4 px clear above the thumb |
| Vertical, rows | 4 px clear of the screen's bottom edge and 4 px clear of the strip's top |

Used by: the timeline along the bottom, both of its scrollers.

### Image size

Stated by Rotem on 2026-09-16.

| Part | Value |
|---|---|
| Container | 100% of the window's width, above the timeline |
| Band | 64 px tall, the size and the tabs centred in its height: the size's 16 px line 25 px under the band's top and 23 above its bottom, which puts the digits themselves in the middle, since Google Sans' digits sit high on their line (Rotem, 2026-09-19, the same evening, from 24 and 24), 16 px above and below the 32 px tabs (Rotem, 2026-09-19, from 48 tall with 16 px above and below the size) |
| Background | the app background |
| Content | the picture's size in pixels, written as 1200x1500 |
| Text | Google Sans, 16 px, medium (500), centred in the window (Rotem, 2026-09-17, from 14 px) |

The text's colour and line are not stated yet. Provisional in the page until they are: the
top bar title's #cfd4da, on a 16 px line.

### Tab bar

Stated by Rotem on 2026-09-16 for the timeline's tabs.

| Part | Value |
|---|---|
| Plus button | an icon button after the last tab, on the tabs' right, and alone at the bar's left until the first tab exists (Rotem, 2026-09-20, from fixed at the left of the image size container): 32 by 32, no fill of its own, a plus icon, 20 by 20, with round ends, the hover fill by A1, on the tabs' own 10 px corners (Rotem, 2026-09-18, from square) |
| Tabs | beside the plus, once it was pressed: Main first, pinned and never deleted, then the tabs the plus made, in the order they are dragged into |
| Tab text | Google Sans, medium (500), 16 px (Rotem, 2026-09-20, from 14 px in the system font; semibold was his word, medium is the one weight bundled) |
| Placement | the row starts 12 px from the window's left, on the first thumbnail's line (Rotem, 2026-09-20, from 8) |
| Tab corners | 10 px radius |
| Tab name | a click on the selected tab's name edits it in place; Enter keeps it |
| Tab delete | a right press on a tab after Main opens a dropdown with Delete, which deletes it; no × on the tab (Rotem, 2026-09-18, from a × pressed twice) |
| Tab spacing | two new tabs as far apart as Main and the first new tab (Rotem, 2026-09-18) |

The rest of the tabs' look is not stated yet. Provisional in the page until it is: 32 px tall,
the top bar title's #cfd4da, 8 px between, the icon button hover colour as the selected tab's
fill and as the hover; the dropdown in the same fill on the tabs' 10 px corners, 14 px text,
Delete red under the pointer, opening above the bar at the pointer; the
bar stops at the middle of the window less 80 px, short of the size text.

### Done mark

Stated by Rotem on 2026-09-16 for the thumbnails in a tab after Main.

| Part | Value |
|---|---|
| Container | round, 24 by 24, at the thumbnail's top left, a 1 px white border |
| Icon | 20 by 20, a 2 px line |
| Shown | on hover over the thumbnail; once pressed, always, ticked |
| Marking | the tick fades in by A1 |
| Effect | none beyond the mark itself |

The fills are not stated yet. Provisional in the page: the dark fill the thumbnail's × had, rgba(10,12,14,.75), and the
restore button's green once ticked.

### Settings

Stated by Rotem on 2026-09-18.

| Part | Value |
|---|---|
| Button | a chevron pointing down (Rotem, 2026-09-18, from three dots the same day), left of minimize in the top bar, in the window buttons' own box and icon size: 48 by 40 with a 10 by 10 icon |
| Opens | a modal |
| Content | a shortcut that opens Recon itself, and the shortcut that starts a capture, each picked by pressing it; under Capture a second row, Capture 2, for a second shortcut that starts the same capture (Rotem, 2026-09-19), empty until picked, with its own × |
| Mouse pointer in captures | a row under Capture 2 with a switch, on until switched off (Rotem, 2026-09-21); it holds from the next capture |

The modal's look is not stated yet. Provisional in the page until it is: a 50% black veil,
a 420 px panel in the icon button hover colour on 13 px corners with 24 px of padding, the
title in Google Sans 16 px medium, 14 px rows in the top bar title's #cfd4da, a 32 px field
in the app background on the tabs' 10 px corners, a 1 px #7aa7ff ring while it waits for a
shortcut, a refusal under its field in #ff5252, the veil fading in by A1. The switch: a 32 by
18 track in the app background with a 12 px #8b9098 knob, the track #7aa7ff and the knob
#f2f2f2 when on, moving by A1.

### Callout

Stated by Rotem on 2026-09-17 for the annotation tools. The pixels are at the default text
size of 20; the bubble's padding, corners and text box are proportions of the text size
(`project-os/Plan.md` §3.5), so they are exactly these pixels at 20 and grow with it.

| Part | Value |
|---|---|
| Bubble | `#2D41D7`, 32 px of padding, 15 px corners, no number shown |
| Shadow | x 0, y 8, blur 16, spread 6, black at 25% |
| Text box | 36 px tall at the least, growing with the lines; not marked while it is typed in, transparent |
| Line | 4 px thick, ending in an arrow at the point the note refers to: an open head of two strokes, never filled |
| Enter | keeps the text and leaves the typing, as a click outside the bubble does |
| Ctrl+Enter | breaks the line inside the text box |

Not stated yet, so provisional in the page until it is: the line's and the head's #7aa7ff,
the head 16 px long and 7 px to each side (the arrow shape's numbers), the text's #f2f2f2, the
selected bubble's 1 px #7aa7ff ring, and the 12 px anchor dot on the point while editing.

### Ruler

Stated by Rotem on 2026-09-16 for the annotation tools.

| Part | Value |
|---|---|
| Use | a drag marks an area; its size in pixels, width and height, is written in a bubble beside the pointer the whole time the drag lasts |
| After the release | the box and its size stay on the picture and in every copy (option 1 of two) |
| Delete button | on hover over the ruler: a round 16 by 16 button with a 12 by 12 X, one press deletes the ruler |
| Magnifier | the capture's zoom circle, below, beside the pointer the whole time the ruler tool is in hand, before the press too; while a ruler is dragged out its size sits above the circle as in a capture, and returns beside the ruler's corner at the release (Rotem, 2026-09-18, both of two choices). Every number is the Magnifier's own; it shows the picture's pixels at any zoom, and is never in a copy |

The rest is not stated yet. Provisional in the page until it is: the box a 2 px line in the
callout line's #7aa7ff; the size bubble beside the corner the drag ended at, 8 px right of
and below it, dark #1b1f24 with a 1 px #3a4149 border, 4 px corners, white 14 px medium
text, all in image pixels like a note; the × centred on the box's top right corner, the
dark fill the thumbnail's × had, rgba(10,12,14,.75), and its red on hover, #c0392b, a 1.5 px line, 16 screen pixels at every zoom.

### Captured pointer

Stated by Rotem on 2026-09-22, in place of the element on top he chose the day before.

| Part | Value |
|---|---|
| Use | a capture holds the mouse pointer where it stood when the shortcut was pressed, as part of the picture, like any other pixel of the screen; it cannot be moved or taken off afterwards |
| While choosing the area | the frozen screen shows that pointer too, so what is seen is what is captured |
| Switch | Settings, Mouse pointer in captures: off, and a capture holds no pointer |

### Crop

Stated by Rotem on 2026-09-17 for the annotation tools.

| Part | Value |
|---|---|
| Use | a crop button in the sidebar; with it in hand, handles on the picture's edges, dragged inward, and the picture is cut to them |
| Where | the crop button in the controls sidebar, an icon button like the others, after Ruler |

The rest is not stated yet. Provisional in the page until it is: eight handles, one at each
corner and at the middle of each side, 10 by 10 white squares with a 1 px dark border, 10
screen pixels at every zoom; a 1 px white frame on the picture's edges while the tool is in
hand; while a handle is dragged the frame follows it and the outside is dimmed by half; the
cut lands at the release, inward only, a side no smaller than 8 px; the key K.

### Color picker

Stated by Rotem on 2026-09-18 for the left sidebar.

| Part | Value |
|---|---|
| Use | a click on the picture copies the HEX of the colour under it to the clipboard, by itself |
| Where | a button in the controls sidebar, an icon button like the others |
| Icon | Rotem's own drawing of a dropper, 21 by 21 at its true size in the middle of the 32 px icon box: a 2 px outline and a filled collar and head, in the icon line colour (Rotem, 2026-09-18, from a provisional eyedropper whose head sat off the tube's line) |
| After the click | the tool is put down, like the other tools (Rotem, 2026-09-18, of two) |
| Magnifier | the ruler's zoom circle follows the pointer the whole time the tool is in hand, the HEX of the pixel under the pointer above it, where the ruler's size sits (Rotem, 2026-09-18, of two) |

The rest is not stated yet. Provisional in the page until it is: the button after Crop;
the key I; the HEX written as `#2D41D7`, upper case with
its hash; the picture's own pixel, whatever note lies over it; a see-through pixel as it
shows, over the app background; a click off the picture copies nothing and keeps the tool;
the line "copied #2D41D7 to the clipboard" where a copy of the picture says its own; it
works over a file only viewed and makes no document for it.

### Capture size bubble

Stated by Rotem on 2026-09-17 for the capture overlay.

| Part | Value |
|---|---|
| Use | while a drag on the capture overlay lasts, the dragged area's width and height in pixels sit beside the crosshair, written as 160x100 |
| Text | 14 px |
| Bubble | the app background as its fill, 8 px corners, 8 px of padding at the sides and 4 above and below |

Since 2026-09-18 the bubble is the magnifier's panel, below, and the text sits above its
circle, at Rotem's word, where the picture he sent had it under; the fill, the corners and the padding around the
text are the panel's. The rest is not stated yet. Provisional in the overlay until it is:
the text the ruler's #f2f2f2 in Segoe UI, regular; every number at 100%, scaled by the
display's own scale as the cursor is. The fill is a constant in the host
(`host/src/overlay.rs`), a second home for the app background beside the page's `--paper`.

### Magnifier

Stated by Rotem on 2026-09-18 for the capture overlay, with a picture of another tool's.

| Part | Value |
|---|---|
| Circle | 112 px across, showing the pixels around the pointer large enough to tell apart |
| Side | on the pointer's right (Rotem, 2026-09-18, from the picture's left) |
| Size | above the circle (Rotem, 2026-09-18, from under it as the picture had) |
| One unit | the panel, circle and text, is drawn by one piece of code for the capture, the Ruler and the Color picker: `host/src/magnifier.rs`; a value changed there changes in all three (Rotem, 2026-09-19) |
| Panel padding | 8 px around the circle on every side (Rotem, 2026-09-19, from 4) |
| Text | Google Sans, medium, 14 px (Rotem, 2026-09-19, from Segoe UI; semibold was his word a moment earlier, and medium is the one weight bundled) |
| Text spacing | 10 px from the panel's top edge to the top of the digits, and 10 px from the line the digits stand on to the circle, measured from the digits themselves and not the font's taller line (Rotem, 2026-09-19, after 8 and 8, from 4 and 4) |
| Guide lines | one across and one down through the pointer, the whole width and height of the screen whatever the capture area is, at 48% (Rotem, 2026-09-18, after 72%), with the circle's lines' settings (Rotem, 2026-09-18, later that day): #2554FB and 1.64 px, centred; the plus at the pointer stays white |

From the picture, not his words, so provisional in the overlay until he states it: the
circle sits in a panel of the size bubble's fill and corners, 4 px around it, with the
selection's size above it, the lit window's while hovering and the dragged area's while a
drag lasts, in the size bubble's text and padding; the panel is up the whole time the
overlay is, below the pointer with its left edge 8 px right of it, and on the pointer's
other side where that would leave the display; 8 px to a source pixel, an odd count of them
so the pointer's own is the middle one; a 1 px grid between them, #808080 at 48% over the pixel (Rotem, 2026-09-18, after #A8A8A8, #8C8C8C, #707070 and 64%), where it was the pixel darkened by
half; #2554FB lines 1.64 px wide, centred on the grid's line (Rotem, 2026-09-18, from the marching frame's #00B9F7 and 1 px, after 1.44 px; the editor's Ruler and Color picker circle the same) on the centre pixel's top and left edges, crossing at its top left corner, as another tool's picture shows (Rotem, 2026-09-18); a 2 px ring in
the text's white; every number at 100%, scaled by the display's own scale as the cursor is.
Provisional for the guide lines: they stop 12 px short of the pointer on every side, since
Windows draws the plus as the inverse of what is under it and the blue turned it red; the
screen is the display the pointer is on, they sit
under the dashed frame and the panel, and the width is not scaled with the display: the line's pixel whole, the one
on each side at 32% of it.

### Scrolling capture

Stated by Rotem on 2026-09-22 for the capture overlay.

| Part | Value |
|---|---|
| Button | round, 56 by 56, an arrow pointing down, 24 px up from the bottom of the capture frame, shown in capture mode over an area that scrolls |
| A press on it | starts the capture with scrolling: he scrolls the page himself, Recon joins what passes (his pick, of two) |
| The flow after the press | left to the assistant, with its messages |

The rest is not stated yet. Provisional in the host until it is (`host/src/scrolling/`):
the button in the middle of the frame's width, the magnifier's lines' #2554FB as its fill
and #4B74FF under the pointer, a 2 px ring and a 24 px arrow on a 2 px line in the
magnifier's #F2F2F2, the hand pointer over it and the magnifier and guide lines out of its
way, up only once the lit area has come to rest. After the press the screen keeps the
capture's dim around the area and a 2 px #00B9F7 frame just inside it, not walking. The bar:
a pill 56 px tall in the magnifier's fill where the button was, 24 px up from the area's
bottom, under or over the area when the area is too small; its text Google Sans medium
14 px in #F2F2F2, 24 px from the bar's left; the messages "Scroll down slowly", then the
size so far as "1240x5380 so far", "Too fast. Scroll back up a little" in #FFB84D, and
"This is as long as a picture gets" before it ends by itself; Done a 40 px tall #2554FB
pill, 18 px of padding at its sides, and cancel a 40 px disc in #21222C with a 2 px X,
each lighter under the pointer, 8 px between them and to the bar's right end; Enter is
Done and Escape is cancel; every number at 100%, scaled by the display's own scale.

### Capture window glide

Stated by Rotem on 2026-09-18 for the capture overlay.

| Part | Value |
|---|---|
| Use | when the pointer moves from one window to the next, the lit area and its dashed frame glide from the one to the other instead of jumping |
| Motion | 192 ms, ease-out (Rotem, 2026-09-18, the same night, after 164 ms, 256 ms with an ease-in, and A1's 144 ms and ease-out, which stood provisionally) |

The rest is not stated yet. Provisional in the overlay until it is: each edge of the area on
its own; a pointer that moves on mid-glide sets it out again from where it is; it stays
inside one display, a move to a window on another display jumping as before; and it is off
when Windows' own animation effects are off, the reduced-motion gate above.

### Window

Stated by Rotem on 2026-09-15.

| Part | Value |
|---|---|
| Size when it opens | 1800 wide by 1390 tall (Rotem, 2026-09-18, from 1600 by 1160), centred in the main display's work area |
| Top bar | 64 px tall (Rotem, 2026-09-18, from 40), the window buttons still 48 by 40 at its top |
| Window button icons | 10 by 10, a 1 px stroke; the close X alone 12 by 12 (Rotem, 2026-09-21, from 10), in the same 48 by 40 box as its neighbours |
| Logo | the Recon mark and word, 103 by 32, at the top bar's left with 16 px above it and to its left, and no title line beside it: the picture's name stays in the window's own title only (Rotem, 2026-09-18) |

### Timeline

Stated by Rotem on 2026-09-16.

| Part | Value |
|---|---|
| Ground | no colour and no top line of its own: the thumbnails sit on the app background (Rotem, 2026-09-19, from a #14171b fill under a 1 px #2a3037 line) |
| Tabs to thumbnails | 16 px (Rotem, 2026-09-19): the 16 px under the tabs in their 64 px band, the thumbnails starting at the timeline's top edge |
| Thumbnail height by default | 112 px, from the timeline's top edge, and the scroller's 12 px band under it (4 px clear, the 4 px thumb, 4 px clear), so the timeline is 124 px tall until it is dragged (Rotem, 2026-09-17: the band, option 1 of three, from 128 with 8 px below, which sat 4 px of every thumbnail under the scroller; 2026-09-19: nothing above, from a 1 px line and 12, by way of 8) |
| Air around the thumbnails | 12 px left of the first, 12 px right of the last, 12 px between them (Rotem, 2026-09-17, from 8) |
| Thumbnail corners | 8 px radius, on the picture and on its cell (Rotem, 2026-09-17, from 13 the same evening, 4 before) |
| Thumbnail delete | a right press on a thumbnail opens the tabs' dropdown with Delete, which deletes it; no × on the thumbnail (Rotem, 2026-09-19, from a × shown on hover) |
| Thumbnail hover | a thumbnail that is not the selected one takes the selected one's 2 px border under the pointer, in `#323A43`, fading in by A1 (Rotem, 2026-09-19, after #414C58 and #536070 the same evening) |
