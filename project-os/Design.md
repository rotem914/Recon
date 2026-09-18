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
| A1 | Fade in | 144 ms | ease-out | The hover fill of an icon button (`editor/index.html`, `#controls button`); the tick of a done mark as it is pressed (Rotem, 2026-09-16). |

A motion respects the reduced-motion preference: gated at its own surface, the final
state reachable with no animation at all (`project-os/QA.md` §7).

## Colours

| Token | Value | Where it is used |
|---|---|---|
| App background | `#0D0E12` | The page ground behind the picture and the panels (Rotem, 2026-09-17, from #111217, which was his on 2026-09-14). |
| Icon button hover | `#21222C` | The fill of an icon button's square under the pointer (Rotem, 2026-09-15). |
| Icon line | `#C3C6CA` | The line of an icon button's icon in the controls sidebar (Rotem, 2026-09-16). |
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
| Hover | the container fills with the icon button hover colour, fading in by A1 |
| Name | every icon button carries its name and its key as a title, so it has a name without its text |

Used by: the controls sidebar down the left edge of the editor, the eight tools, Copy, Save
As and the mode.

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
| Margin | 16 px above and 16 px below |
| Background | the app background |
| Content | the picture's size in pixels, written as 1200x1500 |
| Text | Google Sans, 16 px, medium (500), centred in the window (Rotem, 2026-09-17, from 14 px) |

The text's colour and line are not stated yet. Provisional in the page until they are: the
top bar title's #cfd4da, on a 16 px line.

### Tab bar

Stated by Rotem on 2026-09-16 for the timeline's tabs.

| Part | Value |
|---|---|
| Plus button | an icon button at the left of the image size container: 32 by 32, no fill of its own, a plus icon, 20 by 20, with round ends, the hover fill by A1 |
| Tabs | beside the plus, once it was pressed: Main first, pinned and never deleted, then the tabs the plus made, in the order they are dragged into |
| Tab text | 14 px |
| Tab corners | 10 px radius |
| Tab name | a click on the selected tab's name edits it in place; Enter keeps it |
| Tab delete | the × on a tab after Main: the first press turns it into a ✓, the second deletes |

The rest of the tabs' look is not stated yet. Provisional in the page until it is: 32 px tall,
the top bar title's #cfd4da, 8 px between, the icon button hover colour as the selected tab's
fill and as the hover; the × and ✓ 16 px icons, shown on hover and on the selected tab; the
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

The fills are not stated yet. Provisional in the page: the thumbnail ×'s dark fill, and the
restore button's green once ticked.

### Settings

Stated by Rotem on 2026-09-18.

| Part | Value |
|---|---|
| Button | a chevron pointing down (Rotem, 2026-09-18, from three dots the same day), left of minimize in the top bar, in the window buttons' own box and icon size: 48 by 40 with a 10 by 10 icon |
| Opens | a modal |
| Content | a shortcut that opens Recon itself, and the shortcut that starts a capture, each picked by pressing it |

The modal's look is not stated yet. Provisional in the page until it is: a 50% black veil,
a 420 px panel in the icon button hover colour on 13 px corners with 24 px of padding, the
title in Google Sans 16 px medium, 14 px rows in the top bar title's #cfd4da, a 32 px field
in the app background on the tabs' 10 px corners, a 1 px #7aa7ff ring while it waits for a
shortcut, a refusal under its field in #ff5252, the veil fading in by A1.

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

The rest is not stated yet. Provisional in the page until it is: the box a 2 px line in the
callout line's #7aa7ff; the size bubble beside the corner the drag ended at, 8 px right of
and below it, dark #1b1f24 with a 1 px #3a4149 border, 4 px corners, white 14 px medium
text, all in image pixels like a note; the × centred on the box's top right corner, the
thumbnail ×'s dark fill and its red on hover, a 1.5 px line, 16 screen pixels at every zoom.

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
| Guide lines | one across and one down through the pointer, 1 px each, the whole width and height of the screen whatever the capture area is, in the blue of the lines inside the circle, #00B9F7, at 72% (Rotem, 2026-09-18) |

From the picture, not his words, so provisional in the overlay until he states it: the
circle sits in a panel of the size bubble's fill and corners, 4 px around it, with the
selection's size above it, the lit window's while hovering and the dragged area's while a
drag lasts, in the size bubble's text and padding; the panel is up the whole time the
overlay is, below the pointer with its left edge 8 px right of it, and on the pointer's
other side where that would leave the display; 8 px to a source pixel, an odd count of them
so the pointer's own is the middle one; a 1 px grid between them, the pixel darkened by
half; the marching frame's #00B9F7 through the middle of the centre pixel; a 2 px ring in
the text's white; every number at 100%, scaled by the display's own scale as the cursor is.
Provisional for the guide lines: the screen is the display the pointer is on, they sit
under the dashed frame and the panel, and the 1 px is not scaled with the display.

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
| Size when it opens | 1600 wide by 1160 tall, centred in the main display's work area |

### Timeline

Stated by Rotem on 2026-09-16.

| Part | Value |
|---|---|
| Thumbnail height by default | 112 px, 12 px clear above it below the timeline's 1 px top line, and the scroller's 12 px band under it (4 px clear, the 4 px thumb, 4 px clear), so the timeline is 137 px tall until it is dragged (Rotem, 2026-09-17: the band, option 1 of three, from 128 with 8 px below, which sat 4 px of every thumbnail under the scroller; the 12 above, from 8, later that day) |
| Air around the thumbnails | 12 px left of the first, 12 px right of the last, 12 px between them (Rotem, 2026-09-17, from 8) |
| Thumbnail corners | 8 px radius, on the picture and on its cell (Rotem, 2026-09-17, from 13 the same evening, 4 before) |
