# QA

This file is the standing checklist: what must be true before any change is called
done, and how to report what you checked.

It exists so "done" means the same thing on every task, instead of whatever felt like
enough that day.

## QA ownership

You check everything you can check yourself.

Never hand Rotem a check a tool could have run. If you have a terminal, run the
build. If you have a browser tool, open the page. Asking them to confirm what you could
have confirmed moves your work onto their desk.

If a check genuinely could not run, say so in the reply, in plain words, with the reason.
An unrun check that is named is information. An unrun check that is silently skipped is a
false report.

## 1. Match QA to change type

Each kind of change fails in its own place. A green build says nothing about a route
that errors, and a working route says nothing about a layout that clips.

| Change type | Required checks |
|---|---|
| Docs only | Read the file back. Confirm every cross-link resolves. |
| App code | Run the project check command. It must pass, not "mostly pass". |
| Server route / API | Start it, call the route, call its error path. |
| Data / storage | Write, read back, reload, confirm it still validates. |
| UI / layout | Browser QA (§2) plus the narrow-width check (§9). |
| Client state / cache | Reload. Mutate, then confirm fresh data arrives. |
| Refactor | Re-check the old behavior. Prove nothing moved. |
| Process / rules | Cross-links resolve, and the change is recorded in `project-os/History.md`. |
| Start with Windows | `cargo test --all-features startup -- --ignored` in `host/` writes, reads back and removes a value of its own name under the real Run key; `reg query` of the key before and after shows no `Recon` value unless the tick is on. The tick, the logon start and its hidden window are Rotem's hand. |
| The capture overlay | Build with `--features stage0-checks` and run `--selftest`: C drags, D escapes, H clicks a window near its corner for the whole window, clicks its one child for the part, drags from the same point, all through the real overlay, and writes the lit part beside the executable to be looked at. C and H also copy the screen below and right of the pointer, mid-drag and while hovering the stand-in's part, to `s02-magnifier-dragging.png` and `s02-magnifier-hovering.png` beside the executable and read them: the panel's ground in the app background 2 px in from its left edge on the circle's centre row, the ring's white 9 px in (8 px of padding since 2026-09-19), the nine cells around the centre equal to the frozen pixels around the pointer, and the light pixels of the size text above the circle, the centre row found by the blue line through it; the pointer's guide lines, one pixel on its row just right of the panel and one on its column 100 px down, each #2554FB at 48% over the frozen pixel there, lit or dimmed, within 3 a channel, against the frozen pixel alone that a screen with no line shows, inconclusive where the two cannot be told apart; the numbers, the grid, the blue lines through the centre and the corners are looked at in the files. H also looks at the lit area on its way from the part to the whole window: 80 ms after the pointer moves, the frame's left band sits between the part's edge and the window's in a strip of the screen, and at 300 ms on the window's edge; with Windows' animation effects off, only the arrival; a look that came past 140 ms is inconclusive, not a failure. A real hotkey drag by hand is Rotem's hand. |
| The timeline's tabs | `--editor-check` section 40: the plus 32 by 32 at the band's left with a 20 by 20 icon, the first press making Main and New tab, a probe capture on a tab landing in its feed and in Main, the list read back from `tabs.json` as saved, a pointer drag putting the last tab right after Main and Main refusing a drag, two new tabs 8 px apart as Main and the first, a right press on a tab opening a menu with Delete alone above the bar, a press elsewhere and Escape closing it, Delete in it deleting the selected tab, a right press on Main opening nothing, Main refusing, a click on the selected tab's name editing it and Enter keeping it, the done mark on a thumbnail in a tab shown on hover and kept once pressed, Ctrl+Z and Ctrl+Shift+Z over a tab added, deleted or renamed and over the mark, with a note's step in between, fullscreen away and back with the timeline. A real capture on a tab, a drag by hand, a real right press and a real Ctrl+Z are Rotem's hand. |
| A capture's automatic copy | In the `--editor-check` log, every `editor_capture_probe` announcement is followed by the host's `copied WxH to the clipboard` line at the probe's size, and no `copied` line follows a fixture or a frame opened; the S1.10 held-clipboard check still passes. A real hotkey capture followed by a paste is Rotem's hand. |
| The thumbnail carries the notes | `--editor-check` section 42: a 320 by 200 capture's thumbnail read back through the region scheme is the picture at its size; a note typed and saved is followed by a thumbnail refresh the host accepted, the pixel inside the bubble dark where the picture's own was not, a pixel away from it unchanged, and the strip's cell holding a new picture; the note deleted and saved puts the pixel back; a note that grows the margin makes the thumbnail the whole composition fitted into the box. A real note on a real capture and a look at the strip are Rotem's hand. |
| The callout's two clicks | `--editor-check` section 41: with the callout tool a first click on the reference scene makes one note anchored there, nothing typed, its bubble at the automatic place; a pointer move carries the bubble by the same distance and the line still starts at the anchor; the second click locks the bubble where it is and the typing has the focus; a move after it moves nothing; the committed note is one undo step, taken by Ctrl+Z and back by Ctrl+Shift+Z where it was locked; Escape, Ctrl+Z and a change of tool between the clicks each drop the unplaced bubble with its number given back and no step added; the text tool still types at one click; a second click landing on the bubble itself locks it. Sections 20 and 32 make their notes with two clicks. Two real clicks by hand are Rotem's hand. |
| The ruler | `--editor-check` section 43: M lights the ruler button, after Blur; a drag makes a box over the area crossed with its size written 8 px right of and below the pointer, following it, the × hidden meanwhile; released, the size stays beside the corner the drag ended at and the ruler is one step of history; the × is a round 16 by 16 button with a 12 by 12 X at the top right corner, hidden until hover, 16 screen pixels centred on the corner at zoom 1 and 2 alike; the export carries the box and its size and no button; a drag on the ruler moves it whole and Ctrl+Z puts it back; a click makes nothing; it is back from the disk after a restart; a press on the × deletes it, Ctrl+Z brings it back, Ctrl+Shift+Z deletes it again. The magnifier: gone with the pointer off the stage; with the ruler in hand a 128 by 128 panel 8 px right of and below the pointer, the host's own drawing (`host/src/magnifier.rs`), which names its circle, cell and label on the panel's element for the checks; the middle cell and two neighbours equal to the host's 1:1 pixels there, which an offset of one pixel would turn red on the probe's checkerboard; the blue on the middle pixel's left edge; the size above the circle during the drag with the ruler's own label hidden; the label back and the circle gone at the release. A real hover and a real × press are Rotem's hand. |
| The crop | `--editor-check` section 44: K lights the crop button, after Ruler; with it in hand a frame with eight handles sits on the picture's edges, the handles 10 screen pixels at zoom 1 and 2 alike; a handle dragged inward moves the frame and dims the outside while the drag lasts and cuts the picture at the release, the size band, the composition, the canvas painted and the view's origin all the crop's; a corner moves two edges; a note keeps its image coordinates and the margin stays; an outward drag moves nothing and a side stops at 8 px; the export layer is shifted by the crop's origin with the frame left out, the export check is the crop's size with zero source mismatches and its top left the source's pixel at the crop's origin; the thumbnail after the save is the crop fitted into the box; Ctrl+Z and Ctrl+Shift+Z walk the crops with the size band; the crop is back from the disk after a restart; the frame goes with another tool and returns with the crop tool. A real drag of a handle by hand, and a paste of the cropped copy, are Rotem's hand. |
| The colour picker | `--editor-check` section 45: I lights the picker's button, after Crop; with it in hand the zoom circle is up with the HEX above it equal to the picture's own pixel under the pointer, asked of the host; a click puts that HEX on the clipboard as text, read back the way a paste would, says so, and puts the tool down; the pick is no step of history and makes nothing; a second click on a pixel of another colour copies that one, so the clipboard follows the click; a click off the picture copies nothing, says nothing and keeps the tool; over a file only viewed the button puts the picker in hand, the file stays viewed, the click copies the HEX the circle showed and the store gains no document. `cargo test`: a colour is written as six upper case digits. Section 28 counts eight sidebar buttons. A real click and a real paste, and a look at the eyedropper's drawing, are Rotem's hand. |
| The review 4 fixes | `--editor-check`: section 23, a save refused before a switch of picture comes back as NOT SAVED and lands by the timer once the store is back; section 29, the last document deleted leaves no shape on the empty stage, and a delete whose neighbour cannot be shown still deletes, into the trash, the editor empty; section 39, a capture deleted and undone inside its own encode comes back whole, record and image in one folder; section 41, Escape after the second click with nothing typed gives the number back. `cargo test`: a write beside a folder that is not there is refused and makes no folder. |
| The editor brought up after a capture | In the `--editor-check` log, section 36's "a minimized window comes back when the host shows it" passes: the window minimized by its own button, then not minimized and visible after the host's show. A real hotkey capture with the editor minimized, and one with it open behind another window, is Rotem's hand. |
| The mode follows what is done | `--editor-check` section 20: the C key over a viewed file ends in annotation with the callout in hand and one document made; a finished note leaves no tool in hand and the stage unarmed; a press on that note's text edits it again; a press that draws nothing keeps the rectangle tool, a drawn rectangle puts it down, and Ctrl+Z takes the rectangle back with the tool still down; a tool picked over a file annotated before resumes its one document; no `#mode` button exists and the A key changes no mode. Section 28 counts seven sidebar buttons, with no Copy or Save As among them; section 33 has no highlight button, H picking nothing, and a highlight drawn before shown nowhere. A real note typed and a real click back on it are Rotem's hand. |
| Settings' second capture shortcut | `--editor-check`, the Settings section: a Capture 2 row under Capture, the same size, showing None with no ×; Ctrl+Alt+P picked there is kept beside the first and written to the file as `second_hotkey`; the first capture's shortcut refused there, and Capture 2's refused for Open Recon, in words; its × clears it, in the file too, and the first stays. `cargo test settings::` proves either capture shortcut starts a capture. A real press of the second shortcut is Rotem's hand. |
| The wheel in a fast spin | `--editor-check`, after the wheel's own checks: five notches sent in one turn leave the canvas as wide as its held region is at the new zoom, read before a region can land, with `body.zooming` on and the scene at the new scale; at rest neither `zooming` nor `panning` is on, the region is at the last zoom and the canvas at its own size; the picture's info carries `opaque` as a boolean. `cargo test`: one pixel at alpha 254 is not opaque. A real fast spin on a large picture is Rotem's hand. |
| An extension in the demo editor | Build with `--features stage0-checks`, set `RECON_EXTENSIONS` to a folder holding an unpacked extension and run `--editor-demo`: the log names the folder and the profile `s-webview-ext` beside the executable, the WebView2 process runs on that profile and never on `com.rotem.recon`, the window stays open after the demo's verdict, and the extension answers on the page. Without the variable the demo closes itself as before. A real key held and a real click are Rotem's hand. |
| Rotem's inspector in his own build | The four checks pass, `--all-features` compiling the switch and the plain `cargo build` compiling without it. A build with `--features own-extensions`, started with `RECON_PRODUCT_EXTENSIONS` naming the extensions folder, logs `extensions from ... in the profile ...webview-ext`, its WebView2 process runs on that profile, and the extension answers on the editor's page; without the variable, or in a build without the switch, the log has no such line and the profile is `com.rotem.recon`. It cannot run beside a Recon already open, which answers a second start itself, so the start is Rotem's hand. |
| The thumbnail's delete | `--editor-check`, the timeline's delete section: no × on a thumbnail; a right press on one opens the tabs' menu with Delete alone and deletes nothing, and the system menu stays shut; a press elsewhere closes it; Delete in it sends the document to the trash, the one on screen unchanged; on an opened file it takes the pointer off, the file byte for byte as it was, and Ctrl+Z puts it back. A real right press is Rotem's hand. |

