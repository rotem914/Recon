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
- 2026-09-10 · The stack is a recommended candidate, not a settled decision.
- 2026-09-11 · A note's text size is the user's, with no minimum on-screen size.
- 2026-09-13 · The display proxy is served from a pyramid of halves, by one worker.
- 2026-09-13 · AVIF decodes through the Windows imaging stack, not a bundled libavif.
- 2026-09-13 · The clipboard is published by the host in three formats, PNG first.

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
