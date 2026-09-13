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

use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, UriSchemeResponder, WebviewUrl, WebviewWindowBuilder};

use crate::capture::coords::ImageRect;
use crate::capture::Frame;

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

/// What the editor currently holds. One image, because Stage 0 is one image.
pub struct Editor {
    image: Mutex<Option<Arc<Image>>>,
}

impl Editor {
    fn new() -> Self {
        Self {
            image: Mutex::new(None),
        }
    }

    pub fn set_frame(&self, frame: Frame) {
        if let Ok(mut slot) = self.image.lock() {
            *slot = Some(Arc::new(Image::new(frame)));
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
    let app = app()?;
    state().set_frame(frame);
    let info = editor_image_info()?;
    app.emit("capture-ready", &info)
        .map_err(|err| err.to_string())?;
    show(app)
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
        checks::editor_load_probe,
        checks::editor_load_screen,
        checks::editor_look_at_window,
        checks::editor_show_and_look,
        checks::editor_checks_done,
        checks::editor_wants_checks,
        checks::editor_wants_demo,
        checks::editor_shoot_window,
        checks::editor_export_layer,
        checks::editor_exit,
    ]);
    #[cfg(not(feature = "stage0-checks"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        editor_image_info,
        editor_show,
        editor_window_metrics,
        editor_log,
    ]);
    builder.register_asynchronous_uri_scheme_protocol("region", |_ctx, request, responder| {
        let query = request.uri().query().unwrap_or_default().to_string();
        serve_region(query, responder);
    })
}

// ---------------------------------------------------------------- the product's commands

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
}

#[tauri::command]
pub fn editor_image_info() -> Result<ImageInfo, String> {
    let image = state().image().ok_or("no image is loaded")?;
    Ok(ImageInfo {
        width: image.frame.width(),
        height: image.frame.height(),
        source: image.frame.source.to_string(),
    })
}

#[tauri::command]
pub fn editor_show(app: AppHandle) -> Result<u128, String> {
    show(&app)
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
                .header("Access-Control-Allow-Origin", "*")
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
                    .header("Access-Control-Allow-Origin", "*")
                    .body(bytes)
                    .expect("a response with a body"),
            ),
            Err(err) => pending.responder.respond(
                tauri::http::Response::builder()
                    .status(400)
                    .header("Access-Control-Allow-Origin", "*")
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

    /// What the export spike found, for the S0.6 record.
    #[derive(serde::Serialize)]
    pub struct ExportReport {
        pub webview_version: String,
        pub layer_bytes: usize,
        pub width: u32,
        pub height: u32,
        /// Pixels the annotation layer touched at all, alpha above zero.
        pub covered: usize,
        /// Pixels the layer left alone and that came out different from the source. The
        /// whole point: this has to be zero, exactly.
        pub source_mismatches: usize,
        pub decode_ms: u128,
        pub composite_ms: u128,
        pub encode_ms: u128,
        pub path: String,
    }

    /// The S0.6 export spike: the page sends the annotation layer as a PNG, the host
    /// composites it over the untouched source at the origin (no margin yet), writes the
    /// file, and compares every pixel the layer did not touch against the source.
    ///
    /// Straight-alpha "over": the layer arrives un-premultiplied from the canvas encoder,
    /// and the source is opaque here, so the result is exact where alpha is 0 and a plain
    /// blend elsewhere. The semi-transparent source case is S0.6's own check 1.
    #[tauri::command]
    pub fn editor_export_layer(request: tauri::ipc::Request<'_>) -> Result<ExportReport, String> {
        let bytes = match request.body() {
            tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
            tauri::ipc::InvokeBody::Json(_) => {
                return Err("the layer arrived as JSON, not as bytes".into())
            }
        };
        let image = state().image().ok_or("no image is loaded")?;
        let (w, h) = (image.frame.width(), image.frame.height());

        let started = Instant::now();
        let layer = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .map_err(|err| format!("the layer did not decode: {err}"))?
            .into_rgba8();
        let decode_ms = started.elapsed().as_millis();
        if layer.width() != w || layer.height() != h {
            return Err(format!(
                "the layer is {}x{} and the image is {w}x{h}",
                layer.width(),
                layer.height()
            ));
        }

        let started = Instant::now();
        let src = &image.frame.rgba;
        let lay = layer.as_raw();
        let mut out = src.clone();
        let mut covered = 0usize;
        for i in 0..(w as usize * h as usize) {
            let a = lay[i * 4 + 3] as u32;
            if a == 0 {
                continue;
            }
            covered += 1;
            for c in 0..3 {
                let s = src[i * 4 + c] as u32;
                let l = lay[i * 4 + c] as u32;
                out[i * 4 + c] = ((l * a + s * (255 - a) + 127) / 255) as u8;
            }
            out[i * 4 + 3] = 255;
        }
        let composite_ms = started.elapsed().as_millis();

        let mut source_mismatches = 0usize;
        for i in 0..(w as usize * h as usize) {
            if lay[i * 4 + 3] == 0 && out[i * 4..i * 4 + 4] != src[i * 4..i * 4 + 4] {
                source_mismatches += 1;
            }
        }

        let started = Instant::now();
        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name("s06-export.png"))
            .unwrap_or_else(|_| "s06-export.png".into());
        image::RgbaImage::from_raw(w, h, out)
            .ok_or("the composite did not match its size")?
            .save(&path)
            .map_err(|err| err.to_string())?;
        let encode_ms = started.elapsed().as_millis();

        Ok(ExportReport {
            webview_version: tauri::webview_version().unwrap_or_else(|_| "unknown".into()),
            layer_bytes: bytes.len(),
            width: w,
            height: h,
            covered,
            source_mismatches,
            decode_ms,
            composite_ms,
            encode_ms,
            path: path.display().to_string(),
        })
    }

    /// Loads the synthetic probe as the current image, for the detail check.
    #[tauri::command]
    pub fn editor_load_probe(width: u32, height: u32) -> Result<ImageInfo, String> {
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err(format!("{width}x{height} is not a probe size"));
        }
        state().set_frame(detail_probe_frame(width, height));
        editor_image_info()
    }

    /// Loads a fresh screen capture as the current image, which is the real case.
    #[tauri::command]
    pub fn editor_load_screen() -> Result<ImageInfo, String> {
        let frame = screen::WholeVirtualScreen
            .freeze()
            .map_err(|err| err.to_string())?;
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
