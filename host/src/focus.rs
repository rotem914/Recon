//! The return target: the application the user was in when the capture began, and where
//! the focus goes back to when the editor hides (§3.1, S1.1).
//!
//! Remembered at the hotkey, before the freeze, because that is the last moment the
//! foreground window is still the user's. A capture started from inside Recon keeps the
//! previous target, so a chain of captures still returns to the application the work was
//! for. If that window has closed by the time the editor hides, nothing is activated:
//! Windows picks, and Recon never brings an unrelated window forward.

use std::sync::Mutex;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
};

/// The window to return to, as a raw handle so it can sit in a static.
static TARGET: Mutex<Option<isize>> = Mutex::new(None);

/// What a return did, for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Returned {
    /// The remembered window took the foreground.
    Restored,
    /// Windows refused the activation; the window is still there.
    Refused,
    /// The remembered window has closed since; nothing was activated.
    Gone,
    /// Nothing was ever remembered.
    Nothing,
}

impl Returned {
    pub fn line(self) -> &'static str {
        match self {
            Returned::Restored => "focus returned to the application the capture began in",
            Returned::Refused => "the application the capture began in refused the focus",
            Returned::Gone => "the application the capture began in has closed; nothing activated",
            Returned::Nothing => "no application to return to",
        }
    }
}

fn is_ours(hwnd: HWND) -> bool {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid == std::process::id()
}

/// Remembers the foreground window as the return target, unless it is one of Recon's own,
/// in which case the previous target stands.
pub fn remember_foreground() -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() || is_ours(hwnd) {
        return None;
    }
    remember(hwnd);
    Some(hwnd)
}

pub fn remember(hwnd: HWND) {
    if let Ok(mut slot) = TARGET.lock() {
        *slot = Some(hwnd.0 as isize);
    }
}

/// The return target as it stands, read by a capture before and after it remembers its own,
/// so that one taken with Ctrl held, which brings no editor up, can leave it as it found it.
pub fn target() -> Option<isize> {
    TARGET.lock().ok().and_then(|slot| *slot)
}

/// The target a capture taken with Ctrl held remembered, set aside for the one case where
/// its editor comes up after all: the copy failed (`take_back`).
static ASIDE: Mutex<Option<isize>> = Mutex::new(None);

/// A capture taken with Ctrl held brings no editor up: the target it remembered is set aside
/// and the one from before it put back. Unless a hide spent the target while the overlay was
/// up, in which case nothing is put back, since a spent target is used once (review T4).
pub fn set_aside(remembered: Option<isize>, before: Option<isize>) {
    let Ok(mut slot) = TARGET.lock() else {
        return;
    };
    let unchanged = *slot == remembered;
    if unchanged {
        *slot = before;
    }
    if let Ok(mut aside) = ASIDE.lock() {
        *aside = if unchanged { remembered } else { None };
    }
}

/// A new capture has begun: a target set aside by an earlier one is not the one to return to,
/// even when that one's copy fails late.
pub fn drop_aside() {
    if let Ok(mut aside) = ASIDE.lock() {
        *aside = None;
    }
}

/// The editor came up for that capture after all: its own target is the one to return to.
pub fn take_back() {
    let aside = ASIDE.lock().ok().and_then(|mut aside| aside.take());
    if let Some(raw) = aside {
        remember(HWND(raw as *mut _));
    }
}

/// Brings the remembered window forward, if it is still there. Never activates anything
/// else: a background completion or a closed application must not steal focus (§3.1).
///
/// The target is used once (review T4): a hide with no capture since the last return
/// activates nothing, so an editor opened from the tray or for a file closes without
/// bringing the last capture's application forward.
pub fn return_to_target() -> Returned {
    let remembered = match TARGET.lock() {
        Ok(mut slot) => slot.take(),
        Err(_) => None,
    };
    let Some(raw) = remembered else {
        return Returned::Nothing;
    };
    let hwnd = HWND(raw as *mut _);
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return Returned::Gone;
    }
    if unsafe { SetForegroundWindow(hwnd) }.as_bool() {
        Returned::Restored
    } else {
        Returned::Refused
    }
}
