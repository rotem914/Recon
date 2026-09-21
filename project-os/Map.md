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
| Runs locally at | no server: `host/target/debug/recon-host.exe`, and `--open <file>` opens a file into it. Built with `--features stage0-checks` it also answers `--editor-check`, `--editor-demo`, `--capture-demo`, `--open-demo <file>`, `--make-fixtures <dir>`, `--decode-report <dir>`, `--selftest` and `--bench`; `--release` for any number that will be quoted |
| Checks | in `host/`: `cargo fmt --check`, then `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo build`, then `cargo test --workspace --all-features` |
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
│   ├── tauri.conf.json             # zero windows in the config: the host creates its own, hidden
│   ├── capabilities/default.json   # what the editor page may call: the host's commands and events
│   ├── icons/                      # the Recon logo: icon.ico for the exe and the taskbar, icon.png for the tray
│   ├── pixels/                     # recon-pixels: crop, resample and the scrolling capture's stitcher, optimised in every profile
│   ├── references/s06/             # the six reviewed reference outputs and their environment (S0.6)
│   └── src/
│       ├── main.rs                 # tray, hotkey, capture to editor, the selection guard, the diagnostic flags
│       ├── marks.rs                # S0.7's marks on one clock: hotkey to overlay usable, selection to editor usable
│       ├── platform.rs             # what the platform is: each display's HDR state, who owns the foreground window
│       ├── focus.rs                # the return target: the application the capture began in, back in front on hide
│       ├── registration.rs         # the file types in "Open with" and Default apps, under the user, never a default taken
│       ├── startup.rs              # Start with Windows: the tray's tick, one value under the user's Run key, and --startup for a start in the tray
│       ├── dialog.rs               # Ctrl+O and Ctrl+S: Windows' own picker and Save As, owned by the editor, on their own thread
│       ├── export.rs               # Save As: the suggested name, the last export folder, a new file only, an available name when one exists
│       ├── folder.rs               # the folder context: listed once, logical order, previous and next, a gone file skipped
│       ├── store.rs                # the document store: one folder per document, source.png once, document.json atomically
│       ├── measure.rs              # --measure and --walk: the thirty-run measurement and memory sample, feature-gated
│       ├── config.rs               # the three shortcuts: read at startup, saved by Settings, and where they were read from
│       ├── overlay.rs              # the Win32 selection overlay, one window per display
│       ├── magnifier.rs            # the zoom circle's panel, one unit: drawn here for the capture, the Ruler and the Color picker
│       ├── scrolling/              # the scrolling capture: the person scrolls, Recon joins what passes
│       │   ├── mod.rs              # the live part: the dim with the area clear, the bar, Enter and Escape, the copies handed to the stitcher
│       │   ├── detect.rs           # whether the area under the pointer scrolls, asked on a thread of its own
│       │   └── draw.rs             # a bitmap with its own transparency, round shapes, text; the round button's picture
│       ├── selftest.rs             # --selftest and --capture-demo: S0.2's evidence, feature-gated
│       ├── settings.rs             # Settings: the global shortcuts swapped live, two for the capture, a taken one refused, all let go while one is pressed
│       ├── bench.rs                 # --bench: S0.3's boundary measurement, feature-gated
│       ├── editor.rs                # the editor window, the image and its pyramid, the one region worker, copy, the managed documents in memory
│       ├── compose.rs               # the composer: source exact at the margin offset, the layer over it
│       ├── clipboard.rs             # the clipboard: PNG, CF_DIBV5 and CF_DIB in one transaction
│       ├── capture/
│       │   ├── mod.rs              # the capture interface and the frame it produces
│       │   ├── coords.rs           # the ONE desktop-to-image conversion, with its tests
│       │   ├── display.rs          # DPI awareness and the live display layout
│       │   ├── pointer.rs          # the mouse pointer at the freeze: its picture and place, drawn into the frozen pixels
│       │   └── screen.rs           # the chosen path: one copy of the whole virtual screen
│       └── source/
│           ├── mod.rs              # the image source: sniff, open, the decoded-image contract, sRGB once
│           ├── raster.rs           # PNG, JPEG, BMP, GIF, WebP through the image crate; orientation once
│           ├── svg.rs              # SVG through resvg, rasterized at a chosen scale
│           ├── wic.rs              # TIFF, HEIC, AVIF through the Windows Imaging Component
│           ├── fixtures.rs         # --make-fixtures: the marked test files, feature-gated
│           └── report.rs           # --decode-report: one evidence line per file, and the hash check
├── plans/                          # a feature's complete plan as a file of its own, only at Rotem's word
│   └── 2026-09-21-screen-recording-plan.md  # Stage 5, screen recording: nothing of it is built
├── editor/                         # the web view surface, a placeholder until S0.4
│   ├── index.html                  # the editor: the scene, the callout, the text layer
│   ├── editor-checks.js            # --editor-check: S0.4's evidence, not product code
│   ├── fonts/                      # Google Sans, medium, Latin, from Google Fonts, with its OFL.txt; the .woff2 for the page, the same face as .ttf for the host's magnifier
│   └── bench.html                  # the web view half of S0.3, not product code
└── project-os/                     # the process docs and their enforcement
    ├── Plan.md                     # the whole plan, read at task pickup
    ├── Workflow.md                 # the path every task walks
    ├── QA.md                       # what must be true before anything is done
    ├── Conversations.md            # how every reply is written
    ├── Code_review.md              # the review calibration
    ├── Visual_QA.md                # the hands-on testing method
    ├── Design.md                   # the design system: Rotem's tokens and components, read before a visible change
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
| The two shortcuts | `%APPDATA%\Recon\recon.json`, the keys `hotkey` and `open_hotkey`, and `capture_pointer`, Settings' switch for the mouse pointer | JSON | the host only, when Settings changes a shortcut: every other key kept, flushed and renamed into place. Read at startup, and the log says where the value came from. |
| Start with Windows | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, the value `Recon` | the command `"<exe>" --startup` | the host only, at the tray's "Start with Windows" tick; removed when it is unticked |
| Recon's trash | `%LOCALAPPDATA%\Recon\trash\<number>\` | a deleted document's folder, whole, plus `trashed` with the time; removed for good at a startup thirty days on | the host only, at a delete and at the startup sweep |
| The timeline's tabs | `%LOCALAPPDATA%\Recon\tabs.json` | JSON, schema 1: the selected tab, and every tab after Main with its name and the ids of its captures; rewritten whole through a temporary file. One that does not parse is set aside as `tabs.broken-<ms>.json` | the host only, at every change the page makes to the tabs. The checks use `host/target/debug/tabs.json` instead |
| The opened files | `%LOCALAPPDATA%\Recon\opened.json` | JSON, schema 1: for each file opened by name, its path, its size in pixels and its number; never pixels, and its thumbnail is made in memory. Rewritten whole through a temporary file; one that does not parse is set aside as `opened.broken-<ms>.json` | the host only: when a file is opened by name, taken off the timeline, annotated, or found gone. The checks use `host/target/debug/opened.json` instead |
| Managed documents | `%LOCALAPPDATA%\Recon\documents\<number>\` | `source.png`, the preserved image, written once; `document.json`, schema 1, the size, the source and the page's notes, rewritten whole through a temporary file; `thumb.png`, the timeline's thumbnail, made once from the image and written anew with the notes composed over it at every save of them | the host only: the record and the image at a capture or at Annotate, the notes on every save the page sends. The checks use `host/target/debug/s18-store` instead, never this folder |

## Ownership

Which area owns which files. Use this to answer "who owns this?" before changing
anything.

| Area | Files | Notes |
|---|---|---|
| Process and rules | `CLAUDE.md`, `project-os/*.md` | One rule has one home. Never write the same rule in two files. |
| Enforcement | `project-os/guards/*`, `project-os/hooks-settings.json`, `project-os/install-hooks.mjs` | Hooks are read at session start. Re-run the installer after editing the settings file. |
| Outside servers | `project-os/mcp/*` | One folder per server, read before that server's first call. |
| The plan | `project-os/Plan.md` | One file, and there is never a second: the product, the decisions, the architecture and the stages. Free-standing documents go in `notes/`, created when one is needed, never at the root. |
| A feature's plan of its own | `plans/*` | Only when Rotem asks for one as a file. `project-os/Plan.md` names it from the stage it belongs to, and the part it replaces there says it is superseded. Today: screen recording, Stage 5. |
| The host | `host/*` | Tray, hotkey, freeze, overlay, the region service, the editor window, decode, the composer and the clipboard today; the store later. It never renders an annotation. Check-only code is behind the `stage0-checks` feature. |
| The composer and the clipboard | `host/src/compose.rs`, `host/src/clipboard.rs` | One function makes every output: the source byte for byte at the margin offset, the page's layer over it. The clipboard publishes that output in three formats and never reads it in the product. The references the output is checked against live in `host/references/s06/`, with the environment they are valid for. |
| The pixel crate | `host/pixels/*` | Crop, resample and the stitcher of a scrolling capture, non-generic on purpose so the work is compiled optimised even in a debug build. Knows nothing about screens, windows or files. |
| The scrolling capture | `host/src/scrolling/*`, `host/pixels/src/stitch.rs`, the round button in `host/src/overlay.rs` | A second way a capture gets its pixels: live copies of one area while the person scrolls it, joined into one frame that arrives in the editor as any capture does. Its own windows are excluded from screen copies by Windows. It writes no file. |
| The one conversion | `host/src/capture/coords.rs` | The only place allowed to subtract a frame origin. Part 5 names the four coordinate spaces; this file is the edge between two of them. |
| The image source | `host/src/source/*` | Path in, decoded frame out, for all nine formats. Only ever reads a file (rule 11); the decode report hashes every file before and after to prove it. Orientation and the colour profile are applied here, once. |
| The editor | `editor/*` | The scene, the callout and the text layer. Laid out in image pixels; one CSS transform does the zoom. Embedded into the binary at build time, so an edit here needs a rebuild. |
