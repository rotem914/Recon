// The capture hotkey is configuration, not a constant.
//
// S0.1 has to show two things about it: that it can be changed without a rebuild, and
// that a value which is malformed or already taken is REPORTED rather than swallowed
// (project-os/Plan.md, S0.1 and F15). So every path below produces a hotkey plus a
// sentence saying where it came from, and the caller logs that sentence.

use std::path::PathBuf;
use std::sync::Mutex;

/// Chosen because it is short, close to the keys used for region capture elsewhere, and
/// not a documented Windows shortcut. It is a default, not a decision: S1 asks Rotem.
pub const DEFAULT_HOTKEY: &str = "Ctrl+Shift+4";

pub struct Config {
    pub hotkey: String,
    /// The shortcut that shows Recon's window, from anywhere. None until one is picked in
    /// Settings: a second global shortcut is a second thing to collide with, so it is never
    /// taken by default (the assistant's default, 2026-09-18, Rotem's to veto).
    pub open_hotkey: Option<String>,
    /// A second shortcut that starts a capture, beside the first (Rotem, 2026-09-19). None
    /// until one is picked in Settings, for the same reason as the open shortcut.
    pub second_hotkey: Option<String>,
    /// Whether a capture takes the mouse pointer with it, as an element on top (Rotem,
    /// 2026-09-21). On unless the file says `"capture_pointer": false`.
    pub capture_pointer: bool,
    /// Where the value came from, in words, for the log.
    pub source: String,
}

/// `--hotkey=<shortcut>` overrides the file. It exists so the conflict path in S0.1 can be
/// exercised by starting a second instance, without editing anyone's real config.
pub fn load() -> Config {
    if let Some(hotkey) = arg_override() {
        return Config {
            source: format!("--hotkey on the command line: {hotkey}"),
            hotkey,
            open_hotkey: file_value().as_ref().and_then(open_of),
            second_hotkey: file_value().as_ref().and_then(second_of),
            capture_pointer: file_value().as_ref().is_none_or(pointer_of),
        };
    }

    let Some(path) = config_path() else {
        return Config {
            hotkey: DEFAULT_HOTKEY.into(),
            open_hotkey: None,
            second_hotkey: None,
            capture_pointer: true,
            source: "no application data directory, so the built-in default".into(),
        };
    };

    if !path.exists() {
        return Config {
            hotkey: DEFAULT_HOTKEY.into(),
            open_hotkey: None,
            second_hotkey: None,
            capture_pointer: true,
            source: format!(
                "{} does not exist yet, so the built-in default",
                path.display()
            ),
        };
    }

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            return Config {
                hotkey: DEFAULT_HOTKEY.into(),
                open_hotkey: None,
                second_hotkey: None,
                capture_pointer: true,
                source: format!(
                    "{} could not be read ({err}), so the built-in default",
                    path.display()
                ),
            }
        }
    };

    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => match value.get("hotkey").and_then(serde_json::Value::as_str) {
            Some(hotkey) => Config {
                hotkey: hotkey.to_string(),
                open_hotkey: open_of(&value),
                second_hotkey: second_of(&value),
                capture_pointer: pointer_of(&value),
                source: format!("{}", path.display()),
            },
            None => Config {
                hotkey: DEFAULT_HOTKEY.into(),
                open_hotkey: open_of(&value),
                second_hotkey: second_of(&value),
                capture_pointer: pointer_of(&value),
                source: format!(
                    "{} has no \"hotkey\" key, so the built-in default",
                    path.display()
                ),
            },
        },
        Err(err) => Config {
            hotkey: DEFAULT_HOTKEY.into(),
            open_hotkey: None,
            second_hotkey: None,
            capture_pointer: true,
            source: format!(
                "{} is not valid JSON ({err}), so the built-in default",
                path.display()
            ),
        },
    }
}

/// `%APPDATA%\Recon\recon.json`. Resolved without a Tauri handle so the path can be logged
/// before the application is built, which is where a startup failure would otherwise hide.
/// A check run points it beside its executable first, so it never touches Rotem's own file.
fn config_path() -> Option<PathBuf> {
    if let Some(path) = PATH_OVERRIDE.lock().ok().and_then(|slot| slot.clone()) {
        return Some(path);
    }
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("Recon").join("recon.json"))
}

static PATH_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// The checks' and the tests' own file, in place of `%APPDATA%\Recon\recon.json`.
#[cfg(any(test, feature = "stage0-checks"))]
pub fn set_path(path: PathBuf) {
    if let Ok(mut slot) = PATH_OVERRIDE.lock() {
        *slot = Some(path);
    }
}

