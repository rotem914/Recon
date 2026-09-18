//! Settings: the global shortcuts, chosen in the editor's Settings and swapped live.
//!
//! Capture has always had one (`config.rs`); a second capture shortcut sits beside it
//! (Rotem, 2026-09-19), and another shows Recon's window from anywhere; both are empty
//! until they are picked (Rotem, 2026-09-18). A change is tried before it is kept: the
//! new shortcut is registered first, and only when Windows gives it to Recon is the old
//! one let go and the file written. A shortcut another application holds
//! is refused in words and the old one keeps working, which is what §3.1 of the plan asks:
//! explain the conflict, allow another, never take a shortcut over silently.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::menu::MenuItem;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::config;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Which {
    Capture,
    /// The second capture shortcut, which starts the same capture as the first.
    Capture2,
    Open,
}

impl Which {
    const ALL: [Which; 3] = [Which::Capture, Which::Capture2, Which::Open];

    fn name(self) -> &'static str {
        match self {
            Which::Capture => "Capture",
            Which::Capture2 => "Capture 2",
            Which::Open => "Open Recon",
        }
    }
}

/// One shortcut as Recon holds it: the words Rotem sees, and whether Windows gave it.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Slot {
    /// As shown and as saved, "Ctrl+Shift+4". Empty means none.
    pub label: String,
    /// False when it is empty, malformed, or held by another application.
    pub active: bool,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Keys {
    pub capture: Slot,
    pub capture2: Slot,
    pub open: Slot,
}

impl Keys {
    fn slot(&mut self, which: Which) -> &mut Slot {
        match which {
            Which::Capture => &mut self.capture,
            Which::Capture2 => &mut self.capture2,
            Which::Open => &mut self.open,
        }
    }
    fn get(&self, which: Which) -> &Slot {
        match which {
            Which::Capture => &self.capture,
            Which::Capture2 => &self.capture2,
            Which::Open => &self.open,
        }
    }
}

/// "Win" is what the key is called on the keyboard; the parser knows it as Super.
pub fn parse(label: &str) -> Result<Shortcut, String> {
    let normal = label
        .split('+')
        .map(|part| match part.trim() {
            p if p.eq_ignore_ascii_case("win") => "Super",
            p => p,
        })
        .collect::<Vec<_>>()
        .join("+");
    Shortcut::from_str(&normal).map_err(|err| format!("{label} is not a shortcut ({err})"))
}

/// The change itself, with Windows behind two closures so it can be tested without it.
/// `register` and `unregister` take the label. On any `Err` the keys are as they were.
pub fn change(
    keys: &mut Keys,
    which: Which,
    wanted: Option<&str>,
    register: &mut dyn FnMut(&str) -> Result<(), String>,
    unregister: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let wanted = wanted.map(str::trim).filter(|w| !w.is_empty());
    let Some(wanted) = wanted else {
        if which == Which::Capture {
            return Err("Capture needs a shortcut".into());
        }
        let slot = keys.slot(which);
        if slot.active {
            unregister(&slot.label);
        }
        *slot = Slot::default();
        return Ok(());
    };

    let shortcut = parse(wanted)?;
    for taken_by in Which::ALL.into_iter().filter(|w| *w != which) {
        let other = keys.get(taken_by);
        if !other.label.is_empty() && parse(&other.label).ok() == Some(shortcut) {
            return Err(format!("{wanted} is already {}", taken_by.name()));
        }
    }

    let current = keys.slot(which).clone();
    let same = !current.label.is_empty() && parse(&current.label).ok() == Some(shortcut);
    if same && current.active {
        keys.slot(which).label = wanted.to_string();
        return Ok(());
    }

    register(wanted)
        .map_err(|_| format!("{wanted} is taken by another application. Pick another."))?;
    if current.active && !same {
        unregister(&current.label);
    }
    *keys.slot(which) = Slot {
        label: wanted.to_string(),
        active: true,
    };
    Ok(())
}

/// What a press of the open shortcut does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenPress {
    Show,
    Minimize,
}

/// The second press minimizes (Rotem, 2026-09-18), and only when Recon is the window in
/// front: hidden in the tray, minimized, or open behind another application, a press still
/// brings it forward, since that is what the shortcut is for.
pub fn open_press(visible: bool, minimized: bool, focused: bool) -> OpenPress {
    if visible && !minimized && focused {
        OpenPress::Minimize
    } else {
        OpenPress::Show
    }
}

static KEYS: Mutex<Keys> = Mutex::new(Keys {
    capture: Slot {
        label: String::new(),
        active: false,
    },
    capture2: Slot {
        label: String::new(),
        active: false,
    },
    open: Slot {
        label: String::new(),
        active: false,
    },
});

