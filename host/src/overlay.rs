//! The selection overlay: hand-written Win32, one borderless window per display.
//!
//! Part 5 keeps the web view off the screen entirely, and F5 says this surface in
//! particular must not be one: it is the latency-critical and DPI-critical window. The
//! plan's cost section also says to give these windows their own thread with their own
//! message pump rather than living inside the application runtime's event loop, which is
//! what `select_region` does: the caller hands it a frozen frame on a dedicated thread and
//! gets back a rectangle or a cancellation.
//!
//! The selection is drawn on the FROZEN image, never on the live screen, so nothing can
//! change under the pointer mid-drag. A selection stays inside the display it started on,
//! which is the product rule in §3.2, and is why each window handles its own drag.
//!
//! Before a drag begins, the window under the pointer is the selection: its visible bounds
//! are lit and framed as the pointer moves, and a click with no drag captures them. The
//! windows are listed once, when the overlay comes up, because the picture under it is
//! frozen at that moment too. A drag past the system's own drag threshold takes over and
//! draws a free rectangle, exactly as before.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Instant;

use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, CreateSolidBrush,
    DeleteDC, DeleteObject, EndPaint, FillRect, GetDC, InvalidateRect, ReleaseDC, SelectObject,
    AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP,
    HBRUSH, HDC, HGDIOBJ, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumChildWindows,
    EnumWindows, GetClassNameW, GetCursorPos, GetForegroundWindow, GetMessageW, GetSystemMetrics,
    GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow,
    IsWindowVisible, LoadCursorW, PostMessageW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, SetTimer, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    GWL_EXSTYLE, IDC_CROSS, MSG, SM_CXDRAG, SM_CYDRAG, SW_SHOW, WM_APP, WM_DESTROY, WM_ERASEBKGND,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_TIMER, WNDCLASSEXW,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::capture::coords::DesktopRect;
use crate::capture::display::{monitors, MonitorInfo};
use crate::capture::Frame;

/// How dark the unselected area gets. 0 is untouched, 255 is black.
const DIM_ALPHA: u8 = 140;

/// Posted by the overlay to itself once its windows exist. Its dispatch is S0.7's mark
/// "overlay accepting input": the loop that delivers the pointer to these windows is running.
const WM_ACCEPTING: u32 = WM_APP + 1;

/// The marching frame: a light dash then a dark gap, these many pixels long, a band this
/// thick just inside the lit area, walking one pixel along the frame every tick. Rotem's
/// values, by eye, on 2026-09-14.
const ANTS_DASH: u32 = 12;
const ANTS_GAP: u32 = 8;
const ANTS_WIDTH: i32 = 2;
/// Pixels a second, Rotem's number. The walk is read off the clock, not counted in
/// ticks, and the pixel at every dash edge is blended by the fraction in between, so
/// the dashes glide instead of stepping.
const ANTS_SPEED: f64 = 12.0;
/// The frame's redraw interval: about sixty a second.
const ANTS_FRAME_MS: u32 = 16;
const ANTS_TIMER: usize = 1;
/// The dash, Rotem's #00B9F7, and the gap, near black.
const ANTS_DASH_RGB: (u8, u8, u8) = (0x00, 0xB9, 0xF7);
const ANTS_GAP_RGB: (u8, u8, u8) = (0x20, 0x20, 0x20);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A rectangle in desktop coordinates, physical pixels, origin at the primary display.
    Selected(DesktopRect),
    /// Escape, or a click with no drag. Nothing was captured and nothing was replaced.
    Cancelled,
}

/// One display's overlay window and the pixels it shows.
struct Surface {
    hwnd: HWND,
    monitor: MonitorInfo,
    /// The frozen slice for this display, as a GDI bitmap, bright and undimmed.
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    /// Whether this display's overlay has painted once, for S0.7's "overlay displayed".
    painted: bool,
}

struct Drag {
    /// Desktop coordinates of where the button went down.
    anchor: (i32, i32),
    current: (i32, i32),
    /// Which window owns this drag. A selection never leaves the display it started on.
    hwnd: isize,
    /// Whether the pointer has left the system's drag threshold since the button went
    /// down. Until it has, the press is a click on the window under it, and the window
    /// stays lit; once it has, the drag draws its own rectangle.
    moved: bool,
}

/// One top-level window as it stood when the overlay came up: its visible bounds in
/// desktop space, and its handle for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowBounds {
    hwnd: isize,
    rect: DesktopRect,
}

/// The window under the pointer, lit as the selection while nothing is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hover {
    /// The window's bounds, cut to the display the pointer is on (§3.2).
    rect: DesktopRect,
    /// The surface showing it, which is the display the pointer is on.
    surface: isize,
    /// The window itself, for the log line when it is picked.
    window: isize,
    /// The part of that window under the pointer, when it has parts: a browser's page
    /// area, a folder window's file list. None lights the whole window.
    part: Option<isize>,
}

