# History.md - Scan-log Archive

NOT read by default - consult only when digging into old changes.
Moved here verbatim by project-os/rotate.ps1. Movement only: rows are never rewritten, compressed, or deleted.

| Date | Area | What changed |
|---|---|---|
| 2026-09-10 | project-os | **This project now has its own written rules, and they are repeated to the assistant on every message.** ProjectOS was installed into an empty repository: the rule files, the living files, and two guards that refuse a write outside the project or a one-way command. Values only Rotem can supply are marked as waiting, never guessed. |
| 2026-09-10 | project-os | **The install is now in git, and one process correction is on record.** Rotem pointed out that the commit is mine to make and the push is his, so the waiting-on-you framing was wrong and is logged in Mistakes.md. |
| 2026-09-10 | plan | **The product plan now has a written review beside it, and a build plan for the first stage.** Sixteen findings, three of them blocking, and six product decisions left open for Rotem. The build plan carries the proposed component split, the Stage 0 steps with the evidence that closes each one, and a named fallback for every way the stack can fail. |
| 2026-09-10 | plan | **The plan is one file now, not two.** The review, the six open decisions and the technical stages all sit in `project-os/Plan.md`, which is what was asked for; the separate review document is gone and its content moved in whole. |
