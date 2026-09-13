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

/// Brings the remembered window forward, if it is still there. Never activates anything
/// else: a background completion or a closed application must not steal focus (§3.1).
pub fn return_to_target() -> Returned {
    let remembered = match TARGET.lock() {
        Ok(slot) => *slot,
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