struct State {
    surfaces: Vec<Surface>,
    drag: Option<Drag>,
    /// Every visible top-level window, front to back, as listed when the overlay came up.
    windows: Vec<WindowBounds>,
    /// A window's visible parts, its child windows at every depth, listed the first time
    /// the pointer rests on that window and kept for the rest of the selection.
    parts: HashMap<isize, Vec<WindowBounds>>,
    hover: Option<Hover>,
    /// How far the pointer may move from the press before it is a drag: the system's own
    /// value, so a click here is a click everywhere else on this machine.
    drag_threshold: (i32, i32),
    outcome: Outcome,
    /// A 1x1 black bitmap, stretched by AlphaBlend to dim whatever is not selected.
    dim_dc: HDC,
    dim_bitmap: HBITMAP,
    dim_previous: HGDIOBJ,
    /// The two inks of the marching frame: the light dash and the dark gap.
    border: HBRUSH,
    ink: HBRUSH,
    /// When the overlay came up: the dashes' walk is the time since, times the speed.
    started: Instant,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Shows the overlay on every display and blocks until a rectangle is chosen or cancelled.
///
/// Must be called on a thread that owns nothing else: it registers windows and runs its own
/// message loop until the selection ends.
pub fn select_region(frame: &Frame) -> Outcome {
    let prior_foreground = unsafe { GetForegroundWindow() };
    // Who was in front, elevation included: the S0.8 record of the hotkey and the focus
    // return against an elevated application is read from these lines.
    crate::log(&format!(
        "overlay: the foreground before it was {}",
        crate::platform::window_owner(prior_foreground).line()
    ));
    let list = monitors();
    if list.is_empty() {
        return Outcome::Cancelled;
    }
    // The windows as they stand right now, before the overlay's own exist. The picture
    // under the overlay is frozen at this moment, so the list is too.
    let windows = top_level_windows();

    unsafe {
        register_class();

        let Some(state) = build_state(frame, &list, windows) else {
            return Outcome::Cancelled;
        };
        STATE.with(|cell| *cell.borrow_mut() = Some(state));

        // Foreground goes to the first surface, so Escape has somewhere to land. Which
        // surface does not matter: every window shares one thread and one STATE, and
        // Escape ends the whole selection from any of them.
        STATE.with(|cell| {
            if let Some(state) = cell.borrow().as_ref() {
                if let Some(first) = state.surfaces.first() {
                    let _ = SetForegroundWindow(first.hwnd);
                    let _ = PostMessageW(Some(first.hwnd), WM_ACCEPTING, WPARAM(0), LPARAM(0));
                }
            }
        });

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let outcome = STATE.with(|cell| {
            let mut borrowed = cell.borrow_mut();
            let state = borrowed.take();
            match state {
                Some(state) => {
                    let outcome = state.outcome;
                    teardown(state);
                    outcome
                }
                None => Outcome::Cancelled,
            }
        });

        // Whatever happened, the window that was in front before the overlay appeared gets
        // the foreground back. On a cancellation that is the whole of "restores the previous
        // context"; after a selection it stops the desktop being left with nothing focused.
        if prior_foreground.is_invalid() {
            crate::log("overlay: no foreground to restore");
        } else if !IsWindow(Some(prior_foreground)).as_bool() {
            crate::log(&format!(
                "overlay: the window that was in front is gone, nothing restored; Windows left the foreground on {}",
                crate::platform::foreground_owner().line()
            ));
        } else {
            let restored = SetForegroundWindow(prior_foreground).as_bool();
            crate::log(&format!(
                "overlay: foreground {} to {}",
                if restored {
                    "restored"
                } else {
                    "NOT restored (refused)"
                },
                crate::platform::window_owner(prior_foreground).line()
            ));
        }
        outcome
    }
}

unsafe fn register_class() {
    // Registering twice is harmless: the second call fails and the class from the first
    // remains, which is what a repeated capture in one process needs.
    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: instance.into(),
        hCursor: unsafe { LoadCursorW(None, IDC_CROSS) }.unwrap_or_default(),
        lpszClassName: w!("ReconOverlay"),
        ..Default::default()
    };
    unsafe { RegisterClassExW(&class) };
}

unsafe fn build_state(
    frame: &Frame,
    list: &[MonitorInfo],
    windows: Vec<WindowBounds>,
) -> Option<State> {
    let screen_dc = unsafe { GetDC(None) };
    if screen_dc.is_invalid() {
        return None;
    }

    let mut surfaces = Vec::new();
    for monitor in list {
        match unsafe { build_surface(frame, monitor, screen_dc) } {
            Some(surface) => surfaces.push(surface),
            None => {
                eprintln!(
                    "overlay: no surface for {} at {},{}",
                    monitor.device, monitor.rect.x, monitor.rect.y
                );
            }
        }
    }
    if surfaces.is_empty() {
        unsafe { ReleaseDC(None, screen_dc) };
        return None;
    }

    // The dimming layer: one black pixel, stretched over whatever is not selected. Its
    // alpha byte is 255 on purpose: the blend writes the window's alpha channel too, and
    // a transparent source left the dimmed pixels at 45% alpha, which the compositor then
    // showed over the identical live desktop, cancelling the dim exactly. The screen
    // looked undimmed while every blend call reported success.
    let dim_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    let mut info = solid_header(1, 1);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let dim_bitmap = match unsafe {
        CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
    } {
        Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
        _ => {
            unsafe { ReleaseDC(None, screen_dc) };
            return None;
        }
    };
    unsafe { std::ptr::copy_nonoverlapping([0u8, 0, 0, 255].as_ptr(), bits as *mut u8, 4) };
    let dim_previous = unsafe { SelectObject(dim_dc, HGDIOBJ(dim_bitmap.0)) };
    std::hint::black_box(&mut info);

    unsafe { ReleaseDC(None, screen_dc) };

    Some(State {
        surfaces,
        drag: None,
        windows,
        parts: HashMap::new(),
        hover: None,
        drag_threshold: unsafe {
            (
                GetSystemMetrics(SM_CXDRAG).max(0),
                GetSystemMetrics(SM_CYDRAG).max(0),
            )
        },
        outcome: Outcome::Cancelled,
        dim_dc,
        dim_bitmap,
        dim_previous,
        border: unsafe { CreateSolidBrush(colorref(ANTS_DASH_RGB)) },
        ink: unsafe { CreateSolidBrush(colorref(ANTS_GAP_RGB)) },
        started: Instant::now(),
    })
}

