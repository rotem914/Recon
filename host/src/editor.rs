//! The editor window, and the pixels the page is allowed to see.
//!
//! What the host owns here, and nothing more: the window, the image, and the region service.
//! Part 5 pins that the decoded original never crosses at full resolution and never
//! round-trips a canvas, so the page asks for a REGION at a SCALE and the host resamples it.
//! That is the display proxy, and the policy attached to it in part 5 is the one this module
//! has to keep: the visible region always has enough detail for the current zoom, so an
//! enlarged fit-to-window preview is never what a 100% view is made of.
//!
//! The scene, the callout and the text layer live in the page, because part 5 gives the
//! editor's layout and text to the web view. The host never renders an annotation.
//!
//! The window is created HIDDEN at startup and shown on demand, which is what the latency
//! budget needs and also what S0.4's own hidden-window check exists to distrust.
//!
//! Two kinds of command live here, and the `stage0-checks` cargo feature keeps them apart:
//! the product's (the image, the window metrics, the log) and the checks' (a probe image, a
//! screen freeze on demand, a screenshot of the window, a report written to disk). The
//! second kind lets the page ask the host to touch the screen and the disk, which is part
//! 5's boundary broken, so a product build does not compile them (review T7).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use image::ImageEncoder;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, UriSchemeResponder, WebviewUrl, WebviewWindowBuilder};

use crate::capture::coords::{FrameGeometry, ImageRect};
use crate::capture::Frame;
use crate::marks;
use crate::source::{self, Kind, Opened};

// ---------------------------------------------------------------- the image, and its levels

/// One downscaled copy of the image, at 1/2^shift of its size.
///
/// The pyramid is what makes a fit view cheap (F43): the full image is resampled once per
/// level, and a request at a small scale reads from the nearest level at or above that scale
/// instead of from eight million source pixels every time the view moves.
struct Level {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// The image the editor holds, plus the levels built from it so far.
pub struct Image {
    pub frame: Frame,
    levels: Mutex<Vec<Arc<Level>>>,
}

impl Image {
    fn new(frame: Frame) -> Self {
        Self {
            frame,
            levels: Mutex::new(Vec::new()),
        }
    }

    /// The level at 1/2^shift, built on demand from the one above it. Serialised by the
    /// levels lock, and called only from the region worker, so a level is built once.
    fn level(&self, shift: u32) -> Option<Arc<Level>> {
        let mut levels = self.levels.lock().ok()?;
        while levels.len() < shift as usize {
            let (src_w, src_h, src): (u32, u32, &[u8]) = match levels.last() {
                Some(last) => (last.width, last.height, &last.rgba),
                None => (self.frame.width(), self.frame.height(), &self.frame.rgba),
            };
            let (w, h) = recon_pixels::half(src_w, src_h);
            let started = Instant::now();
            let rgba = recon_pixels::resample(src, src_w, src_h, w, h)?;
            let next = levels.len() as u32 + 1;
            println!(
                "level {next}: {w}x{h} built in {} ms",
                started.elapsed().as_millis()
            );
            levels.push(Arc::new(Level {
                width: w,
                height: h,
                rgba,
            }));
        }
        levels.get(shift as usize - 1).cloned()
    }
}

/// An opened file behind the current image: where it came from, and the frame or page
/// on screen, so the page can step through the rest by index (§3.7's addressable frame).
pub struct Document {
    pub opened: Opened,
    pub index: u32,
}

/// What the editor currently holds. One image, because Stage 0 is one image.
pub struct Editor {
    image: Mutex<Option<Arc<Image>>>,
    /// Set when the image came from a file; a capture leaves it empty.
    document: Mutex<Option<Document>>,
}

impl Editor {
    fn new() -> Self {
        Self {
            image: Mutex::new(None),
            document: Mutex::new(None),
        }
    }

    pub fn set_frame(&self, frame: Frame) {
        if let Ok(mut slot) = self.image.lock() {
            *slot = Some(Arc::new(Image::new(frame)));
        }
    }

    fn set_document(&self, document: Option<Document>) {
        if let Ok(mut slot) = self.document.lock() {
            *slot = document;
        }
    }

    fn image(&self) -> Option<Arc<Image>> {
        self.image.lock().ok().and_then(|slot| slot.clone())
    }
}

pub fn state() -> &'static Editor {
    static STATE: OnceLock<Editor> = OnceLock::new();
    STATE.get_or_init(Editor::new)
}

// ---------------------------------------------------------------- the documents

/// The document on screen, by number. Every capture and every opened file gets the next
/// one, so the page can tell a new document from a new frame of the same one, and keep or
/// stash its notes accordingly (S1.1).
static CURRENT_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_document_id() -> u64 {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    CURRENT_ID.store(id, Ordering::SeqCst);
    id
}

pub fn current_document_id() -> u64 {
    CURRENT_ID.load(Ordering::SeqCst)
}

/// A previous capture, kept when the next one arrives. "Taking a new capture saves the
/// current document and adds another; it never overwrites the previous one" (§3.3). The
/// store that keeps it on disk is S1.8; until then it is kept here, PNG-encoded so thirty
/// captures cost about a hundred megabytes rather than a gigabyte, and capped, so a day of
/// use cannot grow without bound (§3.8). The page keeps the notes of each document by the
/// same number. An opened file is not retained: an external file is never taken (§3.8).
#[cfg_attr(not(feature = "stage0-checks"), allow(dead_code))]
pub struct Retained {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

static HISTORY: Mutex<Vec<Retained>> = Mutex::new(Vec::new());

/// How many previous captures are kept in memory until S1.8 moves them to disk.
const RETAINED_LIMIT: usize = 50;

/// Encodes the image on its own thread and files it under its document number, so the
/// capture path pays nothing for it.
fn retain(image: Arc<Image>, id: u64) {
    std::thread::spawn(move || {
        let started = Instant::now();
        let mut png = Vec::new();
        let encoded = image::codecs::png::PngEncoder::new(&mut png).write_image(
            &image.frame.rgba,
            image.frame.width(),
            image.frame.height(),
            image::ExtendedColorType::Rgba8,
        );
        if let Err(err) = encoded {
            println!("document {id} NOT retained: the PNG did not encode: {err}");
            return;
        }
        let Ok(mut history) = HISTORY.lock() else {
            return;
        };
        history.push(Retained {
            id,
            width: image.frame.width(),
            height: image.frame.height(),
            png,
        });
        let mut dropped = None;
        while history.len() > RETAINED_LIMIT {
            dropped = Some(history.remove(0).id);
        }
        let held: usize = history.iter().map(|r| r.png.len()).sum();
        println!(
            "document {id} retained: {}x{} in {} ms, {} kept, {:.1} MB{}",
            image.frame.width(),
            image.frame.height(),
            started.elapsed().as_millis(),
            history.len(),
            held as f64 / (1024.0 * 1024.0),
            match dropped {
                Some(old) => format!(", the oldest ({old}) dropped at the cap of {RETAINED_LIMIT}"),
                None => String::new(),
            }
        );
    });
}

/// The retained captures, for the checks and, at S1.9, for navigation.
#[cfg(feature = "stage0-checks")]
pub fn history_summary() -> Vec<(u64, u32, u32, usize)> {
    HISTORY
        .lock()
        .map(|h| {
            h.iter()
                .map(|r| (r.id, r.width, r.height, r.png.len()))
                .collect()
        })
        .unwrap_or_default()
}

/// The hotkey's label, for the page's empty state.
static HOTKEY_LABEL: Mutex<String> = Mutex::new(String::new());

pub fn set_hotkey(label: String) {
    if let Ok(mut slot) = HOTKEY_LABEL.lock() {
        *slot = label;
    }
}

/// The live image's pixels, for S0.7's memory sample: the frame, and the pyramid levels built
/// from it so far. A document behind a file is not counted; an animation keeps its decoded
/// frames there.
#[cfg(feature = "stage0-checks")]
pub fn live_bytes() -> (usize, usize) {
    match state().image() {
        Some(image) => {
            let levels = image
                .levels
                .lock()
                .map(|l| l.iter().map(|level| level.rgba.len()).sum())
                .unwrap_or(0);
            (image.frame.rgba.len(), levels)
        }
        None => (0, 0),
    }
}

// ---------------------------------------------------------------- the window

/// The application handle, kept so the capture thread can present a frame. Set once, in
/// setup, before any hotkey can fire.
static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app(app: AppHandle) {
    let _ = APP.set(app);
}

fn app() -> Result<&'static AppHandle, String> {
    APP.get()
        .ok_or_else(|| "the editor has no application handle yet".to_string())
}

