# Frames: a three-frame window manager for one ultrawide screen

Plan, version 5, 2026-09-19. "Frames" is a working name. Versions 4 and 5 fold in three
external reviews; what was taken and what was refused is in "Review log" at the end.

## To the reviewer

You are being asked for a critical second opinion on this plan before any code exists.

**Who is asking.** Rotem, a product and UX designer. He writes no code himself; the tool
will be built entirely through AI coding assistants, in short sessions, one step at a time.
He already runs one Rust + Win32 project this way, with `cargo fmt`, `clippy -D warnings`
and tests as the gate on every commit.

**What is fixed and what is open.** The nine numbered behaviours in "What it does" are the
owner's and are decided; do not redesign them. The block right under them, "Proposed by
the plan", is not his and is open. The items in "Decisions waiting on the owner" are
genuinely open: give the technical consequence of each option if you see one, but the
choice is his. Everything under "Technology", "Technical design", "Prior art", "Steps" and
"Risks" is open to attack.

**What I want back, in this order:**

1. Anything in the technical design that is wrong or will not work on Windows 11, with
   the API or source that says so.
2. Anything important that is missing: a failure mode, an API, a class of app.
3. Your answer to each numbered question in "Questions for the reviewer", at the end.

Be concrete and cite. Say when you are reasoning from memory and when you have a source.
Where you have neither a source nor a first-hand result, say "unknown, needs a prototype"
rather than estimating.

**How the claims below were checked.** The technical design was reviewed against
Microsoft Learn and the source code of three tools: PowerToys FancyZones (C++), komorebi
and GlazeWM (both Rust tiling window managers). These are "the three reference tools"
throughout. Each finding was re-checked by a separate skeptic, and so was every claim the
external reviewers made from memory. Claims are marked:

- **[source]** read in Microsoft's documentation or in the named project's source code.
- **[inference]** engineering reasoning, not confirmed by a source.
- **[untested]** needs a prototype on the real machine to settle.

Today is 2026-09-19. Some cited issues, releases and pull requests are from 2026 and may
be newer than your training data; treat them as given unless you have a source that
contradicts them.

## The setup

- One 49 inch monitor, 5120 px wide, Windows 11 Home (build 26200). No second monitor.
- Rotem works on it as if it were three monitors side by side, all the time, and changes
  the three widths as the work changes.
- Assumed typical apps (not yet confirmed by the owner): Chrome, Figma desktop, VS Code,
  Explorer, a terminal, Photoshop, chat apps.

## Words used in this document

- **Frame**: one of the three fixed side-by-side regions of the screen.
- **Stack**: the windows living in one frame, overlapping at the frame's size, all still
  ordinary windows with their own taskbar buttons.
- **Covered**: a stack member with another member on top of it. It is not hidden from
  Windows, only not visible to the eye.
- **Seam**: the vertical boundary between two frames.
- **Divider**: the seam as the user experiences it, something he can drag.
- **Strip**: the tool's own thin window sitting on a seam to catch the mouse.
- **Widths**: the three frame widths (what a preset stores).
- **Managed window**: a window currently in a stack.
- **Floating window**: a window the tool knows and deliberately leaves alone, because the
  user took it out of a frame. Known, not managed.
- **Suspended**: a managed window the tool has stopped touching for now (it went
  fullscreen, got hidden, or lives on another virtual desktop). It keeps its frame.
- **Refresh**: one display refresh. "Frame" never means this here.

The conceptual reference is i3 or sway with three columns, each a "tabbed" container:
dragging a container border resizes the container, so every member gets the new size
**[source: i3 user guide]**. It is the vocabulary, not evidence: i3 only draws the
focused member, and X11 is not Windows.

## What the owner tried, and where each stopped

| Tool | What works | What blocks |
|---|---|---|
| PowerToys FancyZones | Shift-drag into a zone is instant ("drag, press Shift, release"); several windows can share a zone; reopens an app in its last zone | Zones are not connected and there is no divider to drag. Open windows follow a layout change only when a layout is switched or edited in its editor, never live |
| Windows 11 Snap (Win+Z) | The edge between two snapped windows is connected and resizes both live | Three steps per window (Win+Z, layout, spot), which he finds tedious. Only the two windows on top resize; everything stacked underneath keeps its old width, so the arrangement breaks the moment a divider moves. New apps open at their own size |

Considered and not tried by the owner: an AutoHotkey script that fakes Win+Z, 7, n on one
key (proposed to him, never run), and the tiling window managers, judged from their
source and documentation in "Prior art".

The tool is the FancyZones drop, plus the Snap divider applied to whole stacks, plus
auto-placement that neither has.

## What it does (decided by the owner)

1. **Three frames** side by side across the monitor's work area (taskbar excluded), at any
   widths, for example 1280 / 2560 / 1280.
2. **Connected dividers.** Dragging the line between two frames changes both frames'
   widths live, and every window in both stacks follows, the ones underneath included;
   otherwise the arrangement breaks. This is the reason the tool exists. Whether the
   covered windows must move during the drag or may catch up the moment it ends is open
   (Decision 8).
3. **A frame is a stack.** Any number of windows, all at the frame's size (fixed-size and
   minimum-width windows are the exception, Decision 6), layered one on top of the other.
   A window moved in goes on top; the windows already there are not affected. Frames
   never split. He switches between the windows of a stack from the Windows taskbar.