/// Copies this display's slice out of the frozen frame into a GDI bitmap, and makes the
/// window that will show it.
unsafe fn build_surface(frame: &Frame, monitor: &MonitorInfo, screen_dc: HDC) -> Option<Surface> {
    let w = monitor.rect.width as i32;
    let h = monitor.rect.height as i32;
    if w <= 0 || h <= 0 {
        return None;
    }

    let dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    if dc.is_invalid() {
        return None;
    }

    let mut info = solid_header(w, h);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap = match unsafe {
        CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
    } {
        Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
        _ => {
            let _ = unsafe { DeleteDC(dc) };
            return None;
        }
    };

    // The frame is RGBA in image space; GDI wants BGRA. This is the only place that knows,
    // and it copies row by row out of the frame slice belonging to this display.
    let src_x = (monitor.rect.x - frame.geometry.origin_x) as isize;
    let src_y = (monitor.rect.y - frame.geometry.origin_y) as isize;
    let frame_stride = frame.geometry.width as isize * 4;
    let dst = bits as *mut u8;
    for row in 0..h as isize {
        let src_row = (src_y + row) * frame_stride + src_x * 4;
        if src_row < 0 || src_row + (w as isize) * 4 > frame.rgba.len() as isize {
            continue;
        }
        for col in 0..w as isize {
            let s = (src_row + col * 4) as usize;
            let d = ((row * w as isize + col) * 4) as usize;
            unsafe {
                *dst.add(d) = frame.rgba[s + 2];
                *dst.add(d + 1) = frame.rgba[s + 1];
                *dst.add(d + 2) = frame.rgba[s];
                *dst.add(d + 3) = 255;
            }
        }
    }
    std::hint::black_box(&mut info);

    let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };

    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("ReconOverlay"),
            PCWSTR::null(),
            WS_POPUP,
            monitor.rect.x,
            monitor.rect.y,
            w,
            h,
            None,
            None,
            Some(instance.into()),
            None,
        )
    };
    let hwnd = match hwnd {
        Ok(hwnd) if !hwnd.is_invalid() => hwnd,
        _ => {
            unsafe { SelectObject(dc, previous) };
            let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
            let _ = unsafe { DeleteDC(dc) };
            return None;
        }
    };
    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
    // The frame's clock. It dies with the window; a tick with nothing lit does nothing.
    unsafe { SetTimer(Some(hwnd), ANTS_TIMER, ANTS_FRAME_MS, None) };

    Some(Surface {
        hwnd,
        monitor: monitor.clone(),
        dc,
        bitmap,
        previous,
        painted: false,
    })
}