/// Builds the editor window, hidden.
///
/// Hidden and pre-created is the whole point: showing an existing window is the only way the
/// second latency interval in S0.7 can be met. It is also the thing S0.4 has to distrust,
/// because a hidden window can be throttled by the browser engine and then show a blank or
/// stale first frame, which looks exactly like slowness.
pub fn create_hidden(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .title("Recon")
        .inner_size(1280.0, 800.0)
        .visible(false)
        .build()?;
    Ok(())
}

/// Shows the editor and reports how long the show call itself took.
pub fn show(app: &AppHandle) -> Result<u128, String> {
    let started = Instant::now();
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    Ok(started.elapsed().as_millis())
}

/// Hands a captured frame to the editor and brings the window up.
///
/// This is the join between the capture path and the editor that did not exist until the
/// review (F49): the page is told the image changed, loads it through the region service,
/// and the window is shown. Returns how long the show took.
pub fn present(frame: Frame) -> Result<u128, String> {
    state().set_document(None);
    present_frame(frame)
}

/// Hides the editor and returns the focus to the application the capture began in (§3.1).
pub fn hide(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    window.hide().map_err(|err| err.to_string())?;
    let returned = crate::focus::return_to_target();
    crate::log(&format!("editor hidden; {}", returned.line()));
    Ok(())
}

/// The file's name in the title, so it is visible without hunting (§3.4); a capture is
/// plain "Recon".
fn set_title(app: &AppHandle, info: &ImageInfo) {
    if let Some(window) = app.get_webview_window("editor") {
        let title = if info.file.is_empty() {
            "Recon".to_string()
        } else {
            format!("{} - Recon", info.file)
        };
        let _ = window.set_title(&title);
    }
}

fn present_frame(frame: Frame) -> Result<u128, String> {
    let app = app()?;
    // The previous capture is kept, never overwritten (§3.3), whatever replaces it.
    if let Some(previous) = state().image() {
        if previous.frame.source == "capture" {
            retain(previous, current_document_id());
        }
    }
    next_document_id();
    state().set_frame(frame);
    let info = editor_image_info()?;
    set_title(app, &info);
    app.emit("capture-ready", &info)
        .map_err(|err| err.to_string())?;
    show(app)
}

/// Opens a file through the one image source and shows its first frame or page on the
/// same canvas a capture uses. The file is read once and never written (rule 11).
pub fn open_path(path: &std::path::Path) -> Result<u128, String> {
    let started = Instant::now();
    let mut opened = source::open(path).map_err(|err| err.to_string())?;
    let open_ms = started.elapsed().as_millis();
    let decoded = opened.frame(0).map_err(|err| err.to_string())?;
    println!(
        "opened {}: {} {}x{}, {:?}, open {} ms, frame 0 {} ms, {}, {}",
        path.display(),
        opened.format.name(),
        opened.width,
        opened.height,
        opened.kind,
        open_ms,
        started.elapsed().as_millis() - open_ms,
        opened.notes.color,
        opened.notes.alpha
    );
    let name = opened.format.name();
    state().set_document(Some(Document { opened, index: 0 }));
    present_frame(frame_of(decoded, name))
}

fn frame_of(decoded: source::DecodedFrame, name: &'static str) -> Frame {
    Frame {
        geometry: FrameGeometry {
            origin_x: 0,
            origin_y: 0,
            width: decoded.width,
            height: decoded.height,
        },
        rgba: decoded.rgba,
        source: name,
    }
}

/// The page asks for another frame or page of the opened file, by index.
#[tauri::command]
pub fn editor_frame(index: u32) -> Result<ImageInfo, String> {
    let (decoded, name) = {
        let mut slot = state()
            .document
            .lock()
            .map_err(|_| "the document is poisoned")?;
        let document = slot
            .as_mut()
            .ok_or("the image on screen did not come from a file")?;
        let decoded = document
            .opened
            .frame(index)
            .map_err(|err| err.to_string())?;
        document.index = index;
        (decoded, document.opened.format.name())
    };
    state().set_frame(frame_of(decoded, name));
    let info = editor_image_info()?;
    app()?
        .emit("capture-ready", &info)
        .map_err(|err| err.to_string())?;
    Ok(info)
}

/// Registers everything the editor needs on a builder: the commands, and the region scheme.
///
/// Two command lists, chosen by the `stage0-checks` feature, so the product build has no
/// command that lets the page freeze the screen, load a probe, or write a file.
pub fn with_editor(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    #[cfg(feature = "stage0-checks")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        editor_image_info,
        editor_show,
        editor_window_metrics,
        editor_log,
        editor_frame,
        editor_mark,
        editor_hide,
        editor_hotkey,
        editor_open_dialog,
        editor_fullscreen,
        checks::editor_window_title,
        checks::editor_history,
        checks::editor_dialog_outcome,
        checks::editor_press_escape,
        checks::editor_capture_probe,
        checks::editor_load_probe,
        checks::editor_load_screen,
        checks::editor_look_at_window,
        checks::editor_show_and_look,
        checks::editor_checks_done,
        checks::editor_wants_checks,
        checks::editor_wants_demo,
        checks::editor_shoot_window,
        checks::editor_exit,
        checks::editor_make_fixtures,
        checks::editor_open_fixture,
        checks::editor_window_origin,
        checks::editor_export_check,
        checks::editor_live_compare,
        checks::editor_clipboard_readback,
        checks::editor_environment,
        editor_copy,
        checks::editor_svg_cases,
        checks::editor_svg_compare,
    ]);
    #[cfg(not(feature = "stage0-checks"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        editor_image_info,
        editor_show,
        editor_window_metrics,
        editor_log,
        editor_frame,
        editor_copy,
        editor_mark,
        editor_hide,
        editor_hotkey,
        editor_open_dialog,
        editor_fullscreen,
    ]);
    builder.register_asynchronous_uri_scheme_protocol("region", |_ctx, request, responder| {
        let query = request.uri().query().unwrap_or_default().to_string();
        serve_region(query, responder);
    })
}

// ---------------------------------------------------------------- the product's commands

/// Composes the page's annotation layer over the current image, in canvas space.
///
/// Returns the composite, the decoded layer (the checks count from it) and the time.
fn compose_layer(
    png: &[u8],
    margin: crate::compose::Margin,
) -> Result<(crate::compose::Composite, Vec<u8>, u128), String> {
    let image = state().image().ok_or("no image is loaded")?;
    let (w, h) = (image.frame.width(), image.frame.height());
    let (cw, ch) = margin.canvas(w, h);
    let started = Instant::now();
    let layer = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|err| format!("the layer did not decode: {err}"))?
        .into_rgba8();
    if layer.dimensions() != (cw, ch) {
        return Err(format!(
            "the layer is {}x{} and the canvas is {cw}x{ch}",
            layer.width(),
            layer.height()
        ));
    }
    let layer = layer.into_raw();
    let composite =
        crate::compose::compose(&image.frame.rgba, w, h, &layer, margin, crate::compose::MAT)?;
    Ok((composite, layer, started.elapsed().as_millis()))
}

/// What a copy did, for the page's success line and the log.
#[derive(serde::Serialize)]
pub struct CopyReport {
    pub width: u32,
    pub height: u32,
    pub compose_ms: u128,
    pub published: crate::clipboard::Published,
}

