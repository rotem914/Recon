//! Start with Windows: one value under the current user's Run key (§3.1).
//!
//! The tray's "Start with Windows" tick writes `"<exe>" --startup` as the value `Recon`
//! under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, and unticking it removes
//! exactly that value, nothing else. Everything is under the user's own hive, so no
//! elevation is needed and nothing reaches another account. Started that way, Recon sits
//! in the tray with no window shown; started by hand, it opens its window as before.
//!
//! The everyday copy owns the value (Rotem, 2026-10-19): a start from the Recon in `app\`
//! re-points a value that names another copy to itself, so a release moved out of the
//! build folder does not leave Windows starting the old copy at logon, apart from the
//! taskbar pin. A copy anywhere else, a check build's included, never writes it.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
};

use crate::registration::{self, Entry};

/// Windows runs every value under this key at the user's logon.
pub const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
/// The value name Recon owns there.
pub const VALUE: &str = "Recon";
/// The flag the Run value passes, so a logon start is told from a start by hand.
pub const FLAG: &str = "--startup";
/// The folder whose copy owns the Run value: the everyday Recon lives in `app\`
/// (`project-os/Decisions.md`, 2026-10-15), and no build of the checks ever does.
pub const OWNER_FOLDER: &str = "app";

/// The command the Run value holds for this executable.
pub fn command_line(exe: &Path) -> String {
    format!("\"{}\" {FLAG}", exe.display())
}

/// What a start from `exe` writes over the Run value, if anything: this executable's
/// command, when the tick is on (`existing` is the value as it stands), the executable sits
/// in a folder named `app`, and the value names another command. The tick off, a copy
/// anywhere else, or a value already naming this copy, in any spelling of its case, since a
/// launcher's spelling of the path is what the executable reports: nothing.
pub fn repoint_wanted(exe: &Path, existing: Option<&str>) -> Option<String> {
    let existing = existing?;
    let folder = exe.parent()?.file_name()?.to_str()?;
    if !folder.eq_ignore_ascii_case(OWNER_FOLDER) {
        return None;
    }
    let command = command_line(exe);
    (!existing.eq_ignore_ascii_case(&command)).then_some(command)
}

/// Re-points the Run value to the running executable when `repoint_wanted` says so.
/// Returns the value replaced and the command written, or None when nothing was written.
/// A build of the checks never calls it, and never writes the key, wherever it runs from.
#[cfg_attr(feature = "stage0-checks", allow(dead_code))]
pub fn repoint() -> Result<Option<(String, String)>, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let existing = read(VALUE);
    match repoint_wanted(&exe, existing.as_deref()) {
        Some(command) => {
            set(VALUE, &command)?;
            Ok(Some((existing.unwrap_or_default(), command)))
        }
        None => Ok(None),
    }
}

/// Whether the tick is on: the value exists, whatever it holds.
pub fn is_enabled() -> bool {
    exists(VALUE)
}

/// Writes the Run value for the running executable. Returns the command written.
pub fn enable() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let command = command_line(&exe);
    set(VALUE, &command)?;
    Ok(command)
}

/// Removes Recon's Run value; a value already absent is the state wanted.
pub fn disable() -> Result<(), String> {
    remove(VALUE)
}

fn set(name: &str, command: &str) -> Result<(), String> {
    registration::write(&Entry {
        key: RUN_KEY.into(),
        name: Some(name.into()),
        value: command.into(),
    })
}

fn remove(name: &str) -> Result<(), String> {
    registration::delete_value(RUN_KEY, name)
}

fn exists(name: &str) -> bool {
    let key_name = registration::wide(RUN_KEY);
    let mut handle = HKEY::default();
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_QUERY_VALUE,
            &mut handle,
        )
    };
    if opened.0 != 0 {
        return false;
    }
    let value_name = registration::wide(name);
    let found =
        unsafe { RegQueryValueExW(handle, PCWSTR(value_name.as_ptr()), None, None, None, None) };
    let _ = unsafe { RegCloseKey(handle) };
    found.0 == 0
}

