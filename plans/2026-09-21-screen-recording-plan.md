# Screen recording in Recon: the complete plan

Plan, version 2, 2026-09-21. Asked for by Rotem on 2026-09-20. Nothing of it is built, and
nothing is built until he says so.

**Where this file stands.** It is a file of its own in `plans/` because Rotem asked for one.
It is Stage 5 of `project-os/Plan.md`: §3.10 there keeps video and GIF capture out of v1 and
that stands, and when Stage 5 starts relative to the daily-use release is his word. Part 14
of `project-os/Plan.md` was this plan's first draft, written before the research below. It is
superseded as a whole, not only where the two differ, and it says so in its first line. Its
calls R1 to R7 are retired: here a decided call is a D, an open call is a C, and a step is
S5.n, as every step in the main plan is named.

## How this plan was made, and how to read its claims

Eight readers worked in parallel: six on the technical ground (getting the frames, encoding,
surviving a crash, sound, playing the file, getting it out), one on Recon's own code, one on
how seven other recorders behave. Each of the six technical readers was then re-checked by a
separate skeptic who opened the documentation and the source code again and tried to refute
the claims; about forty corrections came back. The survey of other recorders and the map of
Recon's code were not re-checked that way. The finished draft was then read by four
reviewers, against the research, against the project's rules, as the engineer who must build
from it, and as Rotem; their seventy-nine findings are folded into this version.

Every technical claim carries one of three marks:

- **[source]** read in Microsoft's documentation or in the named project's source code.
- **[inference]** engineering reasoning, not confirmed by a source.
- **[untested]** needs a prototype on this machine to settle. The gates in part 6 exist for these.

The machine everything is judged on: Windows 11 Home 25H2, build 26200.9457, an NVIDIA
RTX 4090, an Intel i9-13900K, one Samsung display at 5120x1440 and 100%, HDR supported and
off, WebView2 153. Several checks cannot run on it, a Windows N edition, a remote desktop
host, a tall display; part 6 says for each what stands in.

---

# Part 1: what it is, as Rotem would use it

1. He presses a shortcut of its own, picked in Settings under the capture shortcuts.
2. Choosing the area works exactly as a capture does today, his call: the screen freezes and
   dims, the window or the part of a window under the pointer is lit and a click takes it, a
   drag draws a free area, the zoom circle and the guide lines are there, Escape cancels, and
   an area stays inside one display. The freeze is only for choosing and lifts when he lets go.
3. A thin outline stays around the area. A small bar appears just under it, or inside it
   when there is no room, with the microphone's name, a live level for his voice, whether the
   computer's sound is on, a word when the file will be scaled, and a count of three.
4. It records: the picture, the pointer, his voice and the computer's sound. The bar shows
   the running time, Pause and Stop. The outline and the bar are never in the recording. The
   area cannot be moved or resized while recording, as in every recorder surveyed.
5. The same shortcut, or Stop, ends it. Escape belongs to the application being recorded the
   whole time: Recon never listens for it once the area is chosen, because a walkthrough of an
   interface presses Escape to close that interface's menus. Throwing a recording away is a
   button on the bar.
6. Recon comes forward with the recording on the timeline like a capture, its thumbnail
   carrying its length. After a short "finishing" state it is ready to play in the window
   where a picture would be, with sound.
7. Copy puts the recording on the clipboard as a file with a readable name. A paste into a
   folder attaches it. Slack, a chat and a mail are each tested by pasting, in step S5.12, and
   are not promised before that; each also has a size limit. Copy and Return works as it does
   on a capture. Save As writes a new MP4 and never writes over anything.
8. "Annotate this frame" takes the picture on screen as a still and makes an ordinary
   document of it, with callouts, as Plan.md §3.5 already does for an animation. That the
   still is the exact frame he was looking at is what gate S5.5 proves.

## Seven things he should know before anything else

- **Claude does not take video at all.** Its upload list is documents and still pictures, and
  of an animated GIF it reads the first frame only [source]. So for Claude the useful output
  of a recording is stills: "annotate this frame" first, "copy the key frames" later (step
  S5.14). Whether ChatGPT takes an MP4 could not be read and is tested by pasting.
- **A clicked window is an area, not a window.** The recording takes the rectangle the window
  had at the click. If the window moves afterwards, the recording does not follow it. The
  Snipping Tool's window mode behaves the same way [source]. The Game Bar and OBS do follow a
  window, by recording the window itself and not an area of the screen; that is a different
  route from D2's and is not in this plan.
- **Anything wider than 4096 px is scaled down to 4096 across,** his call. On this screen that
  is the whole display and any window maximized on it: 5120x1440 becomes 4096x1152. An area up
  to 4096 across is recorded pixel for pixel. Shrinking to four fifths is the worst case for
  small text, so text in a scaled recording may be softer than on screen; gate S5.3 ends with
  him looking at a sample before the method is fixed. "Annotate this frame" gives the
  recording's pixels, so a still from a scaled recording is the scaled size, not the screen's.
- **Windows may draw its own yellow border** around the whole display while a recording runs.
  Recon asks for it to be left off. Whether Windows agrees on this machine is [untested] and
  is the first thing gate S5.2 reports, with whether the border is ever inside the file. A
  Windows setting or a company policy can force it on, and then no code removes it. On any
  Windows 10 it cannot be removed at all [source]. While any recording runs the pointer can
  also feel slightly heavier under load; that is Windows' doing [source].
- **Both sounds means one mixed track, and speakers mean echo.** His voice and the computer's
  sound are mixed as they are recorded and cannot be separated or rebalanced later; separate
  tracks would mean replacing the part that writes the file. With speakers playing, the
  microphone hears them and the file echoes; headphones avoid it, and the first version does
  not remove echo. A Bluetooth headset used as the microphone drops everything he hears to
  telephone quality while a recording runs; that is Bluetooth [source].
- **Locking the computer ends the recording at once,** on purpose, so the lock screen never
  lands in a client's file [source for the leak]. Recon keeps the display awake while a
  recording runs, so idle time cannot lock the session and end it by itself.
- **A recording that dies with the machine is recovered at the next start,** up to its last
  second or so, if gate S5.4 proves the method. Until it does, that is the design and not a
  promise.

## What the first version must have

Taken from what Snagit, the Snipping Tool, ShareX, ScreenToGif, Loom, CleanShot and the Game
Bar share, cut down to a designer recording interface walkthroughs with his voice:

1. The area chosen as a capture is, with its size beside the pointer as today. An area
   smaller than 64 by 64 is refused with a word.
2. The microphone's name, the computer's sound and a live voice level shown before the
   recording starts. The top complaint in every recorder's support forum is a walkthrough
   that turned out silent, or on the wrong microphone, found only afterwards.
3. A count of three, with a switch.
4. The outline and the bar, never in the file; the tray icon showing that a recording runs.
5. Pause and resume, in one file, on the bar and on a key of its own. Six of the seven tools
   surveyed have pause, and ShareX's users asked for about five years before they got it. It
   changes how the recorder keeps time, so adding it later means rebuilding that part.
6. Stop on the shortcut and on the button, and Stop always works, even when the part that
   writes the video has failed.
7. Throwing a recording away, from the bar.
8. The pointer in the recording.
9. Thirty pictures a second, in an MP4 of the kind every browser and chat plays, one sound
   track.
10. At Stop, Recon opens with the recording kept and ready to play. Nothing needs saving.
11. Trim at the start and the end. After a trim, Copy and Save As take a moment, seconds per
    minute of video [untested], because the kept part is written fresh.
12. Copy as a file, Copy and Return, and Save As with a date and time in the name.

And one rule over all of them: a failure in the middle, a full disk, a display unplugged, a
lock, ends the recording cleanly with what it has and says so. After sleep or a crash, at
worst it is recovered at the next start.

## What waits, in the order daily use is likely to ask for it

1. A smaller copy for places with a size limit. It is first because the limits bite early:
   GitHub's free plan stops at 10 MB, which is about a minute.
2. Choosing a microphone other than Windows' default. Until then a wrong microphone means
   Cancel, changing Windows' default, and starting again.
3. The key frames copied as stills for Claude.
4. Rings on clicks or a highlight on the pointer, and a switch to keep the pointer out.
5. A 16:9 lock with 1920x1080 and 1280x720 sizes. "The same area again". Restart.
6. A key that mutes the microphone mid-recording. A soft warning at one hour.
7. GIF export. Sixty pictures a second. Shown keystrokes. Sound as separate tracks.
8. Following a window that moves, by recording the window itself.

## What Recon refuses, each for a reason

- A webcam picture, cloud upload, share links, accounts: Recon is local and has no server.
- Drawing on the screen while recording, automatic zoom, backgrounds, a multi-track editor,
  cutting a piece out of the middle, joining recordings, noise removal, captions, summaries:
  that is a video editor, which this plan rules out here. Trimming the two ends is the only
  edit a recording gets.
- Streaming, background "record what just happened", scheduled recordings.
- Panels of video options: one quality that is right, picked by him once in gate S5.3.
- Switching Windows' notifications off while recording: there is no public way for a program
  like Recon to do it [source]. The README says to turn on Do Not Disturb by hand.
- Recording a protected film: Windows hands such content over as black, silently [source].
- Playing video files from outside Recon. His rule that viewing a file never changes it is
  not touched by this feature at all: a recording is Recon's own file from its first byte, and
  no outside file is opened.

---

# Part 2: the calls

## Decided by Rotem, 2026-09-20

| # | The call | His answer |
|---|---|---|
| D1 | Sound | Both: his voice from the microphone and the computer's own sound. |
| D2 | Choosing what to record | Exactly as the regular capture works today. |
| D3 | An area wider than 4096 px | Scaled down to 4096 across, one file type that plays everywhere. |

## Open, each put as what he would see

The recommendation is the plan's. The answer is his. The first group changes how the
recorder is built and is wanted before any step; each later group is wanted before the step
named over it.

