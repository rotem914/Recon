# Workflow: the mandatory process from idea to delivery

This file is the path every change in Recon walks: request, plan, implementation, QA, documentation, delivery.

## The universal rule

This workflow applies to every task. No exceptions.

Task size does not unlock a shortcut. Adjusting one line of padding walks the same path as building a new screen.

What changes the steps is the owner: a shortcut they type, such as FAST MODE, or their own words for the task at hand (`CLAUDE.md`, "When two rules pull different ways", which also names the rules that bend to neither). `CLAUDE.md` says what each shortcut changes and for how long. Nothing you decide on your own does.

What scales with size is the *depth of the writing*, not the number of steps. A tiny change gets a one-line plan and a one-line history row, but it still gets both.

## Core flow

Idea → classify and state risk → read context → define boundaries → plan → design the QA → implement small → self-check → browser QA → QA checklist → fix loop → auto review on medium+ risk → decisions checkpoint → documentation routing → History row → delivery summary.

## 1. Idea / request

Rotem describes a need. It will be one of:

- a new feature,
- a change to something that exists,
- a bug,
- a UX improvement,
- an infrastructure change,
- a documentation change,
- a process or QA change,
- an architecture decision.

Do not open an editor yet. Understand the job first.

## 2. Classify the task and state its risk

Name what kind of change this is: new feature · change to existing behavior · bug fix · UI change · backend or API change · data or storage change · state and persistence change · styling change · docs or infra change · process change · architecture decision.

The label does not let you skip steps. It tells you which context to read, which checks matter, and which docs will need updating.

**Then state the risk level out loud, at pickup: low, medium, or high.**

| Risk | What it covers |
|---|---|
| Low | Docs. Isolated changes with no behavior attached. |
| Medium | UI behavior, new interaction, multi-file change, anything that changes how you yourself work. |
| High | Data, schema, auth, user records, irreversible actions, security, money. Anything that touches a project invariant (`CLAUDE.md` rule 11). |

This table is the scale's one home: every other file points here.

This is not commentary. Medium or high **arms the automatic review in step 12**, which reads this rating back. Rotem can override your call, and the owner's rating wins.

## 3. Read the context

Before touching anything, read the files `CLAUDE.md` lists under "Read these before you work", as that list says. That list is the one home of the reading order, so this file does not repeat it.

Read them before you form an opinion.

Some files on that list are read on demand rather than every time. The list says when each one applies.

If the task touches an area with its own notes in `project-os/Map.md`, follow that pointer too.

## 4. Define boundaries

Before planning, settle:

- what is in scope,
- what is out of scope,
- which files may change,
- which files must not change,
- what behavior must stay identical,
- what could break,
- whether this touches UI, data, state, persistence, auth, security, or performance.

If scope is ambiguous, ask before implementing, unless the owner already said to proceed on best effort.

## 5. Settle a short plan

Before implementing, settle:

- the goal,
- files likely to change,
- files not to touch,
- the risk level,
- the QA you expect to run,
- how this task would be undone by hand,
- the docs that will need updating,
- and, when the plan runs to more than one step, the model suggested for each
  one with three words on why. EVERY plan with more than one step,
  whatever it is for: a new project, a feature, a migration, a fix broken into
  rounds, a plan written in chat and never saved to a file. The most capable
  one for design, architecture, anything irreversible and anything touching
  data; a faster, cheaper one for mechanical rounds, renames, copy and small
  fixes. It is a suggestion the owner picks from, and a step with no obvious
  fit says so. This applies to a plan in chat as much as to a saved
  `project-os/Plan.md`.

Practical and short. A three-line plan is fine. No plan is not.

### Check every "impossible" before it shapes the plan

Some options get dropped because something seems to forbid them: a platform
limit, a technical claim, an assumption carried over from an earlier doc.
Before such an option leaves the table, or before you call a route impossible,
unavoidable or the only way:

1. Name the constraint in one sentence, and where it comes from. Checked means
   documentation or a probe backs it; a doc, a memory or a habit is inherited,
   and inherited is unchecked.
2. Say which route it limits. A limit belongs to a route, not to the goal, so
   ask what other route reaches the same goal, starting with what the project
   already has. A route that needs something the project lacks is a question
   for the owner (`CLAUDE.md` rule 21).
3. If the option still goes, write down the reason and the routes checked, in
   the saved plan or the Decisions entry that drops it.

An option dropped for cost, taste or scope skips this check. An old written
"impossible" that names no checked routes is unchecked too, so check it before
new work leans on it.