fn solid_header(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

unsafe fn teardown(state: State) {
    for surface in &state.surfaces {
        let _ = unsafe { DestroyWindow(surface.hwnd) };
        unsafe { SelectObject(surface.dc, surface.previous) };
        let _ = unsafe { DeleteObject(HGDIOBJ(surface.bitmap.0)) };
        let _ = unsafe { DeleteDC(surface.dc) };
    }
    unsafe { SelectObject(state.dim_dc, state.dim_previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(state.dim_bitmap.0)) };
    let _ = unsafe { DeleteDC(state.dim_dc) };
    let _ = unsafe { DeleteObject(HGDIOBJ(state.border.0)) };
    let _ = unsafe { DeleteObject(HGDIOBJ(state.ink.0)) };
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1), // every pixel is painted in WM_PAINT
        WM_PAINT => {
            unsafe { paint(hwnd) };
            // S0.7: "overlay displayed" is the moment every display's overlay has painted once.
            let all_painted = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                let surface = state
                    .surfaces
                    .iter_mut()
                    .find(|s| s.hwnd.0 as isize == hwnd.0 as isize)?;
                if surface.painted {
                    return None;
                }
                surface.painted = true;
                Some(state.surfaces.iter().all(|s| s.painted))
            });
            if all_painted == Some(true) {
                crate::marks::mark(crate::marks::OVERLAY_DISPLAYED);
            }
            LRESULT(0)
        }
        WM_ACCEPTING => {
            crate::marks::mark(crate::marks::OVERLAY_ACCEPTING);
            // The pointer may already be resting on a window when the overlay appears, and
            // no move arrives to say so: light that window now rather than on the first
            // twitch.
            let mut at = POINT::default();
            if unsafe { GetCursorPos(&mut at) }.is_ok() {
                let changed = STATE.with(|cell| {
                    let mut borrowed = cell.borrow_mut();
                    let state = borrowed.as_mut()?;
                    let surface = state.surface_at(at.x, at.y)?;
                    Some(state.update_hover(surface, (at.x, at.y)))
                });
                if let Some(changed) = changed {
                    unsafe { invalidate_hover_change(changed) };
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let point = point_of(lparam);
            STATE.with(|cell| {
                if let Some(state) = cell.borrow_mut().as_mut() {
                    if let Some(monitor) = state.monitor_of(hwnd).map(|m| m.rect) {
                        let desktop = (
                            (monitor.x + point.x)
                                .clamp(monitor.x, monitor.x + monitor.width as i32),
                            (monitor.y + point.y)
                                .clamp(monitor.y, monitor.y + monitor.height as i32),
                        );
                        state.drag = Some(Drag {
                            anchor: desktop,
                            current: desktop,
                            hwnd: hwnd.0 as isize,
                            moved: false,
                        });
                    }
                }
            });
            unsafe { SetCapture(hwnd) };
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let point = point_of(lparam);
            // With no button down, the move only changes which window is lit.
            let hover_change = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                if state.drag.is_some() {
                    return None;
                }
                let monitor = state.monitor_of(hwnd)?.rect;
                let desktop = (monitor.x + point.x, monitor.y + point.y);
                Some(state.update_hover(hwnd.0 as isize, desktop))
            });
            if let Some(changed) = hover_change {
                unsafe { invalidate_hover_change(changed) };
                return LRESULT(0);
            }
            let invalidate = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                let monitor = state.monitor_of(hwnd)?.rect;
                let origin = (monitor.x, monitor.y);
                let threshold = state.drag_threshold;
                let mut lit = None;
                let drag = state.drag.as_mut()?;
                if drag.hwnd != hwnd.0 as isize {
                    return None;
                }
                // Inside the threshold the press is still a click, and the lit window is
                // still the selection; nothing to redraw.
                if !drag.moved {
                    let dx = (monitor.x + point.x - drag.anchor.0).abs();
                    let dy = (monitor.y + point.y - drag.anchor.1).abs();
                    if dx <= threshold.0 && dy <= threshold.1 {
                        return None;
                    }
                    drag.moved = true;
                    // The drag has taken over: the lit window goes back to dim, once, and
                    // is not consulted again until the overlay ends.
                    lit = state.hover.take().map(|h| h.rect);
                }
                let drag = state.drag.as_mut()?;
                let dragged = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
                let before = match lit {
                    Some(rect) => union_of(rect, dragged),
                    None => dragged,
                };
                // Clamped to the display the drag started on. Mouse capture keeps
                // delivering moves past the edge, and without this a drag that overshoots
                // produces a rectangle outside the frozen frame, which the conversion then
                // refuses: the user would lose the whole capture for dragging too far.
                // §3.2 says a selection stays inside its starting display, and this is that
                // rule rather than an error message.
                let max_x = monitor.x + monitor.width as i32;
                let max_y = monitor.y + monitor.height as i32;
                drag.current = (
                    (origin.0 + point.x).clamp(monitor.x, max_x),
                    (origin.1 + point.y).clamp(monitor.y, max_y),
                );
                let after = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
                Some((before, after, origin))
            });
            if let Some((before, after, origin)) = invalidate {
                // Only the union of the old and the new rectangle changed, so only that is
                // repainted. A full repaint of a 5120 wide display per mouse move is the
                // easiest way to make this surface feel slow.
                unsafe { invalidate_union(hwnd, origin, before, after) };
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = unsafe { ReleaseCapture() };
            let finished = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                let drag = state.drag.take()?;
                if !drag.moved {
                    // A click with no drag captures the window that was lit under it. With
                    // nothing lit, on bare desktop on a display without one, it is a
                    // cancellation, not a zero-pixel capture.
                    match state.hover {
                        Some(hover) if hover.surface == hwnd.0 as isize => {
                            state.outcome = Outcome::Selected(hover.rect);
                            crate::log(&format!(
                                "overlay: a click picked the window {}{} at {}x{} desktop {},{}",
                                crate::platform::window_owner(HWND(hover.window as *mut _)).line(),
                                match hover.part {
                                    Some(part) =>
                                        format!(", its part {:?}", class_of(HWND(part as *mut _))),
                                    None => String::new(),
                                },
                                hover.rect.width,
                                hover.rect.height,
                                hover.rect.x,
                                hover.rect.y
                            ));
                            crate::marks::mark(crate::marks::SELECTION_COMPLETED);
                        }
                        _ => state.outcome = Outcome::Cancelled,
                    }
                    return Some(());
                }
                let rect = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
                if rect.is_empty() {
                    // A drag that moved past the threshold on one axis only is a line, not
                    // a capture.
                    state.outcome = Outcome::Cancelled;
                } else {
                    state.outcome = Outcome::Selected(rect);
                    crate::marks::mark(crate::marks::SELECTION_COMPLETED);
                }
                Some(())
            });
            if finished.is_some() {
                unsafe { PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 as u16 == VK_ESCAPE.0 => {
            STATE.with(|cell| {
                if let Some(state) = cell.borrow_mut().as_mut() {
                    state.drag = None;
                    state.outcome = Outcome::Cancelled;
                }
            });
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == ANTS_TIMER => {
            // One frame of the dashes, and only the frame itself is repainted: four thin
            // strips, not the lit area, so the walk costs nothing to speak of. The walk
            // is read off the clock at paint time, so a late frame does not slow it.
            let ring = STATE.with(|cell| {
                let borrowed = cell.borrow();
                let state = borrowed.as_ref()?;
                let sel = state.client_selection(hwnd)?;
                Some(ring_rects(sel))
            });
            if let Some(ring) = ring {
                for strip in ring {
                    let _ = unsafe { InvalidateRect(Some(hwnd), Some(&strip), false) };
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn point_of(lparam: LPARAM) -> POINT {
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
    POINT { x, y }
}

impl State {
    /// Desktop coordinates of a window's client origin, which for these windows is the
    /// display's own top-left.
    fn origin_of(&self, hwnd: HWND) -> Option<(i32, i32)> {
        self.surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == hwnd.0 as isize)
            .map(|s| (s.monitor.rect.x, s.monitor.rect.y))
    }

    fn monitor_of(&self, hwnd: HWND) -> Option<&MonitorInfo> {
        self.surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == hwnd.0 as isize)
            .map(|s| &s.monitor)
    }

    fn surface_of(&self, hwnd: HWND) -> Option<&Surface> {
        self.surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == hwnd.0 as isize)
    }

    /// The live selection in this window's client coordinates: the dragged rectangle once
    /// a drag is under way, otherwise the lit window, if either belongs to this surface.
    fn client_selection(&self, hwnd: HWND) -> Option<RECT> {
        let origin = self.origin_of(hwnd)?;
        let rect = match self.drag.as_ref() {
            Some(drag) if drag.moved => {
                if drag.hwnd != hwnd.0 as isize {
                    return None;
                }
                DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                )
            }
            _ => self
                .hover
                .filter(|h| h.surface == hwnd.0 as isize)
                .map(|h| h.rect)?,
        };
        if rect.is_empty() {
            return None;
        }
        Some(RECT {
            left: rect.x - origin.0,
            top: rect.y - origin.1,
            right: rect.x - origin.0 + rect.width as i32,
            bottom: rect.y - origin.1 + rect.height as i32,
        })
    }

    /// The surface whose display holds a desktop point.
    fn surface_at(&self, x: i32, y: i32) -> Option<isize> {
        self.surfaces
            .iter()
            .find(|s| contains(s.monitor.rect, x, y))
            .map(|s| s.hwnd.0 as isize)
    }

    /// Re-reads which window is under the pointer and lights it, cut to the display the
    /// pointer is on. Returns what has to be repainted: the window that was lit before,
    /// on its surface, and the one lit now, on this one.
    fn update_hover(&mut self, surface: isize, desktop: (i32, i32)) -> HoverChange {
        let display = self
            .surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == surface)
            .map(|s| s.monitor.rect);
        let hit = display.and_then(|display| {
            let hit = window_at(&self.windows, desktop.0, desktop.1)?;
            Some((display, hit))
        });
        let next = hit.and_then(|(display, hit)| {
            // The smallest visible part of the window under the pointer is the selection:
            // a browser's page area rather than the browser, the way Snagit picks it. A
            // part can reach past the window's frame, so it is cut to the frame first, and
            // a window with no part under the pointer is lit whole.
            let part = smallest_at(self.parts_of(hit.hwnd), desktop.0, desktop.1);
            let rect = part
                .and_then(|p| clamp_to(p.rect, hit.rect))
                .unwrap_or(hit.rect);
            let rect = clamp_to(rect, display)?;
            Some(Hover {
                rect,
                surface,
                window: hit.hwnd,
                part: part.map(|p| p.hwnd),
            })
        });
        let previous = self.hover;
        if previous == next {
            return HoverChange::default();
        }
        self.hover = next;
        HoverChange {
            before: previous.map(|h| (h.surface, self.origin_of_surface(h.surface), h.rect)),
            after: next.map(|h| (h.surface, self.origin_of_surface(h.surface), h.rect)),
        }
    }

    /// The parts of a top-level window, listed on first use.
    fn parts_of(&mut self, window: isize) -> &[WindowBounds] {
        self.parts
            .entry(window)
            .or_insert_with(|| parts_of_window(window))
    }

    /// How far the dashes have walked: whole pixels, and the fraction of the next one.
    fn phase(&self) -> (u32, f64) {
        let walked = self.started.elapsed().as_secs_f64() * ANTS_SPEED;
        let whole = walked.floor();
        ((whole as u64 % u32::MAX as u64) as u32, walked - whole)
    }

    fn origin_of_surface(&self, surface: isize) -> (i32, i32) {
        self.surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == surface)
            .map(|s| (s.monitor.rect.x, s.monitor.rect.y))
            .unwrap_or((0, 0))
    }
}

