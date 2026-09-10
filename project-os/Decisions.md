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
`project-os/Plan.md` part 3, items D to F.

External files gain the invariant in CLAUDE.md rule 11: viewing never modifies and never
imports, so a folder of originals is safe to point Recon at.

Stage 0 gains a decoding experiment, and Stage 1 grows from seven items to twelve,
including PNG Save As, because an annotated external image has to be saveable on day one.

Cost: the first daily-use release is larger than it was, and the viewer has to be good
enough to replace an existing one rather than merely present.

Revisit only if the decoding evidence in S0.4 shows the viewer is impractical on this
stack.
