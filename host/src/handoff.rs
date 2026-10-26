//! The second instance: a Recon started while another one runs (§3.1).
//!
//! A click on the taskbar pin or the Start menu while Recon sits in the tray, or Explorer
//! opening a file with it, starts a second Recon. It hands what it was asked over to the one
//! running, and quits. Until 2026-10-26 tauri-plugin-single-instance did the hand-off, and
//! quit the moment the message was delivered; the running Recon, told to show its window,
//! then asked Windows for the front and was refused. Windows gives the right to raise a window
//! to the program the click started, not to one that has sat in the tray since logon, and the
//! right handed over with `AllowSetForegroundWindow` (2026-10-10), then asked for again for a
//! second and a half (2026-10-15), did not survive the real click: revoked as the shell
//! finished the launch, or never granted, the product keeps no record of which. So the second
//! instance now does the hand-off itself, with the same message the plugin's receiver reads,
//! and then raises the editor window from this process, with its own right, asking again
//! while the Start menu is still closing, and quits only then (Rotem, 2026-10-26; option 4 of
//! the 2026-10-15 decision). A logon start is left to the plugin: no click is behind it.
//!
//! Only the sending half is Recon's own. The running Recon still receives the message through
//! the plugin's window, named after the identifier, so the format here is the plugin's, version
//! 2.4.4, and a plugin update is read against it.

use std::time::{Duration, Instant};

use windows::core::{w, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindow, IsWindowVisible, SendMessageW, SetForegroundWindow, WM_COPYDATA,
};

/// The running Recon: the plugin's hand-off window and the process that owns it.
pub struct Running {
    hwnd: HWND,
    pub pid: u32,
}

/// The plugin's own mark on the message, in `dwData`: its receiver reads no other.
const SINGLE_INSTANCE_DATA: usize = 1542;

/// What every editor window title ends with (`editor::set_title`): plain for a capture, the
/// file's name in front of it for a file.
const EDITOR_TITLE: &str = "Recon Editor";

/// How long to wait for the editor window to come on screen after the hand-off: a bare start
/// shows it before the message returns, a file opens on a thread of the running Recon, decoded
/// first, and its folder listed, which a large picture or a slow share can stretch. This
/// process has no window, so a long wait costs nothing on screen (review, 2026-10-26); a
/// running Recon that quits meanwhile ends it.
const WAIT_SHOWN: Duration = Duration::from_secs(30);

/// How often, and how many times, a window still behind is asked for the front: the same
/// second and a half the running Recon gives it (`editor::bring_forward_again`).
const ASK_EVERY: Duration = Duration::from_millis(100);
const ASKS: u32 = 15;

/// How long, at most, this process is raising the window after the hand-off: the wait and the
/// asks together. The running Recon leaves its own focus call out for that long
/// (`editor::launcher_raising_for`).
pub const RAISE_FOR: Duration = Duration::from_millis(31_500);

/// The Recon already running, by the hand-off window the single-instance plugin names after
/// the identifier; none on a cold start.
pub fn running() -> Option<Running> {
    let hwnd = unsafe { FindWindowW(w!("com.rotem.recon-sic"), w!("com.rotem.recon-siw")) }.ok()?;
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    (pid != 0).then_some(Running { hwnd, pid })
}

/// Hands this start over to the running Recon, raises its editor window, and says in the log
/// how that went. The caller quits after it.
pub fn run(running: &Running) {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir()
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_default();
    let data = payload(&cwd, &args);
    let cds = COPYDATASTRUCT {
        dwData: SINGLE_INSTANCE_DATA,
        cbData: data.len() as u32,
        lpData: data.as_ptr() as *mut _,
    };
    // Returns once the running Recon has read it: for a bare start, with its window shown.
    unsafe {
        SendMessageW(
            running.hwnd,
            WM_COPYDATA,
            Some(WPARAM(0)),
            Some(LPARAM(&cds as *const COPYDATASTRUCT as isize)),
        )
    };
    crate::log(&format!(
        "second instance: handed over to pid {}",
        running.pid
    ));
    match raise(running) {
        Raised::InFront(asks) => say(&format!(
            "second instance: editor in front after {asks} asks"
        )),
        Raised::Hidden => {
            say("second instance: editor hidden again before it was in front, left alone")
        }
        Raised::Gone => say("second instance: the running Recon quit meanwhile, nothing raised"),
        Raised::NotShown => say(&format!(
            "second instance: no editor window on screen within {} s, nothing raised",
            WAIT_SHOWN.as_secs()
        )),
        Raised::StillBehind => {
            say(&format!(
                "second instance: editor still behind after {ASKS} asks"
            ));
            crate::log(&format!(
                "in front: {}",
                crate::platform::foreground_owner().line()
            ));
        }
    }
}

/// The plugin's wire format: the working folder, then every argument, the executable's path
/// first, all joined by `|` and ended by a NUL. Its receiver splits on `|`, takes the first
/// part as the folder, and the callback skips the path.
fn payload(cwd: &str, args: &[String]) -> Vec<u8> {
    format!("{cwd}|{}\0", args.join("|")).into_bytes()
}

/// One line for the log and, since this process is about to quit, straight into the timing
/// log, where a product run keeps it (2026-10-26). Never a window's name: the file's rule.
fn say(line: &str) {
    crate::log(line);
    if let Err(err) = crate::logfile::write_now(line) {
        crate::log(&format!("timing log NOT written: {err}"));
    }
}

