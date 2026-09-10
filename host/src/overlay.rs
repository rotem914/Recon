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

use std::cell::RefCell;
use std::ffi::c_void;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, CreateSolidBrush,
    DeleteDC, DeleteObject, EndPaint, FrameRect, GetDC, InvalidateRect, ReleaseDC, SelectObject,
    AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP,
    HBRUSH, HDC, HGDIOBJ, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow,
    GetMessageW, LoadCursorW, PostQuitMessage, RegisterClassExW, SetForegroundWindow, ShowWindow,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_CROSS, MSG, SW_SHOW, WM_DESTROY, WM_ERASEBKGND,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WNDCLASSEXW,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::capture::coords::DesktopRect;
use crate::capture::display::{monitors, MonitorInfo};
use crate::capture::Frame;

/// How dark the unselected area gets. 0 is untouched, 255 is black.
const DIM_ALPHA: u8 = 140;

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
}

struct Drag {
    /// Desktop coordinates of where the button went down.
    anchor: (i32, i32),
    current: (i32, i32),
    /// Which window owns this drag. A selection never leaves the display it started on.
    hwnd: isize,
}

struct State {
    surfaces: Vec<Surface>,
    drag: Option<Drag>,
    outcome: Outcome,
    /// A 1x1 black bitmap, stretched by AlphaBlend to dim whatever is not selected.
    dim_dc: HDC,
    dim_bitmap: HBITMAP,
    dim_previous: HGDIOBJ,
    border: HBRUSH,
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
    let list = monitors();
    if list.is_empty() {
        return Outcome::Cancelled;
    }

    unsafe {
        register_class();

        let Some(state) = build_state(frame, &list) else {
            return Outcome::Cancelled;
        };
        STATE.with(|cell| *cell.borrow_mut() = Some(state));

        // Foreground goes to the window under the pointer, so Escape has somewhere to land.
        STATE.with(|cell| {
            if let Some(state) = cell.borrow().as_ref() {
                if let Some(first) = state.surfaces.first() {
                    let _ = SetForegroundWindow(first.hwnd);
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
        if !prior_foreground.is_invalid() {
            let _ = SetForegroundWindow(prior_foreground);
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

unsafe fn build_state(frame: &Frame, list: &[MonitorInfo]) -> Option<State> {
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

    // The dimming layer: one black pixel, stretched over whatever is not selected.
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
    unsafe { std::ptr::write_bytes(bits as *mut u8, 0, 4) };
    let dim_previous = unsafe { SelectObject(dim_dc, HGDIOBJ(dim_bitmap.0)) };
    std::hint::black_box(&mut info);

    unsafe { ReleaseDC(None, screen_dc) };

    Some(State {
        surfaces,
        drag: None,
        outcome: Outcome::Cancelled,
        dim_dc,
        dim_bitmap,
        dim_previous,
        border: unsafe { CreateSolidBrush(windows::Win32::Foundation::COLORREF(0x00F0F0F0)) },
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

    Some(Surface {
        hwnd,
        monitor: monitor.clone(),
        dc,
        bitmap,
        previous,
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
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1), // every pixel is painted in WM_PAINT
        WM_PAINT => {
            unsafe { paint(hwnd) };
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
                        });
                    }
                }
            });
            unsafe { SetCapture(hwnd) };
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let point = point_of(lparam);
            let invalidate = STATE.with(|cell| {
                let mut borrowed = cell.borrow_mut();
                let state = borrowed.as_mut()?;
                let monitor = state.monitor_of(hwnd)?.rect;
                let origin = (monitor.x, monitor.y);
                let drag = state.drag.as_mut()?;
                if drag.hwnd != hwnd.0 as isize {
                    return None;
                }
                let before = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
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
                let rect = DesktopRect::from_points(
                    drag.anchor.0,
                    drag.anchor.1,
                    drag.current.0,
                    drag.current.1,
                );
                if rect.is_empty() {
                    // A click with no drag is a cancellation, not a zero-pixel capture.
                    state.outcome = Outcome::Cancelled;
                } else {
                    state.outcome = Outcome::Selected(rect);
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

    /// The live selection in this window's client coordinates, if the drag belongs to it.
    fn client_selection(&self, hwnd: HWND) -> Option<RECT> {
        let drag = self.drag.as_ref()?;
        if drag.hwnd != hwnd.0 as isize {
            return None;
        }
        let origin = self.origin_of(hwnd)?;
        let rect =
            DesktopRect::from_points(drag.anchor.0, drag.anchor.1, drag.current.0, drag.current.1);
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

            // 3. the marquee border
            if let Some(sel) = selection {
                let _ = unsafe { FrameRect(hdc, &sel, state.border) };
            }
        });
    }

    let _ = unsafe { EndPaint(hwnd, &ps) };
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
}
