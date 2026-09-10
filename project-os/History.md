# Recon — History

The change log. Every completed change lands here, in two layers: a scan table
you always read, and an appendix you read only when digging.

The scan table answers "what happened lately". The appendix answers "what exactly
did that change do, and how do I undo it".

## How you maintain this file

- **After every completed change**, add one scan row AND one appendix row. Both,
  in the same change that did the work.
- **One change = one row.** Not one row per file, not one row per session. A log
  that has to be reassembled from fragments is a log nobody reads.
- Write the scan row for a reader who was not there. Name the behavior that
  changed, not the files.
- Keep appendix rows short — a few lines, not an essay. The full story is in the
  commit diff; a real decision belongs in `project-os/Decisions.md`.
- Record the commit SHA from **before** the change. That is the rollback target.
- Never rewrite or delete a past row. Correct a wrong one by adding a new row —
  an edited log cannot be trusted about anything.
- A `medium` or `high` risk row names its review result under **What was
  checked**: findings found, findings fixed, pre-existing ones flagged. A
  medium-or-higher row without that is a task that is not finished.
- Never write "tested" or "QA passed". Those phrases record nothing. Name the
  input, the screen, and what happened.
- When this file gets long, move the oldest rows into an archive file beside it.
  `project-os/rotate.ps1` does exactly that at `Go commit`, and creates
  the archive the first time it is needed. Rows move **verbatim** — never
  rewritten, never summarized, never merged, because the detail you drop is the
  one the next reader needed. Never hand-move rows: the script dedups, so it is
  safe to run every time, and a hand-move breaks that guarantee.

## Risk scale

The scale's one home is `project-os/Workflow.md` step 2 — read it there, so the
two files can never disagree. State the level at task pickup; the owner's
override wins.

## Scan log

Newest at the bottom.

| Date | Area | What changed |
|---|---|---|
| 2026-09-10 | project-os | **This project now has its own written rules, and they are repeated to the assistant on every message.** ProjectOS was installed into an empty repository: the rule files, the living files, and two guards that refuse a write outside the project or a one-way command. Values only Rotem can supply are marked as waiting, never guessed. |
| 2026-09-10 | project-os | **The install is now in git, and one process correction is on record.** Rotem pointed out that the commit is mine to make and the push is his, so the waiting-on-you framing was wrong and is logged in Mistakes.md. |
| 2026-09-10 | plan | **The product plan now has a written review beside it, and a build plan for the first stage.** Sixteen findings, three of them blocking, and six product decisions left open for Rotem. The build plan carries the proposed component split, the Stage 0 steps with the evidence that closes each one, and a named fallback for every way the stack can fail. |
| 2026-09-10 | plan | **The plan is one file now, not two.** The review, the six open decisions and the technical stages all sit in `project-os/Plan.md`, which is what was asked for; the separate review document is gone and its content moved in whole. |
| 2026-09-10 | plan | **The plan is corrected where it was guessing.** The elevated-window hotkey claim is withdrawn to something to verify, the latency harness gained the mark it was missing, the coordinate contract became four named spaces, the output test became three separate checks, and Stage 0 shrank to one capture path and two real destinations. Callout numbers keep their gaps in every output. |

## Appendix — deep rows

Newest at the bottom, same as the scan log.