/// What a change of the lit window leaves to repaint: each entry is a surface, its
/// desktop origin, and the rectangle on it.
#[derive(Debug, Default, Clone, Copy)]
struct HoverChange {
    before: Option<(isize, (i32, i32), DesktopRect)>,
    after: Option<(isize, (i32, i32), DesktopRect)>,
}

unsafe fn invalidate_hover_change(change: HoverChange) {
    for (surface, origin, rect) in change.before.into_iter().chain(change.after) {
        unsafe { invalidate_union(HWND(surface as *mut _), origin, rect, rect) };
    }
}

fn contains(rect: DesktopRect, x: i32, y: i32) -> bool {
    x >= rect.x && y >= rect.y && x < rect.x + rect.width as i32 && y < rect.y + rect.height as i32
}

/// The frontmost window under a desktop point, given the list front to back.
fn window_at(windows: &[WindowBounds], x: i32, y: i32) -> Option<WindowBounds> {
    windows.iter().copied().find(|w| contains(w.rect, x, y))
}

/// The part of a rectangle inside another, or nothing when they do not meet.
fn clamp_to(rect: DesktopRect, bounds: DesktopRect) -> Option<DesktopRect> {
    let left = rect.x.max(bounds.x);
    let top = rect.y.max(bounds.y);
    let right = (rect.x + rect.width as i32).min(bounds.x + bounds.width as i32);
    let bottom = (rect.y + rect.height as i32).min(bounds.y + bounds.height as i32);
    if left >= right || top >= bottom {
        return None;
    }
    Some(DesktopRect::from_points(left, top, right, bottom))
}

/// The smallest rectangle holding both.
fn union_of(a: DesktopRect, b: DesktopRect) -> DesktopRect {
    DesktopRect::from_points(
        a.x.min(b.x),
        a.y.min(b.y),
        (a.x + a.width as i32).max(b.x + b.width as i32),
        (a.y + a.height as i32).max(b.y + b.height as i32),
    )
}

/// The smallest of the parts under a desktop point: the deepest thing the pointer is on.
fn smallest_at(parts: &[WindowBounds], x: i32, y: i32) -> Option<WindowBounds> {
    parts
        .iter()
        .copied()
        .filter(|p| contains(p.rect, x, y))
        .min_by_key(|p| p.rect.width as u64 * p.rect.height as u64)
}

/// A window's visible parts at every depth, with the rectangle each one covers. A part
/// is not asked whether it takes clicks: a browser's page area is click-through by
/// design, its input going to the window, and it is still the part the eye sees.
fn parts_of_window(window: isize) -> Vec<WindowBounds> {
    let mut out: Vec<WindowBounds> = Vec::new();
    unsafe {
        let _ = EnumChildWindows(
            Some(HWND(window as *mut _)),
            Some(collect_part),
            LPARAM(&mut out as *mut Vec<WindowBounds> as isize),
        );
    }
    out
}

unsafe extern "system" fn collect_part(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = unsafe { &mut *(lparam.0 as *mut Vec<WindowBounds>) };
    let keep = BOOL(1);
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return keep;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return keep;
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return keep;
        }
        out.push(WindowBounds {
            hwnd: hwnd.0 as isize,
            rect: DesktopRect {
                x: rect.left,
                y: rect.top,
                width: width as u32,
                height: height as u32,
            },
        });
    }
    keep
}

/// A window's class name, for the log.
fn class_of(hwnd: HWND) -> String {
    let mut buffer = [0u16; 128];
    let n = unsafe { GetClassNameW(hwnd, &mut buffer) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..n])
}

/// Every window a click could land on, front to back: visible, not minimised, not cloaked
/// (kept by Windows but not drawn: another virtual desktop, a suspended app), not
/// click-through, and never Recon's own. The bounds are the visible frame, without the
/// invisible resize border a Windows 10 or 11 window carries around its edge, so a pick
/// is the window the user sees and not a strip of its neighbour.
fn top_level_windows() -> Vec<WindowBounds> {
    let mut out: Vec<WindowBounds> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_window),
            LPARAM(&mut out as *mut Vec<WindowBounds> as isize),
        );
    }
    out
}

unsafe extern "system" fn collect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = unsafe { &mut *(lparam.0 as *mut Vec<WindowBounds>) };
    let keep = BOOL(1);
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
            return keep;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == std::process::id() {
            return keep;
        }
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if ex_style & WS_EX_TRANSPARENT.0 != 0 {
            return keep;
        }
        let mut cloaked = 0u32;
        let asked = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut c_void,
            std::mem::size_of::<u32>() as u32,
        );
        if asked.is_ok() && cloaked != 0 {
            return keep;
        }
        let mut rect = RECT::default();
        let bounds = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut RECT as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        );
        if bounds.is_err() && GetWindowRect(hwnd, &mut rect).is_err() {
            return keep;
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return keep;
        }
        out.push(WindowBounds {
            hwnd: hwnd.0 as isize,
            rect: DesktopRect {
                x: rect.left,
                y: rect.top,
                width: width as u32,
                height: height as u32,
            },
        });
    }
    keep
}

unsafe fn invalidate_union(
    hwnd: HWND,
    origin: (i32, i32),
    before: DesktopRect,
    after: DesktopRect,
) {
    let union = RECT {
        left: before.x.min(after.x) - origin.0 - 2,
        top: before.y.min(after.y) - origin.1 - 2,
        right: (before.x + before.width as i32).max(after.x + after.width as i32) - origin.0 + 2,
        bottom: (before.y + before.height as i32).max(after.y + after.height as i32) - origin.1 + 2,
    };
    let _ = unsafe { InvalidateRect(Some(hwnd), Some(&union), false) };
}

