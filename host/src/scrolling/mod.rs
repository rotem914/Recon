//! The scrolling capture (Rotem, 2026-09-22): the person scrolls a long page themselves, and
//! Recon joins what passes into one tall picture.
//!
//! It starts from the round button the capture overlay shows on an area that scrolls
//! (`detect`, and the button's drawing in `draw`). The overlay and its frozen picture go
//! away, since the page has to be live and take the wheel, and this module takes over on the
//! same thread: the screen stays dimmed around the area, the area itself is clear and
//! clickable, and a bar near its bottom says what to do and holds Done and cancel. Enter and
//! Escape do the same from the keyboard, taken as global keys for as long as this lasts,
//! because the page has the focus, not Recon.
//!
//! About thirty times a second the area is copied off the screen and handed to the stitcher
//! (`recon_pixels::stitch`), which places it or says it cannot. Recon's own windows here are
//! excluded from screen copies by Windows itself, so they can lie over the area and never be
//! in the picture. Measured here on 2026-09-22: a window excluded that way is absent from a
//! GDI copy of the screen, layered or not. Where Windows refuses the exclusion, the bar goes
//! outside the area, and when there is no room outside, the capture is refused in words.

pub mod detect;
pub mod draw;

use std::cell::RefCell;
use std::ffi::c_void;
use std::time::{Duration, Instant};

use recon_pixels::stitch::{Step, Stitcher};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetDC,
    ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION, CAPTUREBLT, DIB_RGB_COLORS,
    HBITMAP, HDC, HFONT, HGDIOBJ, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, ReleaseCapture, SetCapture, TrackMouseEvent, UnregisterHotKey, MOD_NOREPEAT,
    TME_LEAVE, TRACKMOUSEEVENT, VK_ESCAPE, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, KillTimer,
    LoadCursorW, PostQuitMessage, RegisterClassExW, SetTimer, SetWindowDisplayAffinity, ShowWindow,
    TranslateMessage, UpdateLayeredWindow, IDC_ARROW, MA_NOACTIVATE, MSG, SW_SHOWNOACTIVATE,
    ULW_ALPHA, WDA_EXCLUDEFROMCAPTURE, WM_HOTKEY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_MOUSEMOVE, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

use crate::capture::coords::{DesktopRect, FrameGeometry};
use crate::capture::display::{monitors, MonitorInfo};
use crate::capture::Frame;
use crate::magnifier::{scaled, solid_header, FILL_RGB};
use draw::{Canvas, BLUE_HOT_RGB, BLUE_RGB, BUTTON_RISE_PX, WHITE_RGB};

/// The longest picture: this many rows, and never more than this many pixels in all, which
/// is 320 MB of them in memory before the editor's own copies.
const MAX_ROWS: u32 = 30_000;
const MAX_PIXELS: u64 = 80_000_000;
/// How often the area is copied, and the tick that does it.
const GRAB_MS: u32 = 33;
const GRAB_TIMER: usize = 1;
/// A frame that cannot be placed is said so only once it has lasted this long: a page
/// half way through drawing itself is unplaced for a frame or two and is nobody's fault.
const LOST_AFTER: Duration = Duration::from_millis(350);
/// How long the last message stands before a full picture is handed over by itself.
const FULL_HOLD: Duration = Duration::from_millis(1400);
/// How far in from the area's sides a scrollbar is looked for, at 100%.
const SCROLLBAR_BAND_PX: i32 = 28;
/// How far a window's round corners reach into the area's top and bottom rows, at 100%:
/// Windows 11 rounds them on 8 px, and the blend at their edge reaches a little further.
const CORNER_PX: i32 = 12;
const KEY_DONE: i32 = 1;
const KEY_CANCEL: i32 = 2;

/// The dim around the area: the overlay's own, so the screen does not change its look when
/// the one takes over from the other. The frame is the overlay's dash colour, not walking.
const DIM_ALPHA: u8 = 140;
const FRAME_RGB: (u8, u8, u8) = (0x00, 0xB9, 0xF7);
const FRAME_PX: i32 = 2;