4. **Shift-drag drop.** Hold Shift while dragging a window: three targets light up;
   release over one and the window fills that frame. The targets show only while Shift is
   held; a drag without Shift stays an ordinary Windows drag (the owner's call,
   2026-09-19, to start with; targets on every drag may come back as a setting later).
   **Leaving a frame.** Dragging a managed window WITHOUT Shift takes it out of its frame
   at once: it becomes a floating window, keeps the size the frame gave it (the owner's
   call, 2026-09-19; no restore to an earlier size), no longer follows the dividers, and
   maximizes to the whole screen. The rest of that stack does not move. It rejoins a
   stack by Shift-drag or by shortcut. The app's remembered frame is unchanged while it
   floats. After the moment it is pulled out it behaves like any window; the tool never
   forces it to stay on top. Resizing a managed window by an edge that is not a divider
   (its top or bottom, a corner, the screen's outer edges) takes it out the same way.
   (Both the owner's calls, 2026-09-19.)
5. **Send-to-frame shortcuts.** One key combination each for frame 1, 2 and 3, acting on
   the focused window.
6. **Auto-placement of new windows**, so nothing opens at its own size and nothing has to
   be re-arranged on launch. The first time an app is ever seen: the middle frame
   (predictable beats clever; "the frame under the mouse" was rejected because apps are
   launched from the centred taskbar). Every later launch: the frame that app last lived
   in. Left alone: dialogs, pop-ups, tooltips, splash screens, tool palettes, fixed-size
   windows, and anything on an ignore list.
7. **Maximize fills the frame**, not the screen.
8. **Memory.** The widths and each app's last frame survive a restart.
9. **Tray icon**; starts with Windows.

**Proposed by the plan, not yet asked of the owner:** preset widths on shortcuts; a tray
menu with Pause, Presets, the ignore list and Quit; no tabs and no switcher of the tool's
own, every window staying an ordinary taskbar window; shortcuts that nudge a divider from
the keyboard; a force-manage list beside the ignore list.

**Added by the owner after review, 2026-09-19:** a shortcut that undoes the last
auto-placement, putting the window back exactly where it was, and ignores that app from
then on. And at startup the tool adopts only windows that already sit in a frame; it
never pulls the other open windows in, because he re-arranges his workspace every day.

**Out of scope until the owner asks:** more than one monitor, more or fewer than three
frames, frames that split, virtual desktops as a feature, tabs, any window decoration.
Windows still sees one screen, so a true fullscreen video or game covers all 5120 px; the
plan proposes accepting this, owner to confirm (Decision 10).

## Technology

### The stack proposed

| Piece | Choice | Why |
|---|---|---|
| Language | Rust, stable toolchain | One static exe, no runtime to install; size, idle memory and start time are expected to be small but are **[unmeasured]**. The deciding fact is that the owner already runs this toolchain and its checks on another project. All three reviewers asked said to stay; with no UI to build, C#'s one advantage is gone |
| Win32 bindings | `windows` crate (microsoft/windows-rs), not `windows-sys`, version pinned in `Cargo.toml` | Microsoft-maintained, typed handles, `Result`-returning calls **[source: windows-rs README]**; app identity needs COM, which the typed crate covers. Pinned because call signatures have changed between releases **[inference]**. Every build session takes its function signatures from the pinned version's documentation, never from the assistant's memory; assistants have seen far less `windows` crate code than C# Win32 code, and the compiler plus `clippy` catch what slips through |
| UI framework | None | The only drawn surfaces are three flat drop targets and a thin divider strip |
| Drawing | Layered windows painted with `UpdateLayeredWindow` from a premultiplied 32-bit DIB. Direct2D only if animated fades or text are ever wanted | Zero dependencies, per-pixel alpha, no render thread. FancyZones uses Direct2D on a non-layered window with a render thread **[source: ZonesOverlay.cpp]**; more than three rectangles need **[inference]** |
| Tray icon and menu | `tray-icon` crate (tauri-apps, MIT/Apache-2.0), or raw `Shell_NotifyIcon` | Maintained, needs only a message loop **[source: repo README]** |
| Global shortcuts | `RegisterHotKey` directly, `MOD_NOREPEAT` | Three calls; a crate adds nothing |
| Settings | `serde` + `serde_json`, one versioned file under `%APPDATA%\Frames\`, written to a temp file then renamed, saved at drag END. On load the widths are validated; a corrupt file is kept as a backup and defaults are used | Small, human-readable, hand-editable |
| Settings UI | None in the tray process; a tray menu and the settings file | The widths are set by dragging and the ignore list by a command on the focused window. An embedded WebView2 in an always-running process is ruled out for its memory cost **[inference, unmeasured]** |
| Logging | `tracing` + rolling file log, from the first step | "Window X did not land in its frame" can only be diagnosed from a log of class name, styles, events and the rule that decided. komorebi records which rule matched for the same reason **[source: komorebi window.rs]** |
| Single instance | One `CreateMutexW` call | No crate needed |
| Start with Windows | `HKCU\...\Run` value for the normal build | An elevated mode, if ever wanted, needs a Task Scheduler task with highest privileges **[inference]** |
| DPI | Per-monitor DPI awareness v2, declared in the manifest | `GetWindowRect` is DPI-virtualized and the DWM extended frame bounds are not, so only a PMv2 process gets both in the same physical pixels **[source: Learn, GetWindowRect]** |
| Checks | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, `cargo test` on every commit | Same gate as the owner's other project. They cover geometry, state transitions and cancellation of deferred work. They do not cover how a drag feels or how other apps behave; those are checked by hand in the real apps, step by step |

### Stacks considered and rejected

| Stack | Verdict |
|---|---|
| C# / .NET with CsWin32 | The strongest alternative: AI assistants write it well, and CsWin32 now documents NativeAOT support **[source: CsWin32 docs]**. Its advantage is an easy tray and settings UI, and this tool has almost none; WinForms is not officially supported under NativeAOT **[reviewer, memory]**, so C# with AOT would be raw Win32 anyway |
| C++ Win32 | What FancyZones itself is. Smallest and best documented, but the least safe language to have an assistant write unattended for a process that runs all day **[inference]** |
| AutoHotkey v2 | Fine for a one-day spike of the divider feel; poor at layered overlay windows; compiled scripts are the category antivirus flags most **[inference, widely reported]**; not a base for an open-source release |
| Fork PowerToys FancyZones | MIT, so legally clean, but it runs inside the PowerToys runner process **[source: FancyZones docs]** and cannot be lifted out as a small exe. Read it as reference, do not fork |
| Fork komorebi or GlazeWM | Ruled out by licence alone (below); what each has and lacks is in "Prior art" |

**Licence rule for whoever writes the code:** PowerToys (MIT) may be read and borrowed from
with attribution. komorebi (Komorebi License 2.0.0, based on PolyForm Strict: no
redistribution, personal non-commercial use only **[source: LICENSE.md, README]**), GlazeWM
(GPL-3.0) and FancyWM (PolyForm Perimeter 1.0.1, a non-compete licence, not open source
**[source: LICENSE]**) are read for technique only. No code is copied from them, because
the tool is meant for an open-source release later.

## Technical design

### 0. Threads and state

- **Three kinds of thread.** The hook thread installs the WinEvent hooks and does nothing
  but pump messages, filter and enqueue. The UI thread owns the strips, the overlay, the
  tray and the shortcuts. Anything that can block on another app (a synchronous call to a
  foreign window, a COM property read) runs on a worker, so a stuck app can stall neither
  the event stream nor Quit. COM calls such as `SHGetPropertyStoreForWindow` need
  `CoInitializeEx` on the thread that makes them.
- **One record per window**, keyed by HWND plus the process id, the thread id and a
  generation counter. `IsWindow` is not a guard against reuse: a window can close right
  after the check and its handle can come back as a different window **[source: Learn,
  IsWindow]**. Every piece of deferred work (a settle timer, a pending identity, a queued
  resize) carries the generation it was made for and is dropped on `EVENT_OBJECT_DESTROY`
  or on a mismatch.
- **Window states:** never seen; rejected (with the rule, permanent or passing); managed;
  suspended (fullscreen, hidden, other desktop); floating; pending (seen mid-drag, or
  identity not yet known). HIDE and CLOAKED are not DESTROY: apps like Slack, Discord and
  Spotify "close to tray" by hiding. A hidden or cloaked managed window stays a member,
  suspended, and gets its frame rectangle re-applied if stale when it shows again.
- **Per window the tool keeps four things apart:** the rectangle it last asked for, the
  rectangle last observed, whether a request is in flight, and the sequence number of the
  user action that caused it. A newer action (a new drag, a preset, Escape, the window
  leaving its frame) invalidates older local work. A request already handed to Windows
  cannot be recalled, so the newest intent is always re-asserted last.
- **Pause** stops everything: new input, settle timers, deferred corrections.
- **What changes an app's remembered frame:** a Shift-drop, a send-to-frame shortcut, and
  an auto-placement. Nothing else: not the startup sweep, not the tool's own echoes, not
  several windows of one app opening at once in an order nobody chose.

### 1. Moving and sizing other apps' windows

- **The rule, by the window's current state** **[source: Learn ShowWindow and
  WINDOWPLACEMENT; FancyZones and GlazeWM for the placement path]**:
  - *Normal:* `SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER |
    SWP_ASYNCWINDOWPOS`, screen coordinates. Never `SetWindowPlacement`: `SW_RESTORE`
    "activates and displays", which would steal focus and scramble Z-order for every
    window that is not already in front (the startup sweep, a window that opened in the
    background, a covered stack member).
  - *Maximized or snapped, and it must become a normal managed window:* one
    `SetWindowPlacement` with `showCmd = SW_SHOWNOACTIVATE`, `WPF_RESTORETOMAXIMIZED`
    cleared, `WPF_ASYNCWINDOWPLACEMENT` set, the frame rectangle in `rcNormalPosition`,
    which is in workspace coordinates, not screen coordinates; then the `SetWindowPos`
    above to land the exact rectangle. FancyZones places twice to let DPI scaling settle
    (its issue #365) **[source: WindowUtils.cpp]**.
  - *Minimized* (the plan's default, Decision 9): never restored. Only `rcNormalPosition`
    is rewritten, with `showCmd` kept at `SW_SHOWMINNOACTIVE`, so the taskbar click brings
    it back already at the frame's size **[source: FancyZones comment, "Do not restore
    minimized windows"]**.
  - `SW_RESTORE` is used only on the one window the user is acting on, which is already
    in front. Whether it really activates in these paths is an S0 probe.
- **Flags on the drag path:** the four above and nothing else. The first draft copied
  `SWP_NOSENDCHANGING | SWP_NOCOPYBITS | SWP_FRAMECHANGED` from komorebi and GlazeWM.
  `SWP_NOSENDCHANGING` suppresses `WM_WINDOWPOSCHANGING` **[source: Learn]**, which is
  where a window's own minimum size is enforced **[reviewers; needs the S0 probe]**, so
  it could squeeze apps below their minimum and would blind the learned-minimum logic
  below. `SWP_FRAMECHANGED` is redundant when the size changes anyway and only matters
  after a style change, which Frames never makes. `SWP_NOCOPYBITS` discards the client
  area so every step is a full repaint **[source: Learn]**; tested in S2, off by default.
  `SWP_NOOWNERZORDER` is there for safety, not because a source requires it.
- **No `DeferWindowPos`**, for the right reasons. It is usable on top-level windows, which
  share the desktop as parent, and `EndDeferWindowPos` updates the positions "in a single
  screen-refreshing cycle" **[source: Learn]**. It is rejected because it has no async
  flag and sends its messages to every target, so one slow or hung app stalls the whole
  batch and the drag with it; and because each app still repaints on its own thread, so
  the batch does not synchronize what the eye sees **[inference]**. No API is known that
  presents two apps' new content in the same refresh **[three reviewers, no source]**.
- **Never `MoveWindow`** on a foreign window; komorebi's source notes it cannot be made
  async and "might hang" **[source]**.
- **Hung targets.** A synchronous `SetWindowPos` blocks until the target thread pumps
  messages; one tiling manager was frozen completely this way (mosaico issue #31)
  **[source]**. The windows that froze it were hidden helper windows that the section 3
  filter should never manage, so the residual risk is a genuinely hung real app. Every
  cross-process call is async. `IsHungAppWindow` is only a cheap pre-filter: it reports
  that a window "has not called PeekMessage within the internal timeout period of 5
  seconds", a criterion "subject to change" **[source: Learn]**, so it cannot predict a
  hang that starts after the check. Never `SendMessage` to a foreign window (only
  `SendMessageTimeout` with `SMTO_ABORTIFHUNG`). The tool never attaches its input queue
  to another app's, and never makes one of its windows owned by a foreign window: either
  ties its responsiveness to that app **[source: The Old New Thing, 2013-06-07]**.
- **Async means acknowledge.** An async call only posts the request **[source: Learn]**
  and returns no completion. The acknowledgement is the `EVENT_OBJECT_LOCATIONCHANGE`
  echo for that HWND, OR a timeout of 50 to 100 ms, whichever comes first: if the app
  clamped at its minimum or the rectangle did not change, no event arrives at all. The
  echo is therefore NOT muted during a divider drag (the first draft contradicted itself
  here); it is recognized as the tool's own by comparing it with the rectangle asked for,
  and never read as the user moving the window. One request in flight per window, so a
  slow app cannot queue a backlog that rubber-bands after mouse-up. A timeout never turns
  into unbounded re-sending to a stuck window.
- **The invisible border.** The correction is the difference between `GetWindowRect` and
  `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`, measured per window; komorebi and
  GlazeWM fall back to the plain window rectangle when the DWM call fails **[source: all
  three code bases]**. Our additions: measure only while the window is visible, not
  minimized and not cloaked; cache per HWND and re-measure after a maximize, a style
  change or a DPI change; expect custom-chrome apps to report a different or zero border
  **[inference]**. Good, not pixel-perfect: PowerToys issue #2151, a 1 px gap between
  adjacent windows, is still open **[source]**.
- **Fixed-size and minimum-size windows.** GlazeWM and FancyZones treat a window without
  `WS_THICKFRAME` as not resizable **[source]**; a heuristic, overridden by the
  force-manage list. FancyZones moves such windows into the zone at their own size
  **[source]**. Plan: auto-placement leaves them alone; a manual drop moves them to the
  frame's corner unsized. Minimums are learned, carefully: a rectangle wider than asked
  proves nothing until the request was acknowledged and the window has settled; only then
  is a minimum recorded, and it is dropped when the window's style, DPI or state changes.
  What a frame does with such a window is Decision 6.
- **Frame geometry.** Each frame has a minimum width; the three widths plus any gaps
  always sum to the work area; a divider clamps at its neighbours' minimums.
- **Access denied.** A non-elevated tool's `SetWindowPos` on an elevated window fails.
  The return value is checked and that window is marked unmanaged, never retried forever.

### 2. Watching windows

- **Out-of-context `SetWinEventHook`**, no DLL injection. Events arrive in order
  **[source: Learn]**. The callback only filters and enqueues, as FancyZones (posted
  message) and GlazeWM (channel) do **[source]**.
- **Several narrow hook ranges, not one wide one**, with `WINEVENT_SKIPOWNPROCESS`:
  - `EVENT_OBJECT_DESTROY..EVENT_OBJECT_HIDE` (0x8001..0x8003), which leaves out
    `EVENT_OBJECT_CREATE`, as GlazeWM's range does **[source]**; our reason, that styles
    and size are not final at create, is **[inference]**.
  - `EVENT_OBJECT_LOCATIONCHANGE..EVENT_OBJECT_NAMECHANGE` (0x800B..0x800C).
  - `EVENT_OBJECT_CLOAKED..EVENT_OBJECT_UNCLOAKED`.
  - `EVENT_SYSTEM_FOREGROUND`, `EVENT_SYSTEM_MOVESIZESTART..END`,
    `EVENT_SYSTEM_MINIMIZESTART..END`.
  A single `EVENT_MIN..EVENT_MAX` range cost PowerToys 3 to 5 % CPU from mouse movement
  alone (issue #1264) **[source]**. Frames needs `LOCATIONCHANGE` always on, so it
  follows GlazeWM's always-on narrow hook. Cursor events still reach the process even
  when the callback returns at once, so idle CPU with the hook on is an S0 probe.
- **First lines of the callback:** return unless `idObject == OBJID_WINDOW`,
  `idChild == 0`, `hwnd` non-null **[source: GlazeWM]**; for `LOCATIONCHANGE` return
  unless the HWND has a record or is the foreground window (one hash lookup).
- **"Evaluate this window"** fires on SHOW, UNCLOAKED, FOREGROUND, and NAMECHANGE, for a
  never-seen, non-minimized HWND. FancyZones treats UNCLOAKED, SHOW and CREATE alike
  **[source: FancyZones.cpp]**. Some apps, Firefox by name, send neither create nor show
  at launch, and komorebi guards its NAMECHANGE path against minimized windows because
  background tabs change titles **[source: komorebi comments]**. FancyZones issue #27859
  (auto-placement re-firing on a virtual-desktop switch) exists **[source]**; that
  UNCLOAKED on an already known HWND is the cause is **[inference]**.
- **Auto-placement runs once per HWND, ever.** A floating window, a window rejected for a
  permanent reason and a window already placed are all "seen". Only a window rejected for
  a passing reason (not yet visible, zero-sized, minimized) is looked at again on its
  next SHOW or size change, because apps often create a window hidden or at 0 × 0 and
  only then size and show it.
- **Never place a window born mid-drag.** Tearing a tab out of Chrome creates a new
  top-level window under the cursor; placing it would yank it away. At first sighting the
  window is marked pending, not placed, if ANY of these holds: the primary mouse button
  is physically down (`GetAsyncKeyState`, honouring `SM_SWAPBUTTON`); its GUI thread
  reports `GUI_INMOVESIZE` in `GetGUIThreadInfo` **[source: Learn]**; a `MOVESIZESTART`
  without its END is open for that thread. The button test is mandatory because it covers
  the gap before the move loop starts. Pending resolves at `MOVESIZEEND` as a user drop
  under behaviour 4 (a Shift-drop if Shift was held, floating otherwise), on a timer once
  the button is up, or by `EVENT_OBJECT_DESTROY` when the tab is dropped back.
- **Windows on another virtual desktop** are cloaked. Auto-placement never touches a
  window that is not on the current desktop (`IsWindowOnCurrentVirtualDesktop` as a
  filter only). A cloaked member is suspended. Whether a cloaked window can simply be
  resized in place is an S0 probe **[untested]**.
- **Startup sweep.** WinEvents report only what happens after the hook exists (FancyZones
  issue #14653 is this gap) **[source]**. At launch `EnumWindows` runs through the same
  filter. The owner's call: a window already sitting in a frame's rectangle, within a few
  pixels, is adopted into that stack without being moved, activated or re-ordered; every
  other open window is recorded as floating. So a restart never undoes
  what the owner deliberately pulled out, and never reshuffles the desk. HWNDs are never
  stored across restarts.
- **Display changes** are part of S1, not polish: on a 49 inch DisplayPort screen that
  sleeps, Windows sees the monitor disconnect, throws every window onto a small virtual
  display and floods `LOCATIONCHANGE` **[reviewer, memory]**, which the tool would read
  as the user moving windows. On `WM_DISPLAYCHANGE`, `WM_SETTINGCHANGE` for
  `SPI_SETWORKAREA`, sleep/resume (`WM_POWERBROADCAST`) and session lock/unlock
  (`WTSRegisterSessionNotification`), the tool goes quiet, waits for the display to
  settle, recomputes the frames from the new work area, and re-applies the stacks.
- **Possible cross-check, not used in version 1:** `RegisterShellHookWindow`, which
  Microsoft marks "not intended for general use" **[source: Learn]**.

### 3. Which windows are real

The mental model is "would this window get a taskbar button", because the taskbar is the
switcher (Raymond Chen's Alt+Tab rule; he warns it is an implementation detail)
**[source]**. Four questions are kept apart: may this window be managed at all; is it
managed now; is it suspended for now; did the user take it out. The ordered list answers
only the first, first decisive rule wins:

1. Top-level: `GetAncestor(hwnd, GA_ROOT) == hwnd`, no `WS_CHILD`.
2. `IsWindowVisible`, and not cloaked (`DWMWA_CLOAKED`); a cloaked window keeps
   `WS_VISIBLE` **[source: The Old New Thing, GlazeWM]**. `IsWindowVisible` says nothing
   about being covered **[source: Learn]**.
3. Not minimized at first sighting (Decision 9 covers apps that open minimized).
4. No `WS_EX_TOOLWINDOW`, no `WS_EX_NOACTIVATE` **[source: GlazeWM, FancyZones]**. The
   owner's own utilities must give their overlays `WS_EX_TOOLWINDOW` or sit on the ignore
   list, or Frames will try to manage them.
5. No owner (`GW_OWNER` is null) unless it has `WS_EX_APPWINDOW`, tested whether or not
   the owner is visible; FancyZones tests only a visible owner **[source]**, the likely
   hole behind its open child-window bugs (#30071, #42366) **[inference]**.
6. `WS_POPUP` only if it also has `WS_THICKFRAME` and a caption or a min/max box
   (FancyZones' rule) **[source]**. Expected to be the main source of false negatives,
   on custom-chrome apps; that is what force-manage is for.
7. Not the tool's own process; and, for auto-placement only, at least 100 × 100 px on the
   extended frame bounds, scaled by the window's DPI **[inference]**.
8. Not fullscreen, tested in this order: if `IsZoomed` (or `showCmd` is
   `SW_SHOWMAXIMIZED`) the window is MAXIMIZED, never fullscreen, and takes the maximize
   path of section 8; this matters with an auto-hide taskbar, where `rcWork` almost
   equals `rcMonitor`. Otherwise, extended frame bounds covering `rcMonitor` AND neither
   `WS_CAPTION` nor `WS_THICKFRAME` means fullscreen (Chromium's own F11 removes both)
   **[inference from Chromium's behaviour]**. A window that covers the monitor but keeps
   `WS_THICKFRAME` is not fullscreen by default; a per-app override exists. Precedent:
   GlazeWM keeps a window out of tiling when its frame exceeds the workspace bounds
   **[source: manage_window.rs]**.
9. Built-in class and process exclusions borrowed from FancyZones (`MsoSplash`,
   `Windows.UI.Core.CoreWindow`, system windows) **[source]**.
10. Auto-placement only: skip windows without `WS_THICKFRAME`, windows of class
    `#32770`, and windows with `WS_EX_TOPMOST`. The last keeps Chrome's
    picture-in-picture, the small call windows of Meet, Zoom and Teams, and an
    always-on-top Task Manager from being stretched across a frame **[reviewer; the
    windows' real styles are an S4 check]**. A managed window that BECOMES topmost is
    released to floating. All of these still accept a manual Shift-drop or shortcut.

Two user lists sit over the rules. The ignore list rejects regardless of rules 3 to 10.
The force-manage list accepts regardless of rules 4, 5, 6, 9 and 10 only: it overrides
compatibility heuristics, never the structural rules (1, 2, 7) and never a passing state
such as fullscreen (8). Every evaluation writes one log line: HWND, process, class,
styles, the rule that decided.

**Small helper windows that pass everything** (Calculator, an Outlook reminder, a
password manager's quick window) will land full-size in the middle frame at first. The
answer is not a cleverer filter but a fast way out: the shortcut "undo the last
auto-placement and ignore this app", which the owner approved. The tool knows the rectangle the
window had before it was placed, so the undo is exact.

**Settling.** None of the three reference tools waits for a new window's geometry to
settle by default; komorebi has only an opt-in per-app sleep (20 ms, default list:
firefox.exe) **[source]**. FancyZones' open issue #50633 reports Visual Studio landing a
few pixels off and second Explorer and Terminal windows not landing at all **[source]**;
that the missing settling step causes it is **[inference]**. Plan: place on first
eligible sighting, re-read the rectangle shortly after and on that window's next
`LOCATIONCHANGE`, re-apply at most twice, then stop so the tool never fights an app. The
delay is measured per app in S0 **[untested]**. The re-check is a timer per window, never
a sleep: an app that restores several windows at once must not stall the queue.

### 4. Which app is this (the last-frame memory)

Key order, first that yields a value **[source for each API: Learn; the order itself and
the Squirrel layout were re-checked after review]**:

1. `GetApplicationUserModelId` on the window's process (opened with
   `PROCESS_QUERY_LIMITED_INFORMATION`): the identity of a packaged app, stable across
   updates. The `WindowsApps` path is never used; it carries the version.
2. The normalised exe path: lower-cased, with a path segment matching `app-<version>`
   replaced by `app-*`. Squirrel-installed apps (Figma desktop, Slack, Discord and
   others) live at `%LOCALAPPDATA%\<App>\app-<version>\<App>.exe`, so a raw path makes
   every update look like a never-seen app that goes back to the middle frame. The rest
   of the path is kept, so two different apps with the same exe name stay apart. Which of
   the owner's apps really sit in a versioned folder is a one-line S0 check.
3. The bare exe name, only when the path cannot be read (an elevated process).

The window-level `System.AppUserModel.ID` (`SHGetPropertyStoreForWindow`) is a possible
refinement, not a key, until S0 shows what it returns for Chrome profiles, PWAs and the
Electron apps: a window-level ID overrides the process-level one, a system-assigned ID
cannot be read by another app **[source: Learn, Application User Model IDs]**, and most
Win32 apps set none **[inference]**.

UWP apps hosted by `ApplicationFrameHost.exe` (not "Store apps": the Store also ships
ordinary Win32 apps **[source: Learn]**): the real app is the child window owned by a
different process, which does not exist yet at first sighting; FancyZones polls for it
with a sleep, up to 30 × 5 ms **[source: process_path.h]**. Frames never sleeps and never
lets identity block placement: one immediate look; if absent, place now under a
provisional identity, mark "identity pending", look again when a later event arrives for
that window or for a window whose root it is, backed by a coarse timer (about 50, 150,
500, 2000 ms), then give up. `ApplicationFrameHost.exe` is never stored as an identity.

Proposed, owner to confirm (Decision 10): all plain Explorer windows share one remembered
frame, and an app that restores several windows at once sends them all to its last frame
(FancyZones issue #19122 is the same behaviour **[source]**). Per-app memory cannot
remember a different frame for each of several windows of one app.

### 5. The divider

No Windows tool was found with stacks plus a divider that resizes them live. komorebi is
closest: stacks, and a mouse resize honoured at drag END **[source: process_event.rs]**,
which is also the warning, since its author chose commit-on-release. FancyZones has no
dividers. The only shipping live connected resize found is Windows 11 Snap's own, which
resizes only the visible snapped pair, and AquaSnap's Ctrl+drag of adjacent windows
**[source for each]**; whether AquaSnap's resize reaches windows covered in the same
position is not stated anywhere (its 2015 release notes mention MOVING stacked windows
together) **[untested]**.

- **Which windows resize live. The first draft was wrong here.** It resized only the
  windows visible when the drag began and brought the covered ones to size at release.
  But when a frame SHRINKS, its covered members keep their old width and stick out past
  the new seam into the neighbour's new territory, and they stay out of sight only if the
  neighbour's top window is above them in the global Z-order. It often is not: switch
  between two apps in the right frame and both now sit above the middle frame's window,
  so dragging that divider to the right shows a strip of the covered app between the old
  line and the new one. This is ordinary use, not a corner case **[inference, found
  independently by two reviewers]**. The plan's default (Decision 8) is therefore:
  - *Live:* the top window of each of the two frames; any lower member that already
    shows; and, on the side that is shrinking, every member that is above the
    neighbour's top window in the Z-order, recomputed whenever the drag changes
    direction.
  - *The moment the drag ends:* every other member of both stacks, staggered and async,
    and minimized members through `rcNormalPosition`.
  - *An alternative to measure:* at drag start, pre-shrink every covered member of both
    stacks to the frame's minimum width, anchored to the edge away from the dragged seam,
    so nothing can ever stick out; then size them all at release. Two resizes per covered
    window per drag, none of them live **[inference, untested]**.
  - Updates are coalesced to the latest divider position, at most one per refresh.
  - A window brought forward within about half a second of release may be caught
    mid-resize. Chromium stops rendering a fully covered window, so resizing one is cheap
    but it repaints on reveal, possibly with a flash at the old size; VS Code opts out of
    that behaviour **[source: Chromium docs, windows_native_window_occlusion_tracking]**.
    The tool's layered strip does not count as an occluder.
- **Full-stack live resize** is built behind a flag in the same step and measured.
- **The strip is the main way to drag a divider.** The tool moves both sides itself, so
  the two edges stay in step with each other. Grabbing a window's own inner edge instead
  lets Windows resize that window at the cursor while the neighbour trails by its own
  layout and paint time, tens of milliseconds in Chromium, so a gap opens when the
  grabbed window shrinks **[inference, three reviewers]**. The strip is therefore 12 to
  16 px wide, covering both neighbours' invisible resize borders, which makes the edge
  path almost unreachable. Expect the window to the RIGHT of any seam to look worse than
  the one to its left: a resize from the left edge is a move plus a resize, the weak case
  in Windows **[reviewer, memory]**.
  - *Recipe:* `WS_POPUP`, `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED`, alpha 1
    rather than 0 so it still hit-tests **[source: Learn, Window Features]**, NOT
    `WS_EX_TRANSPARENT`, `MA_NOACTIVATE` on `WM_MOUSEACTIVATE`, `IDC_SIZEWE` cursor.
  - *The drag loop does not depend on mouse capture.* Learn says "only the foreground
    window can capture the mouse" and that a background window gets mouse messages only
    over its own visible part; the same page also says input goes to the capturing
    window of another thread "only if a mouse button is down", which is exactly the
    strip's case. Learn does not settle it **[source, both quotes]**. So:
    `WM_LBUTTONDOWN` calls `SetCapture` AND starts a 10 to 16 ms timer; the divider is
    driven from `GetCursorPos` on the timer; the drag ends on `WM_LBUTTONUP`, on
    `WM_CAPTURECHANGED`, or when `GetAsyncKeyState(VK_LBUTTON)` reads up. A low-level
    mouse hook scoped to the drag, as FancyZones uses **[source: MouseButtonsHook.cpp]**,
    is the fallback if polling proves unreliable over elevated windows. The strip is
    never activated "just for the drag".
  - *Escape never reaches the strip:* a window that never has focus never gets key
    messages **[source: Learn, keyboard input]**. Escape is polled with
    `GetAsyncKeyState(VK_ESCAPE)` on the same timer; it then restores the widths. Two
    accepted limits: the focused app sees that Escape too, and the poll may read nothing
    while an elevated window is in front. A drag-scoped `RegisterHotKey` on Escape would
    swallow the key for the app instead; only if the owner asks.
- **A user's resize of a managed window that is NOT on a seam** (its top or bottom edge,
  a corner, the outer edge of frame 1 or 3) is a different act from a divider drag. Which
  edge moved is read from how the rectangle changed on the first events. The owner's
  call: that window becomes floating, the same as dragging it out. A resize by an inner
  edge, where the strip was missed, is corrected on release: both stacks are brought to
  the new widths.
- **Z-order of the strip.** Two of three reviewers preferred topmost; one preferred a
  non-topmost strip re-inserted above the managed windows on every foreground change.
  Both are prototyped in S2, topmost first. Topmost strips are hidden while anything
  fullscreen is in front, judged by the section 3 rule 8 test, run on every
  `EVENT_SYSTEM_FOREGROUND` AND on every `LOCATIONCHANGE` of the foreground window,
  managed or not, because F11 and a video's fullscreen button change state inside the
  same window without any foreground change. `SHQueryUserNotificationState` and the
  appbar notification `ABN_FULLSCREENAPP` (register with `ABM_NEW`, never `ABM_SETPOS`)
  are wake-up hints at most **[source: Learn for both; reliability untested]**. An owned
  strip is ruled out (section 1, input queues).
- **A keyboard path** for the same action (shortcuts that nudge a divider, and presets),
  since a no-activate strip is unreachable by keyboard or assistive tech **[source:
  Learn, WS_EX_NOACTIVATE]**.
- **Gaps and corners.** A gap of 6 to 8 px between frames would host the strip, hide the
  1 px seam error, and make the rounded-corner wedges a non-issue, so Decision 4 would
  fall away (Decision 3). Without a gap, FancyZones' answer to the wedges is
  `DWMWA_WINDOW_CORNER_PREFERENCE = DWMWCP_DONOTROUND` on snapped windows, restored on
  unsnap **[source]**.

### 6. Shift-drag

- **Drag detection:** `EVENT_SYSTEM_MOVESIZESTART` / `END`. The event does not say move or
  resize **[source: Learn]**. At start the tool records the rectangle, the maximized or
  snapped state, and whether the cursor is a sizing cursor (FancyZones'
  `IsCursorTypeIndicatingSizeEvent`) **[source]**. The cursor is the early hint that lets
  the targets appear before the mouse has moved; the rectangle is the judge, compared
  over the first few `LOCATIONCHANGE` events, adapted from komorebi, which compares at
  drag END **[source: process_event.rs]**. When they disagree the rectangle wins. A size
  change is ignored when the drag began on a maximized or snapped window, which Windows
  itself shrinks as the move starts.
- **The drag is a small state machine,** written out before it is coded: a move that
  begins without Shift and gains it later; Shift released before mouse-up; Alt+Space
  moves; Escape, which makes Windows put the window back; the window destroyed mid-drag;
  the screen locked mid-drag; a preset fired mid-drag.
- **Leaving a frame happens at once,** as behaviour 4 says: a managed window is detached
  from its stack as soon as the act is identified as a move, not at the end of it. If
  Shift is held at release over a target it joins that stack. If the move is cancelled
  with Escape and the window returns to its old rectangle, it is re-attached. Otherwise
  it is floating: no resize, no placement, skipped by the divider and maximize paths.
- **Shift** is read with `GetAsyncKeyState(VK_SHIFT)` at drag start and on a 30 ms timer
  that runs only during a drag, so the targets appear and vanish even with the mouse
  still. The earlier draft used Raw Input with `RIDEV_INPUTSINK`: a permanent global
  keyboard sink that wakes on every keystroke, reads as keylogger-like to antivirus, and
  is just as blind in front of an elevated window **[source for the blindness:
  DraggingState.cpp comment; Learn, GetAsyncKeyState]**. No global keyboard hook.
- **The overlay:** one click-through window over the work area,
  `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, shown only
  while a Shift-drag is live, drawn ABOVE the dragged window. FancyZones instead puts its
  overlay behind the dragged window and makes that foreign window 50 % transparent by
  changing its extended style **[source: WindowMouseSnap.cpp]**; drawing above avoids
  touching another app's styles **[inference]**. It stays ONE window, as FancyZones'
  overlay is one window per work area **[source: WorkArea.cpp]**. `UpdateLayeredWindow`
  always uploads the whole surface **[source: Learn]**, about 27 MB at this size, so it
  is called only when the highlighted target changes, never on mouse-move; the DIB is
  allocated once; the window exists only during a Shift-drag. S3 times the call and
  accepts under one refresh. Fallback: a GDI-painted popup with
  `SetLayeredWindowAttributes(LWA_ALPHA)`; the two layered-window methods cannot be mixed
  on one window **[source: Learn, Window Features]**.
- **Drag state is always cleared** on `MOVESIZEEND` and on destroy of the dragged window
  (FancyZones documents a stuck-drag bug class here) **[source]**.
- **Apps that never raise `MOVESIZESTART`** cannot be Shift-dragged at all. Microsoft's
  compatibility list gives this exact reason for VMware Workstation Player and
  Application Guard **[source: FancyZones docs]**; PowerToys #1289 and #21727 track
  custom-chrome and title-bar-less windows **[source]**. Separately, the foreground app
  can mask Shift (microsoft/terminal #10880) **[source]**. The send-to-frame shortcuts
  are the fallback for all of these.

### 7. Shortcuts

`RegisterHotKey` with `MOD_NOREPEAT`. Win-key combinations are documented as reserved for
the system, and registration fails when a combination is taken **[source: Learn]**, so the
defaults avoid the Win key, every shortcut is configurable, and a failed registration is
shown in the tray. A shortcut that registers can still collide with an app's own local
shortcut or with a keyboard layout; that is checked by hand in the owner's apps. No
low-level keyboard hook.

### 8. Maximize fills the frame, and fullscreen

No supported, injection-free way to stop a maximize before it paints is known
(`WM_GETMINMAXINFO` and `SC_MAXIMIZE` live in the target process; a global `WH_CBT` needs
a DLL) **[inference; three reviewers agree]**. So it is corrected after the fact: on
`LOCATIONCHANGE` of a managed window, `IsZoomed` means one `SetWindowPlacement` with the
frame rectangle, in a single call and not restore-then-move (GlazeWM does this "to avoid
flickering") **[source]**. The same path runs when a new window first appears maximized.
A brief full-screen flash is likely; the owner has not seen or accepted it yet (Decision
10), so maximize is tried early, in S0: the button, a title-bar double-click, the
shortcut, a window that opens maximized, the way out of fullscreen. After the correction
the window is a normal window as far as Windows and the app know, so the restore button
and a drag from the title bar are checked too. Optional polish, only if a probe shows it
works on another process's window: `DWMWA_TRANSITIONS_FORCEDISABLED` to drop the maximize
animation, restored when the window is released **[untested]**. MaxTo 2 shipped
maximize-into-region **[source: MaxTo v2 docs]**; FancyZones has had it requested since
2019 (issue #279) **[source]**.

**In-app fullscreen (F11, a video's fullscreen button) suspends a managed window; it does
not remove it.** Detection is rule 8 of section 3. While suspended it keeps its frame and
its place in the stack, the tool never moves it, and it does not count as the frame's
top. Leaving fullscreen: re-tested on each `LOCATIONCHANGE`; Chromium restores the styles
first and the rectangle second, so the tool waits about 100 to 150 ms, then re-applies
the frame rectangle only if it differs, for example because a divider moved meanwhile
**[inference from Chromium's behaviour]**. The suspended flag is checked before the
maximize path, the snapped-window path and the floating path, so the way back from
fullscreen is never misread as the user pulling the window out.

### 9. Living with Windows Snap

- A window snapped by Windows is "arranged", a state that excludes maximized and
  minimized, and `GetWindowPlacement` can report it as normal. `IsWindowArranged`
  (user32, Windows 10 1903+, resolved with `GetProcAddress`, with a fallback if absent)
  detects it **[source: Learn]**. Plan's default (Decision 7): an arranged managed window
  becomes floating.
- No documented event announces the arranged state; `EVENT_SYSTEM_ARRANGMENTPREVIEW` is
  only the preview **[reviewers; source for the event list]**. So it is checked on any
  `LOCATIONCHANGE` of a managed window the tool did not cause, about 100 ms after
  `MOVESIZEEND`, and after a restore from minimized, since a window can come back
  arranged. S6 logs `IsWindowArranged` after a snap by the maximize-button flyout,
  Win+Arrow, an edge drag and Win+Z. A narrow backstop only if that shows gaps: on
  `MOVESIZEEND`, a managed window whose bounds differ from its frame by more than 2 px,
  that is not suspended and not merely clamped at its own minimum, is treated the same.
- Windows' own edge snapping can be turned off with `SystemParametersInfo`
  (`SPI_SETWINARRANGING`, or the finer `SPI_SETDOCKMOVING`) **[source: Learn]**; its
  effect on the Windows 11 snap bar is undocumented **[untested]**. It is a separate,
  deferrable add-on: one opt-in setting, the previous value saved first. Restoring it on
  the next start is late recovery after a crash, not instant, and must not overwrite a
  change the user made since. The community-documented registry values are never written.

### 10. Elevated windows

From a normal process, moving an elevated window fails, and Shift and Escape are invisible
while one is in front; WinEvents for it still arrive **[source: PowerToys administrator
docs, issue #5540]**. Version 1: detect it, leave the window alone, say so once in the
tray. Later, opt-in: run elevated at logon through a scheduled task. `uiAccess=true`
requires a signed binary installed under Program Files **[source: Learn, UAC settings]**.

### 11. Distribution

- **Personal use:** built locally; no signing needed.
- **Open-source release:** an unsigned exe that hooks window events, registers global
  shortcuts, moves other apps' windows and adds itself to startup is the profile that
  SmartScreen and Defender's heuristics flag **[inference, widely reported]**. Azure
  Artifact Signing accepts individual developers only in the US and Canada, and
  organizations in a list that includes Israel and the EU **[source: Learn quickstart]**.
  The route is decided before release, as its own step.

## Prior art, and why none of it ends the project

Judged from documentation and source, not from hands-on use, except the first two rows.

| Product | Has | Lacks |
|---|---|---|
| PowerToys FancyZones (free, MIT) | Shift-drag targets, several windows per zone, last-known-zone placement, exclude list, windows follow a layout change made in its editor | Any divider; maximize-to-zone (requested since 2019); placement of an app never seen before |
| Windows 11 Snap | Live joint resize of the two visible snapped windows, always on | Anything for windows underneath; fast placement; auto-placement; memory |
| AquaSnap 1.24 (Nurgo) | "AquaGlue": Ctrl+drag moves and resizes ADJACENT windows together, live. A 2020 help article says Professional only; the current pricing page has no Professional edition and lists "move windows together" in the personal-use freeware. v1.16.0 (2015) added MOVING stacked windows together | Persistent frames, auto-placement, maximize-to-region. No source says its RESIZE reaches windows covered in the same position. The owner found the joint resize is a paid feature and dropped it |
| MaxTo 3.1.0 (5 Aug 2026), a rewrite | A hotkey opens a grid picker to place the foreground window; per-app remembered positions with auto-placement of new windows. Home EUR 29 a year or EUR 99 lifetime, personal PCs only; Business EUR 39 a year per PC | Draggable dividers, linked resize, stacks. Maximize-into-region was a v2 feature and is not documented for v3 |
| DisplayFusion Pro monitor splitting | Windows maximize to the split, per-split taskbars, triggers on window creation | Draggable splits (they are typed pixel values); fullscreen cannot be constrained |
| komorebi | Columns layout, stack containers, an "append" mode where new windows join a stack, rules, mouse resize honoured at drag end | Live resize; mouse-first use; stacked windows that stay visible to the taskbar (it hides the non-top ones); a usable licence |
| GlazeWM | Tiling, window rules | Stacks of any kind; the accordion-layout pull request #1410 was closed unmerged in July 2026 |
| FancyWM (source-available, not open source) | Stack panels, mouse drag between panels, respects min/max sizes | Fixed frames; per-app frame memory. A UX reference only |
| Monitor vendors' splitters, Picture-by-Picture | Preset grids; PBP gives genuinely separate displays | Movable dividers, stacks, memory; PBP needs two cables and gives fixed halves |

Not assessed at all: Divvy, WindowGrid, workspacer, bug.n, Amethyst Windows, LG OnScreen
Control.

## Steps

Each step ends in something the owner can use or see, and names a suggested model for the
session that builds it: Fable 5.1 (Anthropic's most capable model, for design and anything
subtle) or Opus 5 (faster and cheaper, for mechanical rounds). You need not review the
model choice, only whether a step labelled mechanical is not.

- [x] **Before anything: look at AquaSnap.** Done by the owner, 2026-09-19: the joint
  resize needs the paid edition, so he dropped it. Whether its resize reaches covered
  windows stays unknown and no longer matters. The build goes ahead.
- [ ] **S0. Probes, each a few minutes, all logged with the Windows build, the display
  scaling and the refresh rate.** A command that places the focused window into a given
  rectangle, run on Chrome, Explorer, Figma, VS Code, a terminal, Photoshop and one UWP
  app: the border delta, the time to settle, whether a second placement call changes
  anything. Then: does `SW_RESTORE` activate, and does `SW_SHOWNOACTIVATE` place a
  maximized background window without stealing focus; does a window shrink below its
  minimum with and without `SWP_NOSENDCHANGING`; the maximize flash, by every way of
  maximizing; a window that ignores the request; what `SHGetPropertyStoreForWindow` and
  `GetApplicationUserModelId` return for two Chrome profiles, two PWAs and the Electron
  apps; which installed apps live in an `app-<version>` folder; resizing a window on
  another virtual desktop; the events of a cold start of Calculator and Settings; idle
  CPU with the `LOCATIONCHANGE` hook on; whether `DWMWA_TRANSITIONS_FORCEDISABLED` can be
  set on another process's window. *Fable 5.1: foundation, everything rests on it.*
- [ ] **S1. Frames, shortcuts, and staying sane.** Three frames from the settings file;
  shortcuts send the focused window to frame 1, 2 or 3; the window record and its states,
  floating included; a minimal guard (rules 1, 2 and own-process) so a shortcut refuses
  anything that is not a real top-level window; access-denied handling; display change,
  sleep and lock handling; a tray icon with Pause and Quit. Usable daily from here.
  *Opus 5 once S0 holds; the display-change part is the subtle one.*
- [ ] **S2. The divider, as the experiment that decides.** First the strip and its input
  alone, then neighbours, then whole stacks. Presets, the keyboard nudge and the
  window's-own-edge correction come after the strip has proved the core. It comes ahead
  of the window filter: until auto-placement arrives in S5 the only windows in a stack are
  ones the owner sent there himself. The owner tries the modes and settles Decision 8.
  *Fable 5.1: the feel is the product.* **Acceptance gate:**
  - The strip drag keeps going when the cursor leaves the strip, moves fast, and is
    released over another app, an elevated window, the taskbar; Escape and a lost capture
    end in a consistent state.
  - With several windows per stack and the two stacks INTERLEAVED in Z-order, shrinking
    and growing in both directions never shows a window at its old width over the
    neighbouring frame.
  - Bringing any covered window forward right after mouse-up: the time until its
    rectangle AND its paint match the new widths is measured.
  - A slow or hung app freezes neither the strip nor Quit, and does not replay a run of
    jumps when it recovers.
  - A new drag, a preset, a window leaving its frame and a cancel never receive a late
    correction from an earlier action.
  - Minimize and restore, F11 inside the same window, a plain maximize, an open dialog
    and a focus change break no membership and show no strip where it should not be.
  - Measured: response time, the lag between the two edges, time to converge after
    release, load, and that focus never moved. An API returning quickly, or low CPU, is
    not proof of a smooth drag.
- [ ] **S3. Shift-drag drop targets.** The drag state machine written out first, then
  move-versus-resize, the Shift timer, the click-through overlay, drop, and leaving a
  frame at once. *Opus 5 with the state machine in hand; FancyZones as reference.*
- [ ] **S4. Which windows are real.** The ordered rule list, the two lists, the decision
  log, "add focused app to ignore list", the undo-and-ignore shortcut. Tested against
  Photoshop panels, Figma's floating windows, Save As dialogs, installers, UWP apps,
  Chrome picture-in-picture, a Meet or Zoom call window, a Chrome tab torn off.
  *Fable 5.1: judgment heavy, wrong calls are very visible.*
- [ ] **S5. Auto-placement and memory.** New windows by the first-launch and last-frame
  rules, the pending state for windows born mid-drag, app identity, verify-and-reapply,
  the startup sweep, persistence. *Fable 5.1: depends on S4 being right.*
- [ ] **S6. Maximize, fullscreen and snapped windows.** Not mechanical: fullscreen,
  arranged, the maximize correction, the floating state and the tool's own echoes all
  compete for the same events. Tested with F11 in Chrome, a YouTube video, F11 in VS
  Code, a divider moved while a window is fullscreen, an auto-hide taskbar, and the
  `IsWindowArranged` manual test. *Fable 5.1.*
- [ ] **S7. Comfort.** Start with Windows, presets in the tray, settings file and reload,
  elevated windows reported once. The opt-in edge-snap switch is a separate add-on that
  can wait. *Opus 5: routine.*
- [ ] **S8. The long tail.** One rule per app that misbehaves, as daily use finds them.
  *Opus 5: small fixes.*
- [ ] **S9. Release.** Licence, signing route, installer or winget, false-positive
  submissions. Only when the owner decides to publish. *Fable 5.1: irreversible choices.*

**Size.** Not estimated until S2 is done. S2 alone is about a week of sessions because of
its measurements; the earlier "S0 to S3 in a week" was optimistic, and two reviewers said
so.

## Risks, most serious first

1. **The divider may not feel good.** No open-source reference exists for live resize of
   foreign windows from outside their process, nothing found does it for whole stacks,
   heavy apps repaint slowly, and the window to the right of a seam is the weak case.
   Mitigation: S2 is a measured experiment with an acceptance gate before anything else
   depends on it; the fallback is a preview line with everything resized on release.
2. **The filter will be wrong for some app every week at first.** Every reference tool
   has open bugs here. Mitigation: decision log, two override lists, a one-key undo.
3. **Apps that fight back**: restore their own position after showing, enforce a minimum
   size, never raise `MOVESIZESTART`, or report odd frame bounds. Mitigation: verify and
   re-apply at most twice, learned minimums, shortcuts as the fallback, per-app rules.
4. **State, not API calls, is where this breaks**: late acknowledgements, cancelled
   actions, reused handles, display changes, several mechanisms reading one event stream.
   Mitigation: section 0, and automated tests for geometry, transitions and cancellation.
5. **Freezing the tool on a hung app.** Mitigation: the thread model, async calls, no
   `SendMessage`, no attached input queues.
6. **Distribution** (section 11). No effect on personal use.

## Decisions waiting on the owner

1. **The name.** "Frames" is a placeholder.
2. **The shortcuts.** Alt+1/2/3 collides with Figma. Ctrl+Alt+1/2/3 and Alt+Shift+1/2/3
   are the usual alternatives; Win-key combinations are best avoided.
3. **Gaps between frames.** Flush, or 6 to 8 px between. A gap gives the strip a clear
   place to live, hides the 1 px seam, and makes Decision 4 unnecessary; flush means the
   strip competes with the apps' own resize edges.
4. **Square corners on managed windows**, only if there is no gap. It means changing a
   setting on another app's window and putting it back afterwards; best left until after
   the divider prototype.
5. ~~Shift-drag only, or targets on every drag.~~ Decided 2026-09-19: Shift-drag only to
   start with.
6. **A window wider than its frame allows:** the divider stops there, which lets even a
   covered window limit the whole frame; or that window overhangs into the neighbour,
   with consequences for Z-order, the strip and clicks.
7. **A window the user snaps with Windows' own Snap:** becomes floating (the plan's
   default), or is pulled back in, which risks fighting what he meant.
8. **The divider's feel:** everything live; the Z-aware live set of section 5 and the
   rest the moment the drag ends (the plan's default); or a preview line with everything
   resized on release. Settled by the owner on the S2 prototype.
9. **Minimized windows:** resized silently in place so they come back at the frame's size
   (the plan's default), or left alone until restored; and whether an app that opens
   minimized is auto-placed.
10. **Limits to confirm:** a true fullscreen video or game covers all three frames; a
    maximize flashes full-screen for a moment before settling into the frame (shown to
    the owner in S0); all Explorer windows share one remembered frame, and an app that
    restores several windows sends them all to its last frame.
11. **The items under "Proposed by the plan"** in "What it does". Pause and Quit come
    early regardless; presets, the nudge and force-manage can be added as needed.
12. ~~Resizing a managed window by an edge that is not a divider.~~ Decided 2026-09-19:
    it becomes floating, the same as dragging it out.
13. ~~Whether a floating window is kept above the frames.~~ Decided 2026-09-19: no. It is
    on top only at the moment it is pulled out, then behaves like any window. The tool
    never forces always-on-top onto another app's window.
14. ~~Windows already open when the tool starts.~~ Decided 2026-09-19: adopt only those
    already sitting in a frame, leave the rest floating. The owner re-arranges his
    workspace every day and does not want open windows pulled in.
15. ~~A shortcut "undo the last auto-placement and ignore this app".~~ Decided
    2026-09-19: wanted. It joins S4.
16. **A topmost window and a manual drop:** may an always-on-top window be Shift-dropped
    into a frame (the plan's default: yes, the user asked), or never.

## Questions for the reviewer

1. The Z-aware live set of section 5 against the pre-shrink alternative: which is more
   robust, and what breaks either? Is there a third way to keep covered windows from
   sticking out during a shrink?
2. The strip's drag loop on a never-foreground thread: does `SetCapture` from
   `WM_LBUTTONDOWN` keep delivering mouse messages outside the strip while the button is
   held? Is the timer plus `GetCursorPos` fallback sound, and what does it miss?
3. `SetWindowPlacement` with `SW_SHOWNOACTIVATE` to turn a maximized or snapped background
   window into a normal one without activating it: does it work, and what else would?
4. Acknowledging an async `SetWindowPos` by its `LOCATIONCHANGE` echo or a 50 to 100 ms
   timeout: what does that get wrong?
5. The window filter in section 3: which rule will cause the most false positives or
   negatives on a designer's machine, and what is missing?
6. `IsWindowArranged` on Windows 11 24H2 and later: dependable for every way of snapping?
   If you have no source or first-hand result, say "unknown, needs the S6 probe".
7. Display sleep and resume on a single DisplayPort ultrawide: what is the most reliable
   signal that the display has settled and windows may be re-applied?
8. Does any product you know already do connected dividers over whole stacks of windows?
   If so, name it and say exactly which of the nine behaviours it has and which it lacks.
9. What in the step order or the S2 acceptance gate would you change?

## Review log

Every memory-tagged or cited claim from a reviewer was re-checked against primary sources
before it was written in.

**Round 1, Gemini, 2026-09-19** (two replies; the first saw the file only up to section 7).
Taken: `IsHungAppWindow` is a snapshot, all cross-process calls async; the strip as the
main drag path; UWP identity never blocks placement and never sleeps; a concrete minimum
size for auto-placement; settling as a timer per window; fullscreen suspend and resume
written out; the arranged state has no event; signatures from the pinned crate's docs.
Refused: splitting the drop overlay into three windows (nothing saved; FancyZones uses
one); dropping the cursor test for move-versus-resize (it is the early hint; the rectangle
judges); moving the window filter ahead of the divider (before S5 only hand-sent windows
are in a stack); switching to C#. Citations that did not check out: "Microsoft Learn,
Layered Windows performance guidelines" (no such page); "FancyZones issue #20171" (an
unrelated launch crash); "Chromium source code, views::Widget" (names no file);
"Windows DWM Architecture" (not a document).

**Round 2, a second Claude session and a GPT (Codex) review, 2026-09-19,** both of
version 3, both reasoning carefully and marking memory as memory. Taken:

- The exposed-strip flaw in "visible live, covered at release", found by both
  independently; the Z-aware live set, the pre-shrink alternative, and the S2 gate that
  tests interleaved stacks (section 5).
- The acknowledgement contradiction: the echo is the ack, so it cannot be muted; plus a
  timeout, sequence numbers and cancellation (sections 0 and 1).
- Escape cannot reach a no-activate strip; mouse capture from a never-foreground thread
  is not settled by Learn, so the loop does not depend on it (section 5).
- `SW_RESTORE` activates; placement now depends on the window's state (section 1).
- Maximized is tested before fullscreen, for auto-hide taskbars (section 3, rule 8).
- The three copied `SWP_` flags come off the drag path (section 1).
- The `DeferWindowPos` rejection keeps its verdict and gets the right reasons (section 1).
- Versioned install folders break path-keyed memory; the identity order was rebuilt
  (section 4). "Store app" corrected to "UWP app hosted by ApplicationFrameHost".
- Windows born mid-drag; topmost windows; hidden-to-tray windows; display, sleep and lock
  events, moved to S1; resizes that are not divider drags; handle reuse; the thread
  model; floating as a known state; what may change an app's remembered frame; a timer
  for Shift in place of a permanent Raw Input sink (sections 0, 2, 3, 5, 6).
- Leaving a frame happens when the move is identified, not when it ends (section 6).
- Force-manage no longer overrides structural rules or passing states (section 3).
- Prior art: AquaSnap's edition and what its release notes really say; MaxTo 3 is a
  rewrite and the v2 evidence was dropped; i3's tabbed columns as the vocabulary.
- Steps: AquaSnap first; S0 as a list of probes including maximize; display events and
  Pause/Quit in S1; an acceptance gate for S2; S6 marked as subtle; no time estimate
  until S2 is done.

Left as experiments, not facts: `ABN_FULLSCREENAPP` as a fullscreen signal;
`DWMWA_TRANSITIONS_FORCEDISABLED` on a foreign window; whether `SWP_NOSENDCHANGING`
really lets apps be squeezed below their minimum.

## Main sources

- Microsoft Learn: `SetWindowPos`, `DeferWindowPos`, `EndDeferWindowPos`, `ShowWindow`,
  `WINDOWPLACEMENT`, `GetWindowRect`, `IsWindow`, `IsWindowVisible`, `IsHungAppWindow`,
  `SetCapture`, mouse and keyboard input overviews, `WM_WINDOWPOSCHANGING`,
  `GetGUIThreadInfo`, `DWMWINDOWATTRIBUTE`, `SetWinEventHook`, event constants,
  `IsWindowArranged`, `RegisterHotKey`, `GetAsyncKeyState`, `SystemParametersInfo`,
  `SHQueryUserNotificationState`, `SHAppBarMessage`, `RegisterShellHookWindow`, extended
  window styles, Window Features, Application User Model IDs,
  `GetApplicationUserModelId`, distributing a Win32 app through the Store, UAC settings,
  PowerToys FancyZones and administrator pages, Artifact Signing quickstart.
- PowerToys source, `src/modules/fancyzones/FancyZonesLib/`: `WindowUtils.cpp`,
  `FancyZones.cpp`, `FancyZonesWindowProcessing.cpp`, `DraggingState.cpp`,
  `WindowMouseSnap.cpp`, `MouseButtonsHook.cpp`, `WorkArea.cpp`, `ZonesOverlay.cpp`,
  `AppZoneHistory.cpp`, `common/utils/process_path.h`. Issues #279, #1264, #1289, #2151,
  #5540, #14653, #19122, #21727, #27859, #30071, #42366, #50633. microsoft/terminal
  issue #10880.
- komorebi source: `windows_api.rs`, `process_event.rs`, `window.rs`,
  `window_manager_event.rs`, `lib.rs`; `LICENSE.md`, README.
- GlazeWM source: `wm-platform/.../native_window.rs`, `window_listener.rs`,
  `wm/.../manage_window.rs`; pull request #1410.
- Chromium: `docs/windows_native_window_occlusion_tracking.md`. Squirrel.Windows install
  layout. CsWin32 documentation. The i3 user guide.
- Raymond Chen, The Old New Thing: which windows appear in Alt+Tab (2007-10-08); cloaked
  windows (2020-03-02); `DeferWindowPos` (2005-07-06); cross-process window
  relationships (2013-06-07); no-activate windows (2016-09-12).
- mosaico issue #31 (a tiling manager frozen by a synchronous `SetWindowPos`).
- Product pages and docs: Nurgo AquaSnap (product, pricing, AquaGlue help article,
  v1.16.0 release notes), MaxTo (releases, current settings reference), DisplayFusion
  monitor splitting, FancyWM (README, LICENSE).