unsafe fn paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    let dirty = ps.rcPaint;
    let width = dirty.right - dirty.left;
    let height = dirty.bottom - dirty.top;

    if width > 0 && height > 0 {
        STATE.with(|cell| {
            let borrowed = cell.borrow();
            let Some(state) = borrowed.as_ref() else {
                return;
            };
            let Some(surface) = state.surface_of(hwnd) else {
                return;
            };

            // 1. the frozen pixels, undimmed, for the dirty region
            let _ = unsafe {
                BitBlt(
                    hdc,
                    dirty.left,
                    dirty.top,
                    width,
                    height,
                    Some(surface.dc),
                    dirty.left,
                    dirty.top,
                    SRCCOPY,
                )
            };

            // 2. black at DIM_ALPHA over the dirty region, minus the selection
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: DIM_ALPHA,
                AlphaFormat: 0,
            };
            let selection = state.client_selection(hwnd);
            for part in dim_parts(dirty, selection) {
                let pw = part.right - part.left;
                let ph = part.bottom - part.top;
                if pw > 0 && ph > 0 {
                    let _ = unsafe {
                        AlphaBlend(
                            hdc,
                            part.left,
                            part.top,
                            pw,
                            ph,
                            state.dim_dc,
                            0,
                            0,
                            1,
                            1,
                            blend,
                        )
                    };
                }
            }

            // 3. the marching frame: light dashes and dark gaps, one pixel wide, walking
            // along the frame with the phase. Only the runs inside the dirty region are
            // drawn, which on a tick is the frame's four strips.
            if let Some(sel) = selection {
                // The walk: whole pixels shift the pattern, and the fraction in between
                // is the blend of the pixel at every edge, dash into gap and gap into
                // dash. Two brushes per frame, made and freed here.
                let (whole, fraction) = state.phase();
                let to_gap = unsafe {
                    CreateSolidBrush(colorref(mix(ANTS_DASH_RGB, ANTS_GAP_RGB, fraction)))
                };
                let to_dash = unsafe {
                    CreateSolidBrush(colorref(mix(ANTS_GAP_RGB, ANTS_DASH_RGB, fraction)))
                };
                for ant in ant_runs(sel, whole) {
                    let run = ant.run;
                    if run.right <= dirty.left
                        || run.left >= dirty.right
                        || run.bottom <= dirty.top
                        || run.top >= dirty.bottom
                    {
                        continue;
                    }
                    let brush = if ant.light { state.border } else { state.ink };
                    let _ = unsafe { FillRect(hdc, &run, brush) };
                    if let Some(edge) = ant.edge {
                        let blend = if ant.light { to_gap } else { to_dash };
                        let _ = unsafe { FillRect(hdc, &edge, blend) };
                    }
                }
                let _ = unsafe { DeleteObject(HGDIOBJ(to_gap.0)) };
                let _ = unsafe { DeleteObject(HGDIOBJ(to_dash.0)) };
            }
        });
    }

    let _ = unsafe { EndPaint(hwnd, &ps) };
}

/// The frame's thickness inside a selection: the set width, or less when the selection
/// is too small to hold two bands.
fn ants_width(sel: RECT) -> i32 {
    let w = (sel.right - sel.left).max(0);
    let h = (sel.bottom - sel.top).max(0);
    ANTS_WIDTH.min(w / 2).min(h / 2).max(1)
}

/// The frame's four bands, in client coordinates, just inside the lit area: what a tick
/// repaints. Top and bottom take the corners; the sides run between them.
fn ring_rects(sel: RECT) -> [RECT; 4] {
    let t = ants_width(sel);
    [
        RECT {
            left: sel.left,
            top: sel.top,
            right: sel.right,
            bottom: sel.top + t,
        },
        RECT {
            left: sel.left,
            top: sel.bottom - t,
            right: sel.right,
            bottom: sel.bottom,
        },
        RECT {
            left: sel.left,
            top: sel.top + t,
            right: sel.left + t,
            bottom: sel.bottom - t,
        },
        RECT {
            left: sel.right - t,
            top: sel.top + t,
            right: sel.right,
            bottom: sel.bottom - t,
        },
    ]
}

/// Whether a point this far along the frame's path is on a dash rather than a gap.
fn ink_at(position: i64) -> bool {
    let period = (ANTS_DASH + ANTS_GAP) as i64;
    position.rem_euclid(period) < ANTS_DASH as i64
}

/// One run of the marching frame: its rectangle, its ink, and the one pixel at its end
/// where the ink changes, when it ends at a dash edge rather than at a band's end. That
/// pixel is the one blended by the fraction of the walk.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ant {
    run: RECT,
    light: bool,
    edge: Option<RECT>,
}

/// A colour for a brush: red, green, blue, which a COLORREF stores the other way round.
fn colorref((r, g, b): (u8, u8, u8)) -> COLORREF {
    COLORREF((b as u32) << 16 | (g as u32) << 8 | r as u32)
}

/// The colour a fraction of the way from one to the other, per channel.
fn mix(from: (u8, u8, u8), to: (u8, u8, u8), fraction: f64) -> (u8, u8, u8) {
    let f = fraction.clamp(0.0, 1.0);
    let one = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * f).round() as u8;
    (one(from.0, to.0), one(from.1, to.1), one(from.2, to.2))
}

