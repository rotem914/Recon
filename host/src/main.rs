// Recon host: a tray entry, a configurable global hotkey, the capture, and the editor.
//
// Stage 0 built this in four islands (project-os/Plan.md, part 7, S0.1 to S0.4) and S0.4b
// joins them: the hotkey freezes the screen, the overlay picks a region, the one conversion
// turns it into image space, and the editor, created hidden at startup, is handed the pixels
// and shown. Every interesting moment is printed with a millisecond stamp, because the
// evidence for each step is the log and S0.7's instrumentation needs the same habit.
//
// The `stage0-checks` cargo feature carries the diagnostic runs (--selftest, --bench,
// --editor-check, --editor-demo, --capture-demo, --measure) and the commands they need. A product
// build has none of them (review T7).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "stage0-checks")]
mod bench;
mod capture;
mod clipboard;
mod compose;
mod config;
mod editor;
mod focus;
mod marks;
#[cfg(feature = "stage0-checks")]
mod measure;
mod overlay;
mod platform;
#[cfg(feature = "stage0-checks")]
mod selftest;
mod source;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use capture::coords::FrameGeometry;
use capture::{desktop_to_image, CaptureSource, Frame};
use overlay::Outcome;

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, RunEvent, WindowEvent};
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

/// Whether the capture hotkey was registered, so a measurement run never presses a key that
/// would reach another application instead (S0.7).
static HOTKEY_REGISTERED: AtomicBool = AtomicBool::new(false);

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

pub(crate) fn log(line: &str) {
    use std::io::Write;
    println!("{} | {}", now_ms(), line);
    // Flushed on every line: this log is the step's evidence, and a buffered tail that
    // never reaches disk would read exactly like a hotkey that never fired.
    let _ = std::io::stdout().flush();
}

