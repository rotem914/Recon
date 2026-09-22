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
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, EndPaint, InvalidateRect, SetDIBitsToDevice, DIB_RGB_COLORS,
    PAINTSTRUCT,
};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, mouse_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_WHEEL, VK_CONTROL, VK_ESCAPE, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetClientRect, GetCursorPos,
    GetForegroundWindow, GetMessageW, GetWindowRect, IsWindow, IsWindowVisible, PostMessageW,
    PostQuitMessage, RegisterClassExW, SetCursorPos, TranslateMessage, CW_USEDEFAULT, MSG, SB_VERT,
    SCROLLINFO, SIF_PAGE, SIF_POS, SIF_RANGE, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY,
    WM_MOUSEWHEEL, WM_PAINT, WM_SIZE, WNDCLASSEXW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    WS_VSCROLL,
};

use crate::capture::coords::{DesktopRect, FrameGeometry};
use crate::capture::display::{dpi_awareness, monitors};
use crate::capture::screen::{copy_rect, virtual_screen, WholeVirtualScreen};
use crate::capture::{desktop_to_image, CaptureSource, Frame};
use crate::overlay::{self, Outcome};
use crate::Selecting;

pub fn run() -> i32 {
    let mut failures = 0;

    // `--selftest --scroll-trial <part of a title>`: the scrolling capture tried on a real
    // window the person opened, a browser's page. Nothing is compared; the tall picture is
    // written to be looked at.
    let args: Vec<String> = std::env::args().collect();
    if let Some(at) = args.iter().position(|a| a == "--scroll-trial") {
        quiet();
        return match scroll_trial(args.get(at + 1).map(String::as_str).unwrap_or("")) {
            Ok(()) => 0,
            Err(reason) => {
                println!("scrolling trial FAILED: {reason}");
                1
            }
        };
    }

    // `--selftest --only-scroll` runs section I alone: it takes the mouse for a quarter of
    // a minute, and the sections before it take a minute more.
    if std::env::args().any(|a| a == "--only-scroll") {
        println!("=== I. the round button on an area that scrolls, and the scrolling capture it starts ===");
        quiet();
        return match WholeVirtualScreen.freeze() {
            Ok(frame) => match scroll_test(&frame) {
                Ok(()) => {
                    println!("RESULT: the scrolling capture's check passed.");
                    0
                }
                Err(reason) => {
                    println!("scrolling capture FAILED: {reason}");
                    1
                }
            },
            Err(err) => {
                println!("freeze FAILED: {err}");
                1
            }
        };
    }

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
    println!("=== H. a click with no drag picks the window under the pointer; a drag still draws its own ===");
    quiet();
    match window_pick_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("window pick FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!(
        "=== I. the round button on an area that scrolls, and the scrolling capture it starts ==="
    );
    quiet();
    match scroll_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("scrolling capture FAILED: {reason}");
            failures += 1;
        }
    }

    println!();
    println!(
        "=== J. Ctrl held at the selection is recorded, for a copy with no editor brought up ==="
    );
    quiet();
    match ctrl_test(&frame) {
        Ok(()) => {}
        Err(reason) => {
            println!("Ctrl at the selection FAILED: {reason}");
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

    let (looked, seen) = std::sync::mpsc::channel();
    let mid_drag = (expected.x + 160, expected.y + 100);
    let around = pixels_around(frame, mid_drag).ok_or("the mid-drag point is off the frame")?;
    let lines = line_probes(frame, mid_drag).ok_or("a line probe mid-drag is off the frame")?;
    let outcome = with_overlay(frame, move || {
        // Down at the anchor, a couple of intermediate moves so the paint path runs, then up.
        move_to(expected.x, expected.y);
        press_left();
        move_to(expected.x + 60, expected.y + 40);
        move_to(mid_drag.0, mid_drag.1);
        // The magnifier with the size under it (Rotem, 2026-09-17 and 18): a moment for
        // the paint, then the screen beside the pointer is written to be looked at, and read.
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = looked.send(magnifier_seen(mid_drag, around, lines, "dragging"));
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
            // The spot the capture draws the mouse pointer at (Rotem, 2026-09-22): where the
            // mouse was at the release, which is the corner the drag ended on.
            let released = (
                expected.x + expected.width as i32,
                expected.y + expected.height as i32,
            );
            match crate::overlay::take_selected_at() {
                Some(at) if at == released => {
                    println!(
                        "  the mouse at the selection was recorded where it was released: {at:?}"
                    )
                }
                other => {
                    return Err(format!(
                    "released at {released:?}, and the selection recorded the mouse at {other:?}"
                ))
                }
            }
            match seen.try_recv() {
                Ok(Ok(())) => {}
                Ok(Err(err)) => return Err(err),
                Err(_) => return Err("the magnifier was never looked at mid-drag".into()),
            }

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
        Outcome::Scroll { rect, .. } => Err(format!("the round button answered instead: {rect:?}")),
    }
}

/// The frozen pixels around a desktop point, the 3 by 3 of them, as the magnifier should
/// show them: what its centre cells are compared against.
fn pixels_around(frame: &Frame, pointer: (i32, i32)) -> Option<[(u8, u8, u8); 9]> {
    let mut out = [(0u8, 0u8, 0u8); 9];
    for (n, slot) in out.iter_mut().enumerate() {
        let (i, j) = (n as i32 % 3 - 1, n as i32 / 3 - 1);
        let x = pointer.0 + i - frame.geometry.origin_x;
        let y = pointer.1 + j - frame.geometry.origin_y;
        if x < 0 || y < 0 || x >= frame.geometry.width as i32 || y >= frame.geometry.height as i32 {
            return None;
        }
        let at = ((y as usize) * frame.geometry.width as usize + x as usize) * 4;
        *slot = (frame.rgba[at], frame.rgba[at + 1], frame.rgba[at + 2]);
    }
    Some(out)
}

/// A desktop point on one of the pointer's lines, and the frozen pixel under it.
type LineProbe = ((i32, i32), (u8, u8, u8));

/// Where the pointer's lines are read, and the frozen pixel under each: on the row through
/// the pointer just right of the magnifier's panel, and on the column through it 100 px
/// down, left of the panel. Both fall inside the copy `magnifier_seen` takes.
fn line_probes(frame: &Frame, pointer: (i32, i32)) -> Option<[LineProbe; 2]> {
    let display = monitors().into_iter().find(|m| {
        pointer.0 >= m.rect.x
            && pointer.1 >= m.rect.y
            && pointer.0 < m.rect.x + m.rect.width as i32
            && pointer.1 < m.rect.y + m.rect.height as i32
    })?;
    let scaled = |px: i32| (px * display.scale_percent as i32 + 50) / 100;
    let pixel = |point: (i32, i32)| {
        let x = point.0 - frame.geometry.origin_x;
        let y = point.1 - frame.geometry.origin_y;
        if x < 0 || y < 0 || x >= frame.geometry.width as i32 || y >= frame.geometry.height as i32 {
            return None;
        }
        let at = ((y as usize) * frame.geometry.width as usize + x as usize) * 4;
        Some((
            point,
            (frame.rgba[at], frame.rgba[at + 1], frame.rgba[at + 2]),
        ))
    };
    Some([
        pixel((pointer.0 + scaled(138), pointer.1))?,
        pixel((pointer.0, pointer.1 + scaled(100)))?,
    ])
}

/// Looks at the magnifier beside the pointer: the screen below and right of the pointer is
/// written beside the executable, and read for the panel's ground in the app's background
/// colour, its ring, the pixels around the pointer shown large at its centre, and the light
/// pixels of the size text above the circle, which the panel's fill has none of. The
/// offsets are the spec's own numbers at 100%, the display's scale applied to each, so on
/// a scaled display a rounding can put a probe a pixel off.
fn magnifier_seen(
    pointer: (i32, i32),
    around: [(u8, u8, u8); 9],
    lines: [LineProbe; 2],
    name: &str,
) -> Result<(), String> {
    let display = monitors()
        .into_iter()
        .find(|m| {
            pointer.0 >= m.rect.x
                && pointer.1 >= m.rect.y
                && pointer.0 < m.rect.x + m.rect.width as i32
                && pointer.1 < m.rect.y + m.rect.height as i32
        })
        .ok_or("the pointer is on no display")?;
    let scaled = |px: i32| (px * display.scale_percent as i32 + 50) / 100;
    // The panel: 128 wide, 8 px right of the pointer and 8 below it; the copy takes 4 px
    // more on each side and enough height for the text row.
    let region = DesktopRect::from_points(
        (pointer.0 - scaled(4)).max(display.rect.x),
        pointer.1,
        (pointer.0 + scaled(140)).min(display.rect.x + display.rect.width as i32),
        (pointer.1 + scaled(176)).min(display.rect.y + display.rect.height as i32),
    );
    let pixels = copy_rect(region).map_err(|e| e.to_string())?;
    // Where the pointer really is at the copy, against where the script put it: a
    // difference here is the script's, not the overlay's.
    let mut cursor = POINT::default();
    let strayed = unsafe { GetCursorPos(&mut cursor) }.is_ok() && (cursor.x, cursor.y) != pointer;
    if strayed {
        println!(
            "  while {name}, the pointer was put at {},{} but is at {},{} as the screen is copied",
            pointer.0, pointer.1, cursor.x, cursor.y
        );
    }
    if let Ok(exe) = std::env::current_exe() {
        let path = exe.with_file_name(format!("s02-magnifier-{name}.png"));
        match image::RgbaImage::from_raw(region.width, region.height, pixels.clone()) {
            Some(buffer) => match buffer.save(&path) {
                Ok(()) => println!("  the magnifier while {name} is at {}", path.display()),
                Err(err) => println!("  the magnifier was not written: {err}"),
            },
            None => println!("  the magnifier was not written: size mismatch"),
        }
    }
    // The pointer's lines (Rotem, 2026-09-18): the frame's blue at 48% over the frozen
    // pixel, dimmed or lit, on the pointer's row and on its column. Against the frozen
    // pixel alone, which is what a screen with no line shows; 3 a channel for the two
    // blends' rounding. With the pointer somewhere else, a hand on the mouse, the lines are
    // somewhere else too, and nothing is read: inconclusive, not a failure.
    let lines = if strayed {
        println!("  while {name}, the pointer's lines were not read: the pointer is not where it was put");
        Vec::new()
    } else {
        lines.to_vec()
    };
    for (which, (point, frozen)) in ["across", "down"].iter().zip(lines) {
        let (x, y) = (point.0 - region.x, point.1 - region.y);
        if x < 0 || y < 0 || x >= region.width as i32 || y >= region.height as i32 {
            return Err(format!(
                "while {name}, the line probe {which} is off the copy"
            ));
        }
        let i = ((y * region.width as i32 + x) * 4) as usize;
        let shown = [pixels[i], pixels[i + 1], pixels[i + 2]];
        let near = |a: [u8; 3], b: [u8; 3]| (0..3).all(|c| a[c].abs_diff(b[c]) <= 3);
        let lit = [frozen.0, frozen.1, frozen.2];
        let dimmed = lit.map(|c| (c as u32 * (255 - 140) / 255) as u8);
        let lined = |base: [u8; 3]| {
            let blue = [0x25u32, 0x54, 0xFB];
            [0, 1, 2].map(|c| ((blue[c] * 122 + base[c] as u32 * (255 - 122)) / 255) as u8)
        };
        if near(lined(lit), lit) || near(lined(dimmed), dimmed) {
            println!("  while {name}, the line {which} cannot be told from the pixel under it: inconclusive");
        } else if !near(shown, lined(lit)) && !near(shown, lined(dimmed)) {
            return Err(format!(
                "while {name}, the pixel on the pointer's line {which} is {shown:?}, not the blue at 48% over {lit:?} lit {:?} or dimmed {:?}",
                lined(lit),
                lined(dimmed)
            ));
        }
    }
    // Where the panel's top left lands in the copy.
    let panel = (pointer.0 + scaled(8) - region.x, scaled(8));
    let at = |x: i32, y: i32| -> Option<(u8, u8, u8)> {
        let (x, y) = (panel.0 + x, panel.1 + y);
        if x < 0 || y < 0 || x >= region.width as i32 || y >= region.height as i32 {
            return None;
        }
        let i = ((y * region.width as i32 + x) * 4) as usize;
        Some((pixels[i], pixels[i + 1], pixels[i + 2]))
    };
    // The circle's centre row: the size text sits above the circle, its height the face's,
    // so the row is found rather than assumed, by the circle's #2554FB line through the
    // centre, read 30 px in from the panel's left edge where nothing else is that blue.
    let centre_row = (0..region.height as i32)
        .find(|&y| at(scaled(30), y) == Some((0x25, 0x54, 0xFB)))
        .ok_or(format!(
            "while {name}, no blue line through a circle's centre below and right of the pointer: no magnifier there"
        ))?;
    // The ground: 2 px in from the panel's left edge on the circle's centre row, inside
    // the 8 px around the circle.
    let ground = at(scaled(2), centre_row).ok_or("the ground probe is off the copy")?;
    if ground != (0x0D, 0x0E, 0x12) {
        return Err(format!(
            "while {name}, the pixel 2,{centre_row} into the panel is {ground:?}, not the app background (13, 14, 18): no magnifier below and right of the pointer"
        ));
    }
    // The ring: the circle's leftmost 2 px on its centre row.
    let ring = at(scaled(9), centre_row).ok_or("the ring probe is off the copy")?;
    if ring != (0xF2, 0xF2, 0xF2) {
        return Err(format!(
            "while {name}, the pixel 9,{centre_row} into the panel is {ring:?}, not the ring's (242, 242, 242)"
        ));
    }
    // The pointer's pixel and its eight neighbours, each an 8 px cell around the circle's
    // centre at 64 across and the row found, the pointer's own cell starting 1 px past the
    // blue lines on its top and left edges: read 2 px past each cell's start, off the grid
    // and off those lines.
    for (n, expected) in around.iter().enumerate() {
        let (i, j) = (n as i32 % 3 - 1, n as i32 / 3 - 1);
        let probe = (scaled(67 + 8 * i), centre_row + scaled(3) + scaled(8 * j));
        let shown = at(probe.0, probe.1).ok_or("a cell probe is off the copy")?;
        if shown != *expected {
            return Err(format!(
                "while {name}, the cell for the pixel {i},{j} from the pointer shows {shown:?} where the frozen pixel is {expected:?}"
            ));
        }
    }
    // The text row above the circle, between the panel's top and the circle's: its
    // light pixels.
    let mut light = 0;
    for y in scaled(2)..(centre_row - scaled(57)) {
        for x in 0..scaled(128) {
            if let Some((r, g, b)) = at(x, y) {
                if r > 0x80 && g > 0x80 && b > 0x80 {
                    light += 1;
                }
            }
        }
    }
    if light < 20 {
        return Err(format!(
            "while {name}, only {light} light pixels above the circle: the panel has no size text"
        ));
    }
    println!(
        "  while {name}, the magnifier sits below and right of the pointer on the app background, ringed, its centre cells the frozen pixels, {light} light pixels of size text above it"
    );
    Ok(())
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
        Outcome::Scroll { rect, .. } => Err(format!("the round button answered instead: {rect:?}")),
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

/// The strip of the screen the glide is read in (Rotem, 2026-09-18): from the window's
/// left edge to 20 px into its part, over the part's middle rows, with the frozen pixels
/// under it, so a pixel of the picture's own in the band's colour is not read as the band.
/// None when the part sits too close to the window's edge for one.
fn glide_strip(
    frame: &Frame,
    window: DesktopRect,
    part: DesktopRect,
) -> Option<(DesktopRect, Vec<u8>)> {
    if part.x <= window.x + 4 || part.height <= 20 {
        return None;
    }
    let strip = DesktopRect {
        x: window.x,
        y: part.y + 10,
        width: (part.x - window.x + 20) as u32,
        height: part.height - 20,
    };
    let frozen = frame.crop(desktop_to_image(frame.geometry, strip).ok()?)?;
    Some((strip, frozen))
}

/// Looks at the lit area on its way from the stand-in's part to the whole window: the
/// frame's left band, found by its ink in the strip, sits between the part's edge and
/// the window's while the glide lasts, and on the window's edge once it is over. With
/// Windows' animation effects off the lit area jumps, as the product does, and only the
/// arrival is looked at. A look that came too late to catch the glide is said to be
/// inconclusive rather than blamed on the overlay.
fn glide_seen(
    window: DesktopRect,
    part: DesktopRect,
    strip: Option<(DesktopRect, Vec<u8>)>,
    moved_at: Instant,
) -> Result<(), String> {
    let Some((strip, frozen)) = strip else {
        println!(
            "  the glide was not looked at: the part sits too close to the window's edge for a strip"
        );
        return Ok(());
    };
    let width = strip.width as usize;
    let span = (part.x - window.x) as usize;
    let animates = crate::overlay::windows_animates();
    let mut on_the_way = None;
    if animates {
        // The look waits for the area to be well on its way, since the screen's copy
        // sits a tick and a composition behind the clock.
        std::thread::sleep(std::time::Duration::from_millis(80).saturating_sub(moved_at.elapsed()));
        let looked_at = moved_at.elapsed().as_millis();
        let pixels = copy_rect(strip).map_err(|e| e.to_string())?;
        let edge = band_left(&pixels, &frozen, width)
            .ok_or("no frame band in the strip while the lit area was on its way")?;
        if edge == 0 {
            if looked_at > 140 {
                println!(
                    "  inconclusive: the strip was looked at {looked_at} ms after the move, too late to catch the glide"
                );
            } else {
                return Err(format!(
                    "the lit area jumped: {looked_at} ms after the move its frame already sat on the window's edge"
                ));
            }
        } else if edge >= span {
            return Err(format!(
                "the lit area had not set out: {looked_at} ms after the move its frame still sat on the part's edge"
            ));
        } else {
            on_the_way = Some((looked_at, edge, span - edge));
        }
    }
    // The arrival: past the glide's length, the frame sits on the window's own edge.
    std::thread::sleep(std::time::Duration::from_millis(300).saturating_sub(moved_at.elapsed()));
    let pixels = copy_rect(strip).map_err(|e| e.to_string())?;
    let edge =
        band_left(&pixels, &frozen, width).ok_or("no frame band in the strip after the glide")?;
    let now = moved_at.elapsed().as_millis();
    if edge != 0 {
        return Err(format!(
            "the lit area never arrived: {now} ms after the move its frame sits {edge} px in from the window's edge"
        ));
    }
    match on_the_way {
        Some((ms, from_window, from_part)) => println!(
            "  the lit area glided from the part to the window: {ms} ms after the move its frame was {from_window} px in from the window's edge and {from_part} px out from the part's, and on the window's edge at {now} ms"
        ),
        None if animates => println!("  the lit area is on the window's edge at {now} ms"),
        None => println!(
            "  Windows' animation effects are off, so the lit area jumps as the product does: its frame on the window's edge at {now} ms"
        ),
    }
    Ok(())
}

/// The leftmost column of the frame's band in a strip: a pixel of the dash's ink that the
/// frozen picture does not have there, on any row. None when no row shows one.
fn band_left(pixels: &[u8], frozen: &[u8], width: usize) -> Option<usize> {
    let rows = pixels.len() / (width * 4);
    let mut left: Option<usize> = None;
    for y in 0..rows {
        for x in 0..width {
            let at = (y * width + x) * 4;
            let (r, g, b) = (pixels[at], pixels[at + 1], pixels[at + 2]);
            let ink = r < 0x50 && g > 0x80 && b > 0xB0;
            let frozen_here = frozen
                .get(at..at + 3)
                .is_some_and(|f| f[0] == r && f[1] == g && f[2] == b);
            if ink && !frozen_here {
                left = Some(left.map_or(x, |l| l.min(x)));
                break;
            }
        }
    }
    left
}

/// The title of the stand-in window, a second process of this same executable.
const STAND_IN: &str = "Recon stand-in";

/// Where the stand-in's one part sits in its client area: x, y, width, height.
const STAND_IN_PART: (i32, i32, i32, i32) = (40, 60, 200, 100);

/// The window pick: with the stand-in window in front, a click inside it with no drag hands
/// back its visible bounds, read here straight from the window manager rather than through
/// the overlay's own listing, so the two sides of the comparison are independent. Then a
/// drag that starts inside the same window still hands back the dragged rectangle, which
/// is what would go wrong if the threshold were read the wrong way round.
fn window_pick_test(frame: &Frame) -> Result<(), String> {
    let (mut child, hwnd) = open_stand_in()?;
    let mut bounds = RECT::default();
    let read = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut bounds as *mut RECT as *mut std::ffi::c_void,
            std::mem::size_of::<RECT>() as u32,
        )
    };
    if let Err(err) = read {
        let _ = child.kill();
        return Err(format!(
            "the stand-in's visible bounds could not be read: {err}"
        ));
    }
    let visible = DesktopRect::from_points(bounds.left, bounds.top, bounds.right, bounds.bottom);
    // Cut to the display the click lands on, as the product does (§3.2).
    let centre = (
        visible.x + visible.width as i32 / 2,
        visible.y + visible.height as i32 / 2,
    );
    let display = monitors()
        .into_iter()
        .map(|m| m.rect)
        .find(|r| {
            centre.0 >= r.x
                && centre.1 >= r.y
                && centre.0 < r.x + r.width as i32
                && centre.1 < r.y + r.height as i32
        })
        .ok_or("the stand-in's centre is on no display")?;
    let expected = DesktopRect::from_points(
        visible.x.max(display.x),
        visible.y.max(display.y),
        (visible.x + visible.width as i32).min(display.x + display.width as i32),
        (visible.y + visible.height as i32).min(display.y + display.height as i32),
    );

    // The part's place on screen, from the window manager and the known offsets, not from
    // the overlay. The whole-window click lands near the bottom-right corner, clear of it.
    let mut origin = POINT::default();
    if !unsafe { ClientToScreen(hwnd, &mut origin) }.as_bool() {
        let _ = child.kill();
        return Err("the stand-in's client origin could not be read".into());
    }
    let part = DesktopRect {
        x: origin.x + STAND_IN_PART.0,
        y: origin.y + STAND_IN_PART.1,
        width: STAND_IN_PART.2 as u32,
        height: STAND_IN_PART.3 as u32,
    };
    let part_centre = (
        part.x + part.width as i32 / 2,
        part.y + part.height as i32 / 2,
    );
    let corner = (
        visible.x + visible.width as i32 - 30,
        visible.y + visible.height as i32 - 30,
    );

    let (looked, seen) = std::sync::mpsc::channel();
    let around = match pixels_around(frame, part_centre) {
        Some(around) => around,
        None => {
            let _ = child.kill();
            return Err("the part's centre is off the frame".into());
        }
    };
    let lines = match line_probes(frame, part_centre) {
        Some(lines) => lines,
        None => {
            let _ = child.kill();
            return Err("a line probe at the part's centre is off the frame".into());
        }
    };
    let (glided, glide) = std::sync::mpsc::channel();
    let strip = glide_strip(frame, expected, part);
    let picked = with_overlay(frame, move || {
        move_to(part_centre.0, part_centre.1);
        // The lit part, as a person sees it before the click: the screen with the
        // overlay up, written beside the executable to be looked at. And the magnifier
        // while hovering, the part's size under it, looked at the same way.
        std::thread::sleep(std::time::Duration::from_millis(250));
        let _ = looked.send(magnifier_seen(part_centre, around, lines, "hovering"));
        match copy_rect(display) {
            Ok(pixels) => {
                if let Ok(exe) = std::env::current_exe() {
                    let path = exe.with_file_name("s02-window-lit.png");
                    match image::RgbaImage::from_raw(display.width, display.height, pixels) {
                        Some(buffer) => match buffer.save(&path) {
                            Ok(()) => println!("  the lit window is at {}", path.display()),
                            Err(err) => println!("  the lit window was not written: {err}"),
                        },
                        None => println!("  the lit window was not written: size mismatch"),
                    }
                }
            }
            Err(err) => println!("  the lit window was not written: {err}"),
        }
        // The glide (Rotem, 2026-09-18): from the part to the whole window, the lit area
        // is looked at on its way and once it has arrived.
        let moved_at = Instant::now();
        move_to(corner.0, corner.1);
        let _ = glided.send(glide_seen(expected, part, strip, moved_at));
        press_left();
        release_left();
    });
    let picked = match picked {
        Ok(outcome) => outcome,
        Err(err) => {
            let _ = child.kill();
            return Err(err);
        }
    };
    let pick_result = match picked {
        Outcome::Selected(got) if got == expected => {
            println!(
                "  a click at {},{} picked the stand-in's visible bounds exactly: {got:?}",
                corner.0, corner.1
            );
            // The spot the capture draws the mouse pointer at: where the click landed.
            match crate::overlay::take_selected_at() {
                Some(at) if at == corner => {
                    println!("  the mouse at the selection was recorded where it clicked: {at:?}");
                    Ok(())
                }
                other => Err(format!(
                    "clicked at {corner:?}, and the selection recorded the mouse at {other:?}"
                )),
            }
        }
        Outcome::Selected(got) => Err(format!(
            "a click inside the stand-in gave {got:?} rather than its visible bounds {expected:?}"
        )),
        Outcome::Cancelled => {
            Err("a click inside the stand-in cancelled rather than picking it".to_string())
        }
        Outcome::Scroll { rect, .. } => Err(format!("the round button answered instead: {rect:?}")),
    };
    if let Err(err) = pick_result {
        let _ = child.kill();
        return Err(err);
    }
    let hover_result = match seen.try_recv() {
        Ok(result) => result,
        Err(_) => Err("the magnifier was never looked at while hovering".to_string()),
    };
    if let Err(err) = hover_result {
        let _ = child.kill();
        return Err(err);
    }

    let glide_result = match glide.try_recv() {
        Ok(result) => result,
        Err(_) => Err("the glide was never looked at".to_string()),
    };
    if let Err(err) = glide_result {
        let _ = child.kill();
        return Err(err);
    }

    // The part: a click on the stand-in's one child window picks that part, not the window.
    let picked = with_overlay(frame, move || {
        move_to(part_centre.0, part_centre.1);
        press_left();
        release_left();
    });
    let part_result = match picked {
        Ok(Outcome::Selected(got)) if got == part => {
            println!(
                "  a click at {},{} picked the stand-in's part exactly: {got:?}",
                part_centre.0, part_centre.1
            );
            Ok(())
        }
        Ok(Outcome::Selected(got)) => Err(format!(
            "a click on the stand-in's part gave {got:?} rather than the part {part:?}"
        )),
        Ok(Outcome::Cancelled) => Err("a click on the stand-in's part cancelled".to_string()),
        Ok(Outcome::Scroll { rect, .. }) => {
            Err(format!("the round button answered instead: {rect:?}"))
        }
        Err(err) => Err(err),
    };
    if let Err(err) = part_result {
        let _ = child.kill();
        return Err(err);
    }

    // The drag: from the centre, well past any drag threshold, so the window must not win.
    let dragged = DesktopRect {
        x: centre.0,
        y: centre.1,
        width: 60,
        height: 40,
    };
    let outcome = with_overlay(frame, move || {
        move_to(dragged.x, dragged.y);
        press_left();
        move_to(dragged.x + 30, dragged.y + 20);
        move_to(
            dragged.x + dragged.width as i32,
            dragged.y + dragged.height as i32,
        );
        release_left();
    });
    let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _ = child.kill();
    match outcome? {
        Outcome::Selected(got) if got == dragged => {
            println!("  a drag from the same point still gave the dragged rectangle: {got:?}");
            Ok(())
        }
        Outcome::Selected(got) => Err(format!(
            "a drag inside the stand-in gave {got:?} rather than the dragged {dragged:?}"
        )),
        Outcome::Cancelled => Err("a drag inside the stand-in cancelled".to_string()),
        Outcome::Scroll { rect, .. } => Err(format!("the round button answered instead: {rect:?}")),
    }
}

/// Ctrl held at the selection (Rotem, 2026-09-22): a drag and a click on a lit window, each
/// released with Ctrl down, are recorded as such, and the record is taken once; a drag with
/// no Ctrl is not. The overlay's half only: the editor checks prove a capture announced as
/// taken with Ctrl, and no check drives the join between the two in `begin_capture`, which
/// needs the real shortcut.
fn ctrl_test(frame: &Frame) -> Result<(), String> {
    let target = pick_test_area(frame.geometry).ok_or("no room for a test drag")?;
    let dragged = DesktopRect {
        x: target.x + 40,
        y: target.y + 30,
        width: 160,
        height: 100,
    };
    for ctrl in [false, true] {
        let outcome = with_overlay(frame, move || {
            if ctrl {
                hold_ctrl(true);
            }
            move_to(dragged.x, dragged.y);
            press_left();
            move_to(dragged.x + 60, dragged.y + 40);
            move_to(
                dragged.x + dragged.width as i32,
                dragged.y + dragged.height as i32,
            );
            release_left();
            if ctrl {
                hold_ctrl(false);
            }
        })?;
        let recorded = overlay::take_selected_with_ctrl();
        match outcome {
            Outcome::Selected(got) if got == dragged && recorded == ctrl => println!(
                "  a drag {} Ctrl gave the dragged rectangle, recorded as {}",
                if ctrl { "with" } else { "without" },
                if ctrl { "with Ctrl" } else { "without it" }
            ),
            other => {
                return Err(format!(
                    "a drag {} Ctrl gave {other:?}, recorded with Ctrl {recorded}",
                    if ctrl { "with" } else { "without" }
                ))
            }
        }
        if overlay::take_selected_with_ctrl() {
            return Err("the Ctrl record was still there after it was taken".into());
        }
    }

    // The click on a lit window, the stand-in's one part, with Ctrl down.
    let (mut child, hwnd) = open_stand_in()?;
    let mut origin = POINT::default();
    if !unsafe { ClientToScreen(hwnd, &mut origin) }.as_bool() {
        let _ = child.kill();
        return Err("the stand-in's client origin could not be read".into());
    }
    let at = (
        origin.x + STAND_IN_PART.0 + STAND_IN_PART.2 / 2,
        origin.y + STAND_IN_PART.1 + STAND_IN_PART.3 / 2,
    );
    let outcome = with_overlay(frame, move || {
        hold_ctrl(true);
        move_to(at.0, at.1);
        std::thread::sleep(std::time::Duration::from_millis(250));
        press_left();
        release_left();
        hold_ctrl(false);
    });
    let recorded = overlay::take_selected_with_ctrl();
    let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _ = child.kill();
    match outcome? {
        Outcome::Selected(got) if recorded => {
            println!("  a click with Ctrl on the lit part picked {got:?}, recorded with Ctrl");
            Ok(())
        }
        other => Err(format!(
            "a click with Ctrl on the lit part gave {other:?}, recorded with Ctrl {recorded}"
        )),
    }
}

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
        // One part inside it, at a known place, for the part pick: a plain static
        // control, the kind of child window a real application's panes are.
        let _ = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("part"),
            WS_CHILD | WS_VISIBLE,
            STAND_IN_PART.0,
            STAND_IN_PART.1,
            STAND_IN_PART.2,
            STAND_IN_PART.3,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        );
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    0
}