A route found goes to the owner as an option; it changes nothing on its own.
No rule, owner ruling, guard or permission limit is a constraint to route
around: when one blocks a better option, say so and ask. When the goal is
permitted and only one tool failed or was denied, switching to another allowed
tool is not routing around (`CLAUDE.md` rule 6); when a rule or guard forbids
the act itself, it is forbidden through every tool (rules 12 and 22). In chat,
run the check silently and give the result with its reason, not the steps.

## 6. Design the QA before you write code

Decide how you will prove this works *before* it exists.

Derive the checks from the goal, the user flows, the states, the data, and the risks:

- the critical flows,
- the expected default behavior,
- loading, empty, error, disabled, and success states where they exist,
- interactions that must be tested by hand,
- input and output checks,
- reload and persistence checks where relevant,
- network checks where the UI depends on data,
- regression checks on whatever sits next to the change,
- the done criteria.

Which checks go to `project-os/QA.md` and which only to this task's History row: that file's opening says.

## 7. Implement the smallest safe change

Implement the requested change and nothing else (`CLAUDE.md` rules 2 and 18).

- One clear change at a time.
- No unrelated styling.
- No hidden behavior changes.
- No new public interfaces unless the task requires them.

If the work reveals a bigger problem, write it down as a follow-up. Do not silently grow the scope to swallow it.

## 8. Run self-checks

Run what the change deserves. Typically the project check command, plus whatever else applies: build, typecheck, lint, unit tests, a smoke run, a request against the API, a schema validation, a write-then-read test.

Record the exact checks and their results. You will need them in step 15.

## 9. Browser QA for anything visible

Anything a person can see or click gets checked in a running app, not by reading the diff (`CLAUDE.md` rule 6).

**The checks build is rebuilt after the four checks, before any probe.** `cargo build` with no
feature, the third of the four checks, writes the PRODUCT build to `host/target/debug/`, the
same path the checks build has. A probe started there afterwards runs Rotem's own Recon on his
own library. So: after the four checks, `cargo build --features stage0-checks` again before a
demo probe; and every probe reads which build it started, the demo's "stays open" line, before
its first call, and stops when the line is not there. Written on 2026-10-06, the second time
the product build was started by a probe (`project-os/Mistakes.md`, Promoted).

### Verification gate — mandatory before "done"

### Verification gate: mandatory before "done"

Before calling any visible change done, complete one of these three paths and record which one, with each check and its result, in the History row.

- [ ] **Path A: the check ran.** You opened the app in a browser-automation tool, ran the flow, and inspected console, network, and DOM. List each concrete check and its result in the History row.
- [ ] **Path B: no tool was available.** You searched the available tooling for a browser automation tool and found none. Follow `CLAUDE.md` rule 6 for what that means here. Then state the search you ran and its empty result, and hand the owner a manual QA list.
- [ ] **Path C: blocked.** A tool was there, and every route failed: the one you started with and every other browser tool you have. Name each tool you tried and the error it gave, hand the owner the manual QA list, and say in the reply's Known limitation line that the visual check did not run. An app that will not load is not Path C by itself: `CLAUDE.md` rule 16 says who starts the server, and only once that route is spent too is it Path C.

A task claiming none of the three paths is not done.

The two traps on the way, a tool assumed missing and a blocked tool taken for a missing one, are `CLAUDE.md` rule 6.

## 10. Run the QA checklist

Go through `project-os/QA.md`.

If a check that future tasks of this kind will need is missing there, add it now.

## 11. Fix loop

When a check fails:

1. Fix the smallest thing that explains the failure.
2. Re-run that check.
3. Re-run the checks around it.
4. Do not move on to documentation until the relevant checks are green, or the remaining limitation is written down plainly.

## 12. Automatic review on medium or high risk

If step 2 rated this medium or high, the task is not finished. Run the review now, after your own QA is green, before any documentation.

1. Load `project-os/Code_review.md` and review this task's own edits plus whatever they touched.
2. Fix every finding this change introduced, at every severity. Re-verify each fix. If a fix affects something visible, re-verify it in the browser.
3. Report pre-existing findings; never auto-fix them. They wait for the owner's verdict.
4. Name the review result in the History row: found, fixed, pre-existing flagged.

A medium-risk task with no review result recorded is unfinished, not sloppily documented.

Low-risk tasks skip this step, except a change that touches something shared: it still runs the blast-radius trace, as `project-os/Code_review.md` says.

## 13. Decisions checkpoint

Before writing documentation, ask whether this work created or exposed a decision worth keeping.

- Did you choose one approach over another?
- Was there a real tradeoff: technical, product, UX, data, or process?
- Will it constrain future work?
- Would someone later reasonably ask "why was this done this way?"

