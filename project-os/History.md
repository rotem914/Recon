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

## Appendix — deep rows

Newest at the bottom, same as the scan log.

| Date | Task | What changed | What was checked | Result | Risk | Commit before | Rollback |
|---|---|---|---|---|---|---|---|
| 2026-09-10 | Install ProjectOS | Copied `CLAUDE.md`, `Installation.md` and `project-os/` in. Filled the project name, the owner, the role and the reply language; wrote the honest state for stack, local address and checks. Replaced the Map skeleton with the real tree and filled its Data and Ownership tables. Deleted the six example blocks. Adapted `hooks-settings.json` to this project and merged it into `.claude/settings.local.json`. | Both hook commands run in bash and in PowerShell: the text came back whole in each. Destructive guard, fake recursive-delete payload: exit 2 with a reason; a `git status` payload: exit 0. Path guard, a write aimed at the home folder: exit 2; the same write inside the project: exit 0. `rotate.ps1 -DryRun`: every target file resolved, nothing to move. `backup.ps1`: 42 files, all 42 verified back out of the ZIP. No placeholder left in `CLAUDE.md` or `project-os/` outside the two `mcp/` setup tables. Every referenced path resolves except three that are conditional by design (`README.md`, `project-os/Plan.md`, the analytics venv). | Pass | medium | none, the repository had no commits before this | Delete `CLAUDE.md`, `Installation.md`, `project-os/`, `.gitignore`, `.claude/settings.local.json` and `backups/`. Nothing else was touched. |
| 2026-09-10 | Commit the install, record the commit slip | One row added to the Open table in Mistakes.md, then the whole install staged and committed. Nothing else in the docs changed. | Ran the archive script: every target resolved, nothing to move. Checked the staged list by hand: no snapshot, no machine-local settings, no secrets. Read the commit back with git log and git show --stat. | Pass | low | none, this is the first commit in the repository | Nothing to revert to. Delete the files the install row lists, or reset the branch to no commits. |