/// Whether the shortcut plugin is in this process. A check run has none, and there a
/// registration is the parse alone, plus whatever the checks declared taken.
static LIVE: AtomicBool = AtomicBool::new(false);
/// Held while the page records a shortcut, so pressing the current one picks it instead of
/// starting a capture.
static RECORDING: AtomicBool = AtomicBool::new(false);
static TRAY_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);
#[cfg(feature = "stage0-checks")]
static PRETEND_TAKEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn set_tray_item(item: MenuItem<tauri::Wry>) {
    if let Ok(mut slot) = TRAY_ITEM.lock() {
        *slot = Some(item);
    }
}

fn register_live(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    let shortcut = parse(label)?;
    #[cfg(feature = "stage0-checks")]
    if PRETEND_TAKEN
        .lock()
        .map(|taken| taken.iter().any(|t| parse(t).ok() == Some(shortcut)))
        .unwrap_or(false)
    {
        return Err("declared taken by the checks".into());
    }
    if !LIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    app.global_shortcut()
        .register(shortcut)
        .map_err(|err| err.to_string())
}

fn unregister_live(app: &tauri::AppHandle, label: &str) {
    if !LIVE.load(Ordering::SeqCst) {
        return;
    }
    if let Ok(shortcut) = parse(label) {
        if let Err(err) = app.global_shortcut().unregister(shortcut) {
            crate::log(&format!("settings: {label} could not be let go ({err})"));
        }
    }
}

/// At startup, once: registers what the config holds and remembers what Windows gave.
/// Neither failure ends the process (F15); Settings shows it and offers another.
pub fn start(app: &tauri::AppHandle, cfg: &config::Config, live: bool) -> Keys {
    LIVE.store(live, Ordering::SeqCst);
    let mut keys = Keys::default();
    for (which, label) in [
        (Which::Capture, Some(cfg.hotkey.clone())),
        (Which::Capture2, cfg.second_hotkey.clone()),
        (Which::Open, cfg.open_hotkey.clone()),
    ] {
        let Some(label) = label else { continue };
        // A file edited by hand can name one shortcut twice; the later one is dropped, so
        // Settings shows it empty rather than taken by another application.
        let shortcut = parse(&label).ok();
        let twice = Which::ALL
            .into_iter()
            .take_while(|&w| w != which)
            .find(|&w| shortcut.is_some() && parse(&keys.get(w).label).ok() == shortcut);
        if let Some(first) = twice {
            crate::log(&format!(
                "hotkey {label} for {} is already {}, so it is left empty",
                which.name(),
                first.name()
            ));
            continue;
        }
        let active = match register_live(app, &label) {
            Ok(()) => {
                crate::log(&format!("hotkey registered: {label} ({})", which.name()));
                true
            }
            Err(err) => {
                crate::log(&format!(
                    "HOTKEY UNAVAILABLE: {label} for {} could not be registered ({err}). \
                     Recon is still running: pick another in Settings.",
                    which.name()
                ));
                false
            }
        };
        *keys.slot(which) = Slot { label, active };
    }
    if let Ok(mut held) = KEYS.lock() {
        *held = keys.clone();
    }
    keys
}

/// What a fired shortcut is for. None while a shortcut is being recorded, and for one
/// Recon does not know.
pub fn fired(shortcut: &Shortcut) -> Option<Which> {
    if RECORDING.load(Ordering::SeqCst) {
        return None;
    }
    let keys = KEYS.lock().ok()?;
    purpose(&keys, shortcut)
}

fn purpose(keys: &Keys, shortcut: &Shortcut) -> Option<Which> {
    Which::ALL.into_iter().find(|&which| {
        let slot = keys.get(which);
        slot.active && parse(&slot.label).ok().as_ref() == Some(shortcut)
    })
}

fn held_keys() -> Result<Keys, String> {
    KEYS.lock()
        .map(|keys| keys.clone())
        .map_err(|_| "the settings are locked".to_string())
}

fn store_keys(keys: Keys) {
    if let Ok(mut held) = KEYS.lock() {
        *held = keys;
    }
}

#[derive(serde::Serialize)]
pub struct SettingsView {
    capture: String,
    capture_active: bool,
    capture2: String,
    capture2_active: bool,
    open: String,
    open_active: bool,
}

fn view(keys: &Keys) -> SettingsView {
    SettingsView {
        capture: keys.capture.label.clone(),
        capture_active: keys.capture.active,
        capture2: keys.capture2.label.clone(),
        capture2_active: keys.capture2.active,
        open: keys.open.label.clone(),
        open_active: keys.open.active,
    }
}

#[tauri::command]
pub fn editor_settings() -> Result<SettingsView, String> {
    KEYS.lock()
        .map(|keys| view(&keys))
        .map_err(|_| "the settings are locked".to_string())
}

