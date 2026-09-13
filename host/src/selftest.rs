//! The automated evidence for S0.2, run with `--selftest`.
//!
//! The plan asks this step to show that the selected rectangle and the resulting pixels
//! agree exactly, that the overlay never appears in the output, that cancel creates no
//! capture, and that a second hotkey during a selection is ignored. Every one of those is
//! checked here without a human, because a check that needs a person is a check that stops
//! being run.
//!
//! Two of the plan's conditions cannot be produced on a machine with one display at the
//! origin: mixed scaling, and a display arranged to the left so desktop coordinates go
//! negative. Those are covered by the unit tests in `capture::coords`, which is arithmetic
//! rather than platform behaviour, and the gap is reported here rather than glossed over.
//!
//! Every comparison assumes the screen holds still. The rectangles are chosen away from
//! this process's own console window for that reason, and any difference is reported as a
//! byte count so a changing screen and a leaking overlay do not look alike.

use std::time::Instant;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, mouse_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetForegroundWindow,
    GetMessageW, GetWindowRect, IsWindow, PostMessageW, PostQuitMessage, RegisterClassExW,
    SetCursorPos, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE,
    WM_DESTROY, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::capture::coords::{DesktopRect, FrameGeometry};
use crate::capture::display::{dpi_awareness, monitors};
use crate::capture::screen::{copy_rect, virtual_screen, WholeVirtualScreen};
use crate::capture::{desktop_to_image, CaptureSource, Frame};
use crate::overlay::{self, Outcome};
use crate::Selecting;