/// The dashes of the marching frame: runs along the four bands, clockwise from the
/// top-left corner, each light or dark, the pattern shifted by the phase so the dashes
/// walk. The walk is one continuous path, so a dash turns a corner instead of restarting.
fn ant_runs(sel: RECT, phase: u32) -> Vec<Ant> {
    let w = (sel.right - sel.left).max(0);
    let h = (sel.bottom - sel.top).max(0);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let t = ants_width(sel);
    let period = (ANTS_DASH + ANTS_GAP) as i64;
    // Each band: where its path starts, how long it is, which way it runs, and the band's
    // rectangle across the path. Top left to right, right top to bottom, bottom right to
    // left, left bottom to top.
    let sides = (h - 2 * t).max(0);
    let ring = ring_rects(sel);
    let bands: [(i32, i32, (i32, i32), RECT); 4] = [
        (sel.left, w, (1, 0), ring[0]),
        (sel.top + t, sides, (0, 1), ring[3]),
        (sel.right - 1, w, (-1, 0), ring[1]),
        (sel.bottom - t - 1, sides, (0, -1), ring[2]),
    ];
    let mut runs = Vec::new();
    let mut walked: i64 = 0;
    for (start, length, (dx, dy), band) in bands {
        let mut at = 0;
        while at < length {
            // The run ends at the next dash or gap boundary, or the band's end.
            let position = walked + at as i64 + phase as i64;
            let into = position.rem_euclid(period);
            let light = ink_at(position);
            let to_boundary = if light {
                ANTS_DASH as i64 - into
            } else {
                period - into
            };
            let take = (to_boundary as i32).min(length - at);
            let from = start + (dx + dy) * at;
            let to = start + (dx + dy) * (at + take - 1);
            let (lo, hi) = (from.min(to), from.max(to) + 1);
            // Across the band, along the path from lo to hi; the edge pixel is the last
            // one along the path, which is `to`.
            let across = |a: i32, b: i32| {
                if dx != 0 {
                    RECT {
                        left: a,
                        top: band.top,
                        right: b,
                        bottom: band.bottom,
                    }
                } else {
                    RECT {
                        left: band.left,
                        top: a,
                        right: band.right,
                        bottom: b,
                    }
                }
            };
            let edge = if take as i64 == to_boundary {
                Some(across(to, to + 1))
            } else {
                None
            };
            runs.push(Ant {
                run: across(lo, hi),
                light,
                edge,
            });
            at += take;
        }
        walked += length as i64;
    }
    runs
}