If a change spans rows, run every row it spans.

## 2. Browser QA for anything visible

Any change a person can see or operate is verified in a running browser, at its local address once one exists.
A passing build proves the code compiles. It proves nothing about what the screen does.

- Open the changed screen in a fresh tab.
- Confirm the changed element is actually there.
- Do the real interaction — click it, type in it, submit it.
- Check the console (§3).
- Read the DOM or the computed style when the change is about state or styling. Pixels
  lie; a rule that never matched usually still looks plausible.
- Look at the neighbours for regressions.
- Reload if anything was saved.

If you have no browser tool, first prove it (§11), then give a manual check list instead —
numbered, specific, one action per line.

## 3. Console check

A visible change is not verified until you have read the console.

Why: most browser failures never reach the screen. An exception stops one script, the
rest of the page renders anyway, and the result looks like it worked.

Four allowed outcomes. Write one of them, verbatim shape:

- `Passed: no new console errors`
- `Passed: known existing error only` — and name it
- `Failed: <the error>` — then fix and re-check
- `Not run: <why>`

There is no fifth outcome. "Console looked fine" is not one of these.

## 4. Persistence and reload

When a change writes anything that outlives the page, prove the write reached the store.

Why: the screen in front of you already holds the value in memory. It renders the same
whether the write succeeded or vanished.