/// `which` is "capture", "capture2" or "open"; an empty `shortcut` clears, which the first
/// capture shortcut never allows.
#[tauri::command]
pub fn editor_settings_set(
    app: tauri::AppHandle,
    which: String,
    shortcut: String,
) -> Result<SettingsView, String> {
    let which = match which.as_str() {
        "capture" => Which::Capture,
        "capture2" => Which::Capture2,
        "open" => Which::Open,
        other => return Err(format!("{other} is not a setting")),
    };
    // Never held across a registration: the plugin finishes one on the main thread, where
    // a fired shortcut asks `fired` for this same lock.
    let before = held_keys()?;
    let mut keys = before.clone();
    change(
        &mut keys,
        which,
        Some(shortcut.as_str()),
        &mut |label| register_live(&app, label),
        &mut |label| unregister_live(&app, label),
    )?;

    // The file last: a shortcut that works and was not saved is said so, and put back.
    let open = (!keys.open.label.is_empty()).then_some(keys.open.label.as_str());
    let second = (!keys.capture2.label.is_empty()).then_some(keys.capture2.label.as_str());
    if let Err(err) = config::save(&keys.capture.label, second, open) {
        let mut back = keys.clone();
        let wanted = before.get(which).label.clone();
        let _ = change(
            &mut back,
            which,
            Some(wanted.as_str()),
            &mut |label| register_live(&app, label),
            &mut |label| unregister_live(&app, label),
        );
        store_keys(back);
        return Err(format!("NOT SAVED: {err}"));
    }
    crate::log(&format!(
        "settings: {} is {}",
        which.name(),
        if shortcut.trim().is_empty() {
            "none"
        } else {
            shortcut.trim()
        }
    ));
    store_keys(keys.clone());

    if which == Which::Capture {
        crate::editor::set_hotkey(keys.capture.label.clone());
        if let Ok(item) = TRAY_ITEM.lock() {
            if let Some(item) = item.as_ref() {
                let _ = item.set_text(format!("Capture: {}", keys.capture.label));
            }
        }
    }
    Ok(view(&keys))
}

/// On while the page waits for a shortcut to be pressed: the global shortcuts are let
/// go, so the keys reach the page and pressing the current one picks it rather than firing
/// it. Off puts them back.
#[tauri::command]
pub fn editor_settings_recording(app: tauri::AppHandle, on: bool) -> Result<(), String> {
    if RECORDING.swap(on, Ordering::SeqCst) == on {
        return Ok(());
    }
    let mut keys = held_keys()?;
    for which in Which::ALL {
        let slot = keys.slot(which);
        if slot.label.is_empty() {
            continue;
        }
        if on {
            if slot.active {
                unregister_live(&app, &slot.label);
            }
        } else if slot.active {
            if let Err(err) = register_live(&app, &slot.label) {
                slot.active = false;
                crate::log(&format!(
                    "HOTKEY UNAVAILABLE: {} did not come back after recording ({err})",
                    slot.label
                ));
            }
        }
    }
    store_keys(keys);
    Ok(())
}

/// Checks only: shortcuts a registration refuses, as if another application held them.
#[cfg(feature = "stage0-checks")]
#[tauri::command]
pub fn editor_settings_pretend_taken(shortcuts: Vec<String>) {
    if let Ok(mut taken) = PRETEND_TAKEN.lock() {
        *taken = shortcuts;
    }
}