/// Copy the composed image to the clipboard (§3.6). The page sends the annotation layer
/// as a PNG in canvas space with the margin in a header; the host composes it over the
/// untouched source and publishes the result in three formats. The web view never touches
/// the clipboard (part 5).
#[tauri::command]
pub fn editor_copy(request: tauri::ipc::Request<'_>) -> Result<CopyReport, String> {
    let margin = request
        .headers()
        .get("margin")
        .and_then(|v| v.to_str().ok())
        .map(crate::compose::Margin::parse)
        .transpose()?
        .unwrap_or_default();
    let bytes = match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
        tauri::ipc::InvokeBody::Json(_) => {
            return Err("the layer arrived as JSON, not as bytes".into())
        }
    };
    let (composite, _, compose_ms) = compose_layer(&bytes, margin)?;
    let published = crate::clipboard::publish(&composite.rgba, composite.width, composite.height)?;
    println!(
        "copied {}x{} to the clipboard: composed in {compose_ms} ms, encoded in {} ms, published in {} ms as {}",
        composite.width,
        composite.height,
        published.encode_ms,
        published.publish_ms,
        published.formats.join(", ")
    );
    let (width, height) = (composite.width, composite.height);
    #[cfg(feature = "stage0-checks")]
    checks::remember_copy(composite);
    Ok(CopyReport {
        width,
        height,
        compose_ms,
        published,
    })
}

/// The page's marks for S0.7: booted, painted, focused. Stamped here when they arrive, so
/// on the host's clock and late by one crossing. Any other name is ignored.
#[tauri::command]
pub fn editor_mark(name: String) {
    match name.as_str() {
        "page painted" => marks::mark(marks::PAGE_PAINTED),
        "page focused" => marks::mark(marks::PAGE_FOCUSED),
        "page booted" => marks::startup(marks::PAGE_BOOTED),
        _ => {}
    }
}

/// Everything the page says, on the terminal.
///
/// Without this a web view failure is invisible from a shell: the page throws, the window is
/// hidden, and the run simply hangs. Which is exactly what happened the first time.
#[tauri::command]
pub fn editor_log(level: String, line: String) {
    println!("page[{level}] {line}");
}

/// What the page needs to know about the image it is showing.
#[derive(serde::Serialize, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub source: String,
    /// The file's name when the image came from one, empty for a capture.
    pub file: String,
    /// "still", "animation", "pages" or "vector".
    pub kind: String,
    /// The frame or page on screen, and how many there are. 0 of 1 for a capture.
    pub index: u32,
    pub count: u32,
    /// The document's number: a new capture or file gets the next one, a frame of the same
    /// file keeps it (S1.1).
    pub document_id: u64,
}

#[tauri::command]
pub fn editor_image_info() -> Result<ImageInfo, String> {
    let image = state().image().ok_or("no image is loaded")?;
    let (file, kind, index, count) = match state().document.lock() {
        Ok(slot) => match slot.as_ref() {
            Some(document) => (
                document
                    .opened
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                match document.opened.kind {
                    Kind::Still => "still",
                    Kind::Animation { .. } => "animation",
                    Kind::Pages { .. } => "pages",
                    Kind::Vector { .. } => "vector",
                }
                .to_string(),
                document.index,
                document.opened.kind.count(),
            ),
            None => (String::new(), "still".to_string(), 0, 1),
        },
        Err(_) => (String::new(), "still".to_string(), 0, 1),
    };
    Ok(ImageInfo {
        width: image.frame.width(),
        height: image.frame.height(),
        source: image.frame.source.to_string(),
        file,
        kind,
        index,
        count,
        document_id: current_document_id(),
    })
}

#[tauri::command]
pub fn editor_show(app: AppHandle) -> Result<u128, String> {
    show(&app)
}

/// Escape in the idle editor hides it and returns the focus (§3.6, §3.1).
#[tauri::command]
pub fn editor_hide(app: AppHandle) -> Result<(), String> {
    hide(&app)
}

/// Fullscreen in and out (§3.4): one key in, the same key or Escape out. Returns the new
/// state so the page knows which Escape it is handling.
#[tauri::command]
pub fn editor_fullscreen(app: AppHandle, on: bool) -> Result<bool, String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    window.set_fullscreen(on).map_err(|err| err.to_string())?;
    window.is_fullscreen().map_err(|err| err.to_string())
}

/// The capture hotkey as configured, for the page's empty state.
#[tauri::command]
pub fn editor_hotkey() -> String {
    HOTKEY_LABEL.lock().map(|s| s.clone()).unwrap_or_default()
}

/// `Ctrl+O`: Windows' own picker, owned by the editor window, on a thread of its own; a
/// chosen file opens like any other (§3.1). Returns at once.
#[tauri::command]
pub fn editor_open_dialog(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    let owner = window.hwnd().map_err(|err| err.to_string())?.0 as isize;
    set_dialog_outcome("");
    std::thread::spawn(move || {
        let owner = windows::Win32::Foundation::HWND(owner as *mut _);
        let outcome = match crate::dialog::pick_image(Some(owner)) {
            Ok(Some(path)) => match open_path(&path) {
                Ok(_) => format!("opened {} (Ctrl+O)", path.display()),
                Err(err) => format!("OPEN FAILED for {} (Ctrl+O): {err}", path.display()),
            },
            Ok(None) => "Ctrl+O: nothing chosen".to_string(),
            Err(err) => format!("Ctrl+O: {err}"),
        };
        crate::log(&outcome);
        set_dialog_outcome(&outcome);
    });
    Ok(())
}

/// What the last Ctrl+O did, for the checks: empty while the picker is open.
static DIALOG_OUTCOME: Mutex<String> = Mutex::new(String::new());

fn set_dialog_outcome(text: &str) {
    if let Ok(mut slot) = DIALOG_OUTCOME.lock() {
        *slot = text.to_string();
    }
}

/// What the window really is, in both coordinate systems.
///
/// The page cannot be trusted to know: this web view reports a device pixel ratio of 1 on a
/// 225% display, so a page that believes `devicePixelRatio` would draw a quarter of the
/// pixels it should and call it actual size. The ratio is therefore derived from these two
/// numbers against the page's own CSS size, and both are logged so the discrepancy stays
/// visible rather than being absorbed.
#[derive(serde::Serialize)]
pub struct WindowMetrics {
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
    /// What Win32 itself says this window is scaled at, in dots per inch. 96 is 100%.
    /// Reported separately because the runtime and the platform disagreed here.
    pub window_dpi: u32,
}

#[tauri::command]
pub fn editor_window_metrics(app: AppHandle) -> Result<WindowMetrics, String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let window_dpi = unsafe {
        windows::Win32::UI::HiDpi::GetDpiForWindow(windows::Win32::Foundation::HWND(hwnd.0))
    };
    Ok(WindowMetrics {
        physical_width: size.width,
        physical_height: size.height,
        scale_factor: window.scale_factor().map_err(|e| e.to_string())?,
        window_dpi,
    })
}

// ---------------------------------------------------------------- the region service

/// The one origin the region scheme answers: the editor page's own. Any other page inside
/// the web view gets no pixels (S1.1, decided with the content security policy).
const APP_ORIGIN: &str = "http://tauri.localhost";

/// One request waiting to be served. Only the newest survives.
struct Pending {
    query: String,
    responder: UriSchemeResponder,
}

struct Mailbox {
    slot: Mutex<Option<Pending>>,
    ready: Condvar,
}

/// The one region worker, started on first use.
///
/// Holding an arrow key sends a request per keypress. The first version spawned a thread and
/// a full resample for each, and the page discarded the late answers; the work was still
/// done, ten times over (review T8). Now there is one worker and one slot: a new request
/// replaces the pending one, which is answered as superseded, and only the newest is served.
fn mailbox() -> &'static Mailbox {
    static MAILBOX: OnceLock<Mailbox> = OnceLock::new();
    MAILBOX.get_or_init(|| {
        std::thread::spawn(region_worker);
        Mailbox {
            slot: Mutex::new(None),
            ready: Condvar::new(),
        }
    })
}

fn serve_region(query: String, responder: UriSchemeResponder) {
    let mb = mailbox();
    let Ok(mut slot) = mb.slot.lock() else {
        return;
    };
    if let Some(old) = slot.replace(Pending { query, responder }) {
        old.responder.respond(
            tauri::http::Response::builder()
                .status(409)
                .header("Access-Control-Allow-Origin", APP_ORIGIN)
                .body(b"superseded by a newer request".to_vec())
                .expect("an error response"),
        );
    }
    mb.ready.notify_one();
}

