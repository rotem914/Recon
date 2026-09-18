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
//! draws a free rectangle, exactly as before. When the pointer moves from one window to
//! the next, the lit area glides from the one to the other rather than jumping, unless
//! Windows' own animations are off.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Instant;

use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, CreateFontW,
    CreateRectRgn, CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, ExcludeClipRect, FillRect,
    GdiFlush, GetDC, GetRegionData, GetTextExtentPoint32W, GetUpdateRgn, InvalidateRect, ReleaseDC,
    RestoreDC, SaveDC, SelectObject, SetBkMode, SetStretchBltMode, SetTextColor, SetViewportOrgEx,
    StretchBlt, TextOutW, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLORONCOLOR, DEFAULT_CHARSET,
    DEFAULT_PITCH, DIB_RGB_COLORS, FF_DONTCARE, FW_NORMAL, HBITMAP, HBRUSH, HDC, HFONT, HGDIOBJ,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, RGNDATA, RGNDATAHEADER, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumChildWindows,
    EnumWindows, GetClassNameW, GetCursorPos, GetForegroundWindow, GetMessageW, GetSystemMetrics,
    GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow,
    IsWindowVisible, LoadCursorW, PostMessageW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, SetTimer, ShowWindow, SystemParametersInfoW, TranslateMessage, CS_HREDRAW,
    CS_VREDRAW, GWL_EXSTYLE, IDC_CROSS, MSG, SM_CXDRAG, SM_CYDRAG, SPI_GETCLIENTAREAANIMATION,
    SW_SHOW, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_APP, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_TIMER, WNDCLASSEXW, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
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
const ANTS_SPEED: f64 = 24.0;
/// The frame's redraw interval: about sixty a second.
const ANTS_FRAME_MS: u32 = 16;
const ANTS_TIMER: usize = 1;
/// The dash, Rotem's #00B9F7, and the gap, near black.
const ANTS_DASH_RGB: (u8, u8, u8) = (0x00, 0xB9, 0xF7);
const ANTS_GAP_RGB: (u8, u8, u8) = (0x20, 0x20, 0x20);

/// The glide (Rotem, 2026-09-18): when the pointer moves from one window to the next, the
/// lit area and its frame glide from the one to the other instead of jumping, each edge on
/// its own, read off the clock on the frame's own ticks. 192 ms and ease-out, Rotem's
/// values, by eye, the same night, after 164 ms, 256 ms with an ease-in, and A1's 144 ms
/// and ease-out before that. Off when Windows' own animations are off.
const GLIDE_MS: f64 = 192.0;

/// The size label (Rotem, 2026-09-17): while a drag lasts, the dragged area's width and
/// height in pixels, 14 px text on the app's background colour, 8 px corners, 8 px of
/// padding at the sides and 4 above and below. Since 2026-09-18 that bubble is the
/// magnifier's panel and the text sits under the circle, as the picture Rotem sent shows.
/// These are the numbers at 100%; each display's overlay scales them by its own scale, as
/// the cursor scales. The text's colour and face are not stated: the ruler's white, in the
/// page's UI font, until they are.
const SIZE_TEXT_PX: i32 = 14;
const SIZE_RADIUS_PX: i32 = 8;
const SIZE_PAD_X_PX: i32 = 8;
const SIZE_PAD_Y_PX: i32 = 4;
/// The app background, `project-os/Design.md`; the page carries the same value as `--paper`.
const SIZE_FILL_RGB: (u8, u8, u8) = (0x0D, 0x0E, 0x12);
const SIZE_TEXT_RGB: (u8, u8, u8) = (0xF2, 0xF2, 0xF2);

/// The magnifier (Rotem, 2026-09-18, with a picture): a 112 px circle showing the frozen
/// pixels around the pointer large enough to tell apart, the whole time the overlay is
/// up, with the selection's size written above it. What the picture shows and his words
/// do not, provisional until he states it: the circle in a panel of the label's fill and
/// corners, 4 px around it; the panel below the pointer with its left edge 8 px right of
/// it (the pointer's right is Rotem's word, from the picture's left), and on the
/// pointer's other side where that would leave the display; 8 px to a
/// source pixel; a 1 px grid between them, the pixel darkened by half; the frame's blue
/// on the centre pixel's top and left edges (his, the same day); a 2 px ring in the text's white.
const MAG_DIAMETER_PX: i32 = 112;
const MAG_CELL_PX: i32 = 8;
const MAG_INSET_PX: i32 = 4;
const MAG_GAP_PX: i32 = 8;
const MAG_RING_PX: i32 = 2;
const MAG_RING_RGB: (u8, u8, u8) = (0xF2, 0xF2, 0xF2);

/// The pointer's lines (Rotem, 2026-09-18): one across and one down through the pointer,
/// 1 px each, the whole width and height of the display it is on whatever is selected, in
/// the blue of the lines inside the magnifier's circle, at 48% (his, the same day, after
/// 72%). They stop 12 px short of the pointer on every side, under the system's cross:
/// it is drawn as the inverse of what is under it, which over the blue made it red where
/// Rotem wants it white. The 12 px is the cross's arm at 100%, by eye, provisional.
const CROSS_RGB: (u8, u8, u8) = ANTS_DASH_RGB;
const CROSS_ALPHA: u8 = 122;
const CROSS_GAP_PX: i32 = 12;

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
    /// The size bubble's text face at this display's scale.
    font: HFONT,
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