/// The bar. None of it is stated, so all of it is provisional (`project-os/Design.md`): as
/// tall as the round button it follows, 24 px up from the area's bottom like it, a pill in
/// the magnifier's fill, its text in the magnifier's face and white, amber when the page
/// got away; Done a blue pill, cancel a round X.
const BAR_HEIGHT_PX: i32 = 56;
const BAR_PAD_LEFT_PX: i32 = 24;
const BAR_PAD_RIGHT_PX: i32 = 8;
const BAR_GAP_PX: i32 = 16;
const BUTTON_HEIGHT_PX: i32 = 40;
const BUTTON_GAP_PX: i32 = 8;
const DONE_PAD_PX: i32 = 18;
const AMBER_RGB: (u8, u8, u8) = (0xFF, 0xB8, 0x4D);
const CANCEL_RGB: (u8, u8, u8) = (0x21, 0x22, 0x2C);
const CANCEL_HOT_RGB: (u8, u8, u8) = (0x32, 0x3A, 0x43);

const TEXT_START: &str = "Scroll down slowly";
const TEXT_LOST: &str = "Too fast. Scroll back up a little";
const TEXT_FULL: &str = "This is as long as a picture gets";
const TEXT_DONE: &str = "Done";
/// The widest the growing size can be written, so the bar never changes its width.
const TEXT_WIDEST_SIZE: &str = "88888x88888 so far";

pub enum Ended {
    /// The tall picture, as a capture's frame.
    Done(Frame),
    Cancelled,
    Failed(String),
}

#[derive(Clone, PartialEq, Eq)]
enum Message {
    Start,
    Growing(u32, u32),
    Lost,
    Full,
}

