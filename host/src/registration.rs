//! The file types Recon can open, registered with Windows for this user (§3.1, S1.2).
//!
//! Two things, and a line that is never crossed. Recon appears in "Open with" for every
//! type it displays, through an `OpenWithProgids` entry on each extension; and it is listed
//! in Windows' own Default apps settings, through a Capabilities key and the
//! RegisteredApplications list, so the user can make it the default viewer there. What it
//! never does is take an association itself: no extension's default value is written, and
//! an association the user changed is not re-taken (§3.1, "Registration and defaults").
//!
//! Everything is under the current user's hive, so no elevation is needed and nothing
//! reaches another account. `--register-types` writes it, `--unregister-types` removes
//! exactly what was written, and both are run by the user, never on their own.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// The programmatic identifier every registered type points at.
pub const PROG_ID: &str = "Recon.Image";

/// The extensions of the formats in §3.7, as Windows spells them.
pub const EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".bmp", ".gif", ".webp", ".tif", ".tiff", ".heic", ".heif", ".avif",
    ".svg",
];

/// One registry value to write: a key path under the current user's hive, a value name
/// (None for the key's default value), and a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub name: Option<String>,
    pub value: String,
}

/// Everything registration writes, as data, so it can be read and tested before a single
/// key is touched. Nothing here names an extension's default value.
pub fn plan(exe: &Path) -> Vec<Entry> {
    let exe = exe.display().to_string();
    let mut out = vec![
        Entry {
            key: format!("Software\\Classes\\{PROG_ID}"),
            name: None,
            value: "Image (Recon)".into(),
        },
        Entry {
            key: format!("Software\\Classes\\{PROG_ID}\\DefaultIcon"),
            name: None,
            value: format!("\"{exe}\",0"),
        },
        Entry {
            key: format!("Software\\Classes\\{PROG_ID}\\shell\\open\\command"),
            name: None,
            value: format!("\"{exe}\" \"%1\""),
        },
        Entry {
            key: "Software\\Recon\\Capabilities".into(),
            name: Some("ApplicationName".into()),
            value: "Recon".into(),
        },
        Entry {
            key: "Software\\Recon\\Capabilities".into(),
            name: Some("ApplicationDescription".into()),
            value: "Image viewer, screen capture and annotation".into(),
        },
        Entry {
            key: "Software\\RegisteredApplications".into(),
            name: Some("Recon".into()),
            value: "Software\\Recon\\Capabilities".into(),
        },
    ];
    for ext in EXTENSIONS {
        // "Open with" lists Recon for the type; the type's own default is untouched.
        out.push(Entry {
            key: format!("Software\\Classes\\{ext}\\OpenWithProgids"),
            name: Some(PROG_ID.into()),
            value: String::new(),
        });
        // Default apps settings can offer Recon for the type.
        out.push(Entry {
            key: "Software\\Recon\\Capabilities\\FileAssociations".into(),
            name: Some((*ext).into()),
            value: PROG_ID.into(),
        });
    }
    out
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn write(entry: &Entry) -> Result<(), String> {
    let key_name = wide(&entry.key);
    let mut key = HKEY::default();
    let created = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
    };
    if created.0 != 0 {
        return Err(format!(
            "{} could not be created ({})",
            entry.key, created.0
        ));
    }
    let name = entry.name.as_deref().map(wide);
    let value = wide(&entry.value);
    let bytes: Vec<u8> = value.iter().flat_map(|c| c.to_le_bytes()).collect();
    let set = unsafe {
        RegSetValueExW(
            key,
            name.as_ref()
                .map(|n| PCWSTR(n.as_ptr()))
                .unwrap_or(PCWSTR::null()),
            None,
            REG_SZ,
            Some(&bytes),
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if set.0 != 0 {
        return Err(format!(
            "{}\\{} could not be set ({})",
            entry.key,
            entry.name.as_deref().unwrap_or("(default)"),
            set.0
        ));
    }
    Ok(())
}

/// Writes every entry of the plan for this executable. Returns how many were written.
pub fn register() -> Result<usize, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let entries = plan(&exe);
    for entry in &entries {
        write(entry)?;
    }
    Ok(entries.len())
}

fn delete_tree(key: &str) -> Result<(), String> {
    let name = wide(key);
    let result = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(name.as_ptr())) };
    // 2 is "not found", which after an unregister is the state wanted.
    if result.0 != 0 && result.0 != 2 {
        return Err(format!("{key} could not be removed ({})", result.0));
    }
    Ok(())
}

fn delete_value(key: &str, value: &str) -> Result<(), String> {
    let key_name = wide(key);
    let mut handle = HKEY::default();
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut handle,
        )
    };
    if opened.0 == 2 {
        return Ok(());
    }
    if opened.0 != 0 {
        return Err(format!("{key} could not be opened ({})", opened.0));
    }
    let value_name = wide(value);
    let deleted = unsafe { RegDeleteValueW(handle, PCWSTR(value_name.as_ptr())) };
    let _ = unsafe { RegCloseKey(handle) };
    if deleted.0 != 0 && deleted.0 != 2 {
        return Err(format!(
            "{key}\\{value} could not be removed ({})",
            deleted.0
        ));
    }
    Ok(())
}

/// Removes exactly what `register` wrote: the ProgId, the capabilities, the registered
/// application, and Recon's entry in each type's "Open with" list. Never the type itself.
pub fn unregister() -> Result<(), String> {
    delete_tree(&format!("Software\\Classes\\{PROG_ID}"))?;
    delete_tree("Software\\Recon")?;
    delete_value("Software\\RegisteredApplications", "Recon")?;
    for ext in EXTENSIONS {
        delete_value(
            &format!("Software\\Classes\\{ext}\\OpenWithProgids"),
            PROG_ID,
        )?;
    }
    Ok(())
}

/// Windows' own Default apps page, where the user makes Recon the default viewer. Recon
/// links there rather than taking the association itself (§3.1).
pub fn open_default_apps_settings() -> Result<(), String> {
    let operation = wide("open");
    let target = wide("ms-settings:defaultapps");
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // Documented: a value above 32 is success.
    if result.0 as usize > 32 {
        Ok(())
    } else {
        Err(format!(
            "the settings page did not open ({})",
            result.0 as usize
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plan_lists_recon_for_every_type_and_takes_no_default() {
        let entries = plan(Path::new("C:\\apps\\recon-host.exe"));
        for ext in EXTENSIONS {
            assert!(entries.iter().any(|e| e.key
                == format!("Software\\Classes\\{ext}\\OpenWithProgids")
                && e.name.as_deref() == Some(PROG_ID)));
            assert!(entries.iter().any(|e| e.key
                == "Software\\Recon\\Capabilities\\FileAssociations"
                && e.name.as_deref() == Some(*ext)
                && e.value == PROG_ID));
            // The extension's own default value is never written.
            assert!(!entries
                .iter()
                .any(|e| e.key == format!("Software\\Classes\\{ext}") && e.name.is_none()));
        }
        let command = entries
            .iter()
            .find(|e| e.key.ends_with("shell\\open\\command"))
            .unwrap();
        assert_eq!(command.value, "\"C:\\apps\\recon-host.exe\" \"%1\"");
        assert!(entries
            .iter()
            .any(|e| e.key == "Software\\RegisteredApplications"
                && e.name.as_deref() == Some("Recon")));
        assert_eq!(EXTENSIONS.len(), 12);
    }
}