/// The lit area on its way from one window to the next: where it set out from, where it
/// is going, when it set out, and where it is now, as last painted. It lives only while a
/// hover on the same surface has `to` as its rectangle.
#[derive(Debug, Clone, Copy)]
struct Glide {
    surface: isize,
    from: DesktopRect,
    to: DesktopRect,
    started: Instant,
    shown: DesktopRect,
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
    /// The lit area's glide from the window it was on to the one it is on, while one lasts.
    glide: Option<Glide>,
    /// Whether the lit area glides at all: Windows' own animation setting, read once.
    animate: bool,
    /// Where the pointer last was, and on which surface: what the magnifier shows while
    /// nothing is being dragged.
    pointer: Option<(isize, (i32, i32))>,
    /// How far the pointer may move from the press before it is a drag: the system's own
    /// value, so a click here is a click everywhere else on this machine.
    drag_threshold: (i32, i32),
    outcome: Outcome,
    /// A 1x1 black bitmap, stretched by AlphaBlend to dim whatever is not selected.
    dim_dc: HDC,
    dim_bitmap: HBITMAP,
    dim_previous: HGDIOBJ,
    /// A 1x1 bitmap of the pointer's lines' blue, stretched by AlphaBlend along each line.
    cross_dc: HDC,
    cross_bitmap: HBITMAP,
    cross_previous: HGDIOBJ,
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

    // The pointer's lines: one pixel of their blue, its alpha byte 255 for the dim's reason.
    let cross_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    let mut info = solid_header(1, 1);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let cross_bitmap = match unsafe {
        CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
    } {
        Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
        _ => {
            unsafe { SelectObject(dim_dc, dim_previous) };
            let _ = unsafe { DeleteObject(HGDIOBJ(dim_bitmap.0)) };
            let _ = unsafe { DeleteDC(dim_dc) };
            let _ = unsafe { DeleteDC(cross_dc) };
            unsafe { ReleaseDC(None, screen_dc) };
            return None;
        }
    };
    let (r, g, b) = CROSS_RGB;
    unsafe { std::ptr::copy_nonoverlapping([b, g, r, 255].as_ptr(), bits as *mut u8, 4) };
    let cross_previous = unsafe { SelectObject(cross_dc, HGDIOBJ(cross_bitmap.0)) };
    std::hint::black_box(&mut info);

    unsafe { ReleaseDC(None, screen_dc) };

    Some(State {
        surfaces,
        drag: None,
        windows,
        parts: HashMap::new(),
        hover: None,
        glide: None,
        animate: windows_animates(),
        pointer: None,
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
        cross_dc,
        cross_bitmap,
        cross_previous,
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

    // The size bubble's face: a negative height is the em size, so 14 px here is the 14 px
    // the page means. A face that cannot be made leaves the bubble unmeasured and undrawn.
    let font = unsafe {
        CreateFontW(
            -scaled(SIZE_TEXT_PX, monitor.scale_percent),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Segoe UI"),
        )
    };

    Some(Surface {
        hwnd,
        monitor: monitor.clone(),
        dc,
        bitmap,
        previous,
        painted: false,
        font,
    })
}

/// A size at 100%, as it is on a display at this scale.
fn scaled(px: i32, scale_percent: u32) -> i32 {
    (px * scale_percent as i32 + 50) / 100
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
        if !surface.font.is_invalid() {
            let _ = unsafe { DeleteObject(HGDIOBJ(surface.font.0)) };
        }
    }
    unsafe { SelectObject(state.dim_dc, state.dim_previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(state.dim_bitmap.0)) };
    let _ = unsafe { DeleteDC(state.dim_dc) };
    unsafe { SelectObject(state.cross_dc, state.cross_previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(state.cross_bitmap.0)) };
    let _ = unsafe { DeleteDC(state.cross_dc) };
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
                    state.pointer = Some((surface, (at.x, at.y)));
                    Some((surface, state.update_hover(surface, (at.x, at.y))))
                });
                if let Some((surface, changed)) = changed {
                    unsafe { invalidate_hover_change(changed) };
                    // And the magnifier, which shows the pointer from now on.
                    unsafe { invalidate_magnifier(HWND(surface as *mut _)) };
                    unsafe { invalidate_cross() };
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
            // The magnifier follows every move, so where it was and where it goes are both
            // repainted; it is laid out on this window's DC with the display's face.
            let hdc = unsafe { GetDC(Some(hwnd)) };
            let panel_before = STATE.with(|cell| {
                let borrowed = cell.borrow();
                borrowed.as_ref()?.magnifier(hwnd, hdc).map(|p| p.rect)
            });
            // The pointer's lines follow it too: where they were, on whichever display,
            // and further down where they are now.
            unsafe { invalidate_cross() };
            // With no button down, the move changes which window is lit, and the pointer.
            let hover_change = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                let monitor = state.monitor_of(hwnd)?.rect;
                let desktop = (monitor.x + point.x, monitor.y + point.y);
                state.pointer = Some((hwnd.0 as isize, desktop));
                if state.drag.is_some() {
                    return None;
                }
                Some(state.update_hover(hwnd.0 as isize, desktop))
            });
            if let Some(changed) = hover_change {
                unsafe { invalidate_hover_change(changed) };
            } else {
                unsafe { drag_move(hwnd, point) };
            }
            let panel_after = STATE.with(|cell| {
                let borrowed = cell.borrow();
                borrowed.as_ref()?.magnifier(hwnd, hdc).map(|p| p.rect)
            });
            let _ = unsafe { ReleaseDC(Some(hwnd), hdc) };
            for panel in panel_before.into_iter().chain(panel_after) {
                let _ = unsafe { InvalidateRect(Some(hwnd), Some(&panel), false) };
            }
            unsafe { invalidate_cross() };
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
            // The glide first, while the lit area is on its way from one window to the
            // next: where it was and where it is now are repainted whole, since the dim
            // moves with it. It is read off the clock too, so a late frame skips ahead.
            let moved = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                state.advance_glide(hwnd.0 as isize)
            });
            if let Some((origin, before, after)) = moved {
                unsafe { invalidate_union(hwnd, origin, before, after) };
            }
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