impl Message {
    fn text(&self) -> String {
        match self {
            Message::Start => TEXT_START.to_string(),
            Message::Growing(width, height) => format!("{width}x{height} so far"),
            Message::Lost => TEXT_LOST.to_string(),
            Message::Full => TEXT_FULL.to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Control {
    Done,
    Cancel,
}

/// The area copied off the screen again and again into one bitmap kept for the purpose.
struct Grabber {
    screen: HDC,
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut c_void,
    rect: DesktopRect,
}

impl Grabber {
    fn new(rect: DesktopRect) -> Option<Grabber> {
        unsafe {
            let screen = GetDC(None);
            if screen.is_invalid() {
                return None;
            }
            let dc = CreateCompatibleDC(Some(screen));
            let mut info = solid_header(rect.width as i32, rect.height as i32);
            let mut bits: *mut c_void = std::ptr::null_mut();
            let bitmap =
                match CreateDIBSection(Some(screen), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                    Ok(bitmap) if !dc.is_invalid() && !bitmap.is_invalid() && !bits.is_null() => {
                        bitmap
                    }
                    _ => {
                        let _ = DeleteDC(dc);
                        ReleaseDC(None, screen);
                        return None;
                    }
                };
            std::hint::black_box(&mut info);
            let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
            Some(Grabber {
                screen,
                dc,
                bitmap,
                previous,
                bits,
                rect,
            })
        }
    }

    /// The area as it is on screen now, BGRA, or None when the copy failed.
    fn grab(&mut self) -> Option<&[u8]> {
        unsafe {
            BitBlt(
                self.dc,
                0,
                0,
                self.rect.width as i32,
                self.rect.height as i32,
                Some(self.screen),
                self.rect.x,
                self.rect.y,
                SRCCOPY | CAPTUREBLT,
            )
            .ok()?;
            let _ = GdiFlush();
            Some(std::slice::from_raw_parts(
                self.bits as *const u8,
                self.rect.width as usize * self.rect.height as usize * 4,
            ))
        }
    }
}

impl Drop for Grabber {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
            ReleaseDC(None, self.screen);
        }
    }
}

/// Where the bar's pieces sit inside it, at one scale.
struct BarLayout {
    width: i32,
    height: i32,
    text_x: i32,
    done: (i32, i32, i32, i32),
    cancel: (i32, i32, i32, i32),
}

struct Session {
    region: DesktopRect,
    scale: u32,
    bar: HWND,
    bar_at: (i32, i32),
    layout: BarLayout,
    face: HFONT,
    grabber: Grabber,
    stitcher: Option<Stitcher>,
    message: Message,
    lost_since: Option<Instant>,
    full_since: Option<Instant>,
    hot: Option<Control>,
    pressed: Option<Control>,
    tracking: bool,
    frames: u32,
    started: Instant,
    ended: Option<Control>,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

/// Runs a scrolling capture of one area of the desktop, and blocks until the person ends
/// it. Called on the capture's own thread, after the overlay has gone; it owns a message
/// loop of its own for as long as it lasts.
pub fn run(region: DesktopRect) -> Ended {
    let Some(display) = monitors().into_iter().find(|m| {
        let (x, y) = (
            region.x + region.width as i32 / 2,
            region.y + region.height as i32 / 2,
        );
        x >= m.rect.x
            && y >= m.rect.y
            && x < m.rect.x + m.rect.width as i32
            && y < m.rect.y + m.rect.height as i32
    }) else {
        return Ended::Failed("the area is on no display".into());
    };
    let max_rows = (MAX_PIXELS / region.width.max(1) as u64).min(MAX_ROWS as u64) as u32;
    let Some(stitcher) = Stitcher::new(
        region.width,
        region.height,
        max_rows,
        scaled(SCROLLBAR_BAND_PX, display.scale_percent) as u32,
        scaled(CORNER_PX, display.scale_percent) as u32,
    ) else {
        return Ended::Failed("the area is empty".into());
    };
    let Some(grabber) = Grabber::new(region) else {
        return Ended::Failed("the screen could not be copied".into());
    };

    unsafe {
        register_classes();
        let face = crate::magnifier::font(display.scale_percent);
        let layout = bar_layout(face, display.scale_percent);

        let Some(bar) = make_window(w!("ReconScrollBar"), 0, 0, layout.width, layout.height) else {
            return Ended::Failed("the bar's window could not be made".into());
        };
        let bar_hidden = SetWindowDisplayAffinity(bar, WDA_EXCLUDEFROMCAPTURE).is_ok();
        let Some(bar_at) = place_bar(region, &display, &layout, bar_hidden) else {
            let _ = DestroyWindow(bar);
            return Ended::Failed(
                "Windows would not keep Recon's bar out of screen copies, and there is no room for it outside the area"
                    .into(),
            );
        };

        let veil = make_window(
            w!("ReconScrollVeil"),
            display.rect.x,
            display.rect.y,
            display.rect.width as i32,
            display.rect.height as i32,
        );
        if let Some(veil) = veil {
            let veil_hidden = SetWindowDisplayAffinity(veil, WDA_EXCLUDEFROMCAPTURE).is_ok();
            show_veil(veil, region, &display, veil_hidden);
        }
        crate::log(&format!(
            "scrolling: {}x{} at {},{}; the bar at {},{}, {} screen copies",
            region.width,
            region.height,
            region.x,
            region.y,
            bar_at.0,
            bar_at.1,
            if bar_hidden {
                "kept out of"
            } else {
                "NOT kept out of"
            }
        ));

        SESSION.with(|cell| {
            *cell.borrow_mut() = Some(Session {
                region,
                scale: display.scale_percent,
                bar,
                bar_at,
                layout,
                face,
                grabber,
                stitcher: Some(stitcher),
                message: Message::Start,
                lost_since: None,
                full_since: None,
                hot: None,
                pressed: None,
                tracking: false,
                frames: 0,
                started: Instant::now(),
                ended: None,
            })
        });
        redraw_bar();
        let _ = ShowWindow(bar, SW_SHOWNOACTIVATE);

        // Enter and Escape, for as long as this lasts: the page has the keyboard, not Recon.
        let enter = RegisterHotKey(Some(bar), KEY_DONE, MOD_NOREPEAT, VK_RETURN.0 as u32).is_ok();
        let escape =
            RegisterHotKey(Some(bar), KEY_CANCEL, MOD_NOREPEAT, VK_ESCAPE.0 as u32).is_ok();
        if !enter || !escape {
            crate::log(&format!(
                "scrolling: Enter {}, Escape {}; the bar's buttons do the same",
                if enter { "taken" } else { "NOT taken" },
                if escape { "taken" } else { "NOT taken" }
            ));
        }

        // The overlay's windows were destroyed a moment ago; the first copy waits until the
        // screen has been put together without them, or the frozen picture's frame and
        // button would be the top of the tall picture.
        let _ = DwmFlush();
        let _ = DwmFlush();
        std::thread::sleep(Duration::from_millis(150));
        SetTimer(Some(bar), GRAB_TIMER, GRAB_MS, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = KillTimer(Some(bar), GRAB_TIMER);
        if enter {
            let _ = UnregisterHotKey(Some(bar), KEY_DONE);
        }
        if escape {
            let _ = UnregisterHotKey(Some(bar), KEY_CANCEL);
        }
        let session = SESSION.with(|cell| cell.borrow_mut().take());
        let _ = DestroyWindow(bar);
        if let Some(veil) = veil {
            let _ = DestroyWindow(veil);
        }
        let Some(mut session) = session else {
            return Ended::Cancelled;
        };
        if !session.face.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(session.face.0));
        }
        if session.ended != Some(Control::Done) {
            crate::log("scrolling: cancelled, nothing captured");
            return Ended::Cancelled;
        }
        let Some(stitcher) = session.stitcher.take() else {
            return Ended::Cancelled;
        };
        let cut = stitcher.scrollbar();
        if cut != (0, 0) {
            crate::log(&format!(
                "scrolling: a scrollbar is cut off, {} px at the left and {} px at the right",
                cut.0, cut.1
            ));
        }
        let (mut pixels, width, height) = stitcher.finish();
        if width == 0 || height == 0 {
            return Ended::Failed("no frame of the area was ever copied".into());
        }
        // The screen's copies are BGRA with nothing in the alpha; a frame is RGBA, opaque.
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        crate::log(&format!(
            "scrolling: done, {width}x{height} from {} copies in {} ms",
            session.frames,
            session.started.elapsed().as_millis()
        ));
        Ended::Done(Frame {
            geometry: FrameGeometry {
                origin_x: session.region.x,
                origin_y: session.region.y,
                width,
                height,
            },
            rgba: pixels,
            source: "capture",
        })
    }
}

unsafe fn register_classes() {
    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    for (name, proc) in [
        (
            w!("ReconScrollBar"),
            bar_proc as unsafe extern "system" fn(_, _, _, _) -> _,
        ),
        (w!("ReconScrollVeil"), veil_proc),
    ] {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
            lpszClassName: name,
            ..Default::default()
        };
        // Registering twice is harmless, as it is for the overlay.
        unsafe { RegisterClassExW(&class) };
    }
}

