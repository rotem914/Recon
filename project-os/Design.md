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
| A1 | Fade in | 144 ms | ease-out | The hover fill of an icon button (`editor/index.html`, `#controls button`). |

A motion respects the reduced-motion preference: gated at its own surface, the final
state reachable with no animation at all (`project-os/QA.md` §7).

## Colours

| Token | Value | Where it is used |
|---|---|---|
| App background | `#111217` | The page ground behind the picture and the panels (Rotem, 2026-09-14). |
| Icon button hover | `#21222C` | The fill of an icon button's square under the pointer (Rotem, 2026-09-15). |
| Scroller | `#21222C` | The thumb of the timeline's scroller once its rows wrap (Rotem, 2026-09-15). |

## Components

### Icon button

Stated by Rotem on 2026-09-15 for the controls sidebar.

| Part | Value |
|---|---|
| Container | 32 by 32, square, no fill of its own |
| Icon | 20 by 20, centred, a stroke in the button's text colour |
| Between buttons | 8 px |
| Hover | the container fills with the icon button hover colour, fading in by A1 |
| Name | every icon button carries its name and its key as a title, so it has a name without its text |

Used by: the controls sidebar down the left edge of the editor, the six tools, Copy, Save
As and the mode.

### Scroller

Stated by Rotem on 2026-09-15 for the timeline along the bottom.

| Part | Value |
|---|---|
| Thumb | 4 px thick, fully rounded, the scroller colour |
| Track | no fill; 4 px clear of the screen's bottom edge and 4 px clear of the strip's top |

Used by: the timeline's vertical scroller, shown once its thumbnails wrap into rows. The
one-row strip's sideways scroller is still the system's.
