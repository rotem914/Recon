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
| App background | `#111217` | The page ground behind the picture and the panels (Rotem, 2026-09-14). |
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

### Window

Stated by Rotem on 2026-09-15.

| Part | Value |
|---|---|
| Size when it opens | 1600 wide by 1160 tall, centred in the main display's work area |

### Timeline

Stated by Rotem on 2026-09-16.

| Part | Value |
|---|---|
| Thumbnail height by default | 112 px, with 8 px clear above and below, so the timeline is 128 px tall until it is dragged |