/// Checks only: the settings file as it is on disk, so a check reads what a restart would.
#[cfg(feature = "stage0-checks")]
#[tauri::command]
pub fn editor_settings_file() -> String {
    config::file_text().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(capture: &str, open: &str) -> Keys {
        keys3(capture, "", open)
    }

    fn keys3(capture: &str, capture2: &str, open: &str) -> Keys {
        Keys {
            capture: Slot {
                label: capture.into(),
                active: !capture.is_empty(),
            },
            capture2: Slot {
                label: capture2.into(),
                active: !capture2.is_empty(),
            },
            open: Slot {
                label: open.into(),
                active: !open.is_empty(),
            },
        }
    }

    /// Runs a change against a Windows that holds `taken`, and returns what was
    /// registered and let go, in order.
    fn run(
        keys: &mut Keys,
        which: Which,
        wanted: Option<&str>,
        taken: &[&str],
    ) -> (Result<(), String>, Vec<String>) {
        let log = std::cell::RefCell::new(Vec::new());
        let result = change(
            keys,
            which,
            wanted,
            &mut |label| {
                if taken.contains(&label) {
                    return Err("taken".into());
                }
                log.borrow_mut().push(format!("+{label}"));
                Ok(())
            },
            &mut |label| log.borrow_mut().push(format!("-{label}")),
        );
        (result, log.into_inner())
    }

    #[test]
    fn a_free_shortcut_is_registered_before_the_old_one_is_let_go() {
        let mut k = keys("Ctrl+Shift+4", "");
        let (result, log) = run(&mut k, Which::Capture, Some("Ctrl+Alt+S"), &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["+Ctrl+Alt+S", "-Ctrl+Shift+4"]);
        assert_eq!(k, keys("Ctrl+Alt+S", ""));
    }

    #[test]
    fn a_taken_shortcut_is_refused_and_the_old_one_stays() {
        let mut k = keys("Ctrl+Shift+4", "");
        let (result, log) = run(&mut k, Which::Capture, Some("Ctrl+Alt+S"), &["Ctrl+Alt+S"]);
        assert!(result.unwrap_err().contains("taken by another application"));
        assert!(log.is_empty());
        assert_eq!(k, keys("Ctrl+Shift+4", ""));
    }

    #[test]
    fn the_two_never_share_a_shortcut_however_it_is_spelled() {
        let mut k = keys("Ctrl+Shift+4", "");
        let (result, log) = run(&mut k, Which::Open, Some("shift+ctrl+Digit4"), &[]);
        assert!(result.unwrap_err().contains("already Capture"));
        assert!(log.is_empty());
    }

    #[test]
    fn open_can_be_cleared_and_capture_cannot() {
        let mut k = keys("Ctrl+Shift+4", "Ctrl+Alt+R");
        let (result, log) = run(&mut k, Which::Open, None, &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["-Ctrl+Alt+R"]);
        assert_eq!(k, keys("Ctrl+Shift+4", ""));
        let (result, log) = run(&mut k, Which::Capture, Some("  "), &[]);
        assert!(result.is_err());
        assert!(log.is_empty());
    }

    #[test]
    fn words_that_are_not_a_shortcut_are_refused_and_win_is_understood() {
        let mut k = keys("Ctrl+Shift+4", "");
        let (result, _) = run(&mut k, Which::Open, Some("Ctrl+Banana"), &[]);
        assert!(result.unwrap_err().contains("is not a shortcut"));
        let (result, log) = run(&mut k, Which::Open, Some("Win+Shift+R"), &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["+Win+Shift+R"]);
    }

    #[test]
    fn the_open_shortcut_minimizes_only_the_window_in_front() {
        assert_eq!(open_press(true, false, true), OpenPress::Minimize);
        // In the tray, minimized, or behind another application: brought forward.
        assert_eq!(open_press(false, false, false), OpenPress::Show);
        assert_eq!(open_press(true, true, false), OpenPress::Show);
        assert_eq!(open_press(true, false, false), OpenPress::Show);
    }

    #[test]
    fn the_second_capture_shortcut_is_picked_cleared_and_never_shared() {
        let mut k = keys("Ctrl+Shift+4", "Ctrl+Alt+R");
        let (result, log) = run(&mut k, Which::Capture2, Some("PrintScreen"), &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["+PrintScreen"]);
        assert_eq!(k, keys3("Ctrl+Shift+4", "PrintScreen", "Ctrl+Alt+R"));
        for (which, wanted, taken_by) in [
            (Which::Capture2, "Ctrl+Shift+4", "already Capture"),
            (Which::Capture2, "Ctrl+Alt+R", "already Open Recon"),
            (Which::Capture, "PrintScreen", "already Capture 2"),
            (Which::Open, "PrintScreen", "already Capture 2"),
        ] {
            let (result, log) = run(&mut k, which, Some(wanted), &[]);
            assert!(result.unwrap_err().contains(taken_by));
            assert!(log.is_empty());
        }
        let (result, log) = run(&mut k, Which::Capture2, None, &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["-PrintScreen"]);
        assert_eq!(k, keys("Ctrl+Shift+4", "Ctrl+Alt+R"));
    }

    #[test]
    fn either_capture_shortcut_starts_a_capture() {
        let k = keys3("Ctrl+Shift+4", "PrintScreen", "Ctrl+Alt+R");
        let of = |label: &str| purpose(&k, &parse(label).unwrap());
        assert_eq!(of("Ctrl+Shift+4"), Some(Which::Capture));
        assert_eq!(of("PrintScreen"), Some(Which::Capture2));
        assert_eq!(of("Ctrl+Alt+R"), Some(Which::Open));
        assert_eq!(of("Ctrl+Alt+Q"), None);
    }

    #[test]
    fn a_shortcut_that_was_unavailable_is_tried_again_when_picked_again() {
        let mut k = keys("Ctrl+Shift+4", "");
        k.capture.active = false;
        let (result, log) = run(&mut k, Which::Capture, Some("Ctrl+Shift+4"), &[]);
        assert_eq!(result, Ok(()));
        assert_eq!(log, ["+Ctrl+Shift+4"]);
        assert!(k.capture.active);
    }
}