**Wanted first, before S5.1**

| # | The call | Recommended |
|---|---|---|
| C1 | **Pause.** A Pause button on the bar and a second shortcut in Settings for pause and resume, and the recording carries on in the same file; or no pause in the first version. Snagit, which he uses today, has one key for start, pause and resume and another for stop; this plan has one for start and stop and one for pause. Saying yes costs roughly one more session in each of the engine and sound steps. | Pause, with its own key. |
| C2 | **How long, and how much disk.** No longest recording and a plain warning when the disk runs low; or a limit. A minute of a 1920x1080 area is expected at 5 to 15 MB [untested]. "No limit" holds only if gate S5.4 shows the working file can pass 4 GB, which Microsoft does not promise [source]; if it cannot, the recording ends cleanly before that point, announced on the bar beforehand and never applied silently. | No limit, a low-disk warning. |
| C3 | **An area taller than 2304 px,** a display turned on its side. It is scaled down to fit too, keeping its shape; or left as it is if gate S5.3 shows Windows' own player and Recon's window play a taller file. The 2304 comes from Microsoft's documentation of its player [source], not from D3, which speaks of width only. | Scaled, until the gate shows a taller file plays. |
| C4 | **Other people's code.** Several open projects do parts of this under licences that allow copying with their copyright lines, which would mean a notices file in Recon's release; or everything is written anew and those projects are read for the idea only, as the first draft said. Rule 21 makes this his word. | Written anew. One less thing in the release, and the licence of Recon is not yet chosen. |

**Wanted before the engine, S5.7**

| # | The call | Recommended |
|---|---|---|
| C5 | **The microphone is pulled out mid-recording.** The picture and the computer's sound carry on, the bar says the microphone is gone, and his voice rejoins if it comes back; or the recording ends at once with what it has and says why. | It carries on. A silent stretch is a smaller loss than a walkthrough cut short. |
| C6 | **A lock, or the display arrangement changing, mid-recording.** The recording ends at once; or it pauses and he resumes it by hand. Both are moments when the wrong picture can land in the file [source], so carrying on is not offered. | It ends. Pausing leaves a recording open across a locked screen. |
| C7 | **Throwing a recording away.** A button on the bar. The recording goes to Recon's trash with no question and Ctrl+Z or the trash brings it back; or it is gone for good and that is why it asks once first. | To the trash, no question. It matches "no confirmation" in the capture flow and keeps it inside undo. |
| C8 | **A recording recovered after a crash.** It simply appears on the timeline at the next start, marked as recovered; or Recon asks first. | It appears, marked. |

**Wanted before the interface, S5.11**

| # | The call | Recommended |
|---|---|---|
| C9 | **The sound switches.** Two small switches on the bar, microphone and computer's sound, that remember how he left them, next to the live level; or the switches only in Settings and the bar only shows their state. They can be changed until the count ends and are locked while recording. Principle 2 says no routine decisions during capture; a remembered switch is not a question, but it is his principle. | On the bar, remembered. |
| C10 | **Recon's own window.** Today a capture does not hide Recon; its window can even be picked. For a recording: the same, Recon stays where it is and comes forward at Stop; or Recon steps out of the way when the recording starts and returns at Stop. | The same as a capture, since D2 says so. It also lets him record Recon itself. |
| C11 | **Where the bar sits.** Normally just under the area, or inside it when the area touches the screen's edge. When the whole screen is recorded there is no outside: it sits inside at the top middle, never in the file, and can be dragged; or at the bottom middle, which on Windows 11 covers the centred taskbar icons; or it collapses to a small timer in a corner. | Top middle, draggable. Its look is his, from the design system or a spec. |
| C12 | **The count of three.** On, off, or a Settings switch. | On, with a switch. The Snipping Tool's cannot be turned off and its users complain. |
| C13 | **A capture shortcut pressed during a recording.** It is ignored; or it takes a still capture in the middle of the recording, whose freeze and dimming would then be in the video. | Ignored in the first version. |
| C14 | **The record shortcut at first.** Empty until he picks one, as "Open Recon" was, with a "Record" item in the tray menu as the way in meanwhile; or a default key. | Empty, with the tray item. Any default can collide with Snagit while both are installed. |
| C15 | **The keys while a recording is shown.** Space plays and pauses; `[` and `]` step one frame, as they do for an animation today; Ctrl+C copies the file; Ctrl+Enter copies and returns; Ctrl+S is Save As; Page Up and Page Down walk the timeline as today. The recording is always fitted to the window, with no zoom or pan in the first version. | As written, his to change. |

**Wanted before getting it out, S5.12**

| # | The call | Recommended |
|---|---|---|
| C16 | **After Stop.** The recording is on the clipboard at once, as a capture is today; or only when he presses Copy. At once costs three things: a wait while the file is finished, seconds for minutes of recording and longer for an hour [untested], with a "finishing" state meanwhile; a second full copy of every recording written to disk at every Stop until it is swept; and whatever was on the clipboard replaced. | At once, to match a capture. His call turns on how often he pastes a recording straight away. |
| C17 | **It plays by itself at Stop, or waits for Space.** | It waits. A recording that starts talking the moment it opens is a surprise. |
| C18 | **The name of a copied or saved file.** `Recon 2026-09-21 14.32.mp4`; or with the recorded window's title in it. | The date and time only. A window's title can carry a client's name into a file he forgot he sent. |
| C19 | **A copied recording in Windows' clipboard history (Win+V) and its synced clipboard.** Kept out; or allowed, as his captures are today. | Kept out, since a client's screen is on it. The same question for still captures is not changed here; it goes to the Backlog at his word. |
| C20 | **How long a deleted recording stays in Recon's trash.** Thirty days like captures; or a shorter stay, since a recording of a client's screen is heavier to keep than a picture. | The same thirty days. |

**Wanted after the first version**

| # | The call | Recommended |
|---|---|---|
| C21 | **Click rings.** Whether he uses Snagit's pointer highlight or click animation today. If yes it moves up to right after the first version; it has to be drawn into the picture, which is a different mechanism from everything else here. | His habit decides. |

---

# Part 3: how it is built

References to "Plan.md" mean `project-os/Plan.md`; a bare section number means this file.

## 3.1 The boundaries

**The pixels never pass through the page while a recording is made.** Plan.md part 5's split
holds: the host records, converts, encodes, stores, copies and saves. The page draws the
controls it is asked to and plays the finished file.

**One stated exception, bounded.** Playing the file in a `<video>` means the web view decodes
the H.264, which Plan.md part 5 forbids for pictures. The exception is playback of Recon's own
finished file and nothing else. A still never comes from the page: "annotate this frame" is
decoded by the host from the file, so "no picture is born in the renderer" stands.

**Four settled decisions are amended, and each step that does so closes with a Decisions
entry naming the one it amends:** "One native decode route, and the original never enters
the web view" (the exception above, S5.5 and S5.10); "The web view's security posture" (one
media address added to the content policy, S5.5); "The clipboard is published by the host in
three formats" (a file as a fourth kind, S5.12); "The store: one folder per document"
(a new kind of source, S5.9).

**The still capture is not changed,** the overlay included. It stays one GDI copy of the
virtual screen, and Plan.md part 8's rule that a second STILL path is earned by a failure
stands. Video is a different job, and the measured figure says why: that copy takes 41 to
50 ms in a release build on this screen (Plan.md part 11), twenty to twenty-four pictures a
second at best with the processor doing all of it. The recorder is a sibling of
`CaptureSource`, which `host/src/capture/mod.rs` already expects in its header, in a module of
its own. `Frame`, a block of RGBA in memory, is the wrong currency for textures on the
graphics card and is not stretched to carry them.

**No codec and no encoder is shipped.** H.264 and AAC are encoded by what Windows and the
graphics driver provide, so the open-source release carries no patent-encumbered library and
no licence question [inference]. The cost is total dependence on the user's Windows and
driver, and 3.10 lists where that bites. A later contributor must not "fix" recording by
bundling FFmpeg or x264: that is the one move that would bring the licence question in. A
rewriter of the MP4 container in Recon's own code, should S5.4 call for one, is container code
only and stays clean.

## 3.2 Getting the frames

Windows.Graphics.Capture, per display.

- `CreateForMonitor` through `IGraphicsCaptureItemInterop`, from an ordinary program with no
  package identity, no picker and no consent window [source]. The overlay returns a desktop
  rectangle inside one display and nothing else, so the recorder finds the display with
  `MonitorFromRect` and turns the rectangle into that display's own pixels through
  `capture/coords.rs`, the one conversion, and nowhere else.
- **The floor for recording is Windows 10 2004,** not the 1903 the capture call itself needs:
  the flag that keeps Recon's own windows out is a black box in the file before 2004, and the
  pointer switch arrives with it [source]. On any Windows 10 the yellow border cannot be
  removed, because the switch exists only from build 20348 [source]. The README says both.
- A frame pool made with `CreateFreeThreaded`, two or three buffers, BGRA8, on a Direct3D 11
  device created on the adapter that drives that display, with BGRA and video support and
  `ID3D11Multithread::SetMultithreadProtected(TRUE)`. With the Sink Writer hosting the encoder
  that last call is mandatory, since its threads and the pool's thread share the device
  [source]; the closest reference recorder omits it because all its work sits on one thread.
  A texture cloned from a frame's description has its `MiscFlags` zeroed [source].
- The handler does three things only: take the frame, copy the area into a texture the
  recorder owns, let the frame go. Anything slower starves the pool and Windows skips frames;
  the API's owner says so [source]. Nothing is freed inside the item's `Closed` handler [source].