If yes, add an entry to `project-os/Decisions.md`. The split between the two files is `CLAUDE.md` rule 9.

## 14. Documentation routing

Update only what needs to change.

| Situation | Update |
|---|---|
| Setup or the human-facing overview changed | `README.md` |
| A working rule or project context changed | `CLAUDE.md` |
| The process itself changed | `project-os/Workflow.md` |
| Structure, routes, or file ownership changed | `project-os/Map.md` |
| A check future tasks will reuse was added, changed, or retired | `project-os/QA.md` |
| The check command or dev address changed, or exists for the first time | `CLAUDE.md` "Where things are" and `Go commit` step 4, plus every file that repeats the old value (search for it) |
| A bug pattern came back | `project-os/BugAtlas.md` |
| A non-obvious choice was made | `project-os/Decisions.md` |
| Reply format or tone rules changed | `project-os/Conversations.md` |
| The review bar or its scope changed | `project-os/Code_review.md` |
| The hands-on testing method changed | `project-os/Visual_QA.md` |
| An open item was added or closed | `project-os/Backlog.md` |
| The owner corrected HOW you worked | `project-os/Mistakes.md`, or its rule's own file on a repeat |
| Why a rule exists: its backstory, the incident behind it | `project-os/Rule-reasons.md` |
| Anything was completed | `project-os/History.md` |

Do not write the same rule in two files. If two files need it, one states it and the other points there.

## 15. Add the History row

Every completed change gets two rows in `project-os/History.md`: one in the scan table, one in the appendix. No exceptions, including docs-only changes. That file shows the shape.

The scan row names the behavior that changed, for a reader who was not there. The appendix row carries the date, the task, what changed and where, what was checked and its result, the risk level, how to undo this one task by hand, and, for medium or high risk, the review result. The undo names which files, and what to put back.

Keep it to an index entry, not an essay. The full story is in the commit diff.

## 16. Delivery summary

**A plan's steps run back to back.** Rotem said on 2026-09-13, at the start of Stage 1, to
keep going through the plan and stop only for a question he has to answer or a problem. So
a finished step is delivered and the next one begins in the same turn, without a GO per
step; the reply still names the step and its model (Conversations rule 19), and a step
whose pickup finds a decision that is his stops there, with the question. The model the
plan names for the next step is a suggestion and never a reason to stop: the run continues
on whichever model is loaded, and the reply says which one the plan wanted. Rotem said so
on 2026-09-14, after a stop at the S1.6 to S1.7 boundary asked him to switch.

**Every committed change to the code is built into the release, unasked.** Rotem said on
2026-09-15: build straight away, never ask each time. So the commit that lands the change
is followed in the same turn by the release build, and a reply never offers BUILD as a next
step; since 2026-10-06 that commit is the one `Go commit` makes (rule 22, the kit's flow,
Rotem's call), so the release follows `Go commit`, not each task. The
build is made from the commit alone: the commit archived into `.tmp/release-<commit>/`,
built with `cargo build --release --features own-extensions` (the switch keeps Rotem's
inspector in the Recon he runs, `project-os/Decisions.md` 2026-09-19; a public release is
built without it) into the private target folder `.tmp/release-target/`,
so another session's uncommitted work stays out and a running Recon's hold on its file
does not stop the build. It is copied over `host/target/release/recon-host.exe` at once
when no Recon from that path is running; otherwise a waiting job copies it the moment that
Recon exits, and copies nothing if one from that path is back by then. The hash of the copy
is read back against the build, and the build and the copy get their History rows like any
change. A change to the docs alone builds nothing.

**The build reaches Rotem's Recon without a command from him.** Rotem said on 2026-09-19,
after a session handed him copy commands round after round: the flow is always the same
one. The assistant finishes and builds; when his Recon is open, the reply asks him to quit
it, in one line; the waiting job copies the build in the moment he does; he opens Recon
again. The reply never carries a copy command for him to run. The waiting job is started
as a background task of the session (the shell tool's own background run of
`.tmp/swap-release-<commit>.ps1`), never as a hidden, detached process, which the
permission check of the app refuses. With no Recon open, the copy is a plain copy at once.
Only when both routes were refused does a reply hand him a command, and it says which
refusal it met.

Close with a short summary:

- what changed,
- what was checked,
- what was not checked or is still risky,
- the next step.

When the task left uncommitted work, the Next line says so, as `CLAUDE.md` rule 22 asks.
Format it per `project-os/Conversations.md`. Leave out implementation noise the owner did not ask for — a summary padded with steps that went fine buries the one line that did not.