/// A window that is its own picture, stays on top, and never takes the focus from the page.
unsafe fn make_window(class: PCWSTR, x: i32, y: i32, width: i32, height: i32) -> Option<HWND> {
    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            PCWSTR::null(),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            None,
        )
    }
    .ok()
    .filter(|hwnd| !hwnd.is_invalid())
}

unsafe fn show_layered(hwnd: HWND, at: (i32, i32), canvas: &Canvas) {
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    let _ = unsafe {
        UpdateLayeredWindow(
            hwnd,
            None,
            Some(&POINT { x: at.0, y: at.1 }),
            Some(&SIZE {
                cx: canvas.width,
                cy: canvas.height,
            }),
            Some(canvas.dc),
            Some(&POINT { x: 0, y: 0 }),
            windows::Win32::Foundation::COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )
    };
}

/// The dim over the whole display with the area cut clear out of it. Where a pixel is
/// wholly see-through the mouse goes through to the page; the dimmed rest takes the clicks
/// and drops them. The frame lies just inside the area, as the overlay's did, and only when
/// Windows keeps this window out of screen copies.
unsafe fn show_veil(veil: HWND, region: DesktopRect, display: &MonitorInfo, framed: bool) {
    let (width, height) = (display.rect.width as i32, display.rect.height as i32);
    let Some(mut canvas) = Canvas::new(width, height) else {
        return;
    };
    let (left, top) = (region.x - display.rect.x, region.y - display.rect.y);
    let (right, bottom) = (left + region.width as i32, top + region.height as i32);
    let frame = if framed { FRAME_PX } else { 0 };
    let (r, g, b) = FRAME_RGB;
    let pixels = canvas.pixels();
    for y in 0..height {
        let row = &mut pixels[(y * width * 4) as usize..((y + 1) * width * 4) as usize];
        for (x, pixel) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let x = x as i32;
            let inside = x >= left && x < right && y >= top && y < bottom;
            let deep =
                x >= left + frame && x < right - frame && y >= top + frame && y < bottom - frame;
            let colour = if !inside {
                [0, 0, 0, DIM_ALPHA]
            } else if !deep {
                [b, g, r, 255]
            } else {
                [0, 0, 0, 0]
            };
            pixel.copy_from_slice(&colour);
        }
    }
    unsafe { show_layered(veil, (display.rect.x, display.rect.y), &canvas) };
    let _ = unsafe { ShowWindow(veil, SW_SHOWNOACTIVATE) };
}

