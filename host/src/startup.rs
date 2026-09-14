//! Start with Windows: one value under the current user's Run key (§3.1).
//!
//! The tray's "Start with Windows" tick writes `"<exe>" --startup` as the value `Recon`
//! under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, and unticking it removes
//! exactly that value, nothing else. Everything is under the user's own hive, so no
//! elevation is needed and nothing reaches another account. Started that way, Recon sits
//! in the tray with no window shown; started by hand, it opens its window as before.

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

/// The command the Run value holds for this executable.
pub fn command_line(exe: &Path) -> String {
    format!("\"{}\" {FLAG}", exe.display())
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

    /// Writes and removes a value of its own name under the real Run key, never Recon's,
    /// so it can run on a machine where the tick is on. Run by hand:
    /// `cargo test startup -- --ignored`.
    #[test]
    #[ignore]
    fn a_run_value_is_written_read_back_and_removed() {
        let name = "Recon (check)";
        let _ = remove(name);
        assert!(!exists(name));
        set(name, "\"C:\\apps\\recon-host.exe\" --startup").unwrap();
        assert!(exists(name));
        remove(name).unwrap();
        assert!(!exists(name));
    }
}