- One recorder thread that lives as long as the process. After a recording it is never torn
  down: closing the capture and then leaving the thread's COM apartment, or unloading
  `graphicscapture.dll`, crashes about one run in ten in FFmpeg's use of the same API, open
  since 2025 [source]. The DLL is pinned, and the file is closed BEFORE the capture is, so a
  hang or crash in the teardown can never cost the recording. A tray program records many
  times in one run, so S5.7 proves thirty recordings in one process.
- The pointer is in the frames by a switch, from Windows 10 2004 [source].
- **The yellow border.** Microsoft's page describes turning it off for packaged apps only.
  Three programs ask for borderless access before setting the switch, the API owner's own
  unpackaged Rust tool, OBS, and the mirror window of PowerToys ZoomIt, whose source says the
  request "is granted without a prompt for desktop apps" and that the switch does nothing
  until it completes; four others, ZoomIt's own recorder among them, never ask [source].
  Whether the request is needed is unresolved and is S5.2's. Recon makes the call, off the
  window's thread because it blocks, and treats an error as harmless; the reference tool's
  form panics on an error and is not the pattern. Windows 11 also has a setting, "Let desktop
  apps turn off the screenshot border", with a company policy behind it: when that is off the
  border stays and no code removes it, so the plan owes a help line, not a bug hunt [source].
  The border is also reported to come back after a resolution change, an open bug [source].
- **A still screen sends nothing.** On Windows 11 24H2 and later a frame arrives only when
  something changed [source]; before 24H2 the opposite bug existed and frames could pour in at
  the display's full rate with nothing changing [source]. So the file is never driven by frame
  arrival. 3.4's pacer is, and it discards surplus frames as well as filling gaps.
- **`MinUpdateInterval` is not set at all in the first version.** On 24H2 and later the
  default is already a cap near sixty a second [source]. A non-zero interval, or a starved
  frame pool, together with a topmost window kept out of the capture, which is exactly Recon's
  bar, stopped frames arriving for some applications on 25H2. The API's owner reproduced it,
  and the reporter says it is gone after KB5086672, builds 26100.8117 and 26200.8117 [source].
  This machine has the fix. An open-source user on an older build does not, which is the
  second reason the handler never holds a frame.
- **HDR.** With Windows' HD colour on, a BGRA8 pool gives clipped, washed-out frames; OBS and
  FFmpeg switch to a wider pool and tone-map it themselves [source]. The first version reads
  the display's state at start, records the standard-range view, says so once, and logs it as
  the still capture does (F61). A change of HDR state or of frame format mid-recording ends
  the recording. How it looks on this panel is [untested]; S5.2 records one clip with HDR
  switched on for Rotem to judge, and the wider pool with a tone map is the named later fix.
- **What ends a recording at once** (C6), each a privacy matter as much as a technical one:
  the session locks, because the lock screen can sometimes land in the frames [source]; the
  display arrangement changes, because a capture has briefly delivered the OTHER display's
  picture at that moment, an open bug [source]; the capture item closes; the graphics device
  is lost; the HDR state changes. The notice of a lock arrives after the lock, so the file is
  cut back by a stated few hundred milliseconds rather than trusted. Sleep and a display
  powering off are [untested] and are treated the same way. Recon asks Windows to keep the
  display on while a recording runs [inference, checked in S5.7].
- **What it cannot see, said in the README:** the consent prompt's secure desktop, true
  exclusive-fullscreen games, protected video (black, with no warning). Inside a remote
  desktop session frames arrive at forty to fifty a second [source]. And one side effect:
  while any capture session runs, Windows draws the pointer in software system-wide, still
  true in 2026, a fix promised [source].
- **Access refused.** Creating the session can fail with "access denied" although the API says
  it is supported, because of Settings, Privacy, "Screenshots and video recording", or a
  company policy [source]. Recon says that in words and points at the setting.

**The fallback if gate S5.2 refuses this route:** Desktop Duplication, which then IS the first
version's route. It honours the keep-out flag too, by field report [source], and costs drawing
the pointer by hand, handling lost access, a limit of four clients, and rotation.

## 3.3 Converting, scaling and encoding

- **Recon does the conversion itself,** with the Direct3D 11 video processor: the BGRA area
  becomes an NV12 texture on the card, scaled in the same pass when D3 applies. The Sink
  Writer's hidden converter is not relied on: the one shipping library that uses it carries a
  fallback for when the Sink Writer refuses the input [source], and whether it runs on the
  card at all is undocumented.
- **The conversion is limited range, 16 to 235, with the BT.709 matrix, set explicitly.**
  Limited range is what every player assumes for a file with no tag [source]. The closest
  reference recorder converts with BT.601 through two magic numbers, 17 and 33, which are not
  carried over; 21 is BT.709 limited [source]. Whether NVIDIA's driver honours those bits
  exactly is [untested], and a mismatch shows as washed-out or crushed contrast, so S5.3
  checks that pure black and pure white come back as 0 and 255.
- The driver's "automatic processing" on the video processor is switched off. Left on, the
  NVIDIA control panel's video settings, edge enhancement, noise reduction, dynamic range, can
  change the pixels of text [source].
- **The scaled case is a shrink to four fifths, the worst case for text,** and the video
  processor's filter is the driver's choice with no setting [inference, confirmed by the
  skeptic]. If S5.3 shows soft text, the fallback is one small shader pass, area or bicubic,
  used only when scaling, with the video processor converting one to one.
- Each frame in flight gets its own texture from a small pool, because the encoder keeps its
  inputs alive; one reused texture corrupts frames [source]. The device context is flushed
  before a texture is handed over: Chromium's source says NVIDIA's encoder "may not see the
  data" otherwise [source].
- **Sizes.** H.264 wants even numbers. An odd area loses its last column or row rather than
  gaining a black one that was never on screen. The floor is 64x64: Chromium decodes in
  hardware only from there and Microsoft's decoder starts at 48x48 [source]; NVIDIA's encoder
  minimum is reported near 145x49 and is unverified, so S5.3 records which encoder takes the
  64x64 case [untested]. The width ceiling is D3's. The height ceiling of 2304 is the plan's,
  from Microsoft's decoder page, and is C3. NVIDIA's H.264 encoder is reported capped at
  4096x4096 on every generation, this card included; that figure is from secondary sources and
  the only authority is the encoder refusing the type [untested], which is fine since Recon
  never asks for more.
- **The level.** Microsoft's own decoder is documented up to level 5.1 [source]. The rate is
  capped from the output size by `floor(983040 / (ceil(w/16) * ceil(h/16)))`. At thirty a
  second everything up to 4096x1440 fits, and so does a 2160x3840 display on its side; only
  4096x2304 itself goes past, and is capped at twenty-six. Whether the encoder picks the level
  by itself and whether real players refuse 5.2 is [untested].
- **The encoder is hosted by the Sink Writer,** NV12 in and H.264 out, hardware transforms
  allowed, the same Direct3D device. That gives one writer for picture and sound, and the
  AAC encoder for free. That it also falls back to software by itself on a machine with no
  hardware encoder is an [inference]; the software encoder fed textures would need each frame
  read back to memory, so S5.3 forces it and reports whether it holds thirty a second at
  4096x1152 on this processor. That matters twice, because the software encoder is also the
  only exit if NVIDIA's output fails the pixel check: both alternatives below use the same
  NVIDIA encoder. The alternatives: driving the encoder by hand and using the Sink Writer as a
  muxer only, which is what the API owner's recorder does and which pins the encoder to the
  right adapter with `MFTEnum2` [source]; or `Windows.Media.Transcoding`, which most reference
  recorders use, does the conversion, the encoding and the AAC inside, and gives up control of
  quality, key frames and colour [source].
- **Every setting is set and read back.** No default can be assumed: the documented ones
  belong to Microsoft's software encoder, and the one dump of NVIDIA's shows different ones
  [source]. Set: High profile, quality-based rate control, no B-frames (so the order frames
  are decoded in is the order they are shown in, which trim and stepping both lean on), CABAC
  on, low-latency mode explicitly off, because Microsoft's software encoder otherwise stays in
  its lower-quality slice mode [source]. The B-frame count must reach the encoder before its
  output type is set, so it goes in through the input type's encoding parameters [source].
  Reading a setting back does not show what is in the file, so S5.1 builds a small box walker
  that prints, from the file itself, the profile and level, the colour description, whether
  B-frames exist, and the time base. The name of the encoder actually loaded is logged.
- **Key frames by the clock, not by count.** The encoder's spacing is counted in frames. A key
  frame is forced every two seconds of wall time, which the certified encoders must support
  [source], so seeking stays tight.
- **The colour tag is an open problem, named.** A file with no colour tag is guessed at, and
  the guesses differ: Chromium reads untagged video under 720 px tall as BT.601 and above as
  BT.709; mpv switches on width as well; Windows' own player is reported to assume 601 even
  for HD [source]. Most area recordings are under 720 tall. Only a real tag in the file gives
  the same colour everywhere, and whether NVIDIA's encoder writes one when asked is
  [untested]; Chromium's source warns that hardware encoders often write "unspecified" and
  that setting the attributes can break some. S5.3 measures it. If no tag can be had, the fix
  is a colour box written into the closed file, which costs almost nothing if S5.4 ends with
  Recon's own rewriter and is real work otherwise; S5.4's decision takes S5.3's finding as an
  input for that reason.
- NVIDIA's H.264 encoder has a history of wrong output from textures, a colour conversion
  inside it failing, date and driver unknown [source]. So the gate's first check compares a
  decoded frame with the truth. "A file was produced" proves nothing. Crashes when its rates
  are changed mid-recording and a possible leak are also on record [source]: one encoder per
  recording, never reconfigured.
- Text in strong colours on a dark ground goes slightly soft in any H.264 file, because the
  colour is stored at half resolution. It is the format's doing and every recorder has it.

## 3.4 The clock and the pacer