fn bar_layout(face: HFONT, scale: u32) -> BarLayout {
    let s = |px: i32| scaled(px, scale);
    let slot = [TEXT_START, TEXT_LOST, TEXT_FULL, TEXT_WIDEST_SIZE]
        .iter()
        .map(|text| draw::measure(face, text).0)
        .max()
        .unwrap_or(0);
    let height = s(BAR_HEIGHT_PX);
    let button = s(BUTTON_HEIGHT_PX);
    let top = (height - button) / 2;
    let done_width = draw::measure(face, TEXT_DONE).0 + 2 * s(DONE_PAD_PX);
    let text_x = s(BAR_PAD_LEFT_PX);
    let done_x = text_x + slot + s(BAR_GAP_PX);
    let cancel_x = done_x + done_width + s(BUTTON_GAP_PX);
    BarLayout {
        width: cancel_x + button + s(BAR_PAD_RIGHT_PX),
        height,
        text_x,
        done: (done_x, top, done_width, button),
        cancel: (cancel_x, top, button, button),
    }
}

/// Where the bar goes, in desktop pixels: in the middle of the area's width, 24 px up from
/// its bottom, where the round button was. An area too small to hold it, or a Windows that
/// will not keep it out of screen copies, puts it under the area, or over it; None when it
/// has to stay out of the area and nothing outside has room.
fn place_bar(
    region: DesktopRect,
    display: &MonitorInfo,
    layout: &BarLayout,
    hidden: bool,
) -> Option<(i32, i32)> {
    let rise = scaled(BUTTON_RISE_PX, display.scale_percent);
    let d = display.rect;
    let x = (region.x + (region.width as i32 - layout.width) / 2)
        .min(d.x + d.width as i32 - layout.width)
        .max(d.x);
    let bottom = region.y + region.height as i32;
    let fits = region.width as i32 >= layout.width + 2 * rise
        && region.height as i32 >= layout.height + 2 * rise;
    if hidden && fits {
        return Some((x, bottom - rise - layout.height));
    }
    let under = bottom + rise;
    if under + layout.height <= d.y + d.height as i32 {
        return Some((x, under));
    }
    let over = region.y - rise - layout.height;
    if over >= d.y {
        return Some((x, over));
    }
    hidden.then_some((x, (bottom - rise - layout.height).max(d.y)))
}

/// The bar's picture: its message, Done and cancel, either of them lighter under the pointer.
fn bar_picture(
    layout: &BarLayout,
    face: HFONT,
    scale: u32,
    message: &Message,
    hot: Option<Control>,
) -> Option<Canvas> {
    let mut canvas = Canvas::new(layout.width, layout.height)?;
    let s = |px: i32| scaled(px, scale);
    canvas.round_rect(
        0,
        0,
        layout.width,
        layout.height,
        layout.height / 2,
        FILL_RGB,
    );

    let (x, y, width, height) = layout.done;
    let done = if hot == Some(Control::Done) {
        BLUE_HOT_RGB
    } else {
        BLUE_RGB
    };
    canvas.round_rect(x, y, width, height, height / 2, done);

    let (cx, cy, side, _) = layout.cancel;
    let cancel = if hot == Some(Control::Cancel) {
        CANCEL_HOT_RGB
    } else {
        CANCEL_RGB
    };
    canvas.round_rect(cx, cy, side, side, side / 2, cancel);

    // The texts, on the solid fills they need, then the X over its disc.
    let text = message.text();
    let (_, text_height) = draw::measure(face, &text);
    let colour = if *message == Message::Lost {
        AMBER_RGB
    } else {
        WHITE_RGB
    };
    let slot = x - s(BAR_GAP_PX) - layout.text_x;
    canvas.text(
        face,
        layout.text_x,
        (layout.height - text_height) / 2,
        &text,
        colour,
        (layout.text_x, y, slot, height),
    );
    let (done_width, done_height) = draw::measure(face, TEXT_DONE);
    canvas.text(
        face,
        x + (width - done_width) / 2,
        y + (height - done_height) / 2,
        TEXT_DONE,
        WHITE_RGB,
        (x + height / 2, y, width - height, height),
    );
    let arm = side as f64 * 0.15;
    let (mx, my) = (cx as f64 + side as f64 / 2.0, cy as f64 + side as f64 / 2.0);
    canvas.lines(
        &[
            ((mx - arm, my - arm), (mx + arm, my + arm)),
            ((mx - arm, my + arm), (mx + arm, my - arm)),
        ],
        s(2) as f64,
        WHITE_RGB,
    );
    Some(canvas)
}

