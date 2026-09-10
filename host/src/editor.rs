//! S0.4: the editor window, the scene, and one callout.
//!
//! What the host owns here, and nothing more: the window, and the pixels the page is
//! allowed to see. Part 5 pins that the decoded original never crosses at full resolution
//! and never round-trips a canvas, so the page asks for a REGION at a SCALE and the host
//! resamples it. That is the display proxy, and the policy attached to it in part 5 is the
//! one this module has to keep: the visible region always has enough detail for the current
//! zoom, so an enlarged fit-to-window preview is never what a 100% view is made of.
//!
//! The scene, the callout and the text layer live in the page, because part 5 gives the
//! editor's layout and text to the web view. The host never renders an annotation.
//!
//! The window is created HIDDEN at startup and shown on demand, which is what the latency
//! budget needs and also what S0.4's own hidden-window check exists to distrust.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::capture::coords::DesktopRect;
use crate::capture::{screen, CaptureSource, Frame};

/// What the editor currently holds. One image, because S0.4 is one image.
pub struct Editor {
    frame: Mutex<Option<Arc<Frame>>>,
}

impl Editor {
    fn new() -> Self {
        Self {
            frame: Mutex::new(None),
        }
    }

    pub fn set_frame(&self, frame: Frame) {
        if let Ok(mut slot) = self.frame.lock() {
            *slot = Some(Arc::new(frame));
        }
    }

    fn frame(&self) -> Option<Arc<Frame>> {
        self.frame.lock().ok().and_then(|slot| slot.clone())
    }
}

pub fn state() -> &'static Editor {
    static STATE: OnceLock<Editor> = OnceLock::new();
    STATE.get_or_init(Editor::new)
}

/// A synthetic image whose whole purpose is to make resampling visible.
///
/// A one-pixel checkerboard survives a 1:1 view and turns to flat grey the moment anything
/// downscales it, so "the detail matches the zoom" becomes a property the page can assert on
/// the pixels rather than a claim someone squints at. The thin lines and the small blocks do
/// the same for scaling errors that are off by a fraction.
pub fn detail_probe_frame(width: u32, height: u32) -> Frame {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            // The left half is a one-pixel checkerboard, the right half thin verticals every
            // eight pixels, and a band of solid blocks across the middle for eyeballing.
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

/// Builds the editor window, hidden.
///
/// Hidden and pre-created is the whole point: showing an existing window is the only way the
/// second latency interval in S0.7 can be met. It is also the thing S0.4 has to distrust,
/// because a hidden window can be throttled by the browser engine and then show a blank or
/// stale first frame, which looks exactly like slowness.
#[allow(dead_code)] // the product startup path; S0.4 builds its own window for the checks
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

// ---------------------------------------------------------------- commands

/// Whether this run wants the page to run its own checks.
///
/// A query string was the first attempt and it does not work: an app URL is resolved as an
/// asset path, so "index.html?selftest=1" is simply not a file. The page asks instead.
static WANTS_CHECKS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn request_checks() {
    WANTS_CHECKS.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
pub fn editor_wants_checks() -> bool {
    WANTS_CHECKS.load(std::sync::atomic::Ordering::SeqCst)
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
#[derive(serde::Serialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub source: String,
}

#[tauri::command]
pub fn editor_image_info() -> Result<ImageInfo, String> {
    let frame = state().frame().ok_or("no image is loaded")?;
    Ok(ImageInfo {
        width: frame.width(),
        height: frame.height(),
        source: frame.source.to_string(),
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

#[tauri::command]
pub fn editor_show(app: AppHandle) -> Result<u128, String> {
    show(&app)
}

/// The result of the host looking at its own window on screen.
#[derive(serde::Serialize)]
pub struct WindowLook {
    /// Fraction of the sampled pixels that were the colour the page says it painted.
    pub matching: f64,
    /// The same fraction in a band just outside the reported window edge.
    pub outside_matching: f64,
    /// Fraction that were pure black, which is what a suspended or unpainted surface gives.
    pub black: f64,
    pub sampled: usize,
    pub rect: String,
}

/// Captures the editor window's own area of the screen and compares it against the colour
/// the page claims to have painted.
///
/// This is the only honest way to test the hidden-window failure: a suspended surface still
/// has a perfectly correct document behind it, so asking the page what it drew proves
/// nothing. Only the screen can say whether those pixels ever reached it.
#[tauri::command]
pub fn editor_look_at_window(app: AppHandle, r: u8, g: u8, b: u8) -> Result<WindowLook, String> {
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
    // larger than the size the runtime reports, that band is the same colour, which is how
    // a logical size masquerading as a physical one is caught.
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

    let frame = state().frame().ok_or("no image is loaded")?;
    let (fw, fh) = (frame.width() as i64, frame.height() as i64);
    if w <= 0 || h <= 0 || out_w <= 0 || out_h <= 0 {
        return Err("a region needs a size".into());
    }
    if out_w > 8192 || out_h > 8192 {
        return Err("that output size is larger than any display".into());
    }
    // Clamped to the image rather than refused: a viewport can legitimately hang over the
    // edge of an image, and the page is told what it actually got.
    let left = x.clamp(0, fw);
    let top = y.clamp(0, fh);
    let right = (x + w).clamp(0, fw);
    let bottom = (y + h).clamp(0, fh);
    if right <= left || bottom <= top {
        return Err("that region is outside the image".into());
    }

    let region = crate::capture::coords::ImageRect {
        x: left as u32,
        y: top as u32,
        width: (right - left) as u32,
        height: (bottom - top) as u32,
    };
    let cropped = frame.crop(region).ok_or("the crop was refused")?;

    // The output size the page asked for, scaled from the region it asked for. The ratio is
    // recomputed from the clamped region so a viewport hanging over the edge does not shift
    // what the page draws.
    let scale_w = out_w as f64 / w as f64;
    let scale_h = out_h as f64 / h as f64;
    let target_w = ((region.width as f64 * scale_w).round() as u32).max(1);
    let target_h = ((region.height as f64 * scale_h).round() as u32).max(1);

    if target_w == region.width && target_h == region.height {
        // The 1:1 case, which is what a 100% view must be: no resampling at all.
        return Ok((
            with_size_prefix(cropped, region.width, region.height),
            region.width,
            region.height,
        ));
    }

    let source = image::RgbaImage::from_raw(region.width, region.height, cropped)
        .ok_or("the region did not match its own dimensions")?;
    // Triangle rather than Lanczos: this is on the interactive path, and S0.7 measures it.
    let resized = image::imageops::resize(
        &source,
        target_w,
        target_h,
        image::imageops::FilterType::Triangle,
    );
    Ok((
        with_size_prefix(resized.into_raw(), target_w, target_h),
        target_w,
        target_h,
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

/// Shows the window and looks at the screen twice: once almost immediately, once settled.
///
/// Two samples because the failure this exists to catch is a first presented frame that is
/// blank or stale. One late sample would miss it, and one early sample alone could not tell
/// a suspended surface from a window that simply had not been asked to paint yet.
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