/// The hotkey path: freeze, choose a rectangle, convert once, crop, present.
///
/// Runs on its own thread because the overlay owns a message pump of its own (part 5), and
/// because the hotkey handler must return immediately: a handler that blocks is a hotkey
/// that stops arriving.
fn begin_capture() {
    // S0.7's first mark, taken before anything else on this path.
    let received = std::time::Instant::now();
    let Some(guard) = Selecting::acquire() else {
        log("ignored: a selection is already on screen");
        return;
    };
    marks::begin(received);
    // The application the user is in right now is where the focus goes back to when the
    // editor hides (§3.1). Read here, before the overlay takes the foreground.
    match focus::remember_foreground() {
        Some(hwnd) => log(&format!(
            "return target: {}",
            platform::window_owner(hwnd).line()
        )),
        None => log("return target: unchanged, the capture began inside Recon"),
    }

    std::thread::spawn(move || {
        // Moved into the thread so it is released when this closure ends, whether that is a
        // return, an error path, or a panic unwinding out of the overlay.
        let _guard = guard;
        let source = capture::screen::WholeVirtualScreen;
        let started = std::time::Instant::now();

        match source.freeze() {
            Ok(frame) => {
                marks::mark(marks::FREEZE_DONE);
                // v1 does not capture HDR: what GDI hands over from a display with HDR on
                // is Windows' SDR rendering of it, so the fact is said, never silent (S0.8).
                // Asked on its own thread: the display query is a few milliseconds, and
                // this thread is inside the hotkey-to-overlay interval (review R12).
                std::thread::spawn(|| {
                    let hdr = platform::hdr_on();
                    if !hdr.is_empty() {
                        log(&format!(
                            "HDR is on for {}: the frozen pixels are the SDR view Windows gives GDI, which v1 keeps as is",
                            hdr.join(", ")
                        ));
                    }
                });
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
                        let selected_at = std::time::Instant::now();
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
                                    // Not while measuring: the product writes nothing here, and a
                                    // 5120-wide PNG encode would sit inside the editor interval (S0.7).
                                    #[cfg(feature = "stage0-checks")]
                                    if !measuring() {
                                        write_capture(image_rect.width, image_rect.height, &pixels);
                                    }
                                    // The capture becomes an image of its own. Its geometry
                                    // keeps where it came from for the log only; nothing
                                    // downstream reads a desktop coordinate (part 5).
                                    let captured = Frame {
                                        geometry: FrameGeometry {
                                            origin_x: rect.x,
                                            origin_y: rect.y,
                                            width: image_rect.width,
                                            height: image_rect.height,
                                        },
                                        rgba: pixels,
                                        source: "capture",
                                    };
                                    match editor::present(captured) {
                                        Ok(show_ms) => {
                                            marks::mark(marks::EDITOR_SHOW_RETURNED);
                                            log(&format!(
                                                "editor shown: {} ms from selection, the show itself {show_ms} ms",
                                                selected_at.elapsed().as_millis()
                                            ))
                                        }
                                        Err(err) => log(&format!("EDITOR NOT SHOWN: {err}")),
                                    }
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
/// Diagnostic output, never the store: the managed document arrives at S1.8, and nothing
/// here writes anywhere near a file the user owns.
#[cfg(feature = "stage0-checks")]
fn write_capture(width: u32, height: u32, pixels: &[u8]) {
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
    match image::RgbaImage::from_raw(width, height, pixels.to_vec()) {
        Some(buffer) => match buffer.save(&path) {
            Ok(()) => log(&format!("capture written: {}", path.display())),
            Err(err) => log(&format!("capture not written: {err}")),
        },
        None => log("capture not written: the pixel buffer did not match its dimensions"),
    }
}

/// Opens the editor with its own checks, or with a demo scene, which is S0.4's evidence.
///
/// The window is created hidden exactly as the product would, so the first-frame check is
/// testing the real thing rather than a window made visible for the occasion.
#[cfg(feature = "stage0-checks")]
fn editor_run(demo: bool) -> i32 {
    let builder = editor::with_editor(tauri::Builder::default()).setup(move |app| {
        if demo {
            editor::request_demo();
        } else {
            editor::request_checks();
        }
        editor::set_app(app.handle().clone());
        editor::set_hotkey(config::load().hotkey);
        tauri::WebviewWindowBuilder::new(
            app,
            "editor",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Recon")
        .inner_size(1280.0, 800.0)
        .visible(demo)
        .build()?;
        Ok(())
    });

    match builder.build(tauri::generate_context!()) {
        Ok(app) => {
            app.run(|_app, _event| {});
            0
        }
        Err(err) => {
            println!("the editor could not be built: {err}");
            1
        }
    }
}

/// The value after a flag, as in `--open C:\\pictures\\a.png` or `--open=...`.
fn arg_value(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_string());
        }
        if arg == flag {
            return args.next();
        }
    }
    None
}

/// Whether this run should fire a capture on its own, drive the overlay with synthesized
/// input, and screenshot the editor that results. The product path, minus the keypress.
#[cfg(feature = "stage0-checks")]
static CAPTURE_DEMO: AtomicBool = AtomicBool::new(false);

/// A file to open, screenshot, step through and screenshot again, then exit.
#[cfg(feature = "stage0-checks")]
static OPEN_DEMO: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// A measurement run: how many captures, and the folder to walk afterwards (S0.7).
#[cfg(feature = "stage0-checks")]
static MEASURE: std::sync::Mutex<Option<(usize, Option<String>)>> = std::sync::Mutex::new(None);

/// Whether a measurement is running, so the checks build's page does not load its probe
/// image and put a picture into the memory floor that the product would never have.
#[cfg(feature = "stage0-checks")]
pub fn measuring() -> bool {
    MEASURE.lock().map(|slot| slot.is_some()).unwrap_or(false)
}

fn main() {
    // Before anything else, and before any window or device context exists.
    let awareness = capture::display::make_per_monitor_aware();
    // Startup is measured from here (S0.7); the operating system's time before main is not.
    marks::process_started();

    #[cfg(feature = "stage0-checks")]
    {
        if std::env::args().any(|a| a == "--editor-check") {
            println!("dpi at startup  : {awareness}");
            std::process::exit(editor_run(false));
        }
        if std::env::args().any(|a| a == "--editor-demo") {
            println!("dpi at startup  : {awareness}");
            std::process::exit(editor_run(true));
        }
        if std::env::args().any(|a| a == "--bench") {
            println!("dpi at startup  : {awareness}");
            std::process::exit(bench::run());
        }
        if std::env::args().any(|a| a == "--stand-in-window") {
            std::process::exit(selftest::stand_in_window());
        }
        if std::env::args().any(|a| a == "--platform-report") {
            println!("dpi at startup  : {awareness}");
            println!(
                "this process    : {}",
                match platform::this_process_is_elevated() {
                    Some(true) => "elevated",
                    Some(false) => "not elevated",
                    None => "elevation unknown",
                }
            );
            println!(
                "web view        : WebView2 {}",
                tauri::webview_version().unwrap_or_else(|_| "unknown".into())
            );
            for m in capture::display::monitors() {
                println!(
                    "display         : {} at {},{} {}x{}, scale {}%{}",
                    m.device,
                    m.rect.x,
                    m.rect.y,
                    m.rect.width,
                    m.rect.height,
                    m.scale_percent,
                    if m.primary { ", primary" } else { "" }
                );
            }
            for d in platform::displays() {
                println!("colour          : {}", d.line());
            }
            std::process::exit(0);
        }
        if std::env::args().any(|a| a == "--selftest") {
            println!("dpi at startup  : {awareness}");
            let failures = selftest::run();
            std::process::exit(if failures == 0 { 0 } else { 1 });
        }
        if std::env::args().any(|a| a == "--capture-demo") {
            CAPTURE_DEMO.store(true, Ordering::SeqCst);
        }
        if let Some(path) = arg_value("--open-demo") {
            *OPEN_DEMO.lock().expect("the open-demo slot") = Some(path);
        }
        if let Some(runs) = arg_value("--measure") {
            *MEASURE.lock().expect("the measure slot") =
                Some((runs.parse().unwrap_or(30), arg_value("--walk")));
        }
        if let Some(dir) = arg_value("--make-fixtures") {
            println!("dpi at startup  : {awareness}");
            let code = match source::fixtures::make(std::path::Path::new(&dir)) {
                Ok(()) => 0,
                Err(err) => {
                    println!("fixtures FAILED: {err}");
                    1
                }
            };
            std::process::exit(code);
        }
        if let Some(dir) = arg_value("--decode-report") {
            println!("dpi at startup  : {awareness}");
            std::process::exit(source::report::run(std::path::Path::new(&dir)));
        }
    }

    // A file to open into the editor at startup, which is how S0.5 shows an opened file on
    // the same canvas a capture uses. The product's own file activation is S1.2.
    let open_at_start = arg_value("--open");

    let cfg = config::load();

    log(&format!("recon-host, pid {}", std::process::id()));
    log(&format!("dpi awareness: {awareness}"));
    for d in platform::displays() {
        log(&format!("display colour: {}", d.line()));
    }
    log(&format!("hotkey source: {}", cfg.source));
    log(&format!("hotkey wanted: {}", cfg.hotkey));

    let hotkey_label = cfg.hotkey.clone();

    let builder = tauri::Builder::default().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|_app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let n = FIRES.fetch_add(1, Ordering::SeqCst) + 1;
                    log(&format!("HOTKEY FIRED: {shortcut} (fire {n})"));
                    begin_capture();
                }
            })
            .build(),
    );

    editor::with_editor(builder)
        .setup(move |app| {
            // ---- the tray ----
            let hotkey_item =
                MenuItemBuilder::with_id("hotkey", format!("Capture: {hotkey_label}"))
                    .enabled(false)
                    .build(app)?;
            let open_item = MenuItemBuilder::with_id("open", "Open Recon").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Recon").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open_item)
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
                    } else if event.id() == "open" {
                        // The editor as it is: the most recent document, or the empty state
                        // naming the hotkey (§3.1).
                        match editor::show(app) {
                            Ok(ms) => log(&format!("editor opened from the tray in {ms} ms")),
                            Err(err) => log(&format!("EDITOR NOT SHOWN from the tray: {err}")),
                        }
                    }
                })
                .build(app)?;
            log("tray icon created");

            // ---- the editor, created hidden so a capture only has to show it ----
            editor::set_app(app.handle().clone());
            editor::set_hotkey(hotkey_label.clone());
            let created = std::time::Instant::now();
            editor::create_hidden(app.handle())?;
            log(&format!(
                "editor window created hidden in {} ms",
                created.elapsed().as_millis()
            ));

            // ---- the hotkey, and the two ways it can fail to arrive ----
            //
            // Neither failure may kill the process. A capture tool that exits because a
            // shortcut was taken is a capture tool that looks broken for no visible reason,
            // and F15 says the conflict is reported and another key can be chosen.
            match cfg.hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                Ok(shortcut) => match app.global_shortcut().register(shortcut) {
                    Ok(()) => {
                        HOTKEY_REGISTERED.store(true, Ordering::SeqCst);
                        log(&format!("hotkey registered: {}", cfg.hotkey))
                    }
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

            log("ready: waiting on the hotkey or the tray");
            marks::startup(marks::READY);

            if let Some(path) = open_at_start.clone() {
                std::thread::spawn(move || {
                    let started = std::time::Instant::now();
                    match editor::open_path(std::path::Path::new(&path)) {
                        Ok(show_ms) => log(&format!(
                            "opened {path}: editor shown {} ms after the open began, the show itself {show_ms} ms",
                            started.elapsed().as_millis()
                        )),
                        Err(err) => log(&format!("OPEN FAILED for {path}: {err}")),
                    }
                });
            }

            #[cfg(feature = "stage0-checks")]
            if CAPTURE_DEMO.load(Ordering::SeqCst) {
                let handle = app.handle().clone();
                std::thread::spawn(move || selftest::capture_demo(handle, begin_capture));
            }
            #[cfg(feature = "stage0-checks")]
            if let Some(path) = OPEN_DEMO.lock().ok().and_then(|slot| slot.clone()) {
                let handle = app.handle().clone();
                std::thread::spawn(move || selftest::open_demo(handle, path));
            }
            #[cfg(feature = "stage0-checks")]
            if let Some((runs, walk)) = MEASURE.lock().ok().and_then(|slot| slot.clone()) {
                let handle = app.handle().clone();
                std::thread::spawn(move || measure::run(handle, runs, walk));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the editor hides it (§3.1): the window is expensive to create and the
            // work in it must survive. Quit is the tray's, and it is explicit.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                match editor::hide(window.app_handle()) {
                    Ok(()) => {}
                    Err(err) => log(&format!("editor NOT hidden on close: {err}")),
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("the Recon host failed to build")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { code, api, .. } = event {
                // With no visible window, the runtime would otherwise be free to exit on its
                // own. NOT VERIFIED: whether a Windows session logoff arrives here with no
                // code, in which case this would refuse it and hold up shutdown. S0.8
                // records platform limits; this one belongs in that list rather than in an
                // assumption. An explicit quit carries a code; anything else is refused, so
                // the only way out of this process is the tray.
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