/// A move with the button down: past the system's threshold the press becomes a drag,
/// the lit window goes dark, and the dragged rectangle follows the pointer, clamped to
/// the display the drag started on.
unsafe fn drag_move(hwnd: HWND, point: POINT) {
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
            lit = state.take_hover();
        }
        let drag = state.drag.as_mut()?;
        let dragged =
            DesktopRect::from_points(drag.anchor.0, drag.anchor.1, drag.current.0, drag.current.1);
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
        let after =
            DesktopRect::from_points(drag.anchor.0, drag.anchor.1, drag.current.0, drag.current.1);
        Some((before, after, origin))
    });
    if let Some((before, after, origin)) = invalidate {
        // Only the union of the old and the new rectangle changed, so only that is
        // repainted. A full repaint of a 5120 wide display per mouse move is the
        // easiest way to make this surface feel slow.
        unsafe { invalidate_union(hwnd, origin, before, after) };
    }
}

/// Marks the pointer's lines for repainting, as they stand now, on the window they are on.
unsafe fn invalidate_cross() {
    let cross = STATE.with(|cell| cell.borrow().as_ref()?.cross());
    if let Some((surface, strips)) = cross {
        for strip in strips {
            let _ = unsafe { InvalidateRect(Some(HWND(surface as *mut _)), Some(&strip), false) };
        }
    }
}

