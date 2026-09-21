//! Whether the thing under the pointer scrolls, asked while the capture overlay is up.
//!
//! The overlay's thread is the one surface that must never wait (part 5), and the answer
//! lives in another application, so it is asked on a thread of its own and sent back as a
//! message to the overlay's window. Two ways of asking, the cheap one first: a classic
//! control carries its scrollbar in its window style and says how far it goes; everything
//! else, a browser's page, a list drawn by its application, is asked through UI Automation,
//! walking down from the window towards the smallest element under the pointer and stopping
//! at the first one on the way that scrolls up and down: the page rather than a box inside
//! it. An element scrolls when it says so, or when something inside it is taller than it and
//! reaches past its top or its bottom, which is what content that has to be scrolled looks
//! like from outside.
//!
//! Measured here on 2026-09-22, against Edge 153 and an Electron application: a Chromium
//! page answers with an empty tree the first time it is asked, because being asked is what
//! makes it build the tree, and answers in full a moment later, so an empty answer is asked
//! again a few times before it counts as a no. And an ordinary web page never says it
//! scrolls, while the element holding its content reports its whole height, 9270 px inside
//! a part 861 px tall; a list inside the Electron application did say so itself. Hence both
//! signs.
//! Not established here: which applications never answer at all. For those the button does
//! not show, and the ordinary capture is what there is.

use std::sync::mpsc::Sender;

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationScrollPattern,
    TreeScope_Children, UIA_ScrollPatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetScrollInfo, GetWindowLongPtrW, PostMessageW, GWL_STYLE, SB_VERT, SCROLLINFO,
    SIF_PAGE, SIF_RANGE, WS_VSCROLL,
};

use crate::capture::coords::DesktopRect;

/// How many elements one question may look at, and how deep it may go.
const MAX_ELEMENTS: u32 = 1500;
const MAX_DEPTH: u32 = 48;
/// How far past its holder's top or bottom an element has to reach before the holder counts
/// as scrolling: a focus ring or a shadow reaches a few pixels out and means nothing.
const OVERFLOW_PX: i32 = 48;
/// An empty tree is asked again this many times, this far apart.
const RETRIES: u32 = 4;
const RETRY_MS: u64 = 300;

/// One answer: the window or part that was asked about, and the area of it that scrolls, in
/// desktop pixels, or None when nothing under the pointer does.
pub struct Answer {
    pub key: isize,
    pub scrolls: Option<DesktopRect>,
}

/// Asks, on a thread of its own, whether the window `key` scrolls at the desktop point
/// `at`. The answer goes down `reply`, and `wake` gets `message` posted so its thread reads
/// it. A window that is gone by then is a post that fails, which is fine.
pub fn ask(key: isize, at: (i32, i32), reply: Sender<Answer>, wake: isize, message: u32) {
    std::thread::spawn(move || {
        let scrolls = classic(key).or_else(|| automation(key, at));
        crate::log(&match scrolls {
            Some(r) => format!(
                "scrolling: the part under the pointer scrolls, {}x{} at {},{}",
                r.width, r.height, r.x, r.y
            ),
            None => "scrolling: nothing under the pointer says it scrolls".to_string(),
        });
        if reply.send(Answer { key, scrolls }).is_ok() {
            let _ =
                unsafe { PostMessageW(Some(HWND(wake as *mut _)), message, WPARAM(0), LPARAM(0)) };
        }
    });
}

/// A classic control: the scrollbar is in its style, and its range is longer than its page.
fn classic(window: isize) -> Option<DesktopRect> {
    let hwnd = HWND(window as *mut _);
    unsafe {
        if GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_VSCROLL.0 == 0 {
            return None;
        }
        let mut info = SCROLLINFO {
            cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
            fMask: SIF_RANGE | SIF_PAGE,
            ..Default::default()
        };
        GetScrollInfo(hwnd, SB_VERT, &mut info).ok()?;
        if info.nMax - info.nMin < info.nPage as i32 {
            return None;
        }
        // The client area: what scrolls, without the scrollbar beside it.
        let mut client = RECT::default();
        GetClientRect(hwnd, &mut client).ok()?;
        let mut corner = windows::Win32::Foundation::POINT::default();
        if !windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut corner).as_bool() {
            return None;
        }
        let rect = DesktopRect::from_points(
            corner.x,
            corner.y,
            corner.x + client.right,
            corner.y + client.bottom,
        );
        (!rect.is_empty()).then_some(rect)
    }
}