/// Draws the bar as the session stands and hands it to Windows as the window's picture.
fn redraw_bar() {
    SESSION.with(|cell| {
        let Ok(borrowed) = cell.try_borrow() else {
            return;
        };
        let Some(session) = borrowed.as_ref() else {
            return;
        };
        let picture = bar_picture(
            &session.layout,
            session.face,
            session.scale,
            &session.message,
            session.hot,
        );
        if let Some(canvas) = picture {
            unsafe { show_layered(session.bar, session.bar_at, &canvas) };
        }
    });
}

/// The bar and the round button as pictures to look at, since Windows keeps the bar out of
/// every copy of the screen: each of the bar's messages, then the bar with Done and with
/// cancel under the pointer, then the button as it is and under the pointer, one under the
/// other over a light ground and a dark one. RGBA, with its width and height.
#[cfg(feature = "stage0-checks")]
pub fn looks(scale: u32) -> Option<(Vec<u8>, u32, u32)> {
    let face = crate::magnifier::font(scale);
    let layout = bar_layout(face, scale);
    let mut pictures = Vec::new();
    for (message, hot) in [
        (Message::Start, None),
        (Message::Growing(1240, 5380), None),
        (Message::Lost, None),
        (Message::Full, None),
        (Message::Start, Some(Control::Done)),
        (Message::Start, Some(Control::Cancel)),
    ] {
        pictures.push(bar_picture(&layout, face, scale, &message, hot)?);
    }
    pictures.push(draw::scroll_button(scale, false)?);
    pictures.push(draw::scroll_button(scale, true)?);
    if !face.is_invalid() {
        let _ = unsafe { DeleteObject(HGDIOBJ(face.0)) };
    }
    let pad = scaled(16, scale);
    let column = layout.width + 2 * pad;
    let width = column * 2;
    let height = pictures.iter().map(|p| p.height + pad).sum::<i32>() + pad;
    let mut rgba = vec![255u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let ground = if x < column {
                [236, 238, 240]
            } else {
                [24, 26, 30]
            };
            let at = ((y * width + x) * 4) as usize;
            rgba[at..at + 3].copy_from_slice(&ground);
        }
    }
    let mut top = pad;
    for picture in pictures.iter_mut() {
        let (w, h) = (picture.width, picture.height);
        let pixels = picture.pixels().to_vec();
        for side in 0..2 {
            for y in 0..h {
                for x in 0..w {
                    let from = ((y * w + x) * 4) as usize;
                    let to = (((top + y) * width + side * column + pad + x) * 4) as usize;
                    let alpha = pixels[from + 3] as u32;
                    for (channel, source) in [(0, 2), (1, 1), (2, 0)] {
                        let under = rgba[to + channel] as u32;
                        rgba[to + channel] = (pixels[from + source] as u32
                            + under * (255 - alpha) / 255)
                            .min(255) as u8;
                    }
                }
            }
        }
        top += h + pad;
    }
    Some((rgba, width as u32, height as u32))
}