/// Marks the magnifier's place on a window for repainting, as it stands now.
unsafe fn invalidate_magnifier(hwnd: HWND) {
    let hdc = unsafe { GetDC(Some(hwnd)) };
    let panel = STATE.with(|cell| {
        let borrowed = cell.borrow();
        borrowed.as_ref()?.magnifier(hwnd, hdc).map(|p| p.rect)
    });
    let _ = unsafe { ReleaseDC(Some(hwnd), hdc) };
    if let Some(panel) = panel {
        let _ = unsafe { InvalidateRect(Some(hwnd), Some(&panel), false) };
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
            _ => {
                let hover = self.hover.filter(|h| h.surface == hwnd.0 as isize)?;
                self.lit_rect(hover)
            }
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

    /// The pointer's lines, and the window they are on, in its client coordinates: the row
    /// left and right of the pointer, then the column above and below it, each stopping
    /// the gap short of it. A drag that has moved has them at its clamped point, as the magnifier
    /// has; otherwise they are at the pointer as last seen.
    fn cross(&self) -> Option<(isize, [RECT; 4])> {
        let (surface, desktop) = match self.drag.as_ref() {
            Some(drag) if drag.moved => (drag.hwnd, drag.current),
            _ => self.pointer?,
        };
        let monitor = &self
            .surfaces
            .iter()
            .find(|s| s.hwnd.0 as isize == surface)?
            .monitor;
        let display = monitor.rect;
        let gap = scaled(CROSS_GAP_PX, monitor.scale_percent);
        let (x, y) = (desktop.0 - display.x, desktop.1 - display.y);
        let (width, height) = (display.width as i32, display.height as i32);
        let strip = |left, top, right, bottom| RECT {
            left,
            top,
            right,
            bottom,
        };
        Some((
            surface,
            [
                strip(0, y, x - gap, y + 1),
                strip(x + gap + 1, y, width, y + 1),
                strip(x, 0, x + 1, y - gap),
                strip(x, y + gap + 1, x + 1, height),
            ],
        ))
    }

    /// The size written under the magnifier: the dragged area's once a drag on this window
    /// has moved, a line's included, otherwise the lit window's on this surface.
    fn selection_size(&self, hwnd: HWND) -> Option<(u32, u32)> {
        match self.drag.as_ref() {
            Some(drag) if drag.moved && drag.hwnd == hwnd.0 as isize => {
                let rect = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
                Some((rect.width, rect.height))
            }
            _ => self
                .hover
                .filter(|h| h.surface == hwnd.0 as isize)
                .map(|h| (h.rect.width, h.rect.height)),
        }
    }

    /// The magnifier on this window: the panel beside the pointer, the circle inside it
    /// and the selection's size above the circle, measured with the display's face on the
    /// given DC, in client coordinates. None while the pointer is on another window, or
    /// before it was seen at all.
    fn magnifier(&self, hwnd: HWND, hdc: HDC) -> Option<Panel> {
        let surface = self.surface_of(hwnd)?;
        let scale = surface.monitor.scale_percent;
        let display = surface.monitor.rect;
        // A drag's pointer is its clamped point, so past the display's edge the circle
        // shows the edge; with no drag the pointer as last seen on this surface.
        let desktop = match self.drag.as_ref() {
            Some(drag) if drag.hwnd == hwnd.0 as isize => drag.current,
            Some(_) => return None,
            None => self.pointer.filter(|(s, _)| *s == hwnd.0 as isize)?.1,
        };
        let source = (desktop.0 - display.x, desktop.1 - display.y);
        let diameter = scaled(MAG_DIAMETER_PX, scale);
        let cell = scaled(MAG_CELL_PX, scale).max(1);
        let inset = scaled(MAG_INSET_PX, scale);
        let text = self.selection_size(hwnd).and_then(|(width, height)| {
            if surface.font.is_invalid() {
                return None;
            }
            let text: Vec<u16> = format!("{width}x{height}").encode_utf16().collect();
            let mut extent = SIZE::default();
            let measured = unsafe {
                let previous = SelectObject(hdc, HGDIOBJ(surface.font.0));
                let ok = GetTextExtentPoint32W(hdc, &text, &mut extent).as_bool();
                SelectObject(hdc, previous);
                ok
            };
            if !measured || extent.cx <= 0 || extent.cy <= 0 {
                return None;
            }
            Some((text, extent))
        });
        let pad = (scaled(SIZE_PAD_X_PX, scale), scaled(SIZE_PAD_Y_PX, scale));
        let width = (diameter + 2 * inset).max(text.as_ref().map_or(0, |(_, e)| e.cx + 2 * pad.0));
        // The size above the circle (Rotem, 2026-09-18, from under it): the text row first,
        // with its padding, then the circle with its inset.
        let above = text.as_ref().map_or(inset, |(_, e)| pad.1 + e.cy + pad.1);
        let height = above + diameter + inset;
        let text = text.map(|(t, e)| (t, ((width - e.cx) / 2, pad.1)));
        let (left, top) = place_panel(
            source,
            (width, height),
            (display.width as i32, display.height as i32),
            scaled(MAG_GAP_PX, scale),
        );
        Some(Panel {
            rect: RECT {
                left,
                top,
                right: left + width,
                bottom: top + height,
            },
            radius: scaled(SIZE_RADIUS_PX, scale),
            circle: (width / 2, above + diameter / 2, diameter / 2),
            ring: scaled(MAG_RING_PX, scale),
            cell,
            cells: cell_count(diameter, cell),
            source,
            text,
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
        // From one window to the next on the same display, the lit area glides: it sets
        // out from wherever it is now, mid-glide included, and the ticks carry it, so
        // there is nothing to repaint yet. A glide already heading there keeps going.
        if let (Some(was), Some(now)) = (previous, next) {
            if self.animate && was.surface == now.surface {
                let shown = self.lit_rect(was);
                let heading_there = self.glide.is_some_and(|g| g.to == now.rect);
                if shown == now.rect {
                    self.glide = None;
                } else if !heading_there {
                    self.glide = Some(Glide {
                        surface: now.surface,
                        from: shown,
                        to: now.rect,
                        started: Instant::now(),
                        shown,
                    });
                }
                return HoverChange::default();
            }
        }
        // Lit for the first time, gone, on another display, or with Windows' animations
        // off: a jump, the area as it was painted going back to dim and the new one lit.
        let before = previous.map(|h| {
            (
                h.surface,
                self.origin_of_surface(h.surface),
                self.lit_rect(h),
            )
        });
        self.glide = None;
        HoverChange {
            before,
            after: next.map(|h| (h.surface, self.origin_of_surface(h.surface), h.rect)),
        }
    }

    /// The rectangle lit for a hover, as painted: the glide's, while the lit area is on
    /// its way there, otherwise the hover's own.
    fn lit_rect(&self, hover: Hover) -> DesktopRect {
        match self.glide {
            Some(glide) if glide.surface == hover.surface => glide.shown,
            _ => hover.rect,
        }
    }

    /// Ends the hover, glide included, for the drag that takes over: the rectangle that
    /// was lit, as painted, so the repaint can put it back to dim.
    fn take_hover(&mut self) -> Option<DesktopRect> {
        let hover = self.hover.take()?;
        let lit = self.lit_rect(hover);
        self.glide = None;
        Some(lit)
    }

    /// Moves the glide on this surface along by the clock: the rectangle painted before
    /// and the one to paint now, with the surface's origin, when it moved. The glide ends
    /// the tick it arrives, and the hover's own rectangle is lit from then on.
    fn advance_glide(&mut self, surface: isize) -> Option<((i32, i32), DesktopRect, DesktopRect)> {
        let mut glide = self.glide.filter(|g| g.surface == surface)?;
        let fraction = glide.started.elapsed().as_secs_f64() * 1000.0 / GLIDE_MS;
        let before = glide.shown;
        glide.shown = glide_rect(glide.from, glide.to, ease_out(fraction));
        let after = glide.shown;
        self.glide = if fraction >= 1.0 { None } else { Some(glide) };
        if before == after {
            return None;
        }
        Some((self.origin_of_surface(surface), before, after))
    }

    /// The parts of a top-level window, listed on first use.
    fn parts_of(&mut self, window: isize) -> &[WindowBounds] {
        self.parts
            .entry(window)
            .or_insert_with(|| parts_of_window(window))
    }

    /// How far the dashes have walked: whole pixels, and the fraction of the next one.
    /// The walk is clockwise, rightwards along the top, so the pattern is pulled back one
    /// pixel more than it has walked and the edge blend runs the other way to make it up.
    fn phase(&self) -> (u32, f64) {
        let walked = self.started.elapsed().as_secs_f64() * ANTS_SPEED;
        let whole = walked.floor();
        (
            ((whole as u64 + 1) % u32::MAX as u64) as u32,
            1.0 - (walked - whole),
        )
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

/// Whether Windows animates: its "Animation effects" setting, off for the people who
/// turned motion off, and the glide is off with it (`project-os/QA.md` §7). A setting
/// that cannot be read counts as on.
pub(crate) fn windows_animates() -> bool {
    let mut on = BOOL(1);
    let asked = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut on as *mut BOOL as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    asked.is_err() || on.as_bool()
}

/// How far along the glide is for how much of its time has passed: quick to set out and
/// slow to arrive, an ease-out, cubic.
fn ease_out(time: f64) -> f64 {
    let t = time.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// The rectangle a fraction of the way from one to the other, each edge on its own,
/// rounded to the pixel: exactly the one at 0 and exactly the other at 1.
fn glide_rect(from: DesktopRect, to: DesktopRect, fraction: f64) -> DesktopRect {
    let f = fraction.clamp(0.0, 1.0);
    let edge = |a: i32, b: i32| (a as f64 + (b as f64 - a as f64) * f).round() as i32;
    DesktopRect::from_points(
        edge(from.x, to.x),
        edge(from.y, to.y),
        edge(from.x + from.width as i32, to.x + to.width as i32),
        edge(from.y + from.height as i32, to.y + to.height as i32),
    )
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

/// The rectangles of a window's update region, read before BeginPaint empties it. The
/// pointer's lines make that region a row and a column, whose bounding box is the whole
/// display: painted piece by piece, a move repaints two strips and the magnifier, not
/// every pixel. Empty when the region cannot be read, and the bounding box is painted.
unsafe fn update_rects(hwnd: HWND) -> Vec<RECT> {
    let region = unsafe { CreateRectRgn(0, 0, 0, 0) };
    if region.is_invalid() {
        return Vec::new();
    }
    let mut rects = Vec::new();
    unsafe {
        let _ = GetUpdateRgn(hwnd, region, false);
        let size = GetRegionData(region, 0, None) as usize;
        let header = std::mem::size_of::<RGNDATAHEADER>();
        if size >= header {
            // Four-byte words, so the header and the rectangles after it are aligned.
            let mut data = vec![0u32; size.div_ceil(4)];
            let at = data.as_mut_ptr() as *mut RGNDATA;
            if GetRegionData(region, (data.len() * 4) as u32, Some(at)) != 0 {
                let head = (*at).rdh;
                let count =
                    (head.nCount as usize).min((size - header) / std::mem::size_of::<RECT>());
                let first = (at as *const u8).add(head.dwSize as usize) as *const RECT;
                rects.extend((0..count).map(|n| *first.add(n)));
            }
        }
        let _ = DeleteObject(HGDIOBJ(region.0));
    }
    rects
}

unsafe fn paint(hwnd: HWND) {
    let mut pieces = unsafe { update_rects(hwnd) };
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    if pieces.is_empty() {
        pieces.push(ps.rcPaint);
    }
    // The magnifier first, off the screen: the layers under it and the panel over them in
    // one bitmap. Then the pieces, with the panel's place cut out of them, and the bitmap
    // copied there in one call. Painted straight onto the window, a piece first put the
    // screen back where the panel is and the panel came after, and the eye caught the
    // moment in between as a flicker, at every move and at every tick of a frame through it.
    let magnifier = unsafe { compose_magnifier(hwnd, hdc, &pieces) };
    let saved = unsafe { SaveDC(hdc) };
    if let Some((rect, _, _, _)) = magnifier.as_ref() {
        unsafe { ExcludeClipRect(hdc, rect.left, rect.top, rect.right, rect.bottom) };
    }
    for dirty in &pieces {
        unsafe { paint_piece(hwnd, hdc, *dirty) };
    }
    let _ = unsafe { RestoreDC(hdc, saved) };
    if let Some((rect, dc, bitmap, previous)) = magnifier {
        unsafe {
            let _ = BitBlt(
                hdc,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                Some(dc),
                rect.left,
                rect.top,
                SRCCOPY,
            );
            SelectObject(dc, previous);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(dc);
        }
    }
    let _ = unsafe { EndPaint(hwnd, &ps) };
}

/// Paints one rectangle of the window: every layer under the magnifier, cut to it.
unsafe fn paint_piece(hwnd: HWND, hdc: HDC, dirty: RECT) {
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

            // The pointer's lines, over the dim and the lit area alike and under the frame:
            // the pieces of them inside the dirty region.
            if let Some((_, strips)) = state.cross().filter(|(s, _)| *s == hwnd.0 as isize) {
                let blend = BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: CROSS_ALPHA,
                    AlphaFormat: 0,
                };
                for strip in strips {
                    let left = strip.left.max(dirty.left);
                    let top = strip.top.max(dirty.top);
                    let right = strip.right.min(dirty.right);
                    let bottom = strip.bottom.min(dirty.bottom);
                    if right > left && bottom > top {
                        let _ = unsafe {
                            AlphaBlend(
                                hdc,
                                left,
                                top,
                                right - left,
                                bottom - top,
                                state.cross_dc,
                                0,
                                0,
                                1,
                                1,
                                blend,
                            )
                        };
                    }
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
}

/// The last layer, where a dirty piece touches it: the magnifier beside the pointer with
/// the layers under it, in a bitmap of the panel's size whose coordinates are the
/// window's, ready to be copied onto the window in one call. Every pixel's alpha is set
/// full at the end: a brush clears it, and this window's pixels are shown by their alpha
/// (see the dim's note). None when nothing dirty touches the panel, or it cannot be made.
unsafe fn compose_magnifier(
    hwnd: HWND,
    hdc: HDC,
    pieces: &[RECT],
) -> Option<(RECT, HDC, HBITMAP, HGDIOBJ)> {
    let rect = STATE.with(|cell| {
        let borrowed = cell.borrow();
        let rect = borrowed.as_ref()?.magnifier(hwnd, hdc)?.rect;
        let touches = pieces.iter().any(|dirty| {
            rect.right > dirty.left
                && rect.left < dirty.right
                && rect.bottom > dirty.top
                && rect.top < dirty.bottom
        });
        touches.then_some(rect)
    })?;
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    if w <= 0 || h <= 0 {
        return None;
    }
    let dc = unsafe { CreateCompatibleDC(Some(hdc)) };
    if dc.is_invalid() {
        return None;
    }
    let mut info = solid_header(w, h);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap =
        match unsafe { CreateDIBSection(Some(hdc), &info, DIB_RGB_COLORS, &mut bits, None, 0) } {
            Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
            _ => {
                let _ = unsafe { DeleteDC(dc) };
                return None;
            }
        };
    std::hint::black_box(&mut info);
    let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    unsafe {
        let _ = SetViewportOrgEx(dc, -rect.left, -rect.top, None);
        paint_piece(hwnd, dc, rect);
    }
    STATE.with(|cell| {
        let borrowed = cell.borrow();
        let Some(state) = borrowed.as_ref() else {
            return;
        };
        let Some(surface) = state.surface_of(hwnd) else {
            return;
        };
        if let Some(panel) = state.magnifier(hwnd, hdc) {
            unsafe { draw_panel(dc, surface, &panel) };
        }
    });
    unsafe {
        let _ = GdiFlush();
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
        for alpha in pixels.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }
    }
    Some((rect, dc, bitmap, previous))
}

/// The magnifier's panel as laid out on one window, at the display's scale.
struct Panel {
    /// Its place, in client coordinates.
    rect: RECT,
    /// Its corners.
    radius: i32,
    /// The circle's centre inside the panel, and its radius.
    circle: (i32, i32, i32),
    ring: i32,
    /// One source pixel's width on the circle, and how many across, an odd count so the
    /// pointer's own pixel sits in the middle.
    cell: i32,
    cells: i32,
    /// The source pixel at the circle's centre: the pointer, in client coordinates.
    source: (i32, i32),
    /// The selection's size above the circle, and where its text sits inside the panel.
    text: Option<(Vec<u16>, (i32, i32))>,
}

/// Where the panel goes: below the pointer by the gap, its left edge the gap right of the
/// pointer, and on the pointer's other side where that would leave the display, so it is
/// never cut off at an edge.
fn place_panel(pointer: (i32, i32), size: (i32, i32), display: (i32, i32), gap: i32) -> (i32, i32) {
    let mut left = pointer.0 + gap;
    if left + size.0 > display.0 {
        left = pointer.0 - gap - size.0;
    }
    let mut top = pointer.1 + gap;
    if top + size.1 > display.1 {
        top = pointer.1 - gap - size.1;
    }
    (left.max(0), top.max(0))
}

/// How many source pixels the circle shows across: enough cells to cover its diameter,
/// and an odd count so the pointer's own pixel is the middle one.
fn cell_count(diameter: i32, cell: i32) -> i32 {
    let n = (diameter + cell - 1) / cell.max(1);
    if n % 2 == 0 {
        n + 1
    } else {
        n
    }
}

/// The pixels around the pointer, each one a cell wide, BGRA: the frozen slice stretched
/// with no smoothing, the grid's line on the last pixel of every cell across and down,
/// darkened by half, and the frame's blue on the centre cell's top and left edges. Past the
/// display's edge the panel's own fill shows, since a stretch reads nothing beyond the
/// bitmap.
unsafe fn magnified(hdc: HDC, surface: &Surface, panel: &Panel) -> Option<Vec<u8>> {
    let cell = panel.cell;
    let count = panel.cells;
    let side = count * cell;
    let dc = unsafe { CreateCompatibleDC(Some(hdc)) };
    if dc.is_invalid() {
        return None;
    }
    let mut info = solid_header(side, side);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap =
        match unsafe { CreateDIBSection(Some(hdc), &info, DIB_RGB_COLORS, &mut bits, None, 0) } {
            Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
            _ => {
                let _ = unsafe { DeleteDC(dc) };
                return None;
            }
        };
    std::hint::black_box(&mut info);
    let len = (side * side * 4) as usize;
    {
        let pixels = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, len) };
        let (r, g, b) = SIZE_FILL_RGB;
        for at in (0..len).step_by(4) {
            pixels[at] = b;
            pixels[at + 1] = g;
            pixels[at + 2] = r;
            pixels[at + 3] = 255;
        }
    }
    let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    // The square of source pixels around the pointer, cut to the display, and the same
    // cut on the destination so each source pixel lands on its own cell.
    let half = count / 2;
    let (sx, sy) = (panel.source.0 - half, panel.source.1 - half);
    let display = (
        surface.monitor.rect.width as i32,
        surface.monitor.rect.height as i32,
    );
    let (left, top) = (sx.max(0), sy.max(0));
    let (right, bottom) = ((sx + count).min(display.0), (sy + count).min(display.1));
    if right > left && bottom > top {
        unsafe {
            SetStretchBltMode(dc, COLORONCOLOR);
            let _ = StretchBlt(
                dc,
                (left - sx) * cell,
                (top - sy) * cell,
                (right - left) * cell,
                (bottom - top) * cell,
                Some(surface.dc),
                left,
                top,
                right - left,
                bottom - top,
                SRCCOPY,
            );
            let _ = GdiFlush();
        }
    }
    let mut bytes = vec![0u8; len];
    {
        let pixels = unsafe { std::slice::from_raw_parts(bits as *const u8, len) };
        bytes.copy_from_slice(pixels);
    }
    unsafe { SelectObject(dc, previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    let _ = unsafe { DeleteDC(dc) };

    // The blue lines run on the centre cell's top and left edges, the grid's line there, so
    // they cross at the pointer's pixel's top left corner and the pixel itself is whole
    // (Rotem, 2026-09-18, from another tool's picture, after the middle of the cell).
    let centre = half * cell - 1;
    for y in 0..side {
        for x in 0..side {
            let at = ((y * side + x) * 4) as usize;
            if x == centre || y == centre {
                bytes[at] = ANTS_DASH_RGB.2;
                bytes[at + 1] = ANTS_DASH_RGB.1;
                bytes[at + 2] = ANTS_DASH_RGB.0;
            } else if x % cell == cell - 1 || y % cell == cell - 1 {
                bytes[at] /= 2;
                bytes[at + 1] /= 2;
                bytes[at + 2] /= 2;
            }
            bytes[at + 3] = 255;
        }
    }
    Some(bytes)
}

/// How much of a pixel a rounded rectangle of this size covers: 1 inside, 0 outside, and
/// the fraction of it inside the arc at the corners, so the corners are smooth. The radius
/// is cut down to half the shorter side, as a page's corners are.
fn corner_coverage(w: i32, h: i32, radius: i32, x: i32, y: i32) -> f64 {
    let r = radius.min(w / 2).min(h / 2).max(0) as f64;
    let px = x as f64 + 0.5;
    let py = y as f64 + 0.5;
    let cx = if px < r {
        r
    } else if px > w as f64 - r {
        w as f64 - r
    } else {
        return 1.0;
    };
    let cy = if py < r {
        r
    } else if py > h as f64 - r {
        h as f64 - r
    } else {
        return 1.0;
    };
    let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
    (r + 0.5 - d).clamp(0.0, 1.0)
}

/// Draws the magnifier's panel: the rounded fill built pixel by pixel, the pixels around
/// the pointer inside the circle with the ring around them, the corners' and the circle's
/// edge pixels covered by the fraction of them inside the arc so both are smooth, the
/// size text under the circle, and the whole blended onto the window in one call. Every
/// pixel it leaves on the window keeps a full alpha: a GDI text call clears the alpha of
/// the pixels it touches, and this window's pixels are shown by their alpha (see the
/// dim's note).
unsafe fn draw_panel(hdc: HDC, surface: &Surface, panel: &Panel) {
    let w = panel.rect.right - panel.rect.left;
    let h = panel.rect.bottom - panel.rect.top;
    if w <= 0 || h <= 0 {
        return;
    }
    let Some(pixels_around) = (unsafe { magnified(hdc, surface, panel) }) else {
        return;
    };
    let dc = unsafe { CreateCompatibleDC(Some(hdc)) };
    if dc.is_invalid() {
        return;
    }
    let mut info = solid_header(w, h);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap =
        match unsafe { CreateDIBSection(Some(hdc), &info, DIB_RGB_COLORS, &mut bits, None, 0) } {
            Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
            _ => {
                let _ = unsafe { DeleteDC(dc) };
                return;
            }
        };
    std::hint::black_box(&mut info);
    let len = (w * h * 4) as usize;

    // The fill, premultiplied by each pixel's coverage, BGRA as GDI wants it; then the
    // circle over it: the ring between its radius and the picture's, the pixels around
    // the pointer inside, each edge blended by its coverage.
    {
        let pixels = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, len) };
        let (r, g, b) = SIZE_FILL_RGB;
        let (cx, cy, radius) = panel.circle;
        let side = panel.cells * panel.cell;
        let centre = (panel.cells / 2) * panel.cell - 1;
        let ring = (MAG_RING_RGB.2, MAG_RING_RGB.1, MAG_RING_RGB.0);
        for y in 0..h {
            for x in 0..w {
                let coverage = corner_coverage(w, h, panel.radius, x, y);
                let at = ((y * w + x) * 4) as usize;
                let over = |c: u8| (c as f64 * coverage).round() as u8;
                pixels[at] = over(b);
                pixels[at + 1] = over(g);
                pixels[at + 2] = over(r);
                pixels[at + 3] = (255.0 * coverage).round() as u8;

                let dx = x as f64 + 0.5 - cx as f64;
                let dy = y as f64 + 0.5 - cy as f64;
                let distance = (dx * dx + dy * dy).sqrt();
                let outer = (radius as f64 + 0.5 - distance).clamp(0.0, 1.0);
                if outer <= 0.0 {
                    continue;
                }
                let inner = ((radius - panel.ring) as f64 + 0.5 - distance).clamp(0.0, 1.0);
                let (mx, my) = (x - cx + centre, y - cy + centre);
                let around = if mx >= 0 && my >= 0 && mx < side && my < side {
                    let from = ((my * side + mx) * 4) as usize;
                    (
                        pixels_around[from],
                        pixels_around[from + 1],
                        pixels_around[from + 2],
                    )
                } else {
                    (b, g, r)
                };
                let blend = |fill: u8, ring: u8, inside: u8| {
                    (fill as f64 * (1.0 - outer)
                        + ring as f64 * (outer - inner)
                        + inside as f64 * inner)
                        .round() as u8
                };
                pixels[at] = blend(b, ring.0, around.0);
                pixels[at + 1] = blend(g, ring.1, around.1);
                pixels[at + 2] = blend(r, ring.2, around.2);
                pixels[at + 3] = 255;
            }
        }
    }

    let previous_bitmap = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    if let Some((text, at)) = &panel.text {
        let previous_font = unsafe { SelectObject(dc, HGDIOBJ(surface.font.0)) };
        unsafe {
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, colorref(SIZE_TEXT_RGB));
            let _ = TextOutW(dc, at.0, at.1, text);
            let _ = GdiFlush();
            SelectObject(dc, previous_font);
        }
        // The text sits in the flat part of the panel, where every pixel is wholly
        // covered; the text call cleared the alpha of the pixels it touched, and this
        // puts it back.
        let pixels = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, len) };
        for y in 0..h {
            for x in 0..w {
                if corner_coverage(w, h, panel.radius, x, y) >= 1.0 {
                    pixels[((y * w + x) * 4 + 3) as usize] = 255;
                }
            }
        }
    }

    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    let _ = unsafe {
        AlphaBlend(
            hdc,
            panel.rect.left,
            panel.rect.top,
            w,
            h,
            dc,
            0,
            0,
            w,
            h,
            blend,
        )
    };
    unsafe { SelectObject(dc, previous_bitmap) };
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    let _ = unsafe { DeleteDC(dc) };
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
            let position = walked + at as i64 - phase as i64;
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
        // One phase step walks the pattern one pixel rightwards along the top: a pixel of
        // gap comes in at the corner, and the first dash starts one pixel further right.
        let sel = rect(0, 0, 50, 30);
        let before = ant_runs(sel, 0);
        let after = ant_runs(sel, 1);
        assert!(before[0].light);
        assert_eq!(before[0].run.left, 0);
        assert!(!after[0].light);
        assert_eq!(after[0].run.right - after[0].run.left, 1);
        assert!(after[1].light);
        assert_eq!(after[1].run.left, 1);
    }

    #[test]
    fn the_panel_sits_below_the_pointer_with_its_left_edge_right_of_it() {
        assert_eq!(
            place_panel((500, 100), (120, 143), (1920, 1080), 8),
            (508, 108)
        );
    }

    #[test]
    fn the_panel_flips_to_the_other_side_at_a_display_edge() {
        // Too close to the right edge: left of the pointer. Too close to the bottom: above.
        assert_eq!(
            place_panel((1850, 100), (120, 143), (1920, 1080), 8),
            (1722, 108)
        );
        assert_eq!(
            place_panel((500, 1000), (120, 143), (1920, 1080), 8),
            (508, 849)
        );
        assert_eq!(
            place_panel((1850, 1000), (120, 143), (1920, 1080), 8),
            (1722, 849)
        );
        // Flipped left on a display too narrow for the gap: kept inside it.
        assert_eq!(
            place_panel((100, 100), (120, 143), (200, 1080), 8),
            (0, 108)
        );
    }

    #[test]
    fn the_circle_shows_an_odd_count_of_pixels_that_covers_it() {
        assert_eq!(cell_count(112, 8), 15);
        assert_eq!(cell_count(112, 7), 17);
        assert_eq!(cell_count(252, 18), 15);
        assert_eq!(cell_count(112, 16), 7);
    }

    #[test]
    fn the_corners_are_covered_by_the_arc_and_the_rest_whole() {
        // Inside; on the flat edges between the corners; the corner pixels themselves.
        assert_eq!(corner_coverage(60, 27, 8, 30, 13), 1.0);
        assert_eq!(corner_coverage(60, 27, 8, 8, 0), 1.0);
        assert_eq!(corner_coverage(60, 27, 8, 0, 8), 1.0);
        assert_eq!(corner_coverage(60, 27, 8, 0, 0), 0.0);
        assert_eq!(corner_coverage(60, 27, 8, 59, 26), 0.0);
        // On the arc: the pixel at 2,2 sits 7.8 from the corner's centre, part in.
        let edge = corner_coverage(60, 27, 8, 2, 2);
        assert!(edge > 0.0 && edge < 1.0, "{edge}");
        // A radius past half the shorter side is cut down to it.
        assert_eq!(corner_coverage(20, 10, 8, 10, 5), 1.0);
    }

    #[test]
    fn a_size_scales_with_the_display() {
        assert_eq!(scaled(14, 100), 14);
        assert_eq!(scaled(14, 150), 21);
        assert_eq!(scaled(14, 225), 32);
        assert_eq!(scaled(8, 125), 10);
    }

    #[test]
    fn the_union_holds_both_rectangles() {
        let u = union_of(desktop(10, 10, 10, 10), desktop(50, 5, 5, 30));
        assert_eq!(u, desktop(10, 5, 45, 30));
    }

    #[test]
    fn the_glide_sets_out_from_one_rectangle_and_arrives_at_the_other() {
        let from = desktop(100, 100, 200, 100);
        let to = desktop(20, 40, 600, 400);
        assert_eq!(glide_rect(from, to, 0.0), from);
        assert_eq!(glide_rect(from, to, 1.0), to);
        // Halfway, every edge is halfway: the left 100 to 20, the top 100 to 40, the right
        // 300 to 620, the bottom 200 to 440.
        assert_eq!(glide_rect(from, to, 0.5), desktop(60, 70, 400, 250));
        // Past either end it stays at that end.
        assert_eq!(glide_rect(from, to, 1.5), to);
        assert_eq!(glide_rect(from, to, -0.5), from);
    }

    #[test]
    fn the_glide_eases_out() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        // Quick to set out, slow to arrive: more than half the way at half the time.
        assert!(ease_out(0.5) > 0.5);
        // And never back: every step is at least as far as the one before.
        let mut last = 0.0;
        for step in 1..=100 {
            let now = ease_out(step as f64 / 100.0);
            assert!(now >= last, "step {step}: {now} after {last}");
            last = now;
        }
    }
}
