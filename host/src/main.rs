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

mod capture;
mod config;
mod overlay;
mod selftest;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use capture::{desktop_to_image, CaptureSource};
use overlay::Outcome;

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::RunEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Held while a selection is on screen, so a second hotkey press is ignored rather than
/// stacking a second overlay on top of the first.
///
/// A plain flag was the first version, and the review caught what it cost: if the selection
/// thread ever panicked, the flag stayed set and the hotkey was dead until a restart. This
/// releases on drop, so the panic path releases it too.
pub struct Selecting;

static SELECTION_ACTIVE: AtomicBool = AtomicBool::new(false);

impl Selecting {
    /// `Some` if no selection was on screen, and nothing else can acquire it until the
    /// returned value is dropped.
    pub fn acquire() -> Option<Selecting> {
        if SELECTION_ACTIVE.swap(true, Ordering::SeqCst) {
            None
        } else {
            Some(Selecting)
        }
    }
}

impl Drop for Selecting {
    fn drop(&mut self) {
        SELECTION_ACTIVE.store(false, Ordering::SeqCst);
    }
}

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

/// The hotkey path: freeze, choose a rectangle, convert once, crop.
///
/// Runs on its own thread because the overlay owns a message pump of its own (part 5), and
/// because the hotkey handler must return immediately: a handler that blocks is a hotkey
/// that stops arriving.
fn begin_capture() {
    let Some(guard) = Selecting::acquire() else {
        log("ignored: a selection is already on screen");
        return;
    };

    std::thread::spawn(move || {
        // Moved into the thread so it is released when this closure ends, whether that is a
        // return, an error path, or a panic unwinding out of the overlay.
        let _guard = guard;
        let source = capture::screen::WholeVirtualScreen;
        let started = std::time::Instant::now();

        match source.freeze() {
            Ok(frame) => {
                log(&format!(
                    "freeze: {}x{} at {},{} in {} ms via {}",
                    frame.width(),
                    frame.height(),
                    frame.geometry.origin_x,
                    frame.geometry.origin_y,
                    started.elapsed().as_millis(),
                    frame.source
                ));

                let shown = std::time::Instant::now();
                match overlay::select_region(&frame) {
                    Outcome::Selected(rect) => {
                        log(&format!(
                            "selected {}x{} at desktop {},{} after {} ms on screen",
                            rect.width,
                            rect.height,
                            rect.x,
                            rect.y,
                            shown.elapsed().as_millis()
                        ));
                        // The one conversion, at the edge, exactly as part 5 requires.
                        match desktop_to_image(frame.geometry, rect) {
                            Ok(image_rect) => match frame.crop(image_rect) {
                                Some(pixels) => {
                                    log(&format!(
                                        "cropped to image space {},{} {}x{}, {} bytes",
                                        image_rect.x,
                                        image_rect.y,
                                        image_rect.width,
                                        image_rect.height,
                                        pixels.len()
                                    ));
                                    write_capture(image_rect.width, image_rect.height, pixels);
                                }
                                None => log("CROP REFUSED: the rectangle is not inside the frame"),
                            },
                            Err(err) => log(&format!(
                                "CONVERSION REFUSED: {err:?}. The rectangle is not inside the frozen frame, \
                                 which means the desktop layout changed under the capture."
                            )),
                        }
                    }
                    Outcome::Cancelled => log("cancelled: nothing captured, nothing replaced"),
                }
            }
            Err(err) => log(&format!("FREEZE FAILED: {err}")),
        }
    });
}

/// Writes the capture beside the executable so a Stage 0 run leaves something to look at.
/// This is diagnostic output, not the store: the managed document arrives at S1.8, and
/// nothing here writes anywhere near a file the user owns.
fn write_capture(width: u32, height: u32, pixels: Vec<u8>) {
    let Ok(exe) = std::env::current_exe() else {
        log("capture not written: the executable path is unknown");
        return;
    };
    let dir = exe.with_file_name("s02-captures");
    if let Err(err) = std::fs::create_dir_all(&dir) {
        log(&format!("capture not written: {err}"));
        return;
    }
    let path = dir.join(format!("capture-{}.png", now_ms()));
    match image::RgbaImage::from_raw(width, height, pixels) {
        Some(buffer) => match buffer.save(&path) {
            Ok(()) => log(&format!("capture written: {}", path.display())),
            Err(err) => log(&format!("capture not written: {err}")),
        },
        None => log("capture not written: the pixel buffer did not match its dimensions"),
    }
}

fn main() {
    // Before anything else, and before any window or device context exists.
    let awareness = capture::display::make_per_monitor_aware();

    if std::env::args().any(|a| a == "--selftest") {
        println!("dpi at startup  : {awareness}");
        let failures = selftest::run();
        std::process::exit(if failures == 0 { 0 } else { 1 });
    }

    let cfg = config::load();

    log(&format!("recon-host, pid {}", std::process::id()));
    log(&format!("dpi awareness: {awareness}"));
    log(&format!("hotkey source: {}", cfg.source));
    log(&format!("hotkey wanted: {}", cfg.hotkey));

    let hotkey_label = cfg.hotkey.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let n = FIRES.fetch_add(1, Ordering::SeqCst) + 1;
                        log(&format!("HOTKEY FIRED: {shortcut} (fire {n})"));
                        begin_capture();
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // ---- the tray, which is the only user interface this step has ----
            let hotkey_item =
                MenuItemBuilder::with_id("hotkey", format!("Capture: {hotkey_label}"))
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