- Inspect the stored data after the write.
- Confirm it reads back intact and passes its own validation.
- Reload, and confirm the state survived — or reset on purpose, if that was the point.
- Never delete or rewrite the owner's content to make a check pass.

## 5. Atomic write guard

This applies where the app writes files itself. A transactional database gives you
the same guarantee already — there, this section asks nothing.

Every write to the data store writes to a temp target first, then swaps it into place.

Why: a crash halfway through a direct write leaves a half-written file where live data used
to be. The swap is the whole point — either the old file or the new one, never a torn one.

If a new code path writes directly over live data, that is a bug in the change, not a style
preference.

## 6. Accessibility basics

A control that only a mouse can reach does not exist for the people who do not use one.
These are the cheap checks that catch most of it.

For anything interactive:

- **Keyboard reachable.** Every control can be reached and operated without a pointer.
- **Focus is visible.** A focused control shows a ring you can see against its own
  background, not just against white.
- **Test focus with a real keypress.** Focusing an element from the console does not
  always trigger the same focus styling a keyboard does, so a healthy ring can measure as
  absent. Send an actual key.
- **Accessible names.** Inputs have labels. Icon-only buttons have names. Images have an
  alt decision — authored text, or an empty alt on purpose for decoration.
- **No pointer-only path to a critical action.** If the only way to submit, confirm, or
  dismiss is a hover or a drag, the action is unreachable for some people.