- **A constant thirty pictures a second.** A timer ticks thirty times a second and hands the
  encoder the newest frame, or the previous one again when nothing arrived, and drops any
  surplus. Microsoft's own ZoomIt ships a variable rate, and so does Snagit; Snagit's
  variable-rate files are a documented cause of sound drifting in Vegas and Premiere. A
  constant rate costs almost nothing, since a repeated frame encodes to next to no bytes,
  keeps the sound and the picture ending together, and plays the same in a browser, an editor
  and a chat's re-encoder. Re-submitting the very same texture in two live samples is
  [untested]; if the encoder refuses, the pool gives the repeat its own copy.
- **One epoch for everything:** the first accepted frame's time. Sound from before it is
  dropped. Picture and sound are stamped from the same clock, QPC in 100-nanosecond units.
  That the capture's timestamp uses those units is a strong [inference], since Microsoft's own
  recorders pass it straight through; the recorder reads the clock's frequency at start, logs
  it, and refuses to assume. The two stamps mark different events, when the screen was
  composed and when the sound card captured, so a constant offset of up to a frame plus the
  device's delay remains and is measured, not assumed away.
- Timestamps are clamped to zero or later and strictly rising. ZoomIt's source records what
  happens otherwise: a first frame stamped a hair before the start makes the file's length
  balloon to the raw clock value [source].
- The first frame can take seconds to arrive. After three seconds with none, the recording
  stops and says the display is delivering no frames, as ZoomIt does for a remote session or
  a virtual machine [source], rather than writing an empty file.
- **Pause** (C1): the paused span is taken out of both clocks, sound packets stamped inside it
  are dropped, and the sound mixer's expected position is moved on purpose, so the jump is
  not mistaken for drift.
- Writing to the file never happens on the capture's thread or a sound thread. The writer
  throttles by blocking, so a slow disk would stall the capture. The writer's own statistics,
  read once a second, show a stall before memory grows [source].

## 3.5 Sound

- Two WASAPI streams in shared mode: the default microphone, and the default output in
  loopback for the computer's sound. Both are asked for one fixed shape, 48 kHz stereo, with
  Windows doing the conversion through two documented flags [source]; two libraries use them
  with loopback, OBS and Chromium do not, and that every device accepts them is [untested].
  On refusal that source falls back to its own format plus Windows' resampler, with a mono
  microphone copied to both sides by hand, since the resampler fills added channels with
  zeros [source].
- **The microphone is opened in the console role, never the communications role.** Opened as a
  communications stream, Windows turns every other sound down by 80% by default, which is the
  very sound being recorded [source].
- **The mixer runs by the clock, not by the arrival of packets.** Loopback delivers nothing at
  all while nothing is playing [source], and how true that is depends on the driver [source].
  So for every ten-millisecond slot the mixer writes what the sources gave it, or silence. It
  runs a fixed 100 to 200 ms behind the clock, because a packet describing a moment arrives
  after that moment, later still from a USB or Bluetooth microphone; mixing "now" writes
  silence where the packet was about to land, which sounds like crackle [inference]. The
  figure comes from S5.6's measured worst lateness. Whether keeping a silent stream playing
  makes loopback steadier is [untested]; OBS does not do it and Chromium does it for another
  reason, so nothing rests on it.
- A packet flagged with a timestamp error, or stamped more than about half a second from now,
  is stamped now minus its length instead [source: Chromium]. A packet flagged silent is
  written as zeros. No capture thread waits without a timeout, or Stop hangs during silence.
  The device-change callback only posts a message [source].
- **One track.** The microphone and the computer's sound are mixed into one stereo AAC track.
  Microsoft's MP4 writer takes exactly one picture stream and one sound stream, so two tracks
  are not an option on this route [source]; and standard players play only the first track
  anyway. Separate tracks later would mean a different muxer.
- Microsoft's AAC encoder takes 16-bit PCM only, at 44.1 or 48 kHz, and four bit rates, 96 to
  192 kbps for stereo. Every sample needs a time and a length that is not zero; a zero length
  crashes it with a division by zero [source]. So the mixer turns its floating-point sum into
  16-bit, through a limiter, itself.
- **Drift is measured first, then corrected.** The sound advances on the sound card's crystal
  and the picture on the computer's clock. Fifty parts in a million is 30 ms in ten minutes
  [inference]; no source gives real figures, so S5.6 logs the difference every ten seconds per
  source and the slope is the answer. The correction is OBS's rule, written anew since OBS is
  GPL: write sound by sample count, compare with each packet's stamp, and re-anchor past a
  threshold by adding silence or dropping samples, inside a quiet moment. OBS's threshold is
  70 ms, about where lips visibly part from words, so Recon's is 20 to 40 ms. Evidence says
  the loopback's stamps are dependable and a USB or Bluetooth microphone's are the risky ones
  [source].
- The MP4 writer cannot express "this track starts a little late", so both tracks start at
  zero, gaps are real silence, and at Stop the sound is cut to the last picture plus one
  frame. The AAC encoder's own start-up delay is a constant offset measured once and taken
  out of the stamps.
- **What changes in the middle** (C5). The default device changing raises no error on the old
  one, it just goes quiet, so Recon listens for the change and rebuilds that one stream while
  the file's format stays fixed. A device pulled out, or taken in exclusive mode by another
  program, is a "busy, trying again" state with its own words, different from "absent" and
  from "blocked by privacy" [source]. The recording carries on meanwhile, with silence from
  that source, and the bar says so.
- **No microphone, or the privacy switch off.** No device is one error and a refused one is
  another [source]. In both, the computer's sound is still recorded, the bar says the
  microphone is off and why, and a button opens the right page of Settings. The microphone is
  opened when the area is set, before the count, so a refusal, and the consent window Windows
  is about to add for desktop programs in 26H2 [press reports], is settled before the first
  frame and never lands inside the recording. The live level comes from the same samples.
- **Recon makes no sound of its own while recording.** Starting a recording pauses any
  recording playing in Recon's window, and playback stays off until Stop. That rule is what
  makes keeping Recon's own sounds out unnecessary.
- **Known limits, said in the README.** Echo from speakers. The Bluetooth headset's drop to
  telephone quality, with a warning on the bar if a Bluetooth microphone can be detected
  [untested]. With Bluetooth headphones or a television as the output, the recorded sound can
  lead the picture by that device's delay; one constant per recording is reserved for it
  [inference]. A 5.1 or 7.1 output is mixed down and can come out quieter [source]. Whether
  the recorded level follows Windows' volume and mute is [untested]; S5.6 measures it, and if
  muted speakers give a silent file the bar shows a level for the computer's sound as well.

## 3.6 The file, and surviving a crash

An ordinary MP4 has its index at the end and is unreadable until it is closed properly. The
Snipping Tool loses a recording when it crashes, the machine sleeps or the disk fills.
Principle 7 says work survives. The design:

1. **Record into a fragmented MP4 working file**, `recording.part.mp4`, inside the document's
   own folder under `%LOCALAPPDATA%\Recon\documents\`. Never under Videos or Documents: those
   are often moved into OneDrive, which would try to upload a growing file of gigabytes, and
   Defender's protected folders block an unsigned program from writing there [inference from
   documented behaviour]. The file is held without write sharing while it is recorded.
2. **At Stop, rewrite it without re-encoding** into `recording.tmp.mp4` beside it, by reading
   the compressed samples and writing them again, which Microsoft's own sample does [source].
   Only when that file is closed is it renamed to `recording.mp4`, and only after the rename
   is the working file deleted. A half-written file under the final name would look complete
   to the loader; a leftover `tmp` is deleted at the next start and the rewrite redone. The
   rewrite is an offline pass: the writer's throttling is switched off, or it paces itself at
   real time; the picture type keeps its sequence header and the sound type its user data, or
   the writer drops samples; no encoder exists in this writer, so asking it for the encoder's
   settings fails by design [source]. The reader may also walk every fragment when it opens, a
   second pass [untested]. The cost is free space equal to the recording, which the low-disk
   floor counts, and a time S5.4 measures on a ten-minute and a sixty-minute file.
3. **Until the rewrite ends the recording shows its thumbnail and a "finishing" state.** Play,
   "annotate this frame", Copy and Save As start when it ends. Playing the working file
   meanwhile is not offered: Chromium walks every fragment before the first frame, thousands
   of requests for a long recording [source], and the frame index does not exist yet.
4. **The leftover working file is the crash marker.** At the next start a working file that
   can be opened exclusively is orphaned: Recon cuts it after its last complete fragment,
   rewrites it the same way, and shows a recovered recording (C8). One with no complete
   fragment is removed quietly. A tail of zeros, which NTFS can leave after a power cut, is
   treated as the end and never as one giant fragment.

This is what OBS and FFmpeg do, in their own ways [source]. OBS converts in place with no
copy; Microsoft holds a patent on in-place conversion, to 2040 [source], so Recon takes the
conventional copy and stays clear of it.

**What nobody has a source for, and gate S5.4 settles before the plan leans on it:**

- When the fragmented writer puts its first index on disk. It demands a seekable file, which
  hints it may patch something at the end. If a killed file has fragments but no description
  of its streams, the design as written fails, and the fourth outcome below is the exit.
- Whether Media Foundation can read its own fragmented file, whole and cut short, with right
  times.
- Whether the writer's buffers die with the process, which would make "at most the last
  fragment" untrue. A crash is expected to lose at most the last fragment; a power cut loses
  an unknown amount unless Recon flushes its own handle every few seconds.
- The fragment settings are not the Sink Writer's. The pattern that works is making the
  fragmented sink directly, setting them on it, and wrapping the writer around it; whether the
  hosted hardware encoder and its settings survive that form is [untested] [source for the
  pattern]. Fragments are reported to come about three a second by default [untested].
- Fragmented files over 4 GB are not promised [source].

**The four outcomes S5.4 can end with,** recorded as a Decisions entry: this route as written;
Recon's own rewriter, a few hundred lines written from the file format's specification and
from no other project's code; Recon's own file object under the writer, written straight
through; or, if a killed file carries no stream description, a small sidecar written at Start
with the descriptions Recon already holds, from which its own rewriter builds the index. If
the writer fails past 4 GB, the recording rolls to a new working file near 3.5 GB on a key
frame, or C2's limit applies.

Traps with sources: never call the writer's `Flush`, which discards queued samples and is a
documented hang before `Finalize` with AAC on Windows 11; never use the "index first" switch,
which costs a second full copy and has an open report of files with no data. The writer is
made with its async callback, so closing completes by callback with a watchdog beside it.

**Stop always answers.** Closing the file runs on a worker with a stated limit, five seconds
to begin with. After the limit the working file is left for recovery and Recon says so. In
ShareX a dead encoder leaves Stop doing nothing, and that report was closed unfixed.

**Shutting down.** A sign-out or a Windows Update restart is far likelier than a power cut.
Windows gives a few seconds: Recon asks it to wait, stops, closes the working file, skips the
rewrite, and leaves recovery to the next start. The request to wait must be made on the
thread that owns the window [source], which Tauri owns, so S5.7 decides where that message is
caught. Quit from the tray during a recording does a full Stop first; today Quit exits within
a second and a half and would leave a broken file.

**The disk.** The writer's documentation does not say which call fails when the disk fills,
so Recon does not wait to find out: it watches free space and stops cleanly above a floor
that still leaves room to close the file and rewrite it.

**One way down.** Every failure, a lost device, a closed capture, a display change, sleep, a
lock, an HDR change, an encoder error, goes through one routine: stop feeding, close the file
if that works inside the limit, otherwise leave the working file for recovery, and say what
happened in plain words.

## 3.7 The recording as a document

What exists today assumes a document is one still picture with a `source.png`. The store
refuses a folder without it, the record's fields are all required, its source is a closed
list of kinds, and every write takes the whole file in memory [source: the code].

- A recording is a new kind of source in `document.json`, with its new fields optional so
  every document already on disk still reads. An older Recon meeting such a folder fails to
  read that one record, logs that it skipped it, and leaves it alone, which is the safe
  outcome. Writing a poster as `source.png` to slip past the old loader would be the unsafe
  one: an old build would show the recording as a still and let it be annotated. One known
  cost: an old build's thirty-day sweep still removes a recording that was already in the trash.
- The folder holds `recording.mp4` written as 3.6 says, `document.json` with the kind, the
  size in pixels, the length, and later the trim, `thumb.png`, and an index of every frame's
  time built by reading the closed file back. The index is not taken from the times handed to
  the writer: an MP4 stores rounded gaps in a time base the writer chooses, undocumented
  [source].
- **The thumbnail** is written at Stop at exactly the size the store expects for the recorded
  size, so it is never judged stale. The host's thumbnail path gets a branch for a recording:
  when the file is missing, as for a recovered recording that has no recorder behind it, the
  frame reader of 3.8 decodes one and writes it. It never falls through to decoding a
  `source.png` that does not exist.
- The store's size figure counts the MP4 without a change, once it is told the folder changed.
- The timeline lists it through the path captures use, with two new fields for the badge. A
  finished recording announces itself with an event of its own, because the capture's event
  also triggers the automatic copy of a composed picture, which does not exist here. That
  event must add the recording to the selected tab exactly as a capture is added (Plan.md
  §3.8's feed rule): tabs store ids and take it for free, but joining the selected tab is
  the capture event's doing today, and without it a recording made with a tab selected would
  not be on the timeline he is looking at.
- Showing it is a new mode of the window: a `<video>` where the canvases are, tools off,
  always fitted (C15). Walking the timeline with Page Up and Page Down has to step onto a
  recording and off it again, which today's code would answer with an error.
- A file being played cannot be moved by Windows, so the server of 3.8 never keeps it open
  between requests, the page lets go of the video before a delete, and the host's frame
  reader is released first.
- `settings.rs` already has a flag named `RECORDING` that means "a shortcut is being picked".
  The recorder's names avoid that word, so a guard written for one is never read as the other.

**Undo (rule 23), each new action named.** A finished recording is not an act in the undo
list, as a finished capture is not: Ctrl+Z after Stop does not remove it. Deleting it is the
way, and that delete is undone by Ctrl+Z as a capture's is, once the trash accepts a folder
with no `source.png`. Throwing a recording away from the bar goes to the same trash (C7), so
it is undone the same way. A trim is one act, undone and redone. "Annotate this frame" makes
an ordinary document and no act, whose own notes have their own undo. Starting, pausing and
stopping are not acts: a recording cannot be un-recorded into the past. The bar's switches
and the count's switch are settings, and Settings stays outside undo.

## 3.8 Playing it

- H.264 with AAC in MP4 plays in WebView2 [source]. HEVC only where its extension is
  installed, which is one more reason the output is H.264.
- The page's `<video>` points at a new address the host serves, by the document's number and
  never by a path, answering each ranged request with a slice of at most a few megabytes. The
  slice must be EXACTLY the bytes it names: WebView2 ignores the range written in the answer
  and reads whatever it is given [source]. A request with no range is refused, never answered
  with a slice and never with the whole file; the video element always sends one [source].
  The arithmetic gets unit tests: the last short slice, a request past the end, a request for
  the last N bytes, a request with no range.
- It must stay on Tauri's `http://<name>.localhost` form. Microsoft says ranged video fails
  on a truly custom scheme and that this is unlikely to change [source]; nobody may tidy it
  into one later.
- The content policy in `tauri.conf.json` has no `media-src` today, so a video shows nothing
  and the only sign is a line in the console. Exactly that one address is added. The
  timeline's thumbnail, and any poster, stay the fetched pictures they are today, since the
  policy allows pictures from the page itself only.
- Every answer carries a strong tag and a modified time, because Chromium reuses what it
  fetched for five minutes [source]. The bytes of a stored recording change only at the
  rewrite's rename and at recovery, and those are the moments this guards.
- Recon's own controls, not the browser's. Its menu offers "Save video as", which would ask
  for the whole file in one piece.
- Each request is answered on its own thread, never through the one worker that serves
  picture regions, which would cancel the editor's own requests.
- **Which frame is on screen.** Chromium has no "next frame" and does not promise exact
  seeking. The page reads the shown frame's time from `requestVideoFrameCallback`, sends that
  number, and the host matches it against the index. Stepping seeks to a frame's time plus a
  millisecond and confirms by the same callback [source for the reasoning].
- **The still for "annotate this frame"** is decoded by the host: the reader seeks to the key
  frame before, then decodes forward to the exact time [source for the seek]. The host asks
  the reader for NV12 and converts with the same matrix and range the recorder used, in
  Recon's own code, so the still matches the recording by construction. The reader's advanced
  processing is not used, because it converts frame rates and can restamp frames; the simple
  one is software only and cannot be combined with the graphics card [source]. The frame is
  cropped to its real size, since the decoder pads to sixteen pixels [source]. With a key
  frame every two seconds that is at most sixty frames. It runs on its own thread, since
  every slice is completed on the window's thread and anything slow there stalls the video
  [source].
- Whether the time Chromium reports and the time Media Foundation reports are the same number
  for the same picture is [untested]. Chromium starts its timeline at the earlier of the two
  tracks [source], which is another reason both start at zero. If the two decoders disagree
  by a constant, that offset is measured and stored with the index.
- **If ranged serving fails in S5.5,** the fallback is WebView2's own mapping of a folder to a
  host name, which one recent report says now seeks a 4 GB file instantly [unverified]. It
  maps a folder by path and not a document by number, which is why it is second choice.

## 3.9 Getting it out

- **Copy** puts a file on the clipboard, the way Explorer and Chromium do, with the "copy"
  hint, as a new kind of publish beside the two `clipboard.rs` has, inside the same single
  transaction. The page names the document and never a path, as the colour picker names a
  colour and never a text. Copy and Return does the same and returns to where he came from;
  if the file is still being finished, both wait and say so.