/// Which of the bar's two controls a point of the bar is on.
fn control_at(layout: &BarLayout, x: i32, y: i32) -> Option<Control> {
    let on = |(left, top, width, height): (i32, i32, i32, i32)| {
        x >= left && x < left + width && y >= top && y < top + height
    };
    if on(layout.done) {
        Some(Control::Done)
    } else if on(layout.cancel) {
        Some(Control::Cancel)
    } else {
        None
    }
}

fn end(with: Control) {
    SESSION.with(|cell| {
        if let Ok(mut borrowed) = cell.try_borrow_mut() {
            if let Some(session) = borrowed.as_mut() {
                session.ended = Some(with);
            }
        }
    });
    unsafe { PostQuitMessage(0) };
}

/// One copy of the area, handed to the stitcher, and what the bar says after it.
fn tick() {
    let outcome = SESSION.with(|cell| {
        let mut borrowed = cell.try_borrow_mut().ok()?;
        let session = borrowed.as_mut()?;
        if let Some(since) = session.full_since {
            return (since.elapsed() >= FULL_HOLD).then_some((false, true));
        }
        let stitcher = session.stitcher.as_mut()?;
        let step = stitcher.push(session.grabber.grab()?);
        session.frames += 1;
        let before = session.message.clone();
        match step {
            Step::First => crate::log("scrolling: the first copy is the picture's top"),
            Step::Unchanged | Step::Placed => session.lost_since = None,
            Step::Grew(_) => {
                session.lost_since = None;
                let (width, height) = stitcher.size();
                session.message = Message::Growing(width, height);
            }
            Step::Lost => {
                let since = *session.lost_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= LOST_AFTER && session.message != Message::Lost {
                    crate::log("scrolling: the page got away, waiting for it to come back");
                    session.message = Message::Lost;
                }
            }
            Step::Full => {
                crate::log("scrolling: the picture is as long as it may be");
                session.message = Message::Full;
                session.full_since = Some(Instant::now());
            }
        }
        if session.message == Message::Lost && session.lost_since.is_none() {
            crate::log("scrolling: the page is back");
            let (width, height) = stitcher.size();
            session.message = Message::Growing(width, height);
        }
        Some((session.message != before, false))
    });
    match outcome {
        Some((_, true)) => end(Control::Done),
        Some((true, _)) => redraw_bar(),
        _ => {}
    }
}

unsafe extern "system" fn veil_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn bar_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    /// WM_MOUSELEAVE, which the windows crate keeps with the controls.
    const WM_MOUSELEAVE: u32 = 0x02A3;
    let point = || {
        (
            (lparam.0 & 0xFFFF) as i16 as i32,
            ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
        )
    };
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_TIMER if wparam.0 == GRAB_TIMER => {
            tick();
            LRESULT(0)
        }
        WM_HOTKEY => {
            match wparam.0 as i32 {
                KEY_DONE => end(Control::Done),
                KEY_CANCEL => end(Control::Cancel),
                _ => {}
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE | WM_MOUSELEAVE => {
            let (x, y) = point();
            let changed = SESSION.with(|cell| {
                let mut borrowed = cell.try_borrow_mut().ok()?;
                let session = borrowed.as_mut()?;
                let hot = if msg == WM_MOUSELEAVE {
                    session.tracking = false;
                    None
                } else {
                    if !session.tracking {
                        let mut track = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        session.tracking = unsafe { TrackMouseEvent(&mut track) }.is_ok();
                    }
                    control_at(&session.layout, x, y)
                };
                let changed = hot != session.hot;
                session.hot = hot;
                Some(changed)
            });
            if changed == Some(true) {
                redraw_bar();
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let (x, y) = point();
            SESSION.with(|cell| {
                if let Ok(mut borrowed) = cell.try_borrow_mut() {
                    if let Some(session) = borrowed.as_mut() {
                        session.pressed = control_at(&session.layout, x, y);
                    }
                }
            });
            unsafe { SetCapture(hwnd) };
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = unsafe { ReleaseCapture() };
            let (x, y) = point();
            let chosen = SESSION.with(|cell| {
                let mut borrowed = cell.try_borrow_mut().ok()?;
                let session = borrowed.as_mut()?;
                let pressed = session.pressed.take()?;
                (control_at(&session.layout, x, y) == Some(pressed)).then_some(pressed)
            });
            if let Some(control) = chosen {
                end(control);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