fn region_worker() {
    let mb = mailbox();
    loop {
        let pending = {
            let Ok(mut slot) = mb.slot.lock() else {
                return;
            };
            while slot.is_none() {
                slot = match mb.ready.wait(slot) {
                    Ok(guard) => guard,
                    Err(_) => return,
                };
            }
            slot.take()
        };
        let Some(pending) = pending else { continue };
        match region_bytes(&pending.query) {
            Ok((bytes, _, _)) => pending.responder.respond(
                tauri::http::Response::builder()
                    .header("Content-Type", "application/octet-stream")
                    .header("Access-Control-Allow-Origin", APP_ORIGIN)
                    .body(bytes)
                    .expect("a response with a body"),
            ),
            Err(err) => pending.responder.respond(
                tauri::http::Response::builder()
                    .status(400)
                    .header("Access-Control-Allow-Origin", APP_ORIGIN)
                    .body(err.into_bytes())
                    .expect("an error response"),
            ),
        }
    }
}

/// What a region request resolves to, before any pixel is touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionPlan {
    /// The region in full-image pixels, clamped to the image.
    pub region: ImageRect,
    pub target_w: u32,
    pub target_h: u32,
    /// Which pyramid level to read from: 0 is the full image, k is 1/2^k.
    pub shift: u32,
}

/// The one-to-one case: no resampling at all, which is what a 100% view must be.
impl RegionPlan {
    pub fn is_one_to_one(&self) -> bool {
        self.shift == 0 && self.target_w == self.region.width && self.target_h == self.region.height
    }
}

/// The display policy, as arithmetic: clamp the request to the image, scale the output the
/// same way so a viewport hanging over the edge does not shift what the page draws, and
/// pick the pyramid level whose scale is at or above the one requested.
#[allow(clippy::too_many_arguments)]
pub fn plan_region(
    image_w: u32,
    image_h: u32,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    out_w: i64,
    out_h: i64,
    max_shift: u32,
) -> Result<RegionPlan, String> {
    let (fw, fh) = (i64::from(image_w), i64::from(image_h));
    if w <= 0 || h <= 0 || out_w <= 0 || out_h <= 0 {
        return Err("a region needs a size".into());
    }
    if out_w > 8192 || out_h > 8192 {
        return Err("that output size is larger than any display".into());
    }
    let left = x.clamp(0, fw);
    let top = y.clamp(0, fh);
    let right = (x + w).clamp(0, fw);
    let bottom = (y + h).clamp(0, fh);
    if right <= left || bottom <= top {
        return Err("that region is outside the image".into());
    }
    let region = ImageRect {
        x: left as u32,
        y: top as u32,
        width: (right - left) as u32,
        height: (bottom - top) as u32,
    };
    let scale_w = out_w as f64 / w as f64;
    let scale_h = out_h as f64 / h as f64;
    let target_w = ((region.width as f64 * scale_w).round() as u32).max(1);
    let target_h = ((region.height as f64 * scale_h).round() as u32).max(1);

    // The level whose scale is still at or above the requested one, so nothing is ever
    // enlarged: a request at 0.4 reads the half-size level (0.5), never the quarter (0.25).
    let scale = scale_w.max(scale_h);
    let mut shift = 0u32;
    while shift < max_shift && 0.5f64.powi(shift as i32 + 1) >= scale {
        shift += 1;
    }
    Ok(RegionPlan {
        region,
        target_w,
        target_h,
        shift,
    })
}

/// Serves one region of the current image, resampled to the size the page asks for.
///
/// This is the display proxy from part 5, and the policy it has to keep is that the visible
/// region carries the detail its zoom implies. So the page asks for the region it can see at
/// the size it will show it, and gets those pixels: at 100% that is a 1:1 copy, and at
/// fit-to-window it is a downscale of only what is on screen. Nothing here enlarges a
/// preview that was made for a smaller view.
pub fn region_bytes(query: &str) -> Result<(Vec<u8>, u32, u32), String> {
    let started = Instant::now();
    let result = region_bytes_inner(query);
    match &result {
        Ok((bytes, w, h)) => println!(
            "region {query} -> {w}x{h}, {:.1} MB, {} ms",
            bytes.len() as f64 / (1024.0 * 1024.0),
            started.elapsed().as_millis()
        ),
        Err(err) => println!("region {query} -> refused: {err}"),
    }
    result
}

/// How many pyramid levels an image may have: enough that a 4K image reaches 1/64.
const MAX_SHIFT: u32 = 6;

fn region_bytes_inner(query: &str) -> Result<(Vec<u8>, u32, u32), String> {
    let mut x = 0i64;
    let mut y = 0i64;
    let mut w = 0i64;
    let mut h = 0i64;
    let mut out_w = 0i64;
    let mut out_h = 0i64;
    for pair in query.trim_start_matches('?').split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default().parse::<i64>().unwrap_or(0);
        match key {
            "x" => x = value,
            "y" => y = value,
            "w" => w = value,
            "h" => h = value,
            "ow" => out_w = value,
            "oh" => out_h = value,
            _ => {}
        }
    }

    let image = state().image().ok_or("no image is loaded")?;
    let plan = plan_region(
        image.frame.width(),
        image.frame.height(),
        x,
        y,
        w,
        h,
        out_w,
        out_h,
        MAX_SHIFT,
    )?;
    let region = plan.region;

    if plan.is_one_to_one() {
        // The 1:1 case, which is what a 100% view must be: no resampling at all.
        let cropped = image.frame.crop(region).ok_or("the crop was refused")?;
        return Ok((
            with_size_prefix(cropped, region.width, region.height),
            region.width,
            region.height,
        ));
    }

    // From the level at or above the requested scale, so the remaining resample is small
    // and never an enlargement. Level 0 is the frame itself.
    if plan.shift == 0 {
        return resample_from(
            &image.frame.rgba,
            image.frame.width(),
            image.frame.height(),
            region,
            plan,
        );
    }
    let level = image
        .level(plan.shift)
        .ok_or("the level could not be built")?;
    let rect = scaled_rect(region, plan.shift, level.width, level.height);
    resample_from(&level.rgba, level.width, level.height, rect, plan)
}

/// The region's rectangle on a level at 1/2^shift, widened outwards to whole pixels and
/// kept inside the level.
fn scaled_rect(region: ImageRect, shift: u32, level_w: u32, level_h: u32) -> ImageRect {
    let f = 0.5f64.powi(shift as i32);
    let max_x = level_w.saturating_sub(1);
    let max_y = level_h.saturating_sub(1);
    let x = (((region.x as f64) * f).floor() as u32).min(max_x);
    let y = (((region.y as f64) * f).floor() as u32).min(max_y);
    let right = ((((region.x + region.width) as f64) * f).ceil() as u32).clamp(x + 1, level_w);
    let bottom = ((((region.y + region.height) as f64) * f).ceil() as u32).clamp(y + 1, level_h);
    ImageRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    }
}

fn resample_from(
    rgba: &[u8],
    src_w: u32,
    src_h: u32,
    rect: ImageRect,
    plan: RegionPlan,
) -> Result<(Vec<u8>, u32, u32), String> {
    let cropped = recon_pixels::crop(rgba, src_w, src_h, rect.x, rect.y, rect.width, rect.height)
        .ok_or("the crop was refused")?;
    let resampled = recon_pixels::resample(
        &cropped,
        rect.width,
        rect.height,
        plan.target_w,
        plan.target_h,
    )
    .ok_or("the resample was refused")?;
    Ok((
        with_size_prefix(resampled, plan.target_w, plan.target_h),
        plan.target_w,
        plan.target_h,
    ))
}

/// Eight bytes of width and height in front of the pixels.
///
/// The dimensions used to travel as response headers, and a cross-origin fetch cannot read
/// a custom header unless the server lists it in Access-Control-Expose-Headers. The page
/// therefore read zeroes and refused a perfectly good buffer. Putting them in the body
/// removes a whole class of that, and the page checks the length against them.
fn with_size_prefix(mut bytes: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 8);
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.append(&mut bytes);
    out
}

// ---------------------------------------------------------------- the checks' commands

#[cfg(feature = "stage0-checks")]
pub use checks::*;

#[cfg(feature = "stage0-checks")]
mod checks {
    use super::*;
    use crate::capture::coords::DesktopRect;
    use crate::capture::{screen, CaptureSource};