/// The file's text, for the checks.
#[cfg(feature = "stage0-checks")]
pub fn file_text() -> Option<String> {
    std::fs::read_to_string(config_path()?).ok()
}

fn file_value() -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(config_path()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn open_of(value: &serde_json::Value) -> Option<String> {
    named(value, "open_hotkey")
}

fn second_of(value: &serde_json::Value) -> Option<String> {
    named(value, "second_hotkey")
}

fn pointer_of(value: &serde_json::Value) -> bool {
    value
        .get("capture_pointer")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
}

fn named(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

/// Writes the three shortcuts Settings chose into the config file, keeping every other key
/// the file already holds. A file that is not a JSON object is replaced, since nothing in
/// it could be read anyway. Written beside itself and renamed, so a failure half way
/// leaves the old file whole.
pub fn save(
    hotkey: &str,
    second_hotkey: Option<&str>,
    open_hotkey: Option<&str>,
) -> Result<PathBuf, String> {
    let path = config_path().ok_or("there is no application data folder to save into")?;
    let mut object = match file_value() {
        Some(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    object.insert("hotkey".into(), hotkey.into());
    for (key, value) in [
        ("second_hotkey", second_hotkey),
        ("open_hotkey", open_hotkey),
    ] {
        match value {
            Some(value) => object.insert(key.into(), value.into()),
            None => object.remove(key),
        };
    }
    write_object(path, object)
}

/// Writes Settings' switch for the mouse pointer into the config file, every other key kept.
pub fn save_pointer(on: bool) -> Result<PathBuf, String> {
    let path = config_path().ok_or("there is no application data folder to save into")?;
    let mut object = match file_value() {
        Some(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    object.insert("capture_pointer".into(), on.into());
    write_object(path, object)
}

fn write_object(
    path: PathBuf,
    object: serde_json::Map<String, serde_json::Value>,
) -> Result<PathBuf, String> {
    let text = serde_json::to_string_pretty(&serde_json::Value::Object(object))
        .map_err(|err| err.to_string())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|err| format!("{} could not be created: {err}", dir.display()))?;
    }
    let beside = path.with_extension("json.new");
    // Flushed before it is named: a rename of unflushed bytes can survive a power cut as
    // an empty file, and an empty settings file is the default shortcut back, silently.
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&beside)
            .map_err(|err| format!("{} could not be written: {err}", beside.display()))?;
        file.write_all(text.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|err| format!("{} could not be written: {err}", beside.display()))?;
    }
    std::fs::rename(&beside, &path)
        .map_err(|err| format!("{} could not be replaced: {err}", path.display()))?;
    Ok(path)
}

fn arg_override() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--hotkey=") {
            return Some(value.to_string());
        }
        if arg == "--hotkey" {
            return args.next();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One test, since the path override is one static: a save keeps the keys it does not
    /// own, a load reads all three shortcuts back, and clearing one removes its key.
    #[test]
    fn save_keeps_other_keys_and_load_reads_both_back() {
        // Beside the test executable, inside the project, never the system temp folder.
        let dir = std::env::current_exe()
            .unwrap()
            .with_file_name(format!("recon-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("recon.json");
        std::fs::write(&path, r#"{"hotkey":"Ctrl+Shift+4","kept":7}"#).unwrap();
        set_path(path.clone());

        save("Ctrl+Alt+S", Some("PrintScreen"), Some("Ctrl+Alt+R")).unwrap();
        let cfg = load();
        assert_eq!(cfg.hotkey, "Ctrl+Alt+S");
        assert_eq!(cfg.second_hotkey.as_deref(), Some("PrintScreen"));
        assert_eq!(cfg.open_hotkey.as_deref(), Some("Ctrl+Alt+R"));
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["kept"], 7);

        save("Ctrl+Alt+S", None, None).unwrap();
        assert_eq!(load().open_hotkey, None);
        assert_eq!(load().second_hotkey, None);
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(value.get("open_hotkey").is_none());
        assert!(value.get("second_hotkey").is_none());
        assert_eq!(value["kept"], 7);

        // The pointer switch: on when the file says nothing, kept across a shortcut save.
        assert!(load().capture_pointer);
        save_pointer(false).unwrap();
        assert!(!load().capture_pointer);
        save("Ctrl+Alt+S", None, None).unwrap();
        assert!(!load().capture_pointer);
        assert_eq!(load().hotkey, "Ctrl+Alt+S");
        save_pointer(true).unwrap();
        assert!(load().capture_pointer);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
