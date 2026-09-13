# Recon — Map

The architecture snapshot: the stack in a line, the folder tree, where the data
lives, what owns what.

Read it at task pickup to find your way around. Keep it true, because a map that
lies costs more than no map.

## How you maintain this file

- **Replace the skeleton tree below with the real one during setup.** Walk the
  project root, then write what is actually there. Until you do, this file
  describes a project that does not exist.
- **Update it in the same change** that adds, moves or removes a folder, a route,
  a data store, or a major file. Never as a follow-up task — a follow-up is a
  task that does not happen.
- One line per entry: what it is, not how it works. The how lives in the code.
- Paths are relative to the repository root.
- This is a snapshot, not a plan. Nothing here describes work that has not landed.

## Stack

A Rust host (`host/`, windows-rs for the screen, Tauri v2 for the tray, the hotkey and the
window) and a WebView2 editor (`editor/`, plain HTML and JavaScript, embedded into the binary
at build time). A recommended candidate under Stage 0, per `project-os/Plan.md` part 5.

| Setting | Value |
|---|---|
| Project root | this repository, wherever this copy of it lives |
| Runs locally at | no server: `host/target/debug/recon-host.exe`, with `--editor-check`, `--editor-demo`, `--selftest` and `--bench` as the diagnostic runs |
| Checks | in `host/`: `cargo fmt --check`, then `cargo clippy --all-targets -- -D warnings`, then `cargo build`; `cargo test` for the unit tests |
## Tree

The real tree. The host is the first code in the project, from step S0.1.

```text
Recon/
├── CLAUDE.md                       # entry file, read first every session
├── Installation.md                 # the record of how ProjectOS was installed here
├── .gitignore                      # keeps backups/, .tmp/ and host/target/ out of git
├── host/                           # the Rust host process: native, owns the pixels
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json             # zero windows on purpose: the host runs without one
│   ├── icons/                      # placeholder tray icon, the real one is a design task
│   └── src/
│       ├── main.rs                 # tray, hotkey, the capture flow, the selection guard
│       ├── config.rs               # the hotkey, and where it was read from
│       ├── overlay.rs              # the Win32 selection overlay, one window per display
│       ├── selftest.rs             # --selftest: S0.2's evidence, without a human
│       ├── bench.rs                 # --bench: S0.3's boundary measurement
│       ├── editor.rs                # the editor window, and the region the page may see
│       └── capture/
│           ├── mod.rs              # the capture interface and the frame it produces
│           ├── coords.rs           # the ONE desktop-to-image conversion, with its tests
│           ├── display.rs          # DPI awareness and the live display layout
│           └── screen.rs           # the chosen path: one copy of the whole virtual screen
├── editor/                         # the web view surface, a placeholder until S0.4
│   ├── index.html                  # the editor: the scene, the callout, the text layer
│   ├── editor-checks.js            # --editor-check: S0.4's evidence, not product code
│   └── bench.html                  # the web view half of S0.3, not product code
└── project-os/                     # the process docs and their enforcement
    ├── Plan.md                     # the whole plan, read at task pickup
    ├── Workflow.md                 # the path every task walks
    ├── QA.md                       # what must be true before anything is done
    ├── Conversations.md            # how every reply is written
    ├── Code_review.md              # the review calibration
    ├── Visual_QA.md                # the hands-on testing method
    ├── Map.md                      # this file
    ├── History.md                  # what changed, one row per task
    ├── Decisions.md                # why non-obvious choices were made
    ├── Backlog.md                  # the owner's open items
    ├── BugAtlas.md                 # recurring bug classes
    ├── Mistakes.md                 # the assistant's corrected slips
    ├── Hooks.md                    # what is enforced mechanically
    ├── hooks-settings.json         # the hooks the installer merges in
    ├── install-hooks.mjs           # installs those hooks
    ├── rotate.ps1                  # archives the growing docs at Go commit
    ├── backup.ps1                  # the Go backup snapshot
    ├── guards/
    │   ├── path-guard.mjs          # refuses any write outside the project
    │   └── destructive-guard.mjs   # refuses one-way commands
    └── mcp/                        # one folder per outside server
        ├── Figma/Figma_MCP_Rules.md
        └── Google_analytics/Google_Analytics_MCP_Rules.md
```

Update this tree in the same change that adds the first application folder.

## Data

Where state lives and who is allowed to write it.

| What | Where | Format | Written by |
|---|---|---|---|
| The process docs | `CLAUDE.md`, `project-os/*.md` | Markdown | the assistant, under the rules each file states |
| The hooks setting | `.claude/settings.local.json` | JSON | `project-os/install-hooks.mjs` only. Machine-local, not committed. |
| The hotkey setting | `%APPDATA%\Recon\recon.json` | JSON | nobody yet. Recon only reads it, and says in its log where the value came from. |
| Managed documents | none yet | | arrive with the store at S1.8 |

## Ownership

Which area owns which files. Use this to answer "who owns this?" before changing
anything.

| Area | Files | Notes |
|---|---|---|
| Process and rules | `CLAUDE.md`, `project-os/*.md` | One rule has one home. Never write the same rule in two files. |
| Enforcement | `project-os/guards/*`, `project-os/hooks-settings.json`, `project-os/install-hooks.mjs` | Hooks are read at session start. Re-run the installer after editing the settings file. |
| Outside servers | `project-os/mcp/*` | One folder per server, read before that server's first call. |
| The plan | `project-os/Plan.md` | One file, and there is never a second: the product, the decisions, the architecture and the stages. Free-standing documents go in `notes/`, created when one is needed, never at the root. |
| The host | `host/*` | Tray, hotkey, freeze, overlay and the region today; decode, the store and the clipboard later. It never renders an annotation. |
| The one conversion | `host/src/capture/coords.rs` | The only place allowed to subtract a frame origin. Part 5 names the four coordinate spaces; this file is the edge between two of them. |
| The editor | `editor/*` | The scene, the callout and the text layer. Laid out in image pixels; one CSS transform does the zoom. Embedded into the binary at build time, so an edit here needs a rebuild. |
