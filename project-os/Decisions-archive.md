# Decisions.md - Archive

NOT read by default - consult only when digging into an old entry.
Moved here verbatim by project-os/rotate.ps1. Movement only: nothing is rewritten, compressed, or deleted.

## Archived decisions
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
