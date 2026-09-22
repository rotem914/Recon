//! Whether the thing under the pointer scrolls, asked while the capture overlay is up.
//!
//! The overlay's thread is the one surface that must never wait (part 5), and the answer
//! lives in another application, so it is asked on a thread of its own and sent back as a
//! message to the overlay's window. Two ways of asking, the cheap one first: a classic
//! control carries its scrollbar in its window style and says how far it goes; everything
//! else, a browser's page, a list drawn by its application, is asked through UI Automation,
//! walking down from the window towards the smallest element under the pointer and stopping
//! at the first one on the way that scrolls up and down: the page rather than a box inside
//! it. An element scrolls when it says so, or when what is inside it, taken together, is
//! taller than it and reaches past its top or its bottom, which is what content that has to
//! be scrolled looks like from outside. Taken together means one run of children that follow
//! each other down the page from the ones in view, never a far one on its own: a link kept
//! out of sight at -9999 px, or a closed menu parked below, says nothing about scrolling. And
//! the run has to reach past the holder by more than a hidden "skip to content" link parked
//! just above a page does. Both distances are at 100% and grow with the display's scale,
//! since UI Automation hands rectangles over in physical pixels and a page's spacing grows
//! with the scale; a page zoomed in its browser still has its spacing grow past them.
//!
//! Measured here on 2026-09-22, against Edge 153 and an Electron application: a Chromium
//! page answers with an empty tree the first time it is asked, because being asked is what
//! makes it build the tree, and answers in full a moment later, so an empty answer is asked
//! again a few times before it counts as a no. And an ordinary web page never says it
//! scrolls, while the element holding its content reports its whole height, 9270 px inside
//! a part 861 px tall; a list inside the Electron application did say so itself. Hence both
//! signs. And a page that scrolls a panel of its own, not the whole page, holds sections each
//! shorter than the panel, which only reach past it together: Rotem's design system page, on
//! 2026-09-22, six sections from 229 to 2127 in a panel from 197 to 1192.
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
/// How far past its holder's top or bottom the content has to reach before the holder counts
/// as scrolling, at 100%: a focus ring, a shadow, or a skip link parked 40 px above the page
/// reach out and mean nothing.
const OVERFLOW_PX: i32 = 64;
/// Children closer than this, one under the next, are one run of content, at 100%: a page's
/// sections have margins between them, never a screen's height.
const RUN_GAP_PX: i32 = 48;
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
/// `scale` is the display's, in percent, for the distances measured in its pixels.
pub fn ask(
    key: isize,
    at: (i32, i32),
    scale: u32,
    reply: Sender<Answer>,
    wake: isize,
    message: u32,
) {
    std::thread::spawn(move || {
        let scrolls = classic(key).or_else(|| automation(key, at, scale));
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
fn automation(window: isize, at: (i32, i32), scale: u32) -> Option<DesktopRect> {
    unsafe {
        let initialised = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let found = (|| {
            let uia: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            for attempt in 0..=RETRIES {
                let (found, looked) = walk(&uia, window, at, scale);
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
unsafe fn walk(
    uia: &IUIAutomation,
    window: isize,
    at: (i32, i32),
    scale: u32,
) -> (Option<DesktopRect>, u32) {
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
        // The smallest child under the point is the way down; the run of children in the
        // holder's width that reaches from the ones in view says whether the holder scrolls.
        let mut next: Option<(IUIAutomationElement, u64)> = None;
        let mut spans: Vec<(i32, i32)> = Vec::new();
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
            if r.right > holder.left && r.left < holder.right && r.bottom > r.top {
                spans.push((r.top, r.bottom));
            }
            if at.0 < r.left || at.0 >= r.right || at.1 < r.top || at.1 >= r.bottom {
                continue;
            }
            let area = (r.right - r.left) as u64 * (r.bottom - r.top) as u64;
            if next.as_ref().is_none_or(|(_, held)| area <= *held) {
                next = Some((child, area));
            }
        }
        if overflows((holder.top, holder.bottom), &spans, scale) {
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

/// Whether a holder, top and bottom as given, scrolls by the children in its width: their
/// run is taller than it and reaches past its top or bottom by more than `OVERFLOW_PX`, both
/// at the display's scale.
fn overflows(holder: (i32, i32), spans: &[(i32, i32)], scale: u32) -> bool {
    let reach = crate::magnifier::scaled(OVERFLOW_PX, scale);
    let gap = crate::magnifier::scaled(RUN_GAP_PX, scale);
    run_through(holder, spans, gap).is_some_and(|(top, bottom)| {
        bottom - top > holder.1 - holder.0 && (top < holder.0 - reach || bottom > holder.1 + reach)
    })
}

/// From the children that are in view, top and bottom as given, out along every child that
/// follows on within `gap` above or below: how high and how low the content reaches. None
/// when no child is in view.
fn run_through(holder: (i32, i32), spans: &[(i32, i32)], gap: i32) -> Option<(i32, i32)> {
    let mut run = spans
        .iter()
        .filter(|(top, bottom)| *bottom > holder.0 && *top < holder.1)
        .fold(None, |run: Option<(i32, i32)>, &(top, bottom)| {
            Some(run.map_or((top, bottom), |(t, b)| (t.min(top), b.max(bottom))))
        })?;
    loop {
        let grown = spans.iter().fold(run, |(t, b), &(top, bottom)| {
            if bottom >= t - gap && top <= b + gap {
                (t.min(top), b.max(bottom))
            } else {
                (t, b)
            }
        });
        if grown == run {
            return Some(run);
        }
        run = grown;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Rotem's design system page as Edge reported it at 100%: a panel from 197 to 1192, a
    /// headline and five sections with 32 px between them, the last three below the panel's
    /// bottom, and the separator the panel's own height.
    const PANEL: (i32, i32) = (197, 1192);
    const SECTIONS: [(i32, i32); 7] = [
        (229, 259),
        (291, 621),
        (653, 1215),
        (1247, 1519),
        (1551, 1823),
        (1855, 2127),
        (197, 1192),
    ];

    fn at_scale(spans: &[(i32, i32)], scale: u32) -> Vec<(i32, i32)> {
        let s = |v: i32| v * scale as i32 / 100;
        spans.iter().map(|&(t, b)| (s(t), s(b))).collect()
    }

    #[test]
    fn sections_that_follow_each_other_past_the_holder_are_one_run() {
        assert_eq!(run_through(PANEL, &SECTIONS, 48), Some((197, 2127)));
    }

    #[test]
    fn the_design_system_page_scrolls_at_every_scale() {
        // Its 32 px margins grow with the display, and so does the gap that joins a run.
        for scale in [100, 125, 150, 225, 300, 400] {
            let s = |v: i32| v * scale as i32 / 100;
            let holder = (s(PANEL.0), s(PANEL.1));
            assert!(
                overflows(holder, &at_scale(&SECTIONS, scale), scale),
                "at {scale}%"
            );
        }
    }

    #[test]
    fn a_skip_link_parked_above_a_page_that_does_not_scroll_is_no_scroll() {
        // A page that fits, header and main filling it, and a "skip to content" link parked
        // 40 px above it: a reviewer's case, at 225%.
        let spans = [(41, 131), (131, 275), (275, 992)];
        assert!(!overflows((131, 992), &spans, 225));
    }

    #[test]
    fn a_child_far_away_on_its_own_is_not_part_of_the_run() {
        // A link kept out of sight far above, a closed menu parked far below: the content
        // in view stays inside the holder.
        let spans = [(-9999, -9970), (220, 900), (940, 1180), (4200, 4600)];
        assert_eq!(run_through((200, 1200), &spans, 48), Some((220, 1180)));
        assert!(!overflows((200, 1200), &spans, 100));
    }

    #[test]
    fn one_child_taller_than_the_holder_is_a_run_on_its_own() {
        // An ordinary web page: the element holding its content is its whole height.
        assert!(overflows((131, 992), &[(187, 9457)], 100));
        // Scrolled half way down, the content reaches above the holder too.
        assert!(overflows((131, 992), &[(-4000, 5270)], 100));
    }

    #[test]
    fn no_child_in_view_is_no_run() {
        assert_eq!(
            run_through((0, 500), &[(900, 1400), (-800, -100)], 48),
            None
        );
        assert_eq!(run_through((0, 500), &[], 48), None);
        assert!(!overflows((0, 500), &[(900, 1400)], 100));
    }
}