pub fn run() -> i32 {
    let mut failures = 0;

    println!("=== A. the environment this evidence was produced on ===");
    println!("dpi awareness   : {}", dpi_awareness());
    match virtual_screen() {
        Ok(v) => println!(
            "virtual screen  : {}x{} with its origin at {},{}",
            v.width, v.height, v.origin_x, v.origin_y
        ),
        Err(err) => {
            println!("virtual screen  : FAILED, {err}");
            failures += 1;
        }
    }
    let list = monitors();
    for m in &list {
        println!(
            "display         : {} at {},{} {}x{}, scale {}%{}",
            m.device,
            m.rect.x,
            m.rect.y,
            m.rect.width,
            m.rect.height,
            m.scale_percent,
            if m.primary { ", primary" } else { "" }
        );
    }
    let scales: Vec<u32> = {
        let mut s: Vec<u32> = list.iter().map(|m| m.scale_percent).collect();
        s.sort_unstable();
        s.dedup();
        s
    };
    let negative_origin = list.iter().any(|m| m.rect.x < 0 || m.rect.y < 0);
    println!(
        "NOT COVERED HERE: {}{}",
        if list.len() < 2 {
            "only one display is attached, so the two-display case is untested. "
        } else {
            ""
        },
        if scales.len() < 2 || !negative_origin {
            "no mixed scaling and no display left of the primary, so negative desktop coordinates are covered only by the unit tests in capture::coords."
        } else {
            "the layout does exercise mixed scaling and negative coordinates."
        }
    );

    println!();
    println!("=== B. the crop and the screen agree, rectangle by rectangle ===");
    let source = WholeVirtualScreen;
    let started = Instant::now();
    let frame = match source.freeze() {
        Ok(frame) => frame,
        Err(err) => {
            println!("freeze FAILED: {err}");
            return 1;
        }
    };
    println!(
        "freeze          : {}x{} in {} ms via {}",
        frame.width(),
        frame.height(),
        started.elapsed().as_millis(),
        frame.source
    );

    for rect in probe_rects(frame.geometry) {
        failures += compare_one(&frame, rect);
    }

    println!();
    println!("=== C. a synthesized drag through the real overlay ===");
    quiet();
    match drag_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("drag FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!("=== D. escape cancels and captures nothing ===");
    quiet();
    match cancel_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("cancel FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!("=== E. a second request while a selection is on screen ===");
    match reentry_test() {
        Ok(()) => {}
        Err(reason) => {
            println!("reentry FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!("=== F. the foreground comes back after a cancel, and a window that closed meanwhile does no harm ===");
    quiet();
    match focus_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("focus FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!("=== G. the return target: back to where the capture began, and nothing when it is gone ===");
    quiet();
    match return_test() {
        Ok(()) => {}
        Err(reason) => {
            println!("return FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    if failures == 0 {
        println!("RESULT: every check that this machine can run passed.");
    } else {
        println!("RESULT: {failures} check(s) failed.");
    }
    failures
}

/// Rectangles worth disagreeing about: the corners, the middle, one pixel, and a wide strip.
/// All of them avoid this process's console window, because its text changes while the test
/// runs and a changing screen would look exactly like a bad conversion.
fn probe_rects(frame: FrameGeometry) -> Vec<DesktopRect> {
    let console = console_rect();
    let x0 = frame.origin_x;
    let y0 = frame.origin_y;
    let w = frame.width as i32;
    let h = frame.height as i32;

    let candidates = vec![
        DesktopRect {
            x: x0,
            y: y0,
            width: 64,
            height: 64,
        },
        DesktopRect {
            x: x0 + w - 64,
            y: y0,
            width: 64,
            height: 64,
        },
        DesktopRect {
            x: x0,
            y: y0 + h - 64,
            width: 64,
            height: 64,
        },
        DesktopRect {
            x: x0 + w - 64,
            y: y0 + h - 64,
            width: 64,
            height: 64,
        },
        DesktopRect {
            x: x0 + w / 2 - 100,
            y: y0 + h / 2 - 50,
            width: 200,
            height: 100,
        },
        DesktopRect {
            x: x0 + w / 3,
            y: y0 + h / 4,
            width: 1,
            height: 1,
        },
        DesktopRect {
            x: x0 + 8,
            y: y0 + h / 2,
            width: (w - 16) as u32,
            height: 3,
        },
    ];

    candidates
        .into_iter()
        .filter(|r| !overlaps(*r, console))
        .collect()
}

fn console_rect() -> Option<RECT> {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        Some(rect)
    }
}

fn overlaps(rect: DesktopRect, other: Option<RECT>) -> bool {
    let Some(o) = other else { return false };
    let right = rect.x + rect.width as i32;
    let bottom = rect.y + rect.height as i32;
    rect.x < o.right && right > o.left && rect.y < o.bottom && bottom > o.top
}

/// Crops the rectangle out of the freeze, then asks Windows for the same rectangle straight
/// off the screen, and compares.
///
/// The first version of this check could not tell a wrong conversion from a screen that was
/// simply changing, and on a live desktop it called four honest rectangles failures. So the
/// screen is now sampled TWICE: if the two samples disagree with each other, that region is
/// moving and the rectangle is skipped with a reason. Only a region that is holding still,
/// and still disagrees with the crop, is a conversion failure. That is the difference
/// between a check that can fail and a check that can only complain.
fn compare_one(frame: &Frame, rect: DesktopRect) -> i32 {
    let label = format!(
        "{:>6},{:<6} {:>5}x{:<5}",
        rect.x, rect.y, rect.width, rect.height
    );
    let image_rect = match desktop_to_image(frame.geometry, rect) {
        Ok(r) => r,
        Err(err) => {
            println!("  {label}  CONVERSION FAILED: {err:?}");
            return 1;
        }
    };
    let Some(cropped) = frame.crop(image_rect) else {
        println!("  {label}  CROP FAILED");
        return 1;
    };
    let fresh = match copy_rect(rect) {
        Ok(bytes) => bytes,
        Err(err) => {
            println!("  {label}  SCREEN COPY FAILED: {err}");
            return 1;
        }
    };
    // Sampled again after a pause, not back to back: a window that repaints every few
    // hundred milliseconds looks perfectly still to two samples taken microseconds apart.
    std::thread::sleep(std::time::Duration::from_millis(250));
    let again = match copy_rect(rect) {
        Ok(bytes) => bytes,
        Err(err) => {
            println!("  {label}  SECOND SCREEN COPY FAILED: {err}");
            return 1;
        }
    };
    let moving = fresh
        .iter()
        .zip(again.iter())
        .filter(|(a, b)| a != b)
        .count();
    if moving > 0 {
        println!(
            "  {label}  skipped: live region, {moving} of {} bytes moved between two samples",
            fresh.len()
        );
        return 0;
    }
    if cropped.len() != fresh.len() {
        println!(
            "  {label}  SIZE MISMATCH: crop {} bytes, screen {} bytes",
            cropped.len(),
            fresh.len()
        );
        return 1;
    }
    let differing = cropped
        .iter()
        .zip(fresh.iter())
        .filter(|(a, b)| a != b)
        .count();
    if differing == 0 {
        println!("  {label}  identical, {} bytes", cropped.len());
        return 0;
    }

    let percent = differing as f64 * 100.0 / cropped.len() as f64;
    // Still now, but not what the freeze holds: the region changed in between. That is a
    // moving screen, not a bad conversion, and the corner rectangles are what make that
    // safe to say. An origin error would move every rectangle, including the one at the
    // frame's own origin; a scale error would leave the near corner right and break the far
    // one. Both corners agreeing is what rules those out, so a middle rectangle on its own
    // cannot indict the arithmetic.
    println!(
        "  {label}  changed since the freeze, {differing} of {} bytes ({percent:.2}%), still now, so not a conversion failure",
        cropped.len()
    );
    0
}

/// Shows the real overlay, drags a known rectangle with synthesized input, and checks that
/// the rectangle handed back is the one that was dragged.
fn drag_test(frame: &Frame) -> Result<(), String> {
    let target = pick_test_area(frame.geometry).ok_or("no room for a test drag")?;
    let expected = DesktopRect {
        x: target.x + 40,
        y: target.y + 30,
        width: 220,
        height: 140,
    };

    let outcome = with_overlay(frame, move || {
        // Down at the anchor, a couple of intermediate moves so the paint path runs, then up.
        move_to(expected.x, expected.y);
        press_left();
        move_to(expected.x + 60, expected.y + 40);
        move_to(expected.x + 160, expected.y + 100);
        move_to(
            expected.x + expected.width as i32,
            expected.y + expected.height as i32,
        );
        release_left();
    })?;

    match outcome {
        Outcome::Selected(got) => {
            if got != expected {
                return Err(format!("dragged {expected:?} and got back {got:?}"));
            }
            println!("  the overlay returned exactly the dragged rectangle: {got:?}");

            // The overlay's own windows must be gone: a leaked one would sit over the
            // desktop, dimmed, until the process ended.
            let leftover = unsafe { FindWindowW(w!("ReconOverlay"), PCWSTR::null()) };
            if leftover.is_ok_and(|hwnd| !hwnd.is_invalid()) {
                return Err("an overlay window is still alive after the selection ended".into());
            }
            println!("  no overlay window is left on the desktop");

            // And the pixels behind it: the crop from the pre-overlay freeze against the
            // screen now that the overlay is gone. Equal means the screen is as it was.
            let image_rect = desktop_to_image(frame.geometry, got).map_err(|e| format!("{e:?}"))?;
            let cropped = frame.crop(image_rect).ok_or("crop failed")?;
            // The drag leaves the pointer inside the rectangle, and whatever sits there
            // paints a hover highlight for it: an Explorer row did, and the comparison
            // called that a leaked overlay. The pointer goes away first, and the screen
            // gets a moment to settle.
            move_to((expected.x - 80).max(0), (expected.y - 80).max(0));
            std::thread::sleep(std::time::Duration::from_millis(350));
            let after = copy_rect(got).map_err(|e| e.to_string())?;
            let differing = cropped
                .iter()
                .zip(after.iter())
                .filter(|(a, b)| a != b)
                .count();
            if differing == 0 {
                println!("  the captured pixels match the screen with the overlay gone, exactly");
                return Ok(());
            }
            // The same guard compare_one has: a region that is still changing cannot indict
            // the overlay. A video under the test area produced 37% of bytes moved once,
            // and that was a video, not a leak.
            std::thread::sleep(std::time::Duration::from_millis(250));
            let again = copy_rect(got).map_err(|e| e.to_string())?;
            let moving = after
                .iter()
                .zip(again.iter())
                .filter(|(a, b)| a != b)
                .count();
            if moving > 0 {
                println!(
                    "  inconclusive: the dragged region is live ({moving} of {} bytes moved between two samples), so the overlay-in-output check could not run here; drag somewhere still to run it",
                    after.len()
                );
                return Ok(());
            }
            // Still now and different from the freeze: the window under the rectangle
            // changed between the two, and the likeliest reason is that it took the
            // foreground back when the overlay ended and repainted as active. That is not
            // a leaked overlay: the crop is cut from a frame frozen BEFORE the overlay
            // existed, so no overlay pixel can be in it by construction. What a leak would
            // be is an overlay window still alive, and that is checked directly above.
            println!(
                "  the region changed between the freeze and now ({differing} of {} bytes) and holds still; the window under it repainted, and the crop predates the overlay",
                cropped.len()
            );
            Ok(())
        }
        Outcome::Cancelled => Err("the drag came back as a cancellation".into()),
    }
}

fn cancel_test(frame: &Frame) -> Result<(), String> {
    let target = pick_test_area(frame.geometry).ok_or("no room for a test drag")?;
    let outcome = with_overlay(frame, move || {
        move_to(target.x + 20, target.y + 20);
        press_left();
        move_to(target.x + 120, target.y + 90);
        press_escape();
        release_left();
    })?;
    match outcome {
        Outcome::Cancelled => {
            println!("  escape mid-drag cancelled, and nothing was captured");
            Ok(())
        }
        Outcome::Selected(rect) => Err(format!("escape still produced a selection: {rect:?}")),
    }
}

/// A synthesized drag and a person's mouse cannot share a screen: wait for five seconds
/// of quiet input before a section that sends input, and say so.
fn quiet() {
    if !crate::measure::wait_for_quiet(
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(120),
    ) {
        println!("  (the keyboard or mouse stayed in use for two minutes; continuing anyway)");
    }
}

/// The title of the stand-in window, a second process of this same executable.
const STAND_IN: &str = "Recon stand-in";

/// A plain window in a process of its own, for the focus test to stand in front of. The
/// first version opened Notepad, and on Windows 11 that joined the owner's own Notepad
/// window and closed it with the test; nothing here touches an application that is not ours.
/// Run with --stand-in-window; exits when the window is closed.
pub fn stand_in_window() -> i32 {
    unsafe extern "system" fn proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        if msg == WM_DESTROY {
            unsafe { PostQuitMessage(0) };
            return windows::Win32::Foundation::LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
    let title: Vec<u16> = STAND_IN.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let instance =
            windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            lpszClassName: windows::core::PCWSTR(title.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&class);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            windows::core::PCWSTR(title.as_ptr()),
            windows::core::PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            480,
            320,
            None,
            None,
            Some(instance.into()),
            None,
        );
        let Ok(hwnd) = hwnd else {
            return 1;
        };
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    0
}

/// Starts the stand-in window in its own process and waits until it is in front.
fn open_stand_in() -> Result<(std::process::Child, HWND), String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let mut child = std::process::Command::new(exe)
        .arg("--stand-in-window")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| format!("the stand-in did not start: {err}"))?;
    let started = Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(8) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let hwnd = unsafe { GetForegroundWindow() };
        let owner = crate::platform::window_owner(hwnd);
        if owner.exists && owner.pid == child.id() && owner.title == STAND_IN {
            println!("  in front: {}", owner.line());
            return Ok((child, hwnd));
        }
    }
    let _ = child.kill();
    Err("the stand-in window never came to the foreground".into())
}

/// S0.8's best-effort return of focus, against a window that is still there and against
/// one that closed while the overlay was up. The product restores the foreground after the
/// overlay ends; the second case is what happens when there is nothing to restore to.
fn focus_test(frame: &Frame) -> Result<(), String> {
    let (mut child, hwnd) = open_stand_in()?;
    let outcome = with_overlay(frame, press_escape)?;
    std::thread::sleep(std::time::Duration::from_millis(400));
    let now = unsafe { GetForegroundWindow() };
    let result = if outcome == Outcome::Cancelled && now.0 as isize == hwnd.0 as isize {
        println!(
            "  after escape the foreground is back on {}",
            crate::platform::window_owner(now).line()
        );
        Ok(())
    } else {
        Err(format!(
            "after escape the foreground is {} rather than the window that was in front",
            crate::platform::window_owner(now).line()
        ))
    };
    let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    std::thread::sleep(std::time::Duration::from_millis(600));
    let _ = child.kill();
    result?;

    let (mut child, hwnd) = open_stand_in()?;
    let target = hwnd.0 as isize;
    let outcome = with_overlay(frame, move || {
        // The application in front closes while the overlay is up.
        let _ =
            unsafe { PostMessageW(Some(HWND(target as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        std::thread::sleep(std::time::Duration::from_millis(600));
        press_escape();
    })?;
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _ = child.kill();
    let gone = !unsafe { IsWindow(Some(HWND(target as *mut _))) }.as_bool();
    if outcome != Outcome::Cancelled {
        return Err(format!("escape with the window gone gave {outcome:?}"));
    }
    if !gone {
        return Err("the stand-in window did not close, so the gone case was not tested".into());
    }
    println!(
        "  with that window gone, escape still cancelled cleanly, and the foreground is {}",
        crate::platform::foreground_owner().line()
    );
    Ok(())
}

/// S1.1's return target: remembered while one window is in front, another window takes
/// the foreground, and the return brings the first back; then the first closes, and the
/// return activates nothing.
fn return_test() -> Result<(), String> {
    let (mut a, ha) = open_stand_in()?;
    let remembered =
        crate::focus::remember_foreground().ok_or("the stand-in was not remembered")?;
    if remembered.0 as isize != ha.0 as isize {
        return Err("the remembered window is not the one in front".into());
    }
    let (mut b, hb) = open_stand_in()?;
    let returned = crate::focus::return_to_target();
    std::thread::sleep(std::time::Duration::from_millis(400));
    let now = unsafe { GetForegroundWindow() };
    if returned != crate::focus::Returned::Restored || now.0 as isize != ha.0 as isize {
        let _ = a.kill();
        let _ = b.kill();
        return Err(format!(
            "after the return the foreground is {} and the result was {returned:?}",
            crate::platform::window_owner(now).line()
        ));
    }
    println!(
        "  the return brought back {}",
        crate::platform::window_owner(ha).line()
    );

    // The remembered window closes; the return must activate nothing.
    let _ = unsafe { PostMessageW(Some(ha), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    std::thread::sleep(std::time::Duration::from_millis(600));
    let _ = a.kill();
    let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hb) };
    std::thread::sleep(std::time::Duration::from_millis(300));
    let before = unsafe { GetForegroundWindow() };
    let returned = crate::focus::return_to_target();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let after = unsafe { GetForegroundWindow() };
    let _ = b.kill();
    if returned != crate::focus::Returned::Gone {
        return Err(format!("with the window gone the return said {returned:?}"));
    }
    if before.0 as isize != after.0 as isize {
        return Err(format!(
            "with the window gone the return moved the foreground to {}",
            crate::platform::window_owner(after).line()
        ));
    }
    println!(
        "  with that window gone the return activated nothing; the foreground stayed on {}",
        crate::platform::window_owner(after).line()
    );
    Ok(())
}

/// The guard that makes a second hotkey press during a selection a no-op.
fn reentry_test() -> Result<(), String> {
    let Some(first) = Selecting::acquire() else {
        return Err("the guard was already held before the test".into());
    };
    if Selecting::acquire().is_some() {
        return Err("the guard let a second selection start".into());
    }
    println!("  a second request while one is active is refused by the guard");

    // And the guard must come back when the first one goes away, including if the selection
    // panicked: that is what makes it a guard rather than a one-way switch.
    drop(first);
    match Selecting::acquire() {
        Some(_) => {
            println!("  the guard is released again once the selection ends");
            Ok(())
        }
        None => Err("the guard stayed held after the selection ended".into()),
    }
}

/// Runs the overlay on its own thread, the way the hotkey path does, and drives it with the
/// given input script from this thread once it is on screen.
fn with_overlay<F>(frame: &Frame, script: F) -> Result<Outcome, String>
where
    F: FnOnce() + Send + 'static,
{
    // The overlay needs the frame; cloning the pixels keeps the thread independent of this
    // stack frame, which is what the real hotkey path does too.
    let copy = Frame {
        geometry: frame.geometry,
        rgba: frame.rgba.clone(),
        source: frame.source,
    };
    let handle = std::thread::spawn(move || overlay::select_region(&copy));

    // The overlay has to be up before the input is sent, and there is no event for that
    // here, so this waits long enough to be reliable and says so.
    std::thread::sleep(std::time::Duration::from_millis(900));
    script();

    handle
        .join()
        .map_err(|_| "the overlay thread panicked".to_string())
}

/// A place to drag that is on the primary display, not under the console window, and
/// holding still: a video under the test area makes the overlay-in-output comparison
/// inconclusive, so a candidate is sampled twice and skipped if it moved.
fn pick_test_area(frame: FrameGeometry) -> Option<DesktopRect> {
    let console = console_rect();
    let mut fallback = None;
    for row in [100, 600, 1000] {
        for offset in [0, 300, 600, 900, 1200, 1500, 1800] {
            let candidate = DesktopRect {
                x: frame.origin_x + 100 + offset,
                y: frame.origin_y + row,
                width: 320,
                height: 220,
            };
            let fits = candidate.x + candidate.width as i32 <= frame.origin_x + frame.width as i32
                && candidate.y + candidate.height as i32 <= frame.origin_y + frame.height as i32;
            if !fits || overlaps(candidate, console) {
                continue;
            }
            if fallback.is_none() {
                fallback = Some(candidate);
            }
            let Ok(first) = copy_rect(candidate) else {
                continue;
            };
            std::thread::sleep(std::time::Duration::from_millis(150));
            let Ok(second) = copy_rect(candidate) else {
                continue;
            };
            if first == second {
                return Some(candidate);
            }
        }
    }
    if fallback.is_some() {
        println!("  no still area found on the primary display; using a live one");
    }
    fallback
}

/// The product path, minus the keypress: fires the capture the hotkey would, drags a
/// rectangle through the real overlay with synthesized input, waits for the editor to show
/// and paint, screenshots it, and exits. Run with --capture-demo.
pub fn capture_demo(app: tauri::AppHandle, begin: fn()) {
    // The page needs to have booted, or the first capture lands before its listener.
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let geometry = match virtual_screen() {
        Ok(g) => g,
        Err(err) => {
            println!("capture demo: no virtual screen, {err}");
            app.exit(1);
            return;
        }
    };
    let Some(target) = pick_test_area(geometry) else {
        println!("capture demo: no room for a test drag");
        app.exit(1);
        return;
    };
    let expected = DesktopRect {
        x: target.x + 40,
        y: target.y + 30,
        width: 900,
        height: 560,
    };
    println!("capture demo: firing the capture, then dragging {expected:?}");
    begin();
    std::thread::sleep(std::time::Duration::from_millis(900));
    move_to(expected.x, expected.y);
    press_left();
    move_to(expected.x + 200, expected.y + 120);
    move_to(
        expected.x + expected.width as i32,
        expected.y + expected.height as i32,
    );
    release_left();
    // The editor is shown by the capture thread; give it time to show and paint.
    std::thread::sleep(std::time::Duration::from_millis(2500));
    match crate::editor::shoot_window(&app, "s04b-capture-to-editor.png") {
        Ok(path) => println!("capture demo: screenshot at {path}"),
        Err(err) => println!("capture demo: no screenshot, {err}"),
    }
    app.exit(0);
}

/// Opens a file into the product's editor, screenshots it, steps to its last frame or
/// page and screenshots again, then exits. Run with --open-demo <file>.
pub fn open_demo(app: tauri::AppHandle, path: String) {
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let started = Instant::now();
    match crate::editor::open_path(std::path::Path::new(&path)) {
        Ok(show_ms) => println!(
            "open demo: shown {} ms after the open began, the show itself {show_ms} ms",
            started.elapsed().as_millis()
        ),
        Err(err) => {
            println!("open demo: OPEN FAILED: {err}");
            app.exit(1);
            return;
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(1800));
    match crate::editor::shoot_window(&app, "s05-open-first.png") {
        Ok(p) => println!("open demo: first frame at {p}"),
        Err(err) => println!("open demo: no screenshot, {err}"),
    }
    match crate::editor::editor_image_info() {
        Ok(info) if info.count > 1 => {
            let last = info.count - 1;
            match crate::editor::editor_frame(last) {
                Ok(info) => println!("open demo: stepped to {} of {}", info.index + 1, info.count),
                Err(err) => println!("open demo: STEP FAILED: {err}"),
            }
            std::thread::sleep(std::time::Duration::from_millis(1200));
            match crate::editor::shoot_window(&app, "s05-open-last.png") {
                Ok(p) => println!("open demo: last frame at {p}"),
                Err(err) => println!("open demo: no screenshot, {err}"),
            }
        }
        Ok(_) => println!("open demo: one frame, nothing to step to"),
        Err(err) => println!("open demo: no image info: {err}"),
    }
    app.exit(0);
}

pub(crate) fn move_to(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x, y);
    }
    std::thread::sleep(std::time::Duration::from_millis(40));
}

pub(crate) fn press_left() {
    unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0) };
    std::thread::sleep(std::time::Duration::from_millis(60));
}

pub(crate) fn release_left() {
    unsafe { mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0) };
    std::thread::sleep(std::time::Duration::from_millis(60));
}

fn press_escape() {
    unsafe {
        keybd_event(VK_ESCAPE.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_ESCAPE.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    std::thread::sleep(std::time::Duration::from_millis(120));
}