| Date | Task | What changed | What was checked | Result | Risk | Commit before | Rollback |
|---|---|---|---|---|---|---|---|
| 2026-09-10 | Install ProjectOS | Copied `CLAUDE.md`, `Installation.md` and `project-os/` in. Filled the project name, the owner, the role and the reply language; wrote the honest state for stack, local address and checks. Replaced the Map skeleton with the real tree and filled its Data and Ownership tables. Deleted the six example blocks. Adapted `hooks-settings.json` to this project and merged it into `.claude/settings.local.json`. | Both hook commands run in bash and in PowerShell: the text came back whole in each. Destructive guard, fake recursive-delete payload: exit 2 with a reason; a `git status` payload: exit 0. Path guard, a write aimed at the home folder: exit 2; the same write inside the project: exit 0. `rotate.ps1 -DryRun`: every target file resolved, nothing to move. `backup.ps1`: 42 files, all 42 verified back out of the ZIP. No placeholder left in `CLAUDE.md` or `project-os/` outside the two `mcp/` setup tables. Every referenced path resolves except three that are conditional by design (`README.md`, `project-os/Plan.md`, the analytics venv). | Pass | medium | none, the repository had no commits before this | Delete `CLAUDE.md`, `Installation.md`, `project-os/`, `.gitignore`, `.claude/settings.local.json` and `backups/`. Nothing else was touched. |
| 2026-09-10 | Commit the install, record the commit slip | One row added to the Open table in Mistakes.md, then the whole install staged and committed. Nothing else in the docs changed. | Ran the archive script: every target resolved, nothing to move. Checked the staged list by hand: no snapshot, no machine-local settings, no secrets. Read the commit back with git log and git show --stat. | Pass | low | none, this is the first commit in the repository | Nothing to revert to. Delete the files the install row lists, or reset the branch to no commits. |
| 2026-09-10 | Review the product plan, write the Stage 0 build plan | Added `notes/2026-09-10-plan-review.md` (16 findings with severities and verdict boxes, plus six owner decisions) and `project-os/Plan.md` (component split, Stage 0 steps S0.1 to S0.8 each with a suggested model and its evidence, go or no-go fallbacks, Stage 1 order). Updated the Map tree and Ownership table for the new folder and file. | Read both documents back: every path they name resolves, the finding counts match the summary line, and each of the six decisions matches its finding. The rule 17 review of my own output found three defects, all fixed: long dashes in three of my headings, a missing decision pointer in F2, and steps naming no model. Nothing was measured or benchmarked, since the project has no code. | Pass | medium | `4757906` | Delete the two new documents and revert the Map tree and Ownership rows. |
| 2026-09-10 | Merge the review into the one plan | `project-os/Plan.md` rewritten as eight parts: the review with its sixteen findings and verdict boxes, the six decisions, the architecture, Stage 0, the go or no-go fallbacks, Stage 1, the later stages, and what is measured against what is assumed. `notes/2026-09-10-plan-review.md` deleted, its content moved into part 1. Map tree and ownership row updated. | Read the merged file back: sixteen findings present, the counts match the summary line, each decision matches its finding, each step names the findings it carries, and no path it names is missing. No long dash in my own text. Nothing measured, the project has no code. | Pass | medium | `caa3340` | Revert the merge commit; the deleted review document is intact in `caa3340`. |
| 2026-09-10 | Revise the plan on Rotem correction pass | `project-os/Plan.md` rewritten as revision 2 in ten parts. F2 rerated from blocking to important and its documentation claim withdrawn. F3 given six timestamp marks and two named intervals, with the drag excluded. F11 resolved against renumbering. Part 4 given four coordinate spaces, with the annotation margin stored as an edge offset so a growing margin rewrites no coordinate. S0.4 split into three fidelity checks, one of them copying while the caret is active. Stage 0 reduced to six steps, one capture path, two destinations, no packaging. Stage 1 stripped of Rogers preparation and given the undo acceptance sequence. Part 9 split into measured, documented, to verify, assumed. New part 3 for the three answers needed before Stage 0, new part 10 for the sources. Two entries added to `project-os/Decisions.md`. | Read the revision back: sixteen findings, counts now two blocking, eight important and six small, and the summary line agrees. Every decision in part 2 points at a step that exists. Every carried-into target exists. Every path named resolves. No long dash in my own text. Self-review found three defects, all fixed: F4 pointed at the wrong part 3 decision, the header said nine parts against ten, and the parts list omitted the sources. Nothing measured, the project has no code. | Pass | medium | `6fb217a` | Revert the revision commit; revision 1 is intact in `6fb217a`. |