    /// A synthetic image whose whole purpose is to make resampling visible.
    ///
    /// A one-pixel checkerboard survives a 1:1 view and turns to flat grey the moment
    /// anything downscales it, so "the detail matches the zoom" becomes a property the page
    /// can assert on the pixels rather than a claim someone squints at. The thin lines and
    /// the small blocks do the same for scaling errors that are off by a fraction.
    pub fn detail_probe_frame(width: u32, height: u32) -> Frame {
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                // The left half is a one-pixel checkerboard, the right half thin verticals
                // every eight pixels, and a band of solid blocks across the middle.
                let checker = (x + y) % 2 == 0;
                let thin = x % 8 == 0;
                let band = (height / 2..height / 2 + 40).contains(&y);
                let value = if band {
                    if (x / 40) % 2 == 0 {
                        30
                    } else {
                        220
                    }
                } else if x < width / 2 {
                    if checker {
                        255
                    } else {
                        0
                    }
                } else if thin {
                    255
                } else {
                    20
                };
                rgba[i] = value;
                rgba[i + 1] = value;
                rgba[i + 2] = value;
                rgba[i + 3] = 255;
            }
        }
        Frame {
            geometry: crate::capture::coords::FrameGeometry {
                origin_x: 0,
                origin_y: 0,
                width,
                height,
            },
            rgba,
            source: "synthetic detail probe",
        }
    }

    /// Whether this run wants the page to run its own checks.
    ///
    /// A query string was the first attempt and it does not work: an app URL is resolved as
    /// an asset path, so "index.html?selftest=1" is simply not a file. The page asks instead.
    static WANTS_CHECKS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    pub fn request_checks() {
        WANTS_CHECKS.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    #[tauri::command]
    pub fn editor_wants_checks() -> bool {
        WANTS_CHECKS.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Whether this run wants the page to place a couple of example callouts, so the editor
    /// can be LOOKED at. The Workflow's step 9 asks for that on anything visible, and no
    /// number of assertions substitutes for seeing the thing once.
    static WANTS_DEMO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    pub fn request_demo() {
        WANTS_DEMO.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    #[tauri::command]
    pub fn editor_wants_demo() -> bool {
        WANTS_DEMO.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Writes a PNG of the editor window's own area of the screen, beside the executable.
    pub fn shoot_window(app: &AppHandle, name: &str) -> Result<String, String> {
        let window = app
            .get_webview_window("editor")
            .ok_or("there is no editor window")?;
        let position = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let rect = DesktopRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        };
        let pixels = screen::copy_rect(rect).map_err(|err| err.to_string())?;
        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name(name))
            .unwrap_or_else(|_| name.into());
        let image = image::RgbaImage::from_raw(rect.width, rect.height, pixels)
            .ok_or("the window pixels did not match its size")?;
        image.save(&path).map_err(|err| err.to_string())?;
        let shown = path.display().to_string();
        println!("editor screenshot: {shown}");
        Ok(shown)
    }

    /// A screenshot of the window, for the demo run to leave behind.
    #[tauri::command]
    pub fn editor_shoot_window(app: AppHandle) -> Result<String, String> {
        shoot_window(&app, "s04-editor.png")
    }

    /// The page's way to end a run.
    #[tauri::command]
    pub fn editor_exit(app: AppHandle, code: i32) {
        app.exit(code);
    }

    /// One SVG fixture for the page to render on its own: the source text and the size
    /// resvg rendered it at.
    #[derive(serde::Serialize)]
    pub struct SvgCase {
        pub name: String,
        pub svg: String,
        pub width: u32,
        pub height: u32,
    }

    fn svg_fixture_dir() -> std::path::PathBuf {
        std::env::current_exe()
            .map(|exe| exe.with_file_name("s05-svg-cases"))
            .unwrap_or_else(|_| "s05-svg-cases".into())
    }

    /// The three SVG fixtures, generated beside the executable if they are not there.
    #[tauri::command]
    pub fn editor_svg_cases() -> Result<Vec<SvgCase>, String> {
        let dir = svg_fixture_dir();
        if !dir.join("svg-mask-and-clip.svg").exists() {
            source::fixtures::make_svgs(&dir).map_err(|err| err.to_string())?;
        }
        // Every SVG in the folder: the three generated ones, plus any real export dropped in
        // beside them for the same comparison.
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .map_err(|err| err.to_string())?
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|n| n.ends_with(".svg"))
            .collect();
        names.sort();
        let mut cases = Vec::new();
        for name in names {
            let path = dir.join(&name);
            let svg = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
            let mut opened = source::open(&path).map_err(|err| err.to_string())?;
            let frame = opened.frame(0).map_err(|err| err.to_string())?;
            cases.push(SvgCase {
                name,
                svg,
                width: frame.width,
                height: frame.height,
            });
        }
        Ok(cases)
    }

    /// The page's rendering of one case, compared pixel by pixel against resvg's. Reports
    /// how many pixels differ by more than a tolerance, and where the worst region is, so a
    /// dropped element shows up as a block of differences rather than a percentage.
    #[derive(serde::Serialize)]
    pub struct SvgVerdict {
        pub name: String,
        pub pixels: usize,
        pub differing: usize,
        pub percent: f64,
        /// The bounding box of the differing pixels, if any: left, top, right, bottom.
        pub bbox: Option<[u32; 4]>,
    }

    #[tauri::command]
    pub fn editor_svg_compare(request: tauri::ipc::Request<'_>) -> Result<SvgVerdict, String> {
        // A raw body carries no JSON arguments, so the case name travels as a header.
        let name = request
            .headers()
            .get("name")
            .and_then(|v| v.to_str().ok())
            .ok_or("no case name header")?
            .to_string();
        let bytes = match request.body() {
            tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
            tauri::ipc::InvokeBody::Json(_) => return Err("the pixels arrived as JSON".into()),
        };
        let path = svg_fixture_dir().join(&name);
        let mut opened = source::open(&path).map_err(|err| err.to_string())?;
        let ours = opened.frame(0).map_err(|err| err.to_string())?;
        let count = (ours.width * ours.height) as usize;
        if bytes.len() != count * 4 {
            return Err(format!(
                "the page sent {} bytes for {}x{}",
                bytes.len(),
                ours.width,
                ours.height
            ));
        }
        let mut differing = 0usize;
        let mut bbox: Option<[u32; 4]> = None;
        for i in 0..count {
            let a = &ours.rgba[i * 4..i * 4 + 4];
            let b = &bytes[i * 4..i * 4 + 4];
            // Both are straight alpha; compare premultiplied so a transparent pixel's
            // colour noise does not count, and allow antialiasing at edges.
            let pa = |p: &[u8], c: usize| (p[c] as u32 * p[3] as u32) / 255;
            let differs =
                (0..3).any(|c| pa(a, c).abs_diff(pa(b, c)) > 48) || a[3].abs_diff(b[3]) > 48;
            if differs {
                differing += 1;
                let x = (i as u32) % ours.width;
                let y = (i as u32) / ours.width;
                bbox = Some(match bbox {
                    None => [x, y, x, y],
                    Some([l, t, r, b]) => [l.min(x), t.min(y), r.max(x), b.max(y)],
                });
            }
        }
        Ok(SvgVerdict {
            name,
            pixels: count,
            differing,
            percent: differing as f64 * 100.0 / count.max(1) as f64,
            bbox,
        })
    }

    /// The last composite made by a check, so the live comparison and the clipboard
    /// read-back have the exact bytes to compare against.
    static LAST_COMPOSITE: Mutex<Option<Arc<crate::compose::Composite>>> = Mutex::new(None);

    fn remember(composite: Arc<crate::compose::Composite>) {
        if let Ok(mut slot) = LAST_COMPOSITE.lock() {
            *slot = Some(composite);
        }
    }

    /// A product copy, remembered for the read-back check.
    pub fn remember_copy(composite: crate::compose::Composite) {
        remember(Arc::new(composite));
    }

    fn last_composite() -> Result<Arc<crate::compose::Composite>, String> {
        LAST_COMPOSITE
            .lock()
            .map_err(|_| "the composite slot is poisoned".to_string())?
            .clone()
            .ok_or_else(|| "no composite has been made yet".into())
    }

    /// Where the S0.6 reference images live: in the repository, beside the code, so a
    /// reviewed reference is a committed file and a changed one is a diff.
    fn reference_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("references/s06")
    }

    fn fixture_dir() -> std::path::PathBuf {
        std::env::current_exe()
            .map(|exe| exe.with_file_name("s06-fixtures"))
            .unwrap_or_else(|_| "s06-fixtures".into())
    }

    /// Generates the files the S0.6 checks open, beside the executable, once.
    #[tauri::command]
    pub fn editor_make_fixtures() -> Result<String, String> {
        let dir = fixture_dir();
        if !dir.join("reference-scene.png").exists() {
            source::fixtures::make_export_set(&dir).map_err(|err| err.to_string())?;
        }
        Ok(dir.display().to_string())
    }

    /// Opens one of those files into the editor, frame 0, without showing the window or
    /// emitting an event: the page called, so the page loads what comes back.
    #[tauri::command]
    pub fn editor_open_fixture(name: String) -> Result<ImageInfo, String> {
        let path = fixture_dir().join(&name);
        let mut opened = source::open(&path).map_err(|err| err.to_string())?;
        let decoded = opened.frame(0).map_err(|err| err.to_string())?;
        let format = opened.format.name();
        // A fixture opened is a new document, as a file opened through the product is.
        next_document_id();
        state().set_document(Some(Document { opened, index: 0 }));
        state().set_frame(frame_of(decoded, format));
        let info = editor_image_info()?;
        if let Ok(app) = app() {
            set_title(app, &info);
        }
        Ok(info)
    }

    /// The physical position of the window's client area on the desktop, so a rectangle
    /// the page measured in CSS pixels can be found on the screen.
    #[derive(serde::Serialize)]
    pub struct WindowOrigin {
        pub x: i32,
        pub y: i32,
    }

    #[tauri::command]
    pub fn editor_window_origin(app: AppHandle) -> Result<WindowOrigin, String> {
        let window = app
            .get_webview_window("editor")
            .ok_or("there is no editor window")?;
        let position = window.inner_position().map_err(|e| e.to_string())?;
        Ok(WindowOrigin {
            x: position.x,
            y: position.y,
        })
    }

    /// What one export check found.
    #[derive(serde::Serialize)]
    pub struct ExportCheck {
        pub webview_version: String,
        pub width: u32,
        pub height: u32,
        /// Pixels the annotation layer touched at all, alpha above zero.
        pub covered: usize,
        /// Source pixels the layer left alone that came out different. Zero, exactly.
        pub source_mismatches: usize,
        /// "none" when no reference was asked for, "new" when this run wrote one, "match"
        /// or "differs" against an existing one.
        pub reference: String,
        pub differing: usize,
        pub percent: f64,
        pub bbox: Option<[u32; 4]>,
        /// The pixel at the point the page asked about, if it did.
        pub sample: Option<[u8; 4]>,
        pub compose_ms: u128,
        pub path: String,
    }

    /// Differing pixels between two same-sized RGBA buffers, premultiplied so a fully
    /// transparent pixel's colour does not count, at a channel tolerance of 48: enough to
    /// absorb glyph antialiasing, far too little to hide a moved box or a different wrap.
    pub fn compare(a: &[u8], b: &[u8], w: u32, h: u32) -> (usize, Option<[u32; 4]>) {
        let mut differing = 0usize;
        let mut bbox: Option<[u32; 4]> = None;
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let pa = a[i + 3] as u32;
                let pb = b[i + 3] as u32;
                let mut off = pa.abs_diff(pb) > 48;
                for c in 0..3 {
                    let ca = a[i + c] as u32 * pa / 255;
                    let cb = b[i + c] as u32 * pb / 255;
                    if ca.abs_diff(cb) > 48 {
                        off = true;
                    }
                }
                if off {
                    differing += 1;
                    bbox = Some(match bbox {
                        None => [x, y, x, y],
                        Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
                    });
                }
            }
        }
        (differing, bbox)
    }

    /// The S0.6 export check: the page sends the annotation layer as a PNG in canvas
    /// space with the margin in a header; the host composes it over the untouched source
    /// exactly as a copy does, counts the source pixels that changed, writes the result
    /// beside the executable, and, when asked, compares it against the committed reference
    /// or creates that reference for review.
    #[tauri::command]
    pub fn editor_export_check(request: tauri::ipc::Request<'_>) -> Result<ExportCheck, String> {
        let header = |name: &str| -> Option<String> {
            request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };
        let name = header("name").unwrap_or_else(|| "export".into());
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!("{name:?} is not a check name"));
        }
        let margin =
            crate::compose::Margin::parse(&header("margin").unwrap_or_else(|| "0,0,0,0".into()))?;
        let mode = header("mode").unwrap_or_else(|| "source".into());
        let sample_at = header("sample").and_then(|s| {
            let mut it = s.split(',').map(|p| p.trim().parse::<u32>().ok());
            Some((it.next()??, it.next()??))
        });
        let bytes = match request.body() {
            tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
            tauri::ipc::InvokeBody::Json(_) => {
                return Err("the layer arrived as JSON, not as bytes".into())
            }
        };

        let (composite, layer, compose_ms) = compose_layer(&bytes, margin)?;
        let image = state().image().ok_or("no image is loaded")?;
        let (sw, sh) = (image.frame.width(), image.frame.height());
        let source_mismatches = crate::compose::source_mismatches(
            &composite,
            &image.frame.rgba,
            sw,
            sh,
            &layer,
            margin,
        );
        let covered = layer
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] != 0)
            .count();
        let sample = sample_at.and_then(|(x, y)| {
            if x < composite.width && y < composite.height {
                let i = ((y * composite.width + x) * 4) as usize;
                Some([
                    composite.rgba[i],
                    composite.rgba[i + 1],
                    composite.rgba[i + 2],
                    composite.rgba[i + 3],
                ])
            } else {
                None
            }
        });

        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name(format!("s06-{name}.png")))
            .unwrap_or_else(|_| format!("s06-{name}.png").into());
        let out =
            image::RgbaImage::from_raw(composite.width, composite.height, composite.rgba.clone())
                .ok_or("the composite did not match its size")?;
        out.save(&path).map_err(|err| err.to_string())?;

        let (reference, differing, percent, bbox) = if mode == "reference" {
            let dir = reference_dir();
            std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
            let reference_path = dir.join(format!("{name}.png"));
            if reference_path.exists() {
                let expected = image::open(&reference_path)
                    .map_err(|err| format!("the reference did not open: {err}"))?
                    .into_rgba8();
                if expected.dimensions() != (composite.width, composite.height) {
                    (
                        format!(
                            "differs: the reference is {}x{}",
                            expected.width(),
                            expected.height()
                        ),
                        composite.rgba.len() / 4,
                        100.0,
                        None,
                    )
                } else {
                    let (differing, bbox) = compare(
                        &composite.rgba,
                        expected.as_raw(),
                        composite.width,
                        composite.height,
                    );
                    let percent =
                        differing as f64 * 100.0 / (composite.rgba.len() / 4).max(1) as f64;
                    (
                        if differing == 0 {
                            "match".into()
                        } else {
                            "differs".into()
                        },
                        differing,
                        percent,
                        bbox,
                    )
                }
            } else {
                out.save(&reference_path).map_err(|err| err.to_string())?;
                ("new".into(), 0, 0.0, None)
            }
        } else {
            ("none".into(), 0, 0.0, None)
        };

        remember(Arc::new(composite));
        Ok(ExportCheck {
            webview_version: tauri::webview_version().unwrap_or_else(|_| "unknown".into()),
            width: sw + margin.left + margin.right,
            height: sh + margin.top + margin.bottom,
            covered,
            source_mismatches,
            reference,
            differing,
            percent,
            bbox,
            sample,
            compose_ms,
            path: path.display().to_string(),
        })
    }

    /// What the screen shows against what the composite holds, for the same rectangle.
    #[derive(serde::Serialize)]
    pub struct LiveCompare {
        pub differing: usize,
        pub percent: f64,
        pub bbox: Option<[u32; 4]>,
        /// Rows carrying text ink, grouped into lines: the wrap, read off the pixels.
        pub live_lines: Vec<[u32; 2]>,
        pub export_lines: Vec<[u32; 2]>,
        pub width: u32,
        pub height: u32,
    }

    /// Bands of consecutive rows that contain bright pixels, which on a dark bubble is
    /// where the text is. Two renders with the same wrap have the same bands.
    fn ink_lines(rgba: &[u8], w: u32, h: u32) -> Vec<[u32; 2]> {
        let mut lines = Vec::new();
        let mut open: Option<u32> = None;
        for y in 0..h {
            let row = &rgba[(y * w * 4) as usize..((y + 1) * w * 4) as usize];
            let ink = row
                .as_chunks::<4>()
                .0
                .iter()
                .any(|px| (px[0] as u32 + px[1] as u32 + px[2] as u32) / 3 > 150);
            match (ink, open) {
                (true, None) => open = Some(y),
                (false, Some(start)) => {
                    lines.push([start, y - 1]);
                    open = None;
                }
                _ => {}
            }
        }
        if let Some(start) = open {
            lines.push([start, h - 1]);
        }
        lines
    }

    /// S0.6's third check: the live editor against the output while typing. The page names
    /// a rectangle twice, on the desktop in physical pixels and in canvas space, and at
    /// zoom 1 those are the same pixels; the host reads the screen and the last composite
    /// and compares them.
    #[tauri::command]
    pub fn editor_live_compare(
        screen_x: i32,
        screen_y: i32,
        canvas_x: u32,
        canvas_y: u32,
        width: u32,
        height: u32,
    ) -> Result<LiveCompare, String> {
        let composite = last_composite()?;
        if canvas_x + width > composite.width || canvas_y + height > composite.height {
            return Err(format!(
                "the rectangle {canvas_x},{canvas_y} {width}x{height} is outside the {}x{} composite",
                composite.width, composite.height
            ));
        }
        let rect = DesktopRect {
            x: screen_x,
            y: screen_y,
            width,
            height,
        };
        let live = screen::copy_rect(rect).map_err(|err| err.to_string())?;
        let mut exported = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let from = (((canvas_y + y) * composite.width + canvas_x) * 4) as usize;
            exported.extend_from_slice(&composite.rgba[from..from + (width * 4) as usize]);
        }
        // The screen is opaque; a semi-transparent composite pixel is what the screen would
        // show over the page's paper, and no check uses one here, so alpha is forced.
        for px in exported.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        let (differing, bbox) = compare(&live, &exported, width, height);
        let name = "s06-live.png";
        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name(name))
            .unwrap_or_else(|_| name.into());
        // Side by side, the live pixels on the left, the export on the right, to look at.
        let mut pair = Vec::with_capacity((width * 2 * height * 4) as usize);
        for y in 0..height as usize {
            let row = width as usize * 4;
            pair.extend_from_slice(&live[y * row..(y + 1) * row]);
            pair.extend_from_slice(&exported[y * row..(y + 1) * row]);
        }
        if let Some(img) = image::RgbaImage::from_raw(width * 2, height, pair) {
            let _ = img.save(&path);
        }
        Ok(LiveCompare {
            differing,
            percent: differing as f64 * 100.0 / (width * height).max(1) as f64,
            bbox,
            live_lines: ink_lines(&live, width, height),
            export_lines: ink_lines(&exported, width, height),
            width,
            height,
        })
    }

    /// The clipboard read back the way a destination would read it, against the last
    /// composite. Checks only: the product never reads the clipboard.
    #[derive(serde::Serialize)]
    pub struct ClipboardReadBack {
        pub png: bool,
        pub dib_v5: bool,
        pub dib: bool,
        pub png_matches: bool,
        pub width: u32,
        pub height: u32,
        pub png_bytes: usize,
    }

    #[tauri::command]
    pub fn editor_clipboard_readback() -> Result<ClipboardReadBack, String> {
        let composite = last_composite()?;
        let back = crate::clipboard::read_back()?;
        let (mut png_matches, mut width, mut height, mut png_bytes) = (false, 0, 0, 0);
        if let Some(bytes) = &back.png {
            png_bytes = bytes.len();
            let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                .map_err(|err| format!("the clipboard PNG did not decode: {err}"))?
                .into_rgba8();
            width = decoded.width();
            height = decoded.height();
            png_matches = decoded.dimensions() == (composite.width, composite.height)
                && decoded.as_raw() == &composite.rgba;
        }
        Ok(ClipboardReadBack {
            png: back.png.is_some(),
            dib_v5: back.dib_v5,
            dib: back.dib,
            png_matches,
            width,
            height,
            png_bytes,
        })
    }

    /// Writes the reference environment beside the reference images: what the images are
    /// meaningful against (part 6a). The page supplies what only it knows.
    #[tauri::command]
    pub fn editor_environment(
        app: AppHandle,
        font: String,
        css_size: String,
    ) -> Result<String, String> {
        let metrics = editor_window_metrics(app)?;
        let monitors: Vec<String> = crate::capture::display::monitors()
            .iter()
            .map(|m| format!("{}x{} at {}%", m.rect.width, m.rect.height, m.scale_percent))
            .collect();
        let text = format!(
            "S0.6 reference environment, written {}\n\
             web view: WebView2 {}\n\
             window: {}x{} physical for {} css, scale factor {}, window dpi {} ({}%)\n\
             displays: {}\n\
             note font: {}\n\
             tolerance: a channel difference above 48, premultiplied, counts a pixel as differing; a reference passes at 0.5% or fewer differing pixels, and a wrap or box change fails at any tolerance\n",
            date_today(),
            tauri::webview_version().unwrap_or_else(|_| "unknown".into()),
            metrics.physical_width,
            metrics.physical_height,
            css_size,
            metrics.scale_factor,
            metrics.window_dpi,
            metrics.window_dpi * 100 / 96,
            monitors.join(", "),
            font,
        );
        let dir = reference_dir();
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        let path = dir.join("environment.txt");
        std::fs::write(&path, text).map_err(|err| err.to_string())?;
        Ok(path.display().to_string())
    }

    /// Today, from the system clock, as a date: no dependency for one line.
    fn date_today() -> String {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let days = secs / 86_400;
        // Civil-from-days, Howard Hinnant's algorithm.
        let z = days as i64 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
    }

    /// The retained captures, for the S1.1 check.
    #[tauri::command]
    pub fn editor_history() -> Vec<(u64, u32, u32, usize)> {
        history_summary()
    }

    /// The window's title, for the S1.3 check.
    #[tauri::command]
    pub fn editor_window_title(app: AppHandle) -> Result<String, String> {
        app.get_webview_window("editor")
            .ok_or("there is no editor window")?
            .title()
            .map_err(|err| err.to_string())
    }

    /// What the last Ctrl+O did, for the S1.2 check.
    #[tauri::command]
    pub fn editor_dialog_outcome() -> String {
        DIALOG_OUTCOME.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Escape, pressed for real, so the picker under test closes the way a person closes it.
    #[tauri::command]
    pub fn editor_press_escape() {
        crate::selftest::press_escape();
    }

    /// A probe through the real capture path: a new document, the previous capture
    /// retained, the page told, the window shown. What a hotkey does, minus the screen.
    #[tauri::command]
    pub fn editor_capture_probe(width: u32, height: u32) -> Result<ImageInfo, String> {
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err(format!("{width}x{height} is not a probe size"));
        }
        let mut frame = detail_probe_frame(width, height);
        frame.source = "capture";
        present(frame)?;
        editor_image_info()
    }

    /// Loads the synthetic probe as the current image, for the detail check.
    #[tauri::command]
    pub fn editor_load_probe(width: u32, height: u32) -> Result<ImageInfo, String> {
        if crate::measuring() {
            return Err("a measurement is running, and the product has no probe".into());
        }
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err(format!("{width}x{height} is not a probe size"));
        }
        next_document_id();
        state().set_frame(detail_probe_frame(width, height));
        editor_image_info()
    }

    /// Loads a fresh screen capture as the current image, which is the real case.
    #[tauri::command]
    pub fn editor_load_screen() -> Result<ImageInfo, String> {
        let frame = screen::WholeVirtualScreen
            .freeze()
            .map_err(|err| err.to_string())?;
        next_document_id();
        state().set_frame(frame);
        editor_image_info()
    }

    /// The result of the host looking at its own window on screen.
    #[derive(serde::Serialize)]
    pub struct WindowLook {
        /// Fraction of the sampled pixels that were the colour the page says it painted.
        pub matching: f64,
        /// The same fraction in a band just outside the reported window edge.
        pub outside_matching: f64,
        /// Fraction that were pure black, which is what a suspended or unpainted surface
        /// gives.
        pub black: f64,
        pub sampled: usize,
        pub rect: String,
    }

    /// Captures the editor window's own area of the screen and compares it against the
    /// colour the page claims to have painted.
    ///
    /// This is the only honest way to test the hidden-window failure: a suspended surface
    /// still has a perfectly correct document behind it, so asking the page what it drew
    /// proves nothing. Only the screen can say whether those pixels ever reached it.
    #[tauri::command]
    pub fn editor_look_at_window(
        app: AppHandle,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<WindowLook, String> {
        let window = app
            .get_webview_window("editor")
            .ok_or("there is no editor window")?;
        let position = window.inner_position().map_err(|e| e.to_string())?;
        let size = window.inner_size().map_err(|e| e.to_string())?;

        // Inset, so the frame, the title bar shadow and any rounded corner are not sampled.
        let inset = 40i32;
        let rect = DesktopRect {
            x: position.x + inset,
            y: position.y + inset,
            width: size.width.saturating_sub(inset as u32 * 2),
            height: size.height.saturating_sub(inset as u32 * 2),
        };
        if rect.width < 8 || rect.height < 8 {
            return Err("the window is too small to sample".into());
        }

        let pixels = screen::copy_rect(rect).map_err(|err| err.to_string())?;
        let mut matching = 0usize;
        let mut black = 0usize;
        let mut sampled = 0usize;
        // Every 37th pixel: enough to be decisive, cheap enough to run inside a check.
        for chunk in pixels.as_chunks::<4>().0.iter().step_by(37) {
            sampled += 1;
            let near = |a: u8, b: u8| a.abs_diff(b) <= 6;
            if near(chunk[0], r) && near(chunk[1], g) && near(chunk[2], b) {
                matching += 1;
            }
            if chunk[0] < 8 && chunk[1] < 8 && chunk[2] < 8 {
                black += 1;
            }
        }

        // And a band just OUTSIDE the reported right edge. If the window is physically
        // larger than the size the runtime reports, that band is the same colour, which is
        // how a logical size masquerading as a physical one is caught.
        let outside = DesktopRect {
            x: position.x + size.width as i32 + 8,
            y: position.y + inset,
            width: 120,
            height: rect.height.min(400),
        };
        let outside_green = match screen::copy_rect(outside) {
            Ok(pixels) => {
                let mut hit = 0usize;
                let mut seen = 0usize;
                for chunk in pixels.as_chunks::<4>().0.iter().step_by(11) {
                    seen += 1;
                    if chunk[0].abs_diff(r) <= 6
                        && chunk[1].abs_diff(g) <= 6
                        && chunk[2].abs_diff(b) <= 6
                    {
                        hit += 1;
                    }
                }
                hit as f64 / seen.max(1) as f64
            }
            Err(_) => -1.0,
        };

        Ok(WindowLook {
            outside_matching: outside_green,
            matching: matching as f64 / sampled.max(1) as f64,
            black: black as f64 / sampled.max(1) as f64,
            sampled,
            rect: format!("{},{} {}x{}", rect.x, rect.y, rect.width, rect.height),
        })
    }

    /// Shows the window and looks at the screen twice: once almost immediately, once
    /// settled.
    ///
    /// Two samples because the failure this exists to catch is a first presented frame that
    /// is blank or stale. One late sample would miss it, and one early sample alone could
    /// not tell a suspended surface from a window that simply had not been asked to paint
    /// yet.
    #[tauri::command]
    pub fn editor_show_and_look(
        app: AppHandle,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<Vec<LabelledLook>, String> {
        let shown_in = show(&app)?;
        let mut looks = Vec::new();

        std::thread::sleep(std::time::Duration::from_millis(40));
        let early = editor_look_at_window(app.clone(), r, g, b)?;
        looks.push(LabelledLook {
            label: format!("40 ms after a show that took {shown_in} ms"),
            outside_matching: early.outside_matching,
            matching: early.matching,
            black: early.black,
            sampled: early.sampled,
            rect: early.rect,
        });

        std::thread::sleep(std::time::Duration::from_millis(260));
        let settled = editor_look_at_window(app, r, g, b)?;
        looks.push(LabelledLook {
            label: "300 ms after the show".to_string(),
            outside_matching: settled.outside_matching,
            matching: settled.matching,
            black: settled.black,
            sampled: settled.sampled,
            rect: settled.rect,
        });

        Ok(looks)
    }

    #[derive(serde::Serialize)]
    pub struct LabelledLook {
        pub label: String,
        pub matching: f64,
        pub outside_matching: f64,
        pub black: f64,
        pub sampled: usize,
        pub rect: String,
    }

    /// The page's report, printed by the host so a run leaves a record in the terminal.
    #[tauri::command]
    pub fn editor_checks_done(app: AppHandle, report: String, failures: u32) {
        println!("{report}");
        println!();
        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name("s04-editor-checks.txt"))
            .unwrap_or_else(|_| "s04-editor-checks.txt".into());
        match std::fs::write(&path, &report) {
            Ok(()) => println!("report written to {}", path.display()),
            Err(err) => println!("the report could not be written: {err}"),
        }
        app.exit(if failures == 0 { 0 } else { 1 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(x: i64, y: i64, w: i64, h: i64, ow: i64, oh: i64) -> RegionPlan {
        plan_region(3840, 2160, x, y, w, h, ow, oh, MAX_SHIFT).unwrap()
    }

    #[test]
    fn an_interior_request_at_actual_size_is_one_to_one() {
        let p = plan(100, 200, 1280, 800, 1280, 800);
        assert!(p.is_one_to_one());
        assert_eq!(p.shift, 0);
        assert_eq!(
            p.region,
            ImageRect {
                x: 100,
                y: 200,
                width: 1280,
                height: 800
            }
        );
    }

    #[test]
    fn a_viewport_past_the_right_edge_is_clamped_and_scaled_the_same_way() {
        // 1280 asked at 1:1 from x=3000, only 840 exist: the output shrinks with it, so the
        // page draws exactly what it got and nothing shifts.
        let p = plan(3000, 0, 1280, 800, 1280, 800);
        assert_eq!(p.region.x, 3000);
        assert_eq!(p.region.width, 840);
        assert_eq!(p.target_w, 840);
        assert!(p.is_one_to_one());
    }

    #[test]
    fn a_fit_view_reads_from_the_half_level() {
        // The whole 4K image into 1280x720 is a third: the half level (0.5) is the nearest
        // one still at or above that scale, never the quarter.
        let p = plan(0, 0, 3840, 2160, 1280, 720);
        assert_eq!(p.shift, 1);
        assert_eq!((p.target_w, p.target_h), (1280, 720));
    }

    #[test]
    fn a_tiny_view_reads_from_a_deep_level_and_never_enlarges() {
        let p = plan(0, 0, 3840, 2160, 300, 169);
        // 300/3840 is 0.078: 1/8 is 0.125 (above), 1/16 is 0.0625 (below), so shift 3.
        assert_eq!(p.shift, 3);
        let p = plan(0, 0, 3840, 2160, 3839, 2159);
        assert_eq!(
            p.shift, 0,
            "a scale just under 1 must not read a half-size level"
        );
    }

    #[test]
    fn the_level_never_goes_past_the_cap() {
        let p = plan(0, 0, 3840, 2160, 1, 1);
        assert_eq!(p.shift, MAX_SHIFT);
    }

    #[test]
    fn empty_and_outside_requests_are_refused() {
        assert!(plan_region(3840, 2160, 0, 0, 0, 10, 10, 10, MAX_SHIFT).is_err());
        assert!(plan_region(3840, 2160, 5000, 0, 10, 10, 10, 10, MAX_SHIFT).is_err());
        assert!(plan_region(3840, 2160, 0, 0, 10, 10, 9000, 10, MAX_SHIFT).is_err());
    }

    #[test]
    fn a_rounded_target_that_equals_the_region_by_accident_is_still_one_to_one() {
        // 1000 asked, 1000 exist, output 1000: exactly the region, no resample.
        let p = plan(0, 0, 1000, 1000, 1000, 1000);
        assert!(p.is_one_to_one());
    }
}