- **Headings stay in order**, and new content sits inside the page's main landmark.
- **Contrast is measured, not eyeballed.** Over a photo or a gradient, sample the real
  pixels behind the text and judge the worst one, not the average.

## 7. Reduced motion

Animation respects the reduced-motion preference. The final state stays reachable with no
animation at all, because motion is how content arrives, never the content itself — and
for some people large motion is physically unpleasant.

Gate each animation at its own surface. Never add one blanket rule that zeroes every
duration everywhere. An entrance that deliberately waits at frame zero (§8) has nothing
left to release it, so it freezes there and hides its content for good.

## 8. Cold-asset check for entrance animations

When an animation reveals something — an image, text measured off a loaded font, an
element whose geometry a script reads — run that entrance once with the thing genuinely not
cached.

Why: a warm cache hides this entire bug class. On your machine the asset is already there,
so the reveal always has something to reveal. A first-time visitor gets the animation
running on an empty box, finishing before the content arrives.

- Force the asset to be cold — a unique query string, a cleared cache, a throttled network
  — and confirm the animation holds at its start until the asset lands, then plays.
- Confirm the gate releases on **both** success and failure. A failed load must not leave
  the gate stuck.
- Confirm it fails open: if the gate never resolves at all, or scripting is off, the
  element ends **visible**. An entrance may degrade to no animation. It may never degrade
  to permanently hidden.
- Confirm the reduced-motion path still lands on the final state (§7).

## 9. Narrow-width check for any layout change

Any change to layout, type, or spacing is checked at your smallest supported width before
it is called done. A narrow screen is a real surface, not a fallback.

**Resize first, then load.** A page that was loaded wide and then narrowed is not the same
page. Scripts that measure on load re-fit on resize. A page that has lived across several
widths reports geometry no fresh visitor ever sees. Set the viewport, then navigate or
reload. A measurement taken without that reload is unproven.

**Check both sides of every breakpoint.** One pixel below it and one pixel above. A rule
that lands on only one side is invisible at both extremes — you will not catch it at a
typical phone width or a typical desktop width.

What to check:

- **No horizontal overflow.** The document's scroll width must not exceed the viewport
  width. Probe several widths, not one.
- **Wide content stays inside its box.** A no-wrap heading inside a clipped parent fails
  silently. Measure the element against its container.