- The clipboard carries only a path, and the pasted file's name is that path's last part
  [source]. So the store's own `documents\12\recording.mp4` is never what goes on the
  clipboard: it would show the store's path to the target, carry a machine's name, and let a
  "cut and paste" move it out of the store. Copy writes a readable copy,
  `Recon 2026-09-21 14.32.mp4`, into `%LOCALAPPDATA%\Recon\outbox\`. A plain copy, not a hard
  link: a link is the same file for Windows' sharing rules, so an upload in progress could
  block deleting the document and the reverse [source]. A copy of tens of megabytes is a
  fraction of a second; of a gigabyte, several seconds [inference, S5.12 measures it].
- A second Copy in the same minute gets a suffix, never the same path, since the first may
  still be open in an uploader. The path stays under 260 characters, is built from the known
  folder and never from Rust's `canonicalize`, whose `\\?\` form Explorer and browsers
  mishandle, and is written as wide characters so a profile folder with Hebrew in it works.
- The outbox is swept by looking at the clipboard itself: a file the clipboard still names is
  kept, anything else older than some minutes goes. Counting clipboard changes does not work,
  because the count dies with the process while the clipboard's content does not [source].
  Deleting a recording for good also removes its outbox copies the clipboard no longer names.
  The outbox counts toward the disk figure. It is a new place Recon writes, and rule 11 is
  untouched because only Recon's own files are ever staged there.
- Whether a recording appears in Windows' clipboard history and its synced clipboard is
  settable with documented flags [source], and is C19. Whether Windows keeps copied FILES in
  that history at all is [untested], so S5.12 first proves a file copied without the flags
  does appear there; only then does the flags' check mean anything.
- **Every paste target is untested until pasted into.** That Chromium hands a pasted file to
  a page is sourced; that Slack, Teams, WhatsApp, Gmail or Notion then attach a video is each
  app's own behaviour. The check also plays the file back FROM the target, since they
  re-encode uploads.
- **Size limits that are real:** GitHub 10 MB on free plans, Notion's free plan 5 MB, Gmail
  25 MB, Teams 100 MB, Slack 1 GB [source, some secondary]. The size is shown at Copy.
- **Save As** is the still picture's: a new file only, a suggested name, the last export
  folder. A protected folder refusing the write gets readable words.
- **Trim.** The stored recording is never cut, so its bytes and its address never change on a
  trim; the trim is two numbers in the document, and the player honours them by clamping its
  time. **Copy and Save As always write the exact cut,** a fresh hardware encode of the kept
  range, so an export never holds a frame outside the handles. A cut without re-encoding can
  only start on a key frame [source], up to two seconds early, which keeps footage he meant to
  remove; it is not offered in the first version. A "smart" splice re-encoding only the first
  second is refused: LosslessCut's tracker shows it glitching, and Media Foundation gives no
  safe way to make the two halves match [source].
- **GIF, later.** The well-known quality encoder, gifski, is AGPL, and libimagequant is GPL,
  both wrong for the release. The `gif` crate already in the tree is MIT or Apache and does
  the job with Recon's own frame differencing; Windows' imaging component can also write a
  GIF with no dependency at all [source]. A GIF's frame delay is whole hundredths of a second,
  so the rates on offer are 10, 12.5, 16.7, 20 and 25 a second, not 15. Expect 5 to 25 MB for
  thirty seconds of interface work and far more for scrolling [untested].

## 3.10 The outline, the bar, and what the platform does to them

- Both are plain Win32 windows of the host's, not web view windows and not drawn with
  per-pixel transparency. The "keep me out of captures" flag fails on windows drawn that way,
  by design, on builds before 24H2 [source]; and a Tauri window with that flag turned black in
  captures after being hidden and shown, an open bug [source]. So the flag is set once, its
  answer is checked, and the windows are never hidden and shown during a recording.
- They take no focus, so the application being recorded keeps it, Escape included.
- They are kept thin and small. A browser stops painting when it believes it is covered, and
  a window kept out of the capture still counts as covering it, so a large one freezes the
  page under it in the recording [source].
- The outline is drawn outside the area wherever there is room. The overlay is reused
  unchanged and shows nothing about scaling; the word that the file will be scaled appears on
  the bar, beside the size, the moment the area is set.
- The flag hides them from other screen-sharing tools too, Teams or Zoom running alongside.
  Snagit shipped a fix in 2025 for its toolbar vanishing from the user's own sight under
  remote desktop; Recon's bar is checked under the sharing tool he uses.
- Whether Recon's own still capture, a GDI copy, leaves these windows out is [untested]. It
  matters only if C13 ever allows a capture during a recording.
- Windows now has a per-recording list of windows to leave out, shipped in 2026 [source]. It
  avoids every trap above, but it needs a newer `windows` crate than the one Tauri shares, so
  it is a later improvement.
- **The tray.** The icon is built once today and no handle to it is kept. A recording state
  needs the handle, a second icon, and "Record", "Pause" and "Stop recording" items.
- **The shortcuts,** two new slots with C1, join six hard-coded places each: the settings
  list, its tests, the saved file and its fixed save signature, the handler in `main.rs`, one
  row and one list in the page. The same press stops a running recording, so the handler asks
  the recorder's state before anything else. The sound switches and the count's switch are the
  first settings in `recon.json` that are not shortcuts, so they bring the first Settings row
  of a new type.
- **Windows N and KN editions ship without Media Foundation.** The `windows` crate binds its
  functions as static imports, so the moment Recon imports them the PROGRAM would not start
  there, the still capture and the viewer with it, not only recording [source for the missing
  DLL, inference for the rest]. Whether those imports can be loaded late is [untested]; if
  not, every Media Foundation call goes through a table of Recon's own filled at run time.
  That decides how the gates' code is written, so it is S5.1's first item, not a later step's.
  Recording then reports itself unavailable with a line naming the Media Feature Pack.
- **The `windows` crate stays at 0.61,** the one copy shared with Tauri. Its feature list
  roughly doubles, which lengthens the build. Waiting on an async call is `.get()` in this
  version and `.join()` in newer samples, and the borderless request needs a feature of its
  own; both are traps for whoever reads a newer example.

## 3.11 Whose code may be read, and whose could be copied

Whether anything is copied at all is C4, and rule 21 makes it Rotem's word; the plan
recommends writing anew. Either way the licences decide what is even possible, so they are
recorded once.

| Licence allows copying with attribution | May be read for the idea only |
|---|---|
| robmikh's repositories (MIT): `displayrecorder`, `Win32CaptureSample`, the window-exclusion demo | OBS Studio (GPL 2) |
| `windows-capture` (MIT) | ShareX (GPL 3) |
| Microsoft PowerToys ZoomIt (MIT), the closest reference: an ordinary Win32 recorder with area crop, both sound sources and MP4 | FFmpeg (LGPL and GPL) |
| WebRTC, Chromium (BSD) | Cap, except its `scap-*` and `cap-camera*` crates |
| NAudio, `wasapi-rs`, `drag-rs` (MIT or Apache) | untrunc, gifski, libimagequant |

---

# Part 4: what could go wrong, and where it is caught

| Risk | Caught at |
|---|---|
| Recon will not start on Windows N | S5.1: the built program's import table; a flag that makes the late loader look for a wrong name, which must give the Media Feature Pack wording while still capture works. A real N edition is listed as unverified until someone has one |
| The border stays, or the user's setting forbids removing it | S5.2 by eye and in the frames, with the request and without it, with Windows' setting on and off; the help line |
| HDR on clips or washes out the picture | S5.2: one clip with HDR switched on, judged by Rotem; a change mid-recording ends it |
| The encoder's output is wrong in colour or content, a known history on NVIDIA | S5.3: a decoded frame against the synthetic source's exact truth; a crop shifted by 1 px and a frame converted with the wrong matrix must both FAIL at the same tolerance (`project-os/QA.md` §12) |
| Washed-out or crushed contrast from a range mismatch | S5.3: pure black and white come back as 0 and 255 in Edge and in the host's decoded frame |
| Colours shift between players because the file carries no tag | S5.3: patches at a height under 720 and one over, the tag read from the file by the box walker |
| Text in a scaled whole-screen recording is too soft to read | S5.3, judged by Rotem beside a still capture, before the scaling method is fixed |
| A still screen breaks the file: no frames, wrong length, a refused repeat | S5.3: a counter that freezes for four seconds and resumes |
| A crash, a kill, a shutdown or a full disk loses the recording | S5.4: kill tests at 2, 10 and 60 s with a pass figure; S5.7: quit, shutdown and the disk floor |
| A half-written file under the final name looks complete | S5.4: the process killed during the rewrite |
| Media Foundation cannot read its own working file, or a killed file has no stream description | S5.4, with Recon's own rewriter and the sidecar as named exits |
| The video does not play or seek in the window, or memory climbs | S5.5: a file over 1 GB scrubbed for ten minutes, growth between minute 2 and minute 10 under a stated figure |
| "This frame" is not the frame that was on screen, or its colours differ | S5.5: the page's time, the host's decoded frame and the counter in the picture agree; the still's patches against the paused player |
| A silent walkthrough, or the wrong microphone | S5.6 for the facts, S5.8 and S5.11 for the name and the live level before the count |
| Sound drifts from the picture, from the computer's sound or from the microphone | S5.8: the corrector proved by mis-stating a source's rate, and the same run with it switched off must FAIL; one run through the microphone |
| Windows turns the computer's sound down when the microphone opens | S5.6: a steady tone's level before and after the microphone opens; more than 1 dB fails |
| The lock screen or the other display lands in a recording | S5.7: a real lock and a real display change; every frame read back must carry the markers |
| Stop does nothing because the writer is blocked | S5.7: the writer blocked on purpose; Stop answers inside the limit |
| The second, tenth or thirtieth recording in one run crashes or leaks | S5.7: thirty short recordings in one process, memory flat from the fifth |
| A recording vanishes after a restart, an old build mangles it, or the new build mangles old documents | S5.9: restart and find it; the previous release against the new store, and this build against the previous release's store, by count and by hash |
| A recording made with a tab selected is not on the timeline he is looking at | S5.9 |
| Deleting a recording that is playing fails | S5.10 |
| The bar hides from the user under a screen share, or Recon steals focus or Escape | S5.11 |
| A pasted file has a machine's name, or moves out of the store, or a Hebrew path breaks | S5.12 |
| A recording of a client's screen sits in clipboard history | S5.12, after first proving a file lands there without the flags |
| A trimmed export still holds the cut footage | S5.13: the export's first and last counters are the handles' |
| Recording slows the thing being recorded | S5.3's load figures, judged by Rotem as F59 was |
| A later contributor bundles FFmpeg to "fix" something | 3.1 and the README say why not |

---

# Part 5: what is known, and what is assumed

**Measured on this machine:** the GDI copy of this screen takes 41 to 50 ms, which is why
video needs another route. The display is 5120x1440 at 100%. The card is an RTX 4090. The
build is 26200.9457, which has the fix for the capture bug of 3.2. The HEVC and AV1
extensions are installed.

**Read in documentation or source code:** everything marked [source] above. The ones the
design rests on most: the capture item and the free-threaded pool; a still screen sending
nothing on 24H2; the 25H2 bug with a set update interval and an excluded topmost window; the
lock screen and display-change leaks; HDR clipping a BGRA8 pool; the AAC encoder's input
rules; the MP4 writer's one picture stream and one sound stream, no edit lists, its index at
the end; loopback's silence; the communications role and ducking; WebView2 reading the whole
stream it is given; ranged video only on an http address; Chromium's five-minute reuse and
its colour guesses; the clipboard file's name being the path's last part; hard links sharing
locks; Claude taking no video; the licences in 3.11.

**Assumed until a gate measures it:** whether the border request is needed and what it
returns here; whether the border's pixels land in the frames; the frame rate delivered by
default on this panel; how HDR looks; whether GDI honours the keep-out flag; sleep and display
power-off; whether Media Foundation can be loaded late; whether NVIDIA's encoder accepts the
settings as asked, honours the range and matrix, writes colour tags, emits B-frames when told
not to, picks the level itself, or takes the same texture twice; whether the Sink Writer falls
back to software by itself and how fast that is; the working file's first index, Media
Foundation reading it back, the writer's buffers at a kill, the fragmented sink with the
hosted encoder, the rewrite's cost and files over 4 GB; the time base the writer picks; the
two decoders agreeing on a frame's time; autoplay; every paste target and whether Windows
keeps copied files in clipboard history; real drift in parts per million; whether every device
accepts the fixed sound format; loopback following the master volume; the megabytes a minute
on a real scrolling session.

---

# Part 6: the steps

Each step closes on evidence, as every stage of Recon has. The diagnostic flags live behind
`stage0-checks`, use a store and settings of their own beside the executable and never the
user's, and the product build has none of them. Every new action's undo is named in 3.7.
Every step that changes behaviour is medium risk or higher and gets the review of
`project-os/Code_review.md`; S5.7, S5.9 and S5.12 touch files the user believes are kept and
are high. A step with letters is several short sessions, and each letter closes on its own
evidence, so a step is never ticked on part of it.

The first six steps are gates. They build no interface and their only product is evidence.
Any of them may send the plan back to a named fallback; that is what they are for. Files the
gates need are opened in Edge, the same engine as the web view, until S5.5 builds the address
the web view needs. `ffprobe`, where Rotem has it, is a second opinion on his machine only,
never shipped and never linked.

- [ ] **S5.0 Rotem answers C1 to C4.** The others wait for the group named over them. His.
- [ ] **S5.1 Gate: the ground.** (a) Build with the Media Foundation features on and list the
      program's import table. Pass: `mfplat.dll`, `mfreadwrite.dll` and `mf.dll` are not static
      imports; late loading or Recon's own run-time table is decided here, with a Decisions
      entry. (b) A synthetic source under `stage0-checks`: a frame counter, corner markers,
      colour patches, optional noise, any size up to 4096x2304, and 48 kHz sound with a click
      each second in step with a flash, feeding the same convert, encode and write path the
      product will use. It gives exact pixel truth, makes a 1 GB file in minutes with noise,
      and lets S5.4 and S5.5 run without waiting on any capture question. (c) The box walker
      of 3.3. Model: Fable 5.1, it decides how everything after it is written.
- [ ] **S5.2 Gate: frames only.** The capture of 3.2 with no encoder, frames read back as
      PNG: the route runs unpackaged; the border, with the request and without it, with
      Windows' setting on and off, by eye and in the frames; the pointer; Recon's keep-out
      windows absent from the frames and from a GDI copy; the frames delivered a second with
      the update interval left alone, on a moving and on a still screen; the clock's frequency
      and the capture's timestamp against it; one clip with HDR switched on by Rotem. Model:
      Fable 5.1, its evidence decides the capture route.
- [ ] **S5.3 Gate: to a file.** (a) One size from the synthetic source, read back frame by
      frame: marker edges on the exact pixel; flat patches averaged over their inner area
      within 3 levels of 255 per channel; black and white as 0 and 255; thirty frames a second
      of wall time, plus or minus one; both tracks starting at zero. A crop shifted by 1 px and
      a frame converted with the wrong matrix must each FAIL at those same tolerances. (b) A
      captured window that counts for four seconds, freezes for four with the pointer still,
      and counts again: the frozen stretch decodes as identical pictures, counters advance by
      the expected step with repeats and skips printed, and the log says whether the encoder
      took the same texture twice. (c) Sizes: odd, 64x64, 1920x1080, 4096 wide, the scaled
      5120x1440 with markers within 1 px of their computed places, and from the synthetic
      source 4096x2304 capped at twenty-six a second and 2160x3840, each with the encoder's
      name, every setting read back, and profile, level, B-frames, colour tag and time base
      read from the file; the software encoder forced, with its rate at 4096x1152. (d) Load
      on processor and card, frames dropped, and the same scripted minute of real scrolling
      and typing at three quality values, each with its megabytes. It ends with files handed
      to Rotem: the scaled whole screen with 12 px Hebrew and English text beside a still
      capture, and a pixel-for-pixel 1920x1080 one at each quality. He judges the text and
      picks the quality, as F59 was judged, and the number goes into 3.3 and C2. The files
      are opened in Edge, in Windows' player and in one video editor. Model: Fable 5.1, its
      evidence decides the encoding route.
- [ ] **S5.4 Gate: a recording that survives.** Through S5.3's exact pipeline, the same
      device, the hosted hardware encoder, the sound track, once with the writer making the
      fragmented file from the container type and once with the fragmented sink made directly
      and its settings on it. The boxes dumped. The process killed at 2, 10 and 60 s: pass is
      a recovered length of at least the kill time minus one fragment minus one second, the
      counter never backwards, the last counter matching the length, the flash-to-click offset
      unchanged. The rewrite timed on a ten-minute and a sixty-minute file, its times compared
      sample by sample with the working file's, and the process killed DURING the rewrite. A
      noise file past 4 GB. A tail of zeros. It ends with the Decisions entry of 3.6's four
      outcomes, taking S5.3's colour-tag finding as an input. Model: Fable 5.1, it decides
      what "kept" means.
- [ ] **S5.5 Gate: playing and the exact frame.** A throwaway page and the ranged address:
      the content policy; seek and play of a synthetic file over 1 GB, with sound; both
      processes' memory logged over ten minutes of scrubbing, growth between minute 2 and
      minute 10 under a stated figure and not rising with bytes served; the range
      arithmetic's unit tests; the page's frame time against the host's decoded frame against
      the counter in the picture; stepping across a run of repeated frames, where the counter
      stays and the index advances; the still's colour patches against the paused player, at a
      height under 720 and one over; a height that is not a multiple of 16, its last row
      picture and not padding; the time from "this frame" to a still; autoplay measured. One
      S5.3 file and one S5.4 recovered file are replayed here in the web view itself. Model:
      Fable 5.1, it settles the exact frame.
- [ ] **S5.6 Gate: sound measured.** Both streams opened and nothing written: every device's
      format logged and the fixed format's refusal forced once; packets a second in thirty
      seconds of true silence, in the four variants 3.5 names; the worst packet lateness per
      source, which sets the mixer's delay; the drift's slope per source over ten minutes; a
      steady 1 kHz tone's loopback level before and after the microphone opens, where a drop
      of more than 1 dB fails; the level at 100%, at 20% and muted. A 7.1 output and a
      Bluetooth headset, each if he has one, otherwise written down as unverified. Model:
      Fable 5.1, two clocks.
- [ ] **S5.7 The engine.** (a) Start, pause, resume, stop, the pacer, the clock, the
      first-frame timeout proved with a flag that withholds frames. (b) The one way down, each
      failure named as real or injected: a lock (real, Win+L by Rotem), a display change
      (real, a resolution changed and restored by the flag), sleep and display power-off (real,
      by Rotem), a lost device and a dead encoder (injected, and named as injected), a full
      disk (the floor set by flag above the present free space). The recorded area is filled
      by the marker window, and afterwards every frame in the file is read back: a frame
      without the markers FAILS. Stop answering inside its limit with the writer blocked on
      purpose. The display kept awake. (c) Quit and Windows shutting down during a recording;
      recovery at the next start; thirty five-second recordings in one process with no crash
      and memory flat from the fifth to the thirtieth; one sixty-minute recording, its size,
      its drift and its seconds from Stop to a closed file; the Windows N wording and the
      privacy refusal's wording. Host only, driven by flags. Model: Fable 5.1, high risk.
- [ ] **S5.8 Sound in the file.** (a) The mixer on S5.7's clock, the limiter, the corrector.
      The proof: a window flashes white for 100 ms each second while the same thread plays a
      click at the same instant; ten minutes are recorded; the file is read back, the first
      white frame and the click's onset found, a line fitted through all six hundred offsets,
      and its slope, the per-source slopes from the packet log and the count and size of
      re-anchors printed. The corrector is proved with a flag that mis-states one source's
      rate by 500 parts in a million: the offset stays under the threshold with at least one
      re-anchor, and the same run with the corrector switched off must FAIL. A second
      ten-minute run has loopback off and the speaker audible to the microphone, or a loop
      cable, so the microphone's clock is the one judged. Both tracks end within one frame of
      each other after a normal Stop, after a pause and after a failure stop. The run starts
      with thirty seconds of true silence. (b) Device states: a default changed, a device
      pulled (C5), the busy state, no microphone, privacy off. Model: Fable 5.1.
- [ ] **S5.9 The recording as a document.** (a) The store: the new kind, the streamed write,
      the thumbnail and its branch, the frame index, the recovered recording, the disk figure,
      the trash, all of it back after a restart. Compatibility in both directions: the
      previous release started with its data folder pointed at a copy of the check store this
      step wrote, every file's hash the same before and after and the log showing the skipped
      line; and this build run against a store and a trash written by the previous release,
      every document, note, tab and trashed item opening, deleting and un-deleting as before,
      by count and by a hash of each folder. (b) The page: the timeline's badge, the event of
      its own, the selected tab, with a tab other than Main selected Stop shows the recording
      in that tab and in Main and after a restart, the timeline walk onto a recording and off
      it, delete and Ctrl+Z. Driven by recordings the flags make. Closes with a Decisions
      entry. Model: Fable 5.1, high risk.
- [ ] **S5.10 Playing.** The ranged address in the product, Recon's own controls, the keys of
      C15, stepping, "annotate this frame" giving the exact frame as an ordinary document,
      delete while it plays, a playing recording paused when a new one starts. Closes with a
      Decisions entry. Model: Fable 5.1 for the serving and the still, Opus 5 for the controls
      once it serves.
- [ ] **S5.11 Choosing and controlling.** (a) The two shortcuts and the first Settings row of
      a new type, refused when taken like the others. (b) The overlay to the engine, with the
      tray's Record, Pause and Stop as the only controls. (c) The outline and the bar, the
      keep-out flag's answer checked, the window in front after the count being the one that
      was in front at the shortcut, Escape reaching the recorded application. (d) The bar's
      live parts: the microphone's name, the level, the switches as C9 decided, the scaled
      word, the count, Pause, Stop, throwing away as C7 decided, the microphone-off state with
      its button, where a failure's notice shows, and C10, C11, C13 and C14 as decided. The
      bar seen under the sharing tool he uses, or Quick Assist. Every size, colour and
      distance comes from `project-os/Design.md` or from Rotem, none guessed (rule 1). Until
      S5.12 closes, the record shortcut exists only in builds made with the checks. Model:
      Fable 5.1, a new interaction.
- [ ] **S5.12 Getting it out.** Copy as a file inside the clipboard's one transaction, Copy
      and Return, the outbox, its names and its sweep, C16 to C20 as decided, the size shown,
      Save As. A file copied WITHOUT the history flags is first shown to appear in Win+V. The
      clipboard's path read back under a profile folder with Hebrew in it. Copy, delete the
      recording, restart Recon, then paste. A cut and paste tried. Pasted into a folder,
      Slack, ChatGPT, Gmail and whatever else Rotem names, each result written down with the
      date, and the file played back from the target. The copy of a 1 GB file timed. Closes
      with a Decisions entry. Model: Fable 5.1, the clipboard's contract changes.
- [ ] **S5.13 Trim.** Two handles on the scrubber, kept in the document, the player clamped to
      them, the stored file and its address unchanged. The exact cut at Copy and Save As: a
      counter recording trimmed at a time that is not a key frame gives an export whose first
      and last counters are exactly the handles', smaller in bytes, its flash-to-click offset
      within one frame of the original's, with the encoder's name and the time taken printed.
      Ctrl+Z and Ctrl+Shift+Z restore and re-apply both numbers. Model: Fable 5.1, it
      re-encodes kept work.
- [ ] **S5.14 By use, each only at Rotem's word,** in Part 1's order: a smaller copy; a
      microphone picker; the key frames as stills for Claude, at most twenty, cropped to the
      area; click rings (C21) and the pointer's switch; the 16:9 lock and sizes, the same
      area again, restart; a mute key and a warning at one hour; GIF, sixty a second, shown
      keystrokes, separate tracks; following a window; the window-exclusion list; the wider
      pool and tone map for HDR. Model: named when one is taken up.
- [ ] **S5.15 The README's known limits,** written from 3.2, 3.3, 3.5 and 3.10, **and a week
      of real recordings,** the friction written to `project-os/History.md` and anything
      deferred to the Backlog at his word. Model: Opus 5, it records.

**The order, and why.** The gates come first because the foundation questions have no source
anywhere and each can change the design; the synthetic source is built first so that no gate
waits on another's answer. Sound is measured before the engine is built, because the mixer's
delay, the cut at Stop and the re-base on pause are part of the engine's clock, and measuring
needs no file and no engine. The document and the player come before the interface because
flags can drive them, and because a shortcut that works while a finished recording has
nowhere to go is a broken product. The cost is that nothing can be used by hand until S5.11,
and nothing reaches the product build until S5.12.

**Depends on:** the store (S1.8), the timeline and its tabs, Settings and its shortcuts, all
built. When it starts relative to the daily-use release and Stage 3 is Rotem's word.

---

# Part 7: sources

Getting the frames
- [CreateForMonitor](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createformonitor) · the capture item from an ordinary program.
- [Direct3D11CaptureFramePool](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframepool) · the free-threaded pool.
- [IsBorderRequired](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturesession.isborderrequired) · the border switch, its build, and the consent call it describes.
- [robmikh/Win32CaptureSample issues](https://github.com/robmikh/Win32CaptureSample/issues) · the API owner's answers: 34 and 104 the software pointer, 35 the lock screen, 59 and 97 display changes, 61 cloned textures, 80 frames on a still screen before 24H2, 92 the default rate, 99 the teardown crash, 100 remote sessions, 103 the 25H2 bug, 106 the exclusion list, 107 access denied.
- [robmikh/displayrecorder](https://github.com/robmikh/displayrecorder) · a Rust display recorder on this route, MIT.
- [PowerToys ZoomIt](https://github.com/microsoft/PowerToys/tree/main/src/modules/ZoomIt/ZoomIt) · Microsoft's own recorder: the borderless note, the first-frame timeout, the negative timestamp.
- [WebRTC's capture session](https://webrtc.googlesource.com/src/+/refs/heads/main/modules/desktop_capture/win/wgc_capture_session.cc) · still frames skipped on 24H2.
- [SetWindowDisplayAffinity](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity), [the layered-window failure](https://github.com/microsoft/Windows.UI.Composition-Win32-Samples/issues/56) and [Tauri issue 14189](https://github.com/tauri-apps/tauri/issues/14189) · keeping Recon's own windows out.
- [Privacy policies](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy) · programmatic capture and the border, as policies.

Encoding
- [H.264 Video Encoder](https://learn.microsoft.com/en-us/windows/win32/medfound/h-264-video-encoder) and [H.264 Video Decoder](https://learn.microsoft.com/en-us/windows/win32/medfound/h-264-video-decoder) · inputs, rate control, the hardware encoder taking over, the decoder's ceiling.
- [MPEG-4 File Sink](https://learn.microsoft.com/en-us/windows/win32/medfound/mpeg-4-file-sink) · one picture stream, one sound stream, no edit lists, fragments, 4 GB.
- [Chromium's Media Foundation encoder](https://raw.githubusercontent.com/chromium/chromium/main/media/gpu/windows/media_foundation_video_encode_accelerator_win.cc) · the NVIDIA flush, colour written as unspecified.
- [Chromium's colour guess](https://raw.githubusercontent.com/chromium/chromium/main/media/ffmpeg/ffmpeg_common.cc) and [mpv's](https://raw.githubusercontent.com/mpv-player/mpv/master/video/csputils.c) · why an untagged file changes colour between players.
- [Automatic processing on the video processor](https://learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11videocontext-videoprocessorsetstreamautoprocessingmode), [MFTEnum2](https://learn.microsoft.com/en-us/windows/win32/api/mfapi/nf-mfapi-mftenum2), [the first-frame offset](https://learn.microsoft.com/en-gb/answers/questions/2117723/mediafoundation-timestamp-of-first-frame-is-not-ge), [NVIDIA's encoder and textures](https://alax.info/blog/2114).
- [Media Feature Pack and the missing DLL](https://learn.microsoft.com/en-us/answers/questions/5915168/mfplat-dll-missing-install-media-feature-pack) · Windows N.

The file
- [MFCreateFMPEG4MediaSink](https://learn.microsoft.com/en-us/windows/win32/api/mfidl/nf-mfidl-mfcreatefmpeg4mediasink) · the fragmented writer.
- [MF_SINK_WRITER_DISABLE_THROTTLING](https://learn.microsoft.com/en-us/windows/win32/medfound/mf-sink-writer-disable-throttling) and [GetServiceForStream](https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-imfsinkwriter-getserviceforstream) · the offline rewrite.
- [FFmpeg's hybrid fragmented mode](https://ffmpeg.org/ffmpeg-formats.html) and OBS's hybrid MP4 · the same pattern, prior practice.
- [Writer statistics](https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/ns-mfreadwrite-mf_sink_writer_statistics), [ShutdownBlockReasonCreate](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-shutdownblockreasoncreate), [protected folders](https://learn.microsoft.com/en-us/defender-endpoint/controlled-folders).

Sound
- [AAC Encoder](https://learn.microsoft.com/en-us/windows/win32/medfound/aac-encoder) · its input rules and the zero-length crash.
- [The communications device](https://learn.microsoft.com/en-us/windows/win32/coreaudio/using-the-communication-device) and [stream attenuation](https://learn.microsoft.com/en-us/windows/win32/coreaudio/stream-attenuation) · ducking.
- [Recovering from an invalid device](https://learn.microsoft.com/en-us/windows/win32/coreaudio/recovering-from-an-invalid-device-error), [IMMNotificationClient](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nn-mmdeviceapi-immnotificationclient), [Bluetooth audio on Windows](https://learn.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-classic-audio).
- [HEnquist/wasapi-rs](https://github.com/HEnquist/wasapi-rs) · a small Rust reference for the capture half, MIT.

Playing
- [Working with local content in WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/working-with-local-content), [WebView2Feedback 2679](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2679) and [3230](https://github.com/MicrosoftEdge/WebView2Feedback/issues/3230) · exact slices, and ranged video on an http address only.
- [Chromium's media cache](https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/platform/media/url_index.cc) and [its renderer's first-frame rule](https://raw.githubusercontent.com/chromium/chromium/main/media/renderers/video_renderer_impl.cc).
- [The source reader's advanced processing](https://learn.microsoft.com/en-us/windows/win32/medfound/mf-source-reader-enable-advanced-video-processing) · why the still's conversion is Recon's own.

Getting it out
- [Clipboard formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats) · the file format and the history flags.
- [CreateHardLinkW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createhardlinkw) · why the outbox holds a copy.
- [Claude's vision limits](https://platform.claude.com/docs/en/build-with-claude/vision), [GitHub's attachment limits](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files), [Notion's](https://www.notion.com/help/images-files-and-media).
- [crabnebula-dev/drag-rs](https://github.com/crabnebula-dev/drag-rs) · dragging a file out of a Tauri window, for later.