enum Raised {
    /// In front, after this many asks from here; 0 when the show itself was enough.
    InFront(u32),
    /// Hidden or minimized again before it was in front: a person's act, left alone.
    Hidden,
    /// The running Recon quit while this one waited: its hand-off window is gone.
    Gone,
    /// No editor window on screen within the wait.
    NotShown,
    /// Still behind after every ask, each refused.
    StillBehind,
}

/// Waits for the running Recon's editor window to be on screen, then asks for the front from
/// this process until the running Recon has it, or the window went again, or the asks ran out.
/// A refused ask changes nothing but the taskbar button's flash, as the refused show did.
fn raise(running: &Running) -> Raised {
    let pid = running.pid;
    let started = Instant::now();
    let hwnd = loop {
        if let Some(hwnd) = editor_window(pid).filter(|&hwnd| shown(hwnd)) {
            break hwnd;
        }
        if !unsafe { IsWindow(Some(running.hwnd)) }.as_bool() {
            return Raised::Gone;
        }
        if started.elapsed() >= WAIT_SHOWN {
            return Raised::NotShown;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    for asked in 0..ASKS {
        if front_belongs_to(pid) {
            return Raised::InFront(asked);
        }
        if !shown(hwnd) {
            return Raised::Hidden;
        }
        let _ = unsafe { SetForegroundWindow(hwnd) };
        std::thread::sleep(ASK_EVERY);
    }
    if front_belongs_to(pid) {
        Raised::InFront(ASKS)
    } else {
        Raised::StillBehind
    }
}

/// Whether the window in front is the running Recon's: the editor, or its capture overlay.
fn front_belongs_to(pid: u32) -> bool {
    let front = unsafe { GetForegroundWindow() };
    if front.is_invalid() {
        return false;
    }
    let mut owner = 0u32;
    unsafe { GetWindowThreadProcessId(front, Some(&mut owner)) };
    owner == pid
}

/// Whether the window is on screen, visible and not minimized, read from Windows itself.
fn shown(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() }
}

/// The running Recon's editor window: its top-level window whose title is the editor's. The
/// plugin's own window carries the identifier as its title, so it is never taken for it.
fn editor_window(pid: u32) -> Option<HWND> {
    let mut wanted = Wanted {
        pid,
        found: HWND::default(),
    };
    // Err when the search stopped at a match, which is the callback's own doing.
    let _ = unsafe {
        EnumWindows(
            Some(find_editor),
            LPARAM(&mut wanted as *mut Wanted as isize),
        )
    };
    (!wanted.found.is_invalid()).then_some(wanted.found)
}

struct Wanted {
    pid: u32,
    found: HWND,
}

unsafe extern "system" fn find_editor(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let wanted = unsafe { &mut *(lparam.0 as *mut Wanted) };
    let keep_looking = BOOL(1);
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid != wanted.pid {
        return keep_looking;
    }
    // Room for a long file name in front of the editor's own words (review, 2026-10-26).
    let mut buffer = [0u16; 1024];
    let n = unsafe { GetWindowTextW(hwnd, &mut buffer) }.max(0) as usize;
    if is_editor_title(&String::from_utf16_lossy(&buffer[..n])) {
        wanted.found = hwnd;
        BOOL(0)
    } else {
        keep_looking
    }
}

fn is_editor_title(title: &str) -> bool {
    title.ends_with(EDITOR_TITLE)
}

#[cfg(test)]
mod tests {
    use super::{is_editor_title, payload, ASKS, ASK_EVERY, RAISE_FOR, WAIT_SHOWN};

    #[test]
    fn the_running_recon_is_told_how_long_the_raising_lasts() {
        assert_eq!(RAISE_FOR, WAIT_SHOWN + ASK_EVERY * ASKS);
    }

    /// The receiver's side of the format, as tauri-plugin-single-instance 2.4.4 reads it: a
    /// C string split on `|`, the folder first, then the arguments the callback gets.
    fn receiver_reads(bytes: &[u8]) -> (String, Vec<String>) {
        let end = bytes
            .iter()
            .position(|&b| b == 0)
            .expect("a NUL ends the message");
        let text = String::from_utf8_lossy(&bytes[..end]);
        let mut parts = text.split('|');
        let cwd = parts.next().unwrap().to_string();
        (cwd, parts.map(|s| s.to_string()).collect())
    }

    #[test]
    fn the_payload_is_what_the_plugins_receiver_reads() {
        let args = vec![
            r"J:\Projects\Recon\app\recon-host.exe".to_string(),
            "--open".to_string(),
            r"C:\pictures\a photo.png".to_string(),
        ];
        let bytes = payload(r"C:\pictures", &args);
        assert_eq!(bytes.last(), Some(&0));
        let (cwd, read) = receiver_reads(&bytes);
        assert_eq!(cwd, r"C:\pictures");
        assert_eq!(read, args, "every argument, the executable's path first");
    }

    #[test]
    fn a_bare_start_hands_over_the_path_alone() {
        let args = vec![r"J:\Projects\Recon\app\recon-host.exe".to_string()];
        let (_, read) = receiver_reads(&payload("", &args));
        assert_eq!(read, args);
        assert_eq!(
            read.iter().skip(1).count(),
            0,
            "nothing to open after the skip"
        );
    }

    #[test]
    fn the_editor_is_known_by_its_title_with_or_without_a_file() {
        assert!(is_editor_title("Recon Editor"));
        assert!(is_editor_title("a photo.png - Recon Editor"));
        assert!(
            !is_editor_title("com.rotem.recon-siw"),
            "the plugin's own window"
        );
        assert!(!is_editor_title("ReconOverlay"));
        assert!(!is_editor_title(""));
    }
}
