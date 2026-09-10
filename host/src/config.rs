// The capture hotkey is configuration, not a constant.
//
// S0.1 has to show two things about it: that it can be changed without a rebuild, and
// that a value which is malformed or already taken is REPORTED rather than swallowed
// (project-os/Plan.md, S0.1 and F15). So every path below produces a hotkey plus a
// sentence saying where it came from, and the caller logs that sentence.

use std::path::PathBuf;

/// Chosen because it is short, close to the keys used for region capture elsewhere, and
/// not a documented Windows shortcut. It is a default, not a decision: S1 asks Rotem.
pub const DEFAULT_HOTKEY: &str = "Ctrl+Shift+4";

pub struct Config {
    pub hotkey: String,
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
        };
    }

    let Some(path) = config_path() else {
        return Config {
            hotkey: DEFAULT_HOTKEY.into(),
            source: "no application data directory, so the built-in default".into(),
        };
    };

    if !path.exists() {
        return Config {
            hotkey: DEFAULT_HOTKEY.into(),
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
                source: format!("{}", path.display()),
            },
            None => Config {
                hotkey: DEFAULT_HOTKEY.into(),
                source: format!(
                    "{} has no \"hotkey\" key, so the built-in default",
                    path.display()
                ),
            },
        },
        Err(err) => Config {
            hotkey: DEFAULT_HOTKEY.into(),
            source: format!(
                "{} is not valid JSON ({err}), so the built-in default",
                path.display()
            ),
        },
    }
}

/// `%APPDATA%\Recon\recon.json`. Resolved without a Tauri handle so the path can be logged
/// before the application is built, which is where a startup failure would otherwise hide.
fn config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("Recon").join("recon.json"))
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