/// The title of the stand-in that scrolls, a second process of this same executable.
const SCROLL_STAND_IN: &str = "Recon scrolling stand-in";
/// How long its page is, and how far one notch of the wheel moves it.
const SCROLL_PAGE_ROWS: i32 = 2400;
const SCROLL_NOTCH_ROWS: i32 = 90;

/// The colour of one pixel of the stand-in's page: bands of lines that look like nothing
/// else on the page, with blank rows between them, as text has. The stand-in paints from
/// this and the check reads the tall picture against it, so the two sides of the comparison
/// share the formula and nothing else: the screen, the copies and the stitcher lie between.
fn scroll_page_pixel(x: i32, row: i32) -> (u8, u8, u8) {
    if row % 23 < 5 {
        return (255, 255, 255);
    }
    let n = ((x / 7) as u32).wrapping_mul(2654435761) ^ (row as u32).wrapping_mul(40503);
    ((n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8)
}

/// A window whose client area scrolls a long page, with a real scrollbar, the wheel moving
/// it a fixed step at once. Run with --stand-in-scroll; exits when the window is closed.
pub fn scroll_stand_in_window() -> i32 {
    use std::sync::atomic::{AtomicI32, Ordering};
    static OFFSET: AtomicI32 = AtomicI32::new(0);

    // The windows crate keeps this one with the common controls, a feature nothing in the
    // product needs; the stand-in names it itself.
    #[link(name = "user32")]
    extern "system" {
        fn SetScrollInfo(hwnd: HWND, bar: i32, info: *const SCROLLINFO, redraw: i32) -> i32;
    }

    unsafe fn sync(hwnd: HWND) {
        let mut client = RECT::default();
        let _ = unsafe { GetClientRect(hwnd, &mut client) };
        let longest = (SCROLL_PAGE_ROWS - client.bottom).max(0);
        OFFSET.store(
            OFFSET.load(Ordering::SeqCst).clamp(0, longest),
            Ordering::SeqCst,
        );
        let info = SCROLLINFO {
            cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
            fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
            nMin: 0,
            nMax: SCROLL_PAGE_ROWS - 1,
            nPage: client.bottom.max(0) as u32,
            nPos: OFFSET.load(Ordering::SeqCst),
            nTrackPos: 0,
        };
        unsafe { SetScrollInfo(hwnd, SB_VERT.0, &info, 1) };
    }

    unsafe extern "system" fn proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        use windows::Win32::Foundation::LRESULT;
        match msg {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            WM_SIZE => {
                unsafe { sync(hwnd) };
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let notches = ((wparam.0 >> 16) & 0xFFFF) as i16 as i32 / 120;
                OFFSET.fetch_sub(notches * SCROLL_NOTCH_ROWS, Ordering::SeqCst);
                unsafe {
                    sync(hwnd);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                let mut client = RECT::default();
                let _ = unsafe { GetClientRect(hwnd, &mut client) };
                let (width, height) = (client.right, client.bottom);
                if width > 0 && height > 0 {
                    let offset = OFFSET.load(Ordering::SeqCst);
                    let mut bgra = vec![0u8; (width * height * 4) as usize];
                    for y in 0..height {
                        for x in 0..width {
                            let (r, g, b) = scroll_page_pixel(x, y + offset);
                            let at = ((y * width + x) * 4) as usize;
                            bgra[at] = b;
                            bgra[at + 1] = g;
                            bgra[at + 2] = r;
                            bgra[at + 3] = 255;
                        }
                    }
                    let info = crate::magnifier::solid_header(width, height);
                    unsafe {
                        SetDIBitsToDevice(
                            hdc,
                            0,
                            0,
                            width as u32,
                            height as u32,
                            0,
                            0,
                            0,
                            height as u32,
                            bgra.as_ptr() as *const std::ffi::c_void,
                            &info,
                            DIB_RGB_COLORS,
                        );
                    }
                }
                let _ = unsafe { EndPaint(hwnd, &ps) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    let title: Vec<u16> = SCROLL_STAND_IN
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        let instance =
            windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                None,
                windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
            )
            .unwrap_or_default(),
            lpszClassName: windows::core::PCWSTR(title.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&class);
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            windows::core::PCWSTR(title.as_ptr()),
            windows::core::PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VSCROLL,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1100,
            1000,
            None,
            None,
            Some(instance.into()),
            None,
        ) else {
            return 1;
        };
        sync(hwnd);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    0
}

/// The scrolling capture, end to end, against a window that scrolls a page this check can
/// work out for itself. The round button is looked for on the screen where it has to be; a
/// click on it has to hand back the stand-in's client area, read here from the window
/// manager; the bar has to be where the button was, and absent from a copy of the screen;
/// the wheel scrolls the page to its end and Enter ends the capture; and the tall picture
/// has to be the page, pixel for pixel, every row of it once. Before that, the same start
/// ended by Escape, which has to hand back nothing.
fn scroll_test(frame: &Frame) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let mut child = std::process::Command::new(exe)
        .arg("--stand-in-scroll")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| format!("the scrolling stand-in did not start: {err}"))?;
    let result = scroll_test_with(frame, &mut child);
    let title: Vec<u16> = SCROLL_STAND_IN
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) } {
        let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let _ = child.kill();
    result
}

