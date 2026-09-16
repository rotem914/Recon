# Recon — Mistakes

The waiting room for the assistant's own mistakes.

A mistake corrected in chat lives as long as the session does, and the next
session repeats it. This file is where a correction waits until it has proven
it needs to become a law.

The mechanism is two steps, and it is the whole file:

1. A slip happens and is corrected. It gets ONE row here.
2. The same slip happens again. It stops being a row and becomes a RULE, in
   the file that owns that behavior. The row moves to Promoted.

## What belongs here

Mistakes in HOW you worked:

- you broke a rule that is already written down,
- you decided something that was the owner's to decide,
- you skipped a step the process requires,
- you assumed instead of asking,
- you reported something as done, or as checked, when it was not.

## What does not

- **A code bug** goes to `project-os/BugAtlas.md`. That file maps bugs in the
  product; this one maps bugs in the way you work.
- **What changed** goes to `project-os/History.md`.
- **A product opinion the owner simply overruled.** Their call, not your
  error. Disagreement is not a mistake.
- **A slip whose rule home is obvious.** Write the rule immediately, in its
  own file, and skip the waiting room. This file is for slips with no clear
  home yet, or where it is not yet clear a law is warranted.

## How you use it

- **Write the row in the same reply where the correction landed**, before the
  work continues. A mistake recorded later is a mistake recorded never.
- **Read this file at task pickup.** It stays short on purpose, so there is no
  excuse to skip it.
- **The two tails rotate, Open never does.** `project-os/rotate.ps1` keeps
  the newest 30 rows in Promoted and in Retired at `Go commit`, moving older
  ones verbatim into `Mistakes-archive.md`. That is a ceiling, not permission
  to let this file grow: the rule above still governs.
- **On a repeat, promote it.** Write the rule into the file that owns the
  behavior, then move the row to Promoted with that file named:

| The slip is about | Its rule goes to |
|---|---|
| How you write replies | `project-os/Conversations.md` |
| A step of the process | `project-os/Workflow.md` |
| What counts as checked | `project-os/QA.md` |
| Deciding, scope, permission | `CLAUDE.md` working rules |
| A review's severity or blind spot | `project-os/Code_review.md` |
| Using an outside tool | that server's file under `project-os/mcp/` |

- **Never let it grow into a diary.** It has exactly two ways out: promoted to
  a rule, or retired unrepeated. Nothing accumulates.
- **It is not a confession log and carries no apology.** One line of fact,
  because the next session needs the fact and not the feeling.

## Open

| Date | What I did | What was wanted | Home if it repeats | Times |
|---|---|---|---|---|
| 2026-09-10 | Made the commit conditional on your answer, and called the commit something waiting on you | The commit itself, unasked: rule 22 gives me the commit and gives you the push | `CLAUDE.md` working rules | 1 |
| 2026-09-10 | Justified the technology choice by the owner having used that stack on another project | The right technologies for this task and for years of maintenance, argued on merits | `CLAUDE.md` rule 21 | 1 |
| 2026-09-14 | Ran the four checks and the commit in one chain that did not stop on a red check, so S2.6 went in with clippy failing | A red check stops the commit (`CLAUDE.md`, Go commit step 4): the commit gated on every check, never chained after them | `project-os/Workflow.md` step 8 | 1 |
| 2026-09-16 | Wrote status lines between tool calls through the timeline tasks, and sent reports past the three-line, sixteen-word ceiling, some with no section headings | One line at pickup, then the report, in the layout and length `project-os/Conversations.md` sets | `project-os/Conversations.md` | 1 |
| 2026-09-16 | Read "the line" in "take the line's thickness to a pixel" as the done mark's tick and thinned it, where Rotem meant the circle's border | When a word can name two elements on screen, ask which before building, as `CLAUDE.md` rule 1 says of direction words | `CLAUDE.md` rule 1 | 1 |

## Promoted

| Date | The slip | Where its rule now lives |
|---|---|---|
| 2026-09-10 | Designed a fidelity check that could not detect the failure it targeted, twice in two passes | `project-os/QA.md` §12 |
| 2026-09-10 | Split the plan into two documents twice: first a review beside the plan, then a product plan beside the build plan | `CLAUDE.md` rule 13, one plan file |
| 2026-09-10 | Argued from claims that were not established, twice: a platform behavior stated as documented, then what competing technologies cannot do | `project-os/QA.md` §13 |
| 2026-09-15 | Read a direction word literally twice: styled only the "vertical" scroller where the spec described the sideways one, then centred the sidebar's buttons "horizontally", across a width they already filled, where Rotem meant in its height | `CLAUDE.md` rule 1, a direction word is read against the screen |
| 2026-09-16 | Committed from a tree another session works in twice: first the shared index, whose stale entries undid that session's commit for ten files; then whole files staged from the working tree, which swept that session's uncommitted callout change into my tick commit | `CLAUDE.md` rule 22, a commit from a shared tree carries only its own hunks |
| 2026-09-16 | Put a choice to Rotem in the words of my own design twice: the capture's dimming and the copy in the words of my checks, then the undo plan's three calls as rule names ("the last-thing-done rule") with no scene of what he would see; both times he could not tell what was asked | `project-os/Conversations.md` rule 14, a choice is shown as scenes |

## Retired

| Date | The slip | Why it left |
|---|---|---|