/// Everything else, through UI Automation.
fn automation(window: isize, at: (i32, i32)) -> Option<DesktopRect> {
    unsafe {
        let initialised = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let found = (|| {
            let uia: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            for attempt in 0..=RETRIES {
                let (found, looked) = walk(&uia, window, at);
                if found.is_some() || looked > 0 || attempt == RETRIES {
                    return found;
                }
                std::thread::sleep(std::time::Duration::from_millis(RETRY_MS));
            }
            None
        })();
        if initialised {
            CoUninitialize();
        }
        found
    }
}

/// One walk down from the window: the first element on the way to the point that scrolls
/// up and down, and how many elements under the window were looked at.
unsafe fn walk(uia: &IUIAutomation, window: isize, at: (i32, i32)) -> (Option<DesktopRect>, u32) {
    let Ok(mut element) = (unsafe { uia.ElementFromHandle(HWND(window as *mut _)) }) else {
        return (None, 0);
    };
    let Ok(everything) = (unsafe { uia.CreateTrueCondition() }) else {
        return (None, 0);
    };
    let mut looked = 0;
    for _ in 0..MAX_DEPTH {
        if let Some(rect) = unsafe { scrolls(&element) } {
            return (Some(rect), looked);
        }
        let holder = unsafe { element.CurrentBoundingRectangle() }.unwrap_or_default();
        let Ok(children) = (unsafe { element.FindAll(TreeScope_Children, &everything) }) else {
            break;
        };
        let count = unsafe { children.Length() }.unwrap_or(0);
        // The smallest child under the point is the way down; any child taller than its
        // holder and reaching past it says the holder scrolls.
        let mut next: Option<(IUIAutomationElement, u64)> = None;
        let mut overflows = false;
        for index in 0..count {
            looked += 1;
            if looked > MAX_ELEMENTS {
                return (None, looked);
            }
            let Ok(child) = (unsafe { children.GetElement(index) }) else {
                continue;
            };
            let Ok(r) = (unsafe { child.CurrentBoundingRectangle() }) else {
                continue;
            };
            if r.bottom - r.top > holder.bottom - holder.top
                && r.right > holder.left
                && r.left < holder.right
                && (r.top < holder.top - OVERFLOW_PX || r.bottom > holder.bottom + OVERFLOW_PX)
            {
                overflows = true;
            }
            if at.0 < r.left || at.0 >= r.right || at.1 < r.top || at.1 >= r.bottom {
                continue;
            }
            let area = (r.right - r.left) as u64 * (r.bottom - r.top) as u64;
            if next.as_ref().is_none_or(|(_, held)| area <= *held) {
                next = Some((child, area));
            }
        }
        if overflows {
            let rect =
                DesktopRect::from_points(holder.left, holder.top, holder.right, holder.bottom);
            if !rect.is_empty() {
                return (Some(rect), looked);
            }
        }
        match next {
            Some((child, _)) => element = child,
            None => break,
        }
    }
    (None, looked)
}

/// The element's area when it says it scrolls up and down.
unsafe fn scrolls(element: &IUIAutomationElement) -> Option<DesktopRect> {
    let pattern: IUIAutomationScrollPattern =
        unsafe { element.GetCurrentPatternAs(UIA_ScrollPatternId) }.ok()?;
    if !unsafe { pattern.CurrentVerticallyScrollable() }
        .ok()?
        .as_bool()
    {
        return None;
    }
    let r = unsafe { element.CurrentBoundingRectangle() }.ok()?;
    let rect = DesktopRect::from_points(r.left, r.top, r.right, r.bottom);
    (!rect.is_empty()).then_some(rect)
}