/// The string a value under the Run key holds; None when there is no such value, or it
/// cannot be read, which is taken as nothing to re-point.
fn read(name: &str) -> Option<String> {
    let key_name = registration::wide(RUN_KEY);
    let mut handle = HKEY::default();
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_QUERY_VALUE,
            &mut handle,
        )
    };
    if opened.0 != 0 {
        return None;
    }
    let value_name = registration::wide(name);
    let mut size = 0u32;
    let sized = unsafe {
        RegQueryValueExW(
            handle,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            None,
            Some(&mut size),
        )
    };
    let mut text = None;
    if sized.0 == 0 {
        // One unit of slack past the size, so an odd byte count never overruns; the text's
        // end is found below, at the first terminator or at the size, never relied on.
        let mut buffer = vec![0u16; (size as usize).div_ceil(2) + 1];
        let filled = unsafe {
            RegQueryValueExW(
                handle,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
        };
        if filled.0 == 0 {
            let units = (size as usize) / 2;
            let end = buffer[..units]
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(units);
            text = Some(String::from_utf16_lossy(&buffer[..end]));
        }
    }
    let _ = unsafe { RegCloseKey(handle) };
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_value_quotes_the_executable_and_passes_the_flag() {
        assert_eq!(
            command_line(Path::new("C:\\apps\\recon-host.exe")),
            "\"C:\\apps\\recon-host.exe\" --startup"
        );
    }

    #[test]
    fn the_everyday_copy_repoints_a_value_naming_another_copy() {
        let app = Path::new("J:\\Projects\\Recon\\app\\recon-host.exe");
        let old = "\"J:\\Projects\\Recon\\host\\target\\release\\recon-host.exe\" --startup";
        assert_eq!(repoint_wanted(app, Some(old)), Some(command_line(app)));
        // Spelled APP by hand: the same folder to Windows.
        let upper = Path::new("J:\\Projects\\Recon\\APP\\recon-host.exe");
        assert_eq!(repoint_wanted(upper, Some(old)), Some(command_line(upper)));
    }

    #[test]
    fn nothing_is_written_with_the_tick_off_or_the_value_naming_this_copy() {
        let app = Path::new("J:\\Projects\\Recon\\app\\recon-host.exe");
        assert_eq!(repoint_wanted(app, None), None);
        // The same copy in a launcher's own spelling of the case: nothing to re-point.
        assert_eq!(
            repoint_wanted(
                app,
                Some("\"j:\\projects\\recon\\APP\\recon-host.exe\" --startup")
            ),
            None
        );
        assert_eq!(repoint_wanted(app, Some(&command_line(app))), None);
    }

    #[test]
    fn a_copy_outside_app_never_repoints() {
        let value = "\"J:\\Projects\\Recon\\app\\recon-host.exe\" --startup";
        for other in [
            "J:\\Projects\\Recon\\host\\target\\release\\recon-host.exe",
            "J:\\Projects\\Recon\\host\\target\\debug\\recon-host.exe",
            "J:\\Projects\\Recon\\app\\previous\\recon-host.exe",
            "J:\\Projects\\Recon\\.tmp\\release-target\\release\\recon-host.exe",
        ] {
            assert_eq!(
                repoint_wanted(Path::new(other), Some(value)),
                None,
                "{other}"
            );
        }
    }

    /// Writes and removes a value of its own name under the real Run key, never Recon's,
    /// so it can run on a machine where the tick is on. Run by hand:
    /// `cargo test startup -- --ignored`.
    #[test]
    #[ignore]
    fn a_run_value_is_written_read_back_and_removed() {
        let name = "Recon (check)";
        let _ = remove(name);
        assert!(!exists(name));
        assert_eq!(read(name), None);
        set(name, "\"C:\\apps\\recon-host.exe\" --startup").unwrap();
        assert!(exists(name));
        assert_eq!(
            read(name).as_deref(),
            Some("\"C:\\apps\\recon-host.exe\" --startup")
        );
        remove(name).unwrap();
        assert!(!exists(name));
        assert_eq!(read(name), None);
    }
}