- **Wide layouts are untouched.** Re-measure anything the narrow rule could have moved — a
  shared token, a base rule you overrode. A narrow fix that shifts the wide layout is a
  regression.
- **Specificity, not source order.** A media query adds no specificity of its own. A
  narrow-width rule must out-rank the rule it overrides, and still come after it.
- **Tap targets** stay large enough to hit after any shrink.

Measure, do not eyeball. Read the numbers out of the page; a screenshot at the wrong scale
will agree with whatever you already believe.

## 10. No vague QA

Never write `manual QA passed`, `looks good`, or `tested`. They say nothing, and they read
exactly like a check that was skipped.

| Bad | Good |
|---|---|
| `Manual QA passed` | `Opened the settings screen, changed the name field, confirmed the save indicator fired and the stored record updated.` |
| `Looks good` | `Changed list-row padding, opened the screen, checked alignment, hover, and that nothing clipped at the narrow width.` |
| `Tested` | `Ran the build and the tests; passed. Loaded the page; no console errors.` |

Two lines, always: what you ran, and what it returned.

```md
**Verification run**
Ran the project checks, then opened the changed screen, made an edit, reloaded.

**Verification result**
Passed: checks green, the edit rendered, the record updated, the change survived the
reload. Console: no new errors.
```

## 11. Prove a tool is missing before you claim it is

In many setups, tools are not loaded until something asks for them. They are invisible by
default, which makes "I don't see a browser tool" feel true when it is not.

Before you ever write that no browser tool exists:

1. Search the available tools for one.
2. If anything matches, load it and use it. No exceptions, and no "the owner can check
   this manually".
3. Only if the search returns nothing, say that you searched and found none — then fall
   back to the manual check list.

A tool that loaded but was **refused** is a different case. Name the tool, say it was
refused, use another route, and never report it as "no tools available".

The failure mode this blocks: declaring early in a task that you have no browser, then
repeating it for the rest of the task to stay consistent with yourself.

## 12. A check must be able to fail

Name the two things a check compares, and the defect it would reveal. If you cannot say
what failure would turn it red, it is not a check.

Two ways a comparison lies, and both have already happened here:

- **It compares two outputs of the same code.** That proves the code is deterministic,
  not that either output is correct. Whatever you actually doubt has to sit on one side
  of the comparison.
- **It compares things that were never meant to match.** A check that fails by design
  gets muted, and a muted check protects nothing.

State the tolerance and the environment wherever either one matters. An exact
comparison and a tolerant one answer different questions, and neither answers both.

## 13. A claim carries its evidence class

Every claim in a project document is one of three things, and it says which: **measured
here**, **documented with a citation**, or **inferred**. The third one is written as
inferred, in the sentence, not left for the reader to work out.

This bites hardest on a claim about what some other technology cannot do. "That framework
has no editing model", "that platform cannot be made accessible", "this API is the only one
that can do X": those are the shape that gets believed and then decides an architecture.
Cite it, or write what is actually known, which is usually "not established here".

Why this is a rule: two plan revisions on one day argued from claims that were not
established, one about a platform behavior and one about competing technologies. Both
survived a review because they read like facts. A wrong fact in a plan is more expensive
than a missing one, because nobody goes looking for it.

## Checklist before delivery

- [ ] Task type identified, risk level stated.
- [ ] Context files read.
- [ ] Scope boundaries named — including what you did not touch.
- [ ] Smallest safe change used.
- [ ] The project checks run and passing.
- [ ] Browser QA run for anything visible.
- [ ] Layout, type, or spacing touched → narrow-width check run, both sides of each
      breakpoint, wide layout re-measured (§9).
- [ ] Entrance animation touched → cold-asset check run (§8).
- [ ] Storage touched → persistence and reload check run (§4).
- [ ] Medium or high risk → code review run (`project-os/Code_review.md`), result named.
- [ ] QA wording is concrete, not vague (§10).
- [ ] `project-os/History.md` row added.
- [ ] `project-os/Decisions.md` updated if a non-obvious choice was made.
- [ ] Every check names what it compares and what failure would turn it red (§12).
- [ ] Every claim about a platform or another technology is measured, cited, or written as inferred (§13).
- [ ] Reply names the checks that failed or surprised you — not the ones that passed as
      expected.