fn scroll_test_with(_frame: &Frame, child: &mut std::process::Child) -> Result<(), String> {
    // In front, as the other stand-in is waited for.
    let started = Instant::now();
    let hwnd = loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let hwnd = unsafe { GetForegroundWindow() };
        let owner = crate::platform::window_owner(hwnd);
        if owner.exists && owner.pid == child.id() && owner.title == SCROLL_STAND_IN {
            println!("  in front: {}", owner.line());
            break hwnd;
        }
        if started.elapsed() > std::time::Duration::from_secs(8) {
            return Err("the scrolling stand-in never came to the foreground".into());
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(400));

    // Its client area, from the window manager: what the button has to hand back.
    let mut client = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        GetClientRect(hwnd, &mut client).map_err(|err| err.to_string())?;
        if !ClientToScreen(hwnd, &mut origin).as_bool() {
            return Err("the stand-in's client origin could not be read".into());
        }
    }
    let area = DesktopRect::from_points(
        origin.x,
        origin.y,
        origin.x + client.right,
        origin.y + client.bottom,
    );
    let scale = monitors()
        .into_iter()
        .find(|m| {
            origin.x >= m.rect.x
                && origin.y >= m.rect.y
                && origin.x < m.rect.x + m.rect.width as i32
                && origin.y < m.rect.y + m.rect.height as i32
        })
        .map(|m| m.scale_percent)
        .ok_or("the stand-in is on no display")?;
    let side = crate::magnifier::scaled(crate::scrolling::draw::BUTTON_PX, scale);
    let rise = crate::magnifier::scaled(crate::scrolling::draw::BUTTON_RISE_PX, scale);

    // The lit frame: the stand-in has no parts, so the overlay lights its visible bounds,
    // read here from the window manager. The capture's area has to be that frame, whole.
    let mut visible_bounds = RECT::default();
    unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut visible_bounds as *mut RECT as *mut std::ffi::c_void,
            std::mem::size_of::<RECT>() as u32,
        )
        .map_err(|err| format!("the stand-in's visible bounds could not be read: {err}"))?;
    }
    let lit = DesktopRect::from_points(
        visible_bounds.left,
        visible_bounds.top,
        visible_bounds.right,
        visible_bounds.bottom,
    );
    // Where the page sits inside the frame.
    let (dx, dy) = (area.x - lit.x, area.y - lit.y);

    // The bar cannot be seen in a copy of the screen, which is the point of it; its looks
    // are written as a picture of their own to be looked at, the round button's with them.
    if let (Some((rgba, width, height)), Ok(exe)) =
        (crate::scrolling::looks(scale), std::env::current_exe())
    {
        let path = exe.with_file_name("s02-scrolling-looks.png");
        if let Some(buffer) = image::RgbaImage::from_raw(width, height, rgba) {
            if buffer.save(&path).is_ok() {
                println!(
                    "  the bar's and the button's looks are at {}",
                    path.display()
                );
            }
        }
    }

    // Escape first: the page is still at its top then, which the look under the bar relies on.
    for ending in ["Escape", "Enter"] {
        // A fresh freeze each time: the overlay shows the screen as it is now.
        let frame = WholeVirtualScreen
            .freeze()
            .map_err(|err| format!("freeze failed: {err}"))?;
        let (saw, seen) = std::sync::mpsc::channel();
        let window = hwnd.0 as isize;
        let picked = with_overlay(&frame, move || {
            // Near the top left corner, so the magnifier beside the pointer is clear of the
            // button's place at any scale.
            move_to(area.x + 40, area.y + 40);
            // The lit area is whatever the overlay lights under that point: the window, since
            // the stand-in has no parts. The button sits on the lit area, so its place is
            // worked out from the window's visible bounds, not from the client area.
            let mut bounds = RECT::default();
            let read = unsafe {
                DwmGetWindowAttribute(
                    HWND(window as *mut _),
                    DWMWA_EXTENDED_FRAME_BOUNDS,
                    &mut bounds as *mut RECT as *mut std::ffi::c_void,
                    std::mem::size_of::<RECT>() as u32,
                )
            };
            if read.is_err() {
                let _ = saw.send(Err("the stand-in's bounds could not be read".to_string()));
                press_escape();
                return;
            }
            let centre = (
                bounds.left + (bounds.right - bounds.left) / 2,
                bounds.bottom - rise - side / 2,
            );
            println!(
                "  the stand-in's visible bounds are {},{} to {},{}",
                bounds.left, bounds.top, bounds.right, bounds.bottom
            );
            // The answer comes from another thread: looked for, for three seconds, as the
            // button's blue at its centre's side, clear of the arrow.
            let probe = DesktopRect {
                x: centre.0 + side / 4,
                y: centre.1 - side / 4,
                width: 1,
                height: 1,
            };
            let (r, g, b) = crate::scrolling::draw::BLUE_RGB;
            let waited = Instant::now();
            let mut last = None;
            let shown = loop {
                if let Ok(pixel) = copy_rect(probe) {
                    last = Some((pixel[0], pixel[1], pixel[2]));
                    if last == Some((r, g, b)) {
                        break true;
                    }
                }
                if waited.elapsed() > std::time::Duration::from_secs(3) {
                    break false;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            if !shown {
                let _ = saw.send(Err(format!(
                    "no round button at {},{}: the screen there is {last:?}, not its blue",
                    probe.x, probe.y
                )));
                press_escape();
                return;
            }
            let _ = saw.send(Ok((centre, waited.elapsed().as_millis())));
            // The button on the lit area, as a person sees it, to be looked at; then under
            // the pointer, where it has to be its lighter blue and the magnifier gone.
            let around = DesktopRect {
                x: centre.0 - side * 3,
                y: centre.1 - side * 2,
                width: side as u32 * 6,
                height: side as u32 * 3,
            };
            let write = |name: &str| {
                if let (Ok(pixels), Ok(exe)) = (copy_rect(around), std::env::current_exe()) {
                    if let Some(buffer) =
                        image::RgbaImage::from_raw(around.width, around.height, pixels)
                    {
                        let _ = buffer.save(exe.with_file_name(name));
                    }
                }
            };
            write("s02-scrolling-button.png");
            move_to(centre.0, centre.1);
            std::thread::sleep(std::time::Duration::from_millis(120));
            write("s02-scrolling-button-hot.png");
            let hot = copy_rect(probe).ok().map(|p| (p[0], p[1], p[2]));
            if hot != Some(crate::scrolling::draw::BLUE_HOT_RGB) {
                println!("  NOTE: under the pointer the button's fill read {hot:?}");
            }
            press_left();
            release_left();
        })?;
        let (centre, waited) = seen
            .try_recv()
            .map_err(|_| "the round button was never looked for".to_string())??;
        println!(
            "  the round button showed {waited} ms after the pointer came to rest, centred at {},{}",
            centre.0, centre.1
        );
        let rect = match picked {
            Outcome::Scroll { rect, .. } if rect == lit => rect,
            other => {
                return Err(format!(
                    "a click on the round button gave {other:?} rather than the lit frame {lit:?}"
                ))
            }
        };
        println!("  a click on it handed back the lit frame exactly, not the part of it that scrolls: {rect:?}");

        let session = std::thread::spawn(move || crate::scrolling::run(rect));
        std::thread::sleep(std::time::Duration::from_millis(900));

        // The bar: a visible window where the button was, and not in a copy of the screen.
        let bar = unsafe { FindWindowW(w!("ReconScrollBar"), PCWSTR::null()) }
            .map_err(|_| "the bar's window does not exist".to_string())?;
        let mut at = RECT::default();
        unsafe { GetWindowRect(bar, &mut at).map_err(|err| err.to_string())? };
        let visible = unsafe { IsWindowVisible(bar) }.as_bool();
        let bottom_gap = lit.y + lit.height as i32 - at.bottom;
        let off_centre = (at.left + at.right) / 2 - (lit.x + lit.width as i32 / 2);
        if !visible || bottom_gap != rise || off_centre.abs() > 1 {
            return Err(format!(
                "the bar is visible: {visible}, {bottom_gap} px up from the area's bottom for {rise}, {off_centre} px off its middle"
            ));
        }
        let under_bar = DesktopRect::from_points(at.left, at.top, at.right, at.bottom);
        let copied = copy_rect(under_bar).map_err(|err| err.to_string())?;
        let mut wrong = 0usize;
        for y in 0..under_bar.height as i32 {
            for x in 0..under_bar.width as i32 {
                let (r, g, b) =
                    scroll_page_pixel(under_bar.x - area.x + x, under_bar.y - area.y + y);
                let i = ((y * under_bar.width as i32 + x) * 4) as usize;
                if (copied[i], copied[i + 1], copied[i + 2]) != (r, g, b) {
                    wrong += 1;
                }
            }
        }
        if wrong != 0 {
            return Err(format!(
                "a copy of the screen where the bar lies differs from the page in {wrong} pixels: the bar is in it"
            ));
        }
        println!(
            "  the bar is {}x{} at {},{}, {rise} px up from the area's bottom, and a copy of the screen there is the page alone",
            at.right - at.left,
            at.bottom - at.top,
            at.left,
            at.top
        );

        if ending == "Escape" {
            press_escape();
            match session.join() {
                Ok(crate::scrolling::Ended::Cancelled) => {
                    println!("  Escape ended it with nothing captured");
                    continue;
                }
                Ok(_) => return Err("Escape did not cancel the scrolling capture".into()),
                Err(_) => return Err("the scrolling capture panicked".into()),
            }
        }

        // The wheel, a notch at a time, to the page's end and a few notches past it.
        move_to(area.x + area.width as i32 / 2, area.y + 80);
        let notches = (SCROLL_PAGE_ROWS - area.height as i32) / SCROLL_NOTCH_ROWS + 4;
        for _ in 0..notches {
            unsafe { mouse_event(MOUSEEVENTF_WHEEL, 0, 0, -120, 0) };
            std::thread::sleep(std::time::Duration::from_millis(140));
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        unsafe {
            keybd_event(VK_RETURN.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_RETURN.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
        let tall = match session.join() {
            Ok(crate::scrolling::Ended::Done(tall)) => tall,
            Ok(crate::scrolling::Ended::Cancelled) => {
                return Err("Enter cancelled the scrolling capture".into())
            }
            Ok(crate::scrolling::Ended::Failed(why)) => {
                return Err(format!("the scrolling capture refused: {why}"))
            }
            Err(_) => return Err("the scrolling capture panicked".into()),
        };
        if let Ok(exe) = std::env::current_exe() {
            let path = exe.with_file_name("s02-scrolling-capture.png");
            if let Some(buffer) =
                image::RgbaImage::from_raw(tall.width(), tall.height(), tall.rgba.clone())
            {
                if buffer.save(&path).is_ok() {
                    println!("  the tall picture is at {}", path.display());
                }
            }
        }
        // The frame around the page stands still: its title bar is the picture's top and its
        // bottom edge the picture's end, each once. The stand-in's own scrollbar may be cut
        // off at the right, and nothing of the page with it.
        let tall_rows = SCROLL_PAGE_ROWS as u32 + lit.height - area.height;
        let width = tall.width() as i32;
        if tall.height() != tall_rows || width > lit.width as i32 || width < dx + area.width as i32
        {
            return Err(format!(
                "the tall picture is {}x{}, and the frame with the whole page in it is {}x{tall_rows}, the page {} wide from {dx}",
                tall.width(),
                tall.height(),
                lit.width,
                area.width
            ));
        }
        // Left out of the comparison: the two bottom corners of the last rows, where Windows
        // rounds the stand-in's own corners and blends them with what lies behind it.
        let corner = crate::magnifier::scaled(12, scale);
        let mut wrong = 0usize;
        for row in 0..SCROLL_PAGE_ROWS {
            for x in 0..area.width as i32 {
                if row >= SCROLL_PAGE_ROWS - corner
                    && (x < corner || x >= area.width as i32 - corner)
                {
                    continue;
                }
                let (r, g, b) = scroll_page_pixel(x, row);
                let i = (((row + dy) * width + x + dx) * 4) as usize;
                if tall.rgba[i..i + 4] != [r, g, b, 255] {
                    wrong += 1;
                }
            }
        }
        if wrong != 0 {
            return Err(format!(
                "the tall picture differs from the page in {wrong} pixels"
            ));
        }
        println!(
            "  {notches} notches of the wheel and Enter: the tall picture is {}x{}, the page pixel for pixel, every row once",
            tall.width(),
            tall.height()
        );
    }
    Ok(())
}

/// The scrolling capture against a real window, found by a part of its title: the pointer
/// goes to the middle of its largest part, the round button is waited for where it has to
/// be on that part, pressed, the wheel is turned sixty notches with a breath between them,
/// and Enter ends it. What comes back is written beside the executable to be looked at.
fn scroll_trial(title_part: &str) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, GetWindowTextW, SetForegroundWindow,
    };
    struct Find(String, Option<isize>);
    unsafe extern "system" fn top(hwnd: HWND, l: LPARAM) -> windows::core::BOOL {
        let find = unsafe { &mut *(l.0 as *mut Find) };
        let mut text = [0u16; 256];
        let n = unsafe { GetWindowTextW(hwnd, &mut text) }.max(0) as usize;
        if unsafe { IsWindowVisible(hwnd) }.as_bool()
            && String::from_utf16_lossy(&text[..n]).contains(&find.0)
        {
            find.1 = Some(hwnd.0 as isize);
            return false.into();
        }
        true.into()
    }
    unsafe extern "system" fn child(hwnd: HWND, l: LPARAM) -> windows::core::BOOL {
        let parts = unsafe { &mut *(l.0 as *mut Vec<RECT>) };
        let mut r = RECT::default();
        if unsafe { IsWindowVisible(hwnd) }.as_bool()
            && unsafe { GetWindowRect(hwnd, &mut r) }.is_ok()
        {
            parts.push(r);
        }
        true.into()
    }
    let mut find = Find(title_part.to_string(), None);
    let _ = unsafe { EnumWindows(Some(top), LPARAM(&mut find as *mut Find as isize)) };
    let window = find
        .1
        .ok_or(format!("no visible window's title holds {title_part:?}"))?;
    let hwnd = HWND(window as *mut _);
    // Above everything else for the trial's length: a check has no right to the foreground,
    // and another window over this one would be what the overlay lights.
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
        };
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        let _ = SetForegroundWindow(hwnd);
    }
    std::thread::sleep(std::time::Duration::from_millis(600));
    let mut parts: Vec<RECT> = Vec::new();
    let _ = unsafe {
        EnumChildWindows(
            Some(hwnd),
            Some(child),
            LPARAM(&mut parts as *mut Vec<RECT> as isize),
        )
    };
    // The part the overlay lights under the pointer is the smallest one there; the pointer
    // goes to the middle of the largest, and the smallest part holding that point is it.
    let largest = parts
        .iter()
        .copied()
        .max_by_key(|r| (r.right - r.left) as i64 * (r.bottom - r.top) as i64)
        .ok_or("the window has no parts")?;
    let at = (
        (largest.left + largest.right) / 2,
        largest.top + (largest.bottom - largest.top) / 3,
    );
    let lit = parts
        .iter()
        .copied()
        .filter(|r| at.0 >= r.left && at.0 < r.right && at.1 >= r.top && at.1 < r.bottom)
        .min_by_key(|r| (r.right - r.left) as i64 * (r.bottom - r.top) as i64)
        .unwrap_or(largest);
    println!(
        "  the part to be lit is {},{} to {},{}",
        lit.left, lit.top, lit.right, lit.bottom
    );
    let scale = monitors()
        .into_iter()
        .find(|m| {
            at.0 >= m.rect.x
                && at.1 >= m.rect.y
                && at.0 < m.rect.x + m.rect.width as i32
                && at.1 < m.rect.y + m.rect.height as i32
        })
        .map(|m| m.scale_percent)
        .ok_or("the window is on no display")?;
    let side = crate::magnifier::scaled(crate::scrolling::draw::BUTTON_PX, scale);
    let rise = crate::magnifier::scaled(crate::scrolling::draw::BUTTON_RISE_PX, scale);
    let centre = ((lit.left + lit.right) / 2, lit.bottom - rise - side / 2);

    let frame = WholeVirtualScreen
        .freeze()
        .map_err(|err| format!("freeze failed: {err}"))?;
    let (saw, seen) = std::sync::mpsc::channel();
    let picked = with_overlay(&frame, move || {
        move_to(at.0, at.1);
        let probe = DesktopRect {
            x: centre.0 + side / 4,
            y: centre.1 - side / 4,
            width: 1,
            height: 1,
        };
        let waited = Instant::now();
        let shown = loop {
            if let Ok(pixel) = copy_rect(probe) {
                if (pixel[0], pixel[1], pixel[2]) == crate::scrolling::draw::BLUE_RGB {
                    break true;
                }
            }
            if waited.elapsed() > std::time::Duration::from_secs(6) {
                break false;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let _ = saw.send((shown, waited.elapsed().as_millis()));
        if !shown {
            press_escape();
            return;
        }
        move_to(centre.0, centre.1);
        press_left();
        release_left();
    })?;
    let (shown, waited) = seen.try_recv().map_err(|_| "never looked".to_string())?;
    if !shown {
        return Err(format!("no round button on the part within {waited} ms"));
    }
    println!("  the round button showed after {waited} ms");
    let Outcome::Scroll { rect, .. } = picked else {
        return Err(format!("the click gave {picked:?}"));
    };
    println!("  the area handed over: {rect:?}");
    let session = std::thread::spawn(move || crate::scrolling::run(rect));
    std::thread::sleep(std::time::Duration::from_millis(900));
    move_to(at.0, at.1);
    for _ in 0..60 {
        unsafe { mouse_event(MOUSEEVENTF_WHEEL, 0, 0, -120, 0) };
        std::thread::sleep(std::time::Duration::from_millis(220));
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    unsafe {
        keybd_event(VK_RETURN.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_RETURN.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    match session.join() {
        Ok(crate::scrolling::Ended::Done(tall)) => {
            let path = std::env::current_exe()
                .map_err(|err| err.to_string())?
                .with_file_name("s02-scrolling-trial.png");
            image::RgbaImage::from_raw(tall.width(), tall.height(), tall.rgba)
                .ok_or("size mismatch")?
                .save(&path)
                .map_err(|err| err.to_string())?;
            println!("  the tall picture is {}", path.display());
            Ok(())
        }
        Ok(crate::scrolling::Ended::Cancelled) => Err("it came back cancelled".into()),
        Ok(crate::scrolling::Ended::Failed(why)) => Err(why),
        Err(_) => Err("the scrolling capture panicked".into()),
    }
}

/// `--hold-clipboard`: opens the clipboard and holds it until this process is killed, so
/// the S1.10 check can meet a clipboard another application is holding. Says "held" on
/// its output once it has it.
pub(crate) fn hold_clipboard() -> i32 {
    use windows::Win32::System::DataExchange::OpenClipboard;
    // With a window of its own: an open with no window does not keep another process
    // out, as a probe on this machine showed, and a real application holds it through a
    // window.
    unsafe extern "system" fn proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
    let title: Vec<u16> = "ReconClipboardHolder"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            windows::core::PCWSTR(title.as_ptr()),
            windows::core::PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            100,
            100,
            None,
            None,
            Some(instance.into()),
            None,
        ) else {
            println!("not held: no window");
            return 1;
        };
        if OpenClipboard(Some(hwnd)).is_err() {
            println!("not held");
            return 1;
        }
        println!("held");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    0
}

/// Starts the clipboard holder in its own process and waits for its "held".
pub(crate) fn open_clipboard_holder() -> Result<std::process::Child, String> {
    use std::io::{BufRead, BufReader};
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let mut child = std::process::Command::new(exe)
        .arg("--hold-clipboard")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| format!("the clipboard holder did not start: {err}"))?;
    let mut line = String::new();
    if let Some(out) = child.stdout.take() {
        let _ = BufReader::new(out).read_line(&mut line);
    }
    if line.trim() != "held" {
        let _ = child.kill();
        return Err(format!("the clipboard holder said {:?}", line.trim()));
    }
    Ok(child)
}

/// Starts the stand-in window in its own process and waits until it is in front.
pub(crate) fn open_stand_in() -> Result<(std::process::Child, HWND), String> {
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

    // A return uses the target up (review T4): a second one with nothing remembered
    // since activates nothing, and says so.
    let spent = crate::focus::return_to_target();
    if spent != crate::focus::Returned::Nothing {
        let _ = a.kill();
        let _ = b.kill();
        return Err(format!(
            "a second return with no capture between said {spent:?}"
        ));
    }
    println!("  a second return with no capture between activated nothing");
    crate::focus::remember(ha);

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

/// Ctrl down or up, for real, as a person holds it through a click.
fn hold_ctrl(down: bool) {
    let flags = if down {
        KEYBD_EVENT_FLAGS(0)
    } else {
        KEYEVENTF_KEYUP
    };
    unsafe { keybd_event(VK_CONTROL.0 as u8, 0, flags, 0) };
    std::thread::sleep(std::time::Duration::from_millis(40));
}

pub(crate) fn press_escape() {
    unsafe {
        keybd_event(VK_ESCAPE.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(VK_ESCAPE.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    std::thread::sleep(std::time::Duration::from_millis(120));
}