/// The dirty region minus the selection, as up to four rectangles. Splitting it this way
/// keeps the dim in one pass and leaves the selected pixels untouched, rather than dimming
/// everything and then repainting the bright part on top.
fn dim_parts(dirty: RECT, selection: Option<RECT>) -> Vec<RECT> {
    let Some(sel) = selection else {
        return vec![dirty];
    };
    let inner = RECT {
        left: sel.left.max(dirty.left),
        top: sel.top.max(dirty.top),
        right: sel.right.min(dirty.right),
        bottom: sel.bottom.min(dirty.bottom),
    };
    if inner.left >= inner.right || inner.top >= inner.bottom {
        return vec![dirty];
    }
    vec![
        RECT {
            left: dirty.left,
            top: dirty.top,
            right: dirty.right,
            bottom: inner.top,
        },
        RECT {
            left: dirty.left,
            top: inner.bottom,
            right: dirty.right,
            bottom: dirty.bottom,
        },
        RECT {
            left: dirty.left,
            top: inner.top,
            right: inner.left,
            bottom: inner.bottom,
        },
        RECT {
            left: inner.right,
            top: inner.top,
            right: dirty.right,
            bottom: inner.bottom,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(l: i32, t: i32, r: i32, b: i32) -> RECT {
        RECT {
            left: l,
            top: t,
            right: r,
            bottom: b,
        }
    }

    #[test]
    fn with_no_selection_the_whole_dirty_region_is_dimmed() {
        let d = rect(0, 0, 100, 100);
        assert_eq!(dim_parts(d, None).len(), 1);
    }

    #[test]
    fn the_four_parts_never_overlap_the_selection() {
        let d = rect(0, 0, 100, 100);
        let sel = rect(20, 30, 60, 70);
        for part in dim_parts(d, Some(sel)) {
            let overlaps = part.left < sel.right
                && part.right > sel.left
                && part.top < sel.bottom
                && part.bottom > sel.top;
            assert!(!overlaps, "part {part:?} overlaps the selection");
        }
    }

    #[test]
    fn the_four_parts_cover_everything_the_selection_does_not() {
        let d = rect(0, 0, 40, 40);
        let sel = rect(10, 10, 20, 20);
        let parts = dim_parts(d, Some(sel));
        let mut covered = 0usize;
        for y in d.top..d.bottom {
            for x in d.left..d.right {
                let in_sel = x >= sel.left && x < sel.right && y >= sel.top && y < sel.bottom;
                let in_parts = parts
                    .iter()
                    .any(|p| x >= p.left && x < p.right && y >= p.top && y < p.bottom);
                if in_sel {
                    assert!(!in_parts, "{x},{y} is selected and also dimmed");
                } else {
                    assert!(in_parts, "{x},{y} is neither selected nor dimmed");
                    covered += 1;
                }
            }
        }
        assert_eq!(covered, 40 * 40 - 10 * 10);
    }

    #[test]
    fn a_selection_outside_the_dirty_region_dims_all_of_it() {
        let d = rect(0, 0, 40, 40);
        let sel = rect(100, 100, 120, 120);
        assert_eq!(dim_parts(d, Some(sel)).len(), 1);
    }

    fn desktop(x: i32, y: i32, width: u32, height: u32) -> DesktopRect {
        DesktopRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn the_frontmost_window_under_the_point_wins() {
        // A small window in front of a large one: a point on both picks the front one, a
        // point only on the large one picks that, and a point on neither picks nothing.
        let front = WindowBounds {
            hwnd: 1,
            rect: desktop(100, 100, 200, 100),
        };
        let back = WindowBounds {
            hwnd: 2,
            rect: desktop(0, 0, 1000, 1000),
        };
        let list = [front, back];
        assert_eq!(window_at(&list, 150, 150), Some(front));
        assert_eq!(window_at(&list, 50, 50), Some(back));
        assert_eq!(window_at(&list, 1000, 1000), None);
        // The right and bottom edges are outside, as in every other rectangle here.
        assert_eq!(window_at(&list, 300, 150), Some(back));
    }

    #[test]
    fn a_window_is_cut_to_the_display_the_pointer_is_on() {
        let display = desktop(0, 0, 1920, 1080);
        // Hangs off the right and the bottom: the visible part stays.
        assert_eq!(
            clamp_to(desktop(1800, 1000, 400, 300), display),
            Some(desktop(1800, 1000, 120, 80))
        );
        // Wholly on another display: nothing to light here.
        assert_eq!(clamp_to(desktop(1920, 0, 400, 300), display), None);
        // A maximised window's frame reaches past the display by its border, and comes
        // back as the display itself.
        assert_eq!(
            clamp_to(desktop(-8, -8, 1936, 1096), display),
            Some(display)
        );
    }

    #[test]
    fn the_smallest_part_under_the_point_wins() {
        // A browser's shape: a part covering the whole client area, and inside it the
        // page area. On the page the page wins; on the toolbar only the big part is
        // there; off both there is nothing.
        let client = WindowBounds {
            hwnd: 10,
            rect: desktop(0, 0, 1705, 1391),
        };
        let page = WindowBounds {
            hwnd: 11,
            rect: desktop(0, 121, 1705, 1270),
        };
        let parts = [client, page];
        assert_eq!(smallest_at(&parts, 800, 700), Some(page));
        assert_eq!(smallest_at(&parts, 800, 50), Some(client));
        assert_eq!(smallest_at(&parts, 1705, 700), None);
    }

    /// Every pixel the runs cover, with its ink, in visiting order.
    fn painted(sel: RECT, phase: u32) -> Vec<((i32, i32), bool)> {
        let mut out = Vec::new();
        for ant in ant_runs(sel, phase) {
            let run = ant.run;
            for y in run.top..run.bottom {
                for x in run.left..run.right {
                    out.push(((x, y), ant.light));
                }
            }
        }
        out
    }

    #[test]
    fn the_edge_pixel_sits_at_the_end_of_a_run_that_meets_a_dash_edge() {
        let sel = rect(0, 0, 100, 60);
        let runs = ant_runs(sel, 0);
        // The first run is a whole dash along the top band, ending at a dash edge: its
        // edge pixel is its last column, the band's full thickness.
        let first = runs[0];
        assert!(first.light);
        let edge = first.edge.expect("the first dash ends at an edge");
        assert_eq!(
            (edge.left, edge.right),
            (first.run.right - 1, first.run.right)
        );
        assert_eq!((edge.top, edge.bottom), (first.run.top, first.run.bottom));
        // A run cut short by the band's end has no edge pixel: it continues round the
        // corner in the next band, so the ink does not change there.
        let cut = runs
            .iter()
            .find(|a| a.edge.is_none())
            .expect("some run ends at a band's end");
        let next = runs
            .iter()
            .skip_while(|a| *a != cut)
            .nth(1)
            .expect("a run follows it");
        assert_eq!(cut.light, next.light);
        // The edge pixel is always inside its run, so the blend never spills.
        for ant in &runs {
            if let Some(e) = ant.edge {
                assert!(
                    e.left >= ant.run.left
                        && e.right <= ant.run.right
                        && e.top >= ant.run.top
                        && e.bottom <= ant.run.bottom
                );
            }
        }
    }

    #[test]
    fn the_blend_walks_from_one_ink_to_the_other() {
        assert_eq!(mix(ANTS_DASH_RGB, ANTS_GAP_RGB, 0.0), ANTS_DASH_RGB);
        assert_eq!(mix(ANTS_DASH_RGB, ANTS_GAP_RGB, 1.0), ANTS_GAP_RGB);
        let half = mix((0, 100, 200), (100, 100, 0), 0.5);
        assert_eq!(half, (50, 100, 100));
        // A COLORREF stores blue in the high byte and red in the low one.
        assert_eq!(colorref((0x00, 0xB9, 0xF7)).0, 0x00F7B900);
    }

    #[test]
    fn the_band_covers_the_frame_once_at_its_width() {
        let sel = rect(10, 20, 70, 55); // 60 by 35
        let t = ANTS_WIDTH;
        let pixels = painted(sel, 0);
        // Two full-width bands and two side bands between them, each pixel painted once.
        let expected = 2 * 60 * t + 2 * (35 - 2 * t) * t;
        assert_eq!(pixels.len() as i32, expected);
        let mut seen: Vec<(i32, i32)> = pixels.iter().map(|p| p.0).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), pixels.len());
        for ((x, y), _) in &pixels {
            let inside = *x >= 10 && *x < 70 && *y >= 20 && *y < 55;
            let in_band = *x < 10 + t || *x >= 70 - t || *y < 20 + t || *y >= 55 - t;
            assert!(inside && in_band, "{x},{y} is not on the band");
        }
    }

    #[test]
    fn the_runs_follow_the_dash_and_gap_lengths_around_the_corners() {
        let sel = rect(0, 0, 100, 60);
        let runs = ant_runs(sel, 0);
        // Consecutive runs alternate, and no run is longer than its ink allows.
        for pair in runs.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let same_band = (a.run.top == b.run.top && a.run.bottom == b.run.bottom)
                || (a.run.left == b.run.left && a.run.right == b.run.right);
            if same_band {
                assert_ne!(a.light, b.light, "two runs of one ink side by side");
            }
        }
        for ant in &runs {
            let run = ant.run;
            let along = (run.right - run.left).max(run.bottom - run.top) as u32;
            let limit = if ant.light { ANTS_DASH } else { ANTS_GAP };
            assert!(along <= limit, "a run of {along} for a limit of {limit}");
        }
        // The path is one: the ink at a path position is the pattern's, corners included.
        let mut position = 0i64;
        for ant in &runs {
            let run = ant.run;
            assert_eq!(ant.light, ink_at(position));
            position += (run.right - run.left).max(run.bottom - run.top) as i64;
        }
        assert_eq!(position, 2 * 100 + 2 * (60 - 2 * ANTS_WIDTH) as i64);
    }

    #[test]
    fn a_step_of_phase_walks_the_pattern_one_pixel() {
        // The dash is ANTS_DASH long, then the gap ANTS_GAP long.
        let light: Vec<bool> = (0..(ANTS_DASH + ANTS_GAP) as i64).map(ink_at).collect();
        assert_eq!(light.iter().filter(|l| **l).count() as u32, ANTS_DASH);
        assert!(light[..ANTS_DASH as usize].iter().all(|l| *l));
        assert!(light[ANTS_DASH as usize..].iter().all(|l| !*l));
        // One phase step: what a pixel shows is what its neighbour showed a step before.
        let sel = rect(0, 0, 50, 30);
        let before = ant_runs(sel, 0);
        let after = ant_runs(sel, 1);
        let first_before = before[0].run.right - before[0].run.left;
        let first_after = after[0].run.right - after[0].run.left;
        assert_eq!(first_after, first_before - 1);
    }

    #[test]
    fn the_union_holds_both_rectangles() {
        let u = union_of(desktop(10, 10, 10, 10), desktop(50, 5, 5, 30));
        assert_eq!(u, desktop(10, 5, 45, 30));
    }
}
