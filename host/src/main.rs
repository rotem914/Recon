// Recon host, step S0.1: a tray entry, a configurable global hotkey, and a clean quit.
//
// What this step exists to prove (project-os/Plan.md, part 7, S0.1):
//   1. the hotkey fires while the process has never shown a window,
//   2. a hotkey another application already holds is reported, not silently lost,
//   3. quit leaves no process behind.
//
// Nothing here captures anything. The freeze, the overlay and the region are S0.2, and
// this file must not grow into them.
//
// Every interesting moment is printed with a millisecond stamp, because the evidence for
// S0.1 is the log and because S0.7's instrumentation will need the same habit.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::RunEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// How many times the hotkey has fired this run. Printed with each fire so a missed
/// keypress and a repeated one look different in the log.
static FIRES: AtomicU32 = AtomicU32::new(0);

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

fn log(line: &str) {
    use std::io::Write;
    println!("{} | {}", now_ms(), line);
    // Flushed on every line: this log is the step's evidence, and a buffered tail that
    // never reaches disk would read exactly like a hotkey that never fired.
    let _ = std::io::stdout().flush();
}

fn main() {
    let cfg = config::load();

    log(&format!("recon-host S0.1, pid {}", std::process::id()));
    log(&format!("hotkey source: {}", cfg.source));
    log(&format!("hotkey wanted: {}", cfg.hotkey));

    let hotkey_label = cfg.hotkey.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let n = FIRES.fetch_add(1, Ordering::SeqCst) + 1;
                        log(&format!(
                            "HOTKEY FIRED: {shortcut} (fire {n}; no window has been created this run)"
                        ));
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // ---- the tray, which is the only user interface this step has ----
            let hotkey_item = MenuItemBuilder::with_id("hotkey", format!("Capture: {hotkey_label}"))
                .enabled(false)
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Recon").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&hotkey_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

            TrayIconBuilder::with_id("recon-tray")
                .icon(icon)
                .tooltip(format!("Recon, capture with {hotkey_label}"))
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        log("quit chosen in the tray menu");
                        app.exit(0);
                    }
                })
                .build(app)?;
            log("tray icon created");

            // ---- the hotkey, and the two ways it can fail to arrive ----
            //
            // Neither failure may kill the process. A capture tool that exits because a
            // shortcut was taken is a capture tool that looks broken for no visible reason,
            // and F15 says the conflict is reported and another key can be chosen.
            match cfg.hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                Ok(shortcut) => match app.global_shortcut().register(shortcut) {
                    Ok(()) => log(&format!("hotkey registered: {}", cfg.hotkey)),
                    Err(err) => log(&format!(
                        "HOTKEY UNAVAILABLE: {} could not be registered ({err}). \
                         Another application is holding it. Recon is still running: \
                         change \"hotkey\" in the config file and restart.",
                        cfg.hotkey
                    )),
                },
                Err(err) => log(&format!(
                    "HOTKEY NOT UNDERSTOOD: {:?} is not a shortcut ({err}). \
                     Recon is still running: fix \"hotkey\" in the config file and restart.",
                    cfg.hotkey
                )),
            }

            log("ready: no window created, waiting on the hotkey or the tray");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("the Recon host failed to build")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { code, api, .. } = event {
                // With no windows, the runtime would otherwise be free to exit on its own.
                // NOT VERIFIED: whether a Windows session logoff arrives here with no code, in
                // which case this would refuse it and hold up shutdown. S0.8 records platform
                // limits; this one belongs in that list rather than in an assumption.
                // An explicit quit carries a code; anything else is refused, so the only
                // way out of this process is the tray, which is what S0.1 has to show.
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
