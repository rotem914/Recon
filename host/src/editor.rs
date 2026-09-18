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
    /// The folder the opened file came from, listed once, for previous and next (S1.4).
    folder: Mutex<Option<crate::folder::Context>>,
    /// Which list previous and next walk (§3.4): Recon history when true, the folder
    /// otherwise. A capture, or a document shown from history, sets it; opening a file
    /// clears it; Annotate never touches it.
    history_active: std::sync::atomic::AtomicBool,
}

impl Editor {
    fn new() -> Self {
        Self {
            image: Mutex::new(None),
            document: Mutex::new(None),
            folder: Mutex::new(None),
            history_active: std::sync::atomic::AtomicBool::new(false),
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
/// The last number issued, so two in one millisecond still differ and a number never
/// repeats in a session. Since S1.8 a number is the document's creation time in
/// milliseconds, so the numbers of documents read back from disk never collide with new
/// ones, and the page keys its notes by the same number before and after a restart.
static LAST_ID: AtomicU64 = AtomicU64::new(0);

fn next_document_id() -> u64 {
    let now = crate::store::millis(std::time::SystemTime::now());
    let id = loop {
        let last = LAST_ID.load(Ordering::SeqCst);
        let next = now.max(last + 1);
        if LAST_ID
            .compare_exchange(last, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            break next;
        }
    };
    CURRENT_ID.store(id, Ordering::SeqCst);
    id
}

pub fn current_document_id() -> u64 {
    CURRENT_ID.load(Ordering::SeqCst)
}

/// Where a managed document's image came from (§3.8). A capture's is the screen. An
/// annotated file's is the path and the frame or page that was on screen when Annotate
/// was pressed, with the file's size and stamp as they were then, so a resume can say the
/// source has moved on without reading it again.
#[derive(Clone, Debug)]
pub enum Source {
    Capture,
    File {
        path: std::path::PathBuf,
        frame: u32,
        modified: Option<std::time::SystemTime>,
        len: u64,
    },
}

/// The preserved image: the decoded pixels until the encode thread is done, then the PNG,
/// so a document costs a few megabytes rather than tens. The entry exists from the first
/// moment either way, so a second Annotate can never find nothing and create a second one.
#[derive(Clone)]
enum Preserved {
    Decoded(Arc<Image>),
    Encoded(Vec<u8>),
    /// Written to the store as source.png, once; read back when the document is shown.
    OnDisk(std::path::PathBuf),
}

/// A managed document, kept in memory until the store exists (S1.8). "Taking a new
/// capture saves the current document and adds another; it never overwrites the previous
/// one" (§3.3), and "annotation is what creates a managed document, and only for the file
/// being annotated" (§3.8). Captured pixels arrive here when the next capture replaces
/// them; an annotated file's arrive at Annotate, preserved as decoded, and the external
/// file is never read again for that document. The list is capped so a day of use cannot
/// grow without bound. The page keeps the notes of each document by the same number.
pub struct Managed {
    pub id: u64,
    pub source: Source,
    pub width: u32,
    pub height: u32,
    preserved: Preserved,
    pub created: std::time::SystemTime,
    /// When annotation last touched it: created, resumed, or saved.
    pub annotated: std::time::SystemTime,
    /// The page's notes as last saved or read back, carried whole (part 5).
    pub notes: serde_json::Value,
    /// What the last write to disk said, so a failure stays visible until a save succeeds.
    pub save_error: Option<String>,
}

static DOCUMENTS: Mutex<Vec<Managed>> = Mutex::new(Vec::new());

// ---------------------------------------------------------------- the opened files

/// A file the user opened, listed in the timeline beside the documents as a pointer to
/// where it lives (Rotem, 2026-09-18). Nothing of the file is kept (rule 11): no pixels,
/// and its thumbnail is made from the file when asked for and held in memory only. A file
/// that is gone leaves the list; Annotate turns the pointer into a managed document, which
/// preserves the image as it always did. Only a file opened by name joins, never one
/// stepped onto in its folder.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Link {
    id: u64,
    path: std::path::PathBuf,
    width: u32,
    height: u32,
    created_ms: u64,
}

static LINKS: Mutex<Vec<Link>> = Mutex::new(Vec::new());
/// Pointers removed from the timeline in this session, so Ctrl+Z can put one back.
static REMOVED_LINKS: Mutex<Vec<Link>> = Mutex::new(Vec::new());
static LINK_THUMBS: Mutex<Option<std::collections::HashMap<u64, Vec<u8>>>> = Mutex::new(None);

#[derive(serde::Serialize, serde::Deserialize)]
struct LinksFile {
    schema: u32,
    files: Vec<Link>,
}

/// Writes the list whole, through a temporary file and a rename (QA §5). Called with the
/// list's lock held, so two writers never interleave.
fn save_links(links: &[Link]) {
    let result = crate::store::opened_path().and_then(|path| {
        let text = serde_json::to_vec_pretty(&LinksFile {
            schema: 1,
            files: links.to_vec(),
        })
        .map_err(|err| err.to_string())?;
        crate::store::write_atomic(&path, &text)
    });
    if let Err(err) = result {
        crate::log(&format!("the opened files NOT saved: {err}"));
    }
}

/// Reads the list at startup. A file that does not parse is set aside under a dated name,
/// as the tabs are, and the list starts empty.
fn load_links() {
    let Ok(path) = crate::store::opened_path() else {
        return;
    };
    let files = match std::fs::read(&path) {
        Ok(text) => match serde_json::from_slice::<LinksFile>(&text) {
            Ok(file) => file.files,
            Err(err) => {
                let aside = path.with_extension(format!(
                    "broken-{}.json",
                    crate::store::millis(std::time::SystemTime::now())
                ));
                let moved = std::fs::rename(&path, &aside).is_ok();
                crate::log(&format!(
                    "{}: {err}; set aside as {}: {}",
                    path.display(),
                    aside.display(),
                    if moved { "moved" } else { "NOT moved" }
                ));
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };
    if let Ok(mut links) = LINKS.lock() {
        *links = files;
    }
    if let Ok(mut removed) = REMOVED_LINKS.lock() {
        removed.clear();
    }
    if let Ok(mut thumbs) = LINK_THUMBS.lock() {
        *thumbs = None;
    }
}

fn link(id: u64) -> Option<Link> {
    LINKS
        .lock()
        .ok()
        .and_then(|links| links.iter().find(|l| l.id == id).cloned())
}

/// Drops the pointers a managed document replaces: the one with its number, and any to
/// the same file. Called when Annotate preserves the image.
fn drop_links_for(id: u64, path: Option<&std::path::Path>) {
    let Ok(mut links) = LINKS.lock() else {
        return;
    };
    let before = links.len();
    links.retain(|l| l.id != id && !path.is_some_and(|p| same_path(&l.path, p)));
    if links.len() != before {
        save_links(&links);
    }
}

/// Puts a pointer on screen: the file is opened again from where it lives, under the
/// pointer's own number, with history navigation as for any thumbnail chosen (§3.4). A
/// file that is gone leaves the list.
fn show_link(link: &Link) -> Result<(), String> {
    let opened = source::open(&link.path).and_then(|mut opened| {
        let decoded = opened.frame(0)?;
        Ok((opened, decoded))
    });
    let (opened, decoded) = match opened {
        Ok(pair) => pair,
        Err(err) => {
            if !link.path.is_file() {
                if let Ok(mut links) = LINKS.lock() {
                    links.retain(|l| l.id != link.id);
                    save_links(&links);
                }
            }
            return Err(format!("{}: {err}", link.path.display()));
        }
    };
    let name = opened.format.name();
    state().set_document(Some(Document { opened, index: 0 }));
    if let Ok(mut slot) = state().folder.lock() {
        *slot = None;
    }
    state().history_active.store(true, Ordering::SeqCst);
    CURRENT_ID.store(link.id, Ordering::SeqCst);
    state().set_frame(frame_of(decoded, name));
    if let Ok(app) = app() {
        if let Ok(info) = editor_image_info() {
            set_title(app, &info);
        }
    }
    Ok(())
}

/// A pointer's thumbnail: made from the file itself, held in memory, never written.
fn link_thumbnail(link: &Link) -> Result<Vec<u8>, String> {
    if let Ok(thumbs) = LINK_THUMBS.lock() {
        if let Some(png) = thumbs.as_ref().and_then(|t| t.get(&link.id)) {
            return Ok(png.clone());
        }
    }
    let mut opened = source::open(&link.path).map_err(|err| err.to_string())?;
    let frame = opened.frame(0).map_err(|err| err.to_string())?;
    let (tw, th) = thumb_size(frame.width, frame.height);
    let small = recon_pixels::resample(&frame.rgba, frame.width, frame.height, tw, th)
        .ok_or_else(|| format!("{}: the thumbnail did not resample", link.path.display()))?;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&small, tw, th, image::ExtendedColorType::Rgba8)
        .map_err(|err| {
            format!(
                "{}: the thumbnail did not encode: {err}",
                link.path.display()
            )
        })?;
    if let Ok(mut thumbs) = LINK_THUMBS.lock() {
        thumbs
            .get_or_insert_with(std::collections::HashMap::new)
            .insert(link.id, png.clone());
    }
    Ok(png)
}

fn record_of(document: &Managed) -> crate::store::Record {
    crate::store::Record {
        schema: crate::store::SCHEMA,
        id: document.id,
        created_ms: crate::store::millis(document.created),
        modified_ms: crate::store::millis(document.annotated),
        width: document.width,
        height: document.height,
        source: match &document.source {
            Source::Capture => crate::store::SourceRecord::Capture,
            Source::File {
                path,
                frame,
                modified,
                len,
            } => crate::store::SourceRecord::File {
                path: path.display().to_string(),
                frame: *frame,
                modified_ms: modified.map(crate::store::millis),
                len: *len,
            },
        },
        notes: document.notes.clone(),
    }
}

/// Writes one document's record, under the lock the caller holds, and remembers the result.
fn write_record_of(document: &mut Managed) -> Result<(), String> {
    let result = crate::store::write_record(&record_of(document));
    document.save_error = result.as_ref().err().cloned();
    result
}

/// Files the document under its number at once, decoded, then encodes it on its own thread
/// and swaps the PNG in, so the capture path and the Annotate key pay nothing for it.
fn preserve(image: Arc<Image>, id: u64, source: Source) {
    let (width, height) = (image.frame.width(), image.frame.height());
    // The document takes the place of the file's pointer in the timeline.
    drop_links_for(
        id,
        match &source {
            Source::File { path, .. } => Some(path.as_path()),
            Source::Capture => None,
        },
    );
    if let Ok(mut documents) = DOCUMENTS.lock() {
        if documents.iter().any(|d| d.id == id) {
            return;
        }
        let now = std::time::SystemTime::now();
        documents.push(Managed {
            id,
            source,
            width,
            height,
            preserved: Preserved::Decoded(image.clone()),
            created: now,
            annotated: now,
            notes: serde_json::Value::Null,
            save_error: None,
        });
        // The record first, so the document exists on disk from its first moment; the
        // image follows from the thread below.
        if let Some(document) = documents.iter_mut().find(|d| d.id == id) {
            if let Err(err) = write_record_of(document) {
                crate::log(&format!("document {id} NOT saved: {err}"));
            }
        }
    }
    std::thread::spawn(move || {
        let started = Instant::now();
        let mut png = Vec::new();
        let encoded = image::codecs::png::PngEncoder::new(&mut png).write_image(
            &image.frame.rgba,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        );
        if let Err(err) = encoded {
            println!("document {id} kept decoded: the PNG did not encode: {err}");
            return;
        }
        let encode_ms = started.elapsed().as_millis();
        // To disk, once, under the list's lock (review T2): a delete moves the folder under
        // the same lock, so the image lands where the record is. A document deleted while
        // it encoded gets its image in the trash, where Restore expects it, and never a
        // folder of its own among the documents again.
        let Ok(mut documents) = DOCUMENTS.lock() else {
            return;
        };
        let count = documents.len();
        let Some(document) = documents.iter_mut().find(|d| d.id == id) else {
            // Wherever its folder is now: among the documents while an undo has it back and
            // waits for this image (review 4, T2), or in the trash. Written beside what is
            // there, never into a folder made for it (review 4, T3).
            let folder = crate::store::folder(id)
                .ok()
                .filter(|dir| dir.is_dir())
                .or_else(|| {
                    crate::store::trash_root()
                        .ok()
                        .map(|trash| trash.join(id.to_string()))
                        .filter(|dir| dir.is_dir())
                });
            match folder {
                Some(dir) => match crate::store::write_beside(&dir.join("source.png"), &png) {
                    Ok(()) => crate::log(&format!(
                        "document {id} was deleted while its image encoded; the image joined its folder at {}",
                        dir.display()
                    )),
                    Err(err) => crate::log(&format!(
                        "document {id} was deleted while its image encoded, and the image was NOT kept: {err}"
                    )),
                },
                None => crate::log(&format!(
                    "document {id} is gone from the list with no folder anywhere; its image is dropped"
                )),
            }
            return;
        };
        match crate::store::write_source(id, &png) {
            Ok(path) => {
                document.preserved = Preserved::OnDisk(path);
                println!(
                    "document {id} preserved: {width}x{height} encoded in {encode_ms} ms, written in {} ms, {count} documents",
                    started.elapsed().as_millis() - encode_ms,
                );
            }
            Err(err) => {
                document.preserved = Preserved::Encoded(png);
                document.save_error = Some(err.clone());
                crate::log(&format!(
                    "document {id} image NOT saved, kept in memory: {err}"
                ));
            }
        }
    });
}

impl Managed {
    /// The PNG's size, or zero while the encode is still running; the checks' history line.
    #[cfg(feature = "stage0-checks")]
    fn bytes(&self) -> usize {
        match &self.preserved {
            Preserved::Encoded(png) => png.len(),
            Preserved::OnDisk(path) => std::fs::metadata(path)
                .map(|m| m.len() as usize)
                .unwrap_or(0),
            Preserved::Decoded(_) => 0,
        }
    }

    /// What `preserved_frame` needs, cloned, so a caller can decode outside the list's
    /// lock (review T7, and review 4's T6 for the last two callers): the handle is cheap,
    /// the decode is not. This is the document's own image; the file on disk is not read.
    fn preserved_handle(&self) -> (u64, Preserved, u32, u32) {
        (self.id, self.preserved.clone(), self.width, self.height)
    }
}

/// The frame behind a preserved image, decoded here; see `Managed::frame`.
fn preserved_frame(
    id: u64,
    preserved: &Preserved,
    width: u32,
    height: u32,
    name: &'static str,
) -> Result<Frame, String> {
    let decode = |png: &[u8]| {
        image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .map_err(|err| format!("document {id}: its preserved PNG did not decode: {err}"))
            .map(|img| img.into_rgba8().into_raw())
    };
    let rgba = match preserved {
        Preserved::Decoded(image) => image.frame.rgba.clone(),
        Preserved::Encoded(png) => decode(png)?,
        Preserved::OnDisk(path) => {
            let png = std::fs::read(path).map_err(|err| {
                format!("document {id}: {} could not be read: {err}", path.display())
            })?;
            decode(&png)?
        }
    };
    Ok(Frame {
        geometry: FrameGeometry {
            origin_x: 0,
            origin_y: 0,
            width,
            height,
        },
        rgba,
        source: name,
    })
}

impl Managed {
    fn is_for(&self, path: &std::path::Path, frame: u32) -> bool {
        match &self.source {
            Source::File {
                path: own,
                frame: own_frame,
                ..
            } => *own_frame == frame && same_path(own, path),
            Source::Capture => false,
        }
    }
}

/// The same file under Windows' case-insensitive names, as the folder context compares.
fn same_path(a: &std::path::Path, b: &std::path::Path) -> bool {
    a.to_string_lossy()
        .eq_ignore_ascii_case(&b.to_string_lossy())
}

/// The file's stamp and size, for the "source has moved on" line; None when it is gone.
fn file_stamp(path: &std::path::Path) -> (Option<std::time::SystemTime>, u64) {
    match std::fs::metadata(path) {
        Ok(meta) => (meta.modified().ok(), meta.len()),
        Err(_) => (None, 0),
    }
}

/// The documents, for the checks and, at S1.9, for navigation: number, size, PNG bytes.
#[cfg(feature = "stage0-checks")]
pub fn history_summary() -> Vec<(u64, u32, u32, usize)> {
    DOCUMENTS
        .lock()
        .map(|h| {
            h.iter()
                .map(|d| (d.id, d.width, d.height, d.bytes()))
                .collect()
        })
        .unwrap_or_default()
}

/// How many of the newest records the startup reads before the window is shown (S2.8).
/// The last-modified one among them reopens at once; the rest of the store follows on a
/// thread, so a startup costs the same at thirty thousand documents as at fifty.
const EAGER_READ: usize = 50;

/// Which startup read is current: a reset or a reload bumps it, and a thread from before
/// stops pushing into a list that is no longer its own.
static STORE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Whether the whole store is in the list. What must see every document, the one-
/// document-per-file lookup at Annotate (§3.3), waits on it; the timeline and the history
/// keys show what has arrived and are told when the rest has.
static STORE_LOADED: (Mutex<bool>, Condvar) = (Mutex::new(true), Condvar::new());

fn set_store_loaded(loaded: bool) {
    if let Ok(mut flag) = STORE_LOADED.0.lock() {
        *flag = loaded;
        STORE_LOADED.1.notify_all();
    }
}

/// Blocks until the startup read has the whole store in the list; at once when it has.
pub fn wait_store_loaded() {
    if let Ok(mut flag) = STORE_LOADED.0.lock() {
        while !*flag {
            flag = match STORE_LOADED.1.wait(flag) {
                Ok(flag) => flag,
                Err(_) => return,
            };
        }
    }
}

/// Puts records into the list, those already there left alone, and says how many went in.
fn add_records(found: Vec<(crate::store::Record, std::path::PathBuf)>) -> usize {
    let Ok(mut documents) = DOCUMENTS.lock() else {
        return 0;
    };
    let mut added = 0;
    for (record, source) in found {
        if documents.iter().any(|d| d.id == record.id) {
            continue;
        }
        documents.push(managed_from_record(record, source));
        added += 1;
    }
    added
}

/// Reads the store into the list and reopens the latest document (§3.8: after a restart,
/// the latest document reopens). Since S2.8 the folder names are listed once, the newest
/// `EAGER_READ` records are read here, and the last-modified of them is shown before this
/// returns; every older record is read on a thread, newest first, and the page is told
/// with `store-loaded` when the list is whole. The image is not decoded until it is shown.
pub fn load_store() {
    load_links();
    let generation = STORE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let ids = crate::store::folder_ids();
    let count = ids.len();
    let (rest, head) = ids.split_at(count.saturating_sub(EAGER_READ));
    let head: Vec<(crate::store::Record, std::path::PathBuf)> = head
        .iter()
        .rev()
        .filter_map(|id| crate::store::read_one(*id))
        .collect();
    let latest = head
        .iter()
        .max_by_key(|(record, _)| (record.modified_ms, record.id))
        .map(|(record, _)| record.id);
    let read = add_records(head);
    set_store_loaded(rest.is_empty());
    if let Some(id) = latest {
        match show_document(id) {
            Ok(()) => crate::log(&format!("store: document {id} reopened")),
            Err(err) => crate::log(&format!("store: document {id} NOT reopened: {err}")),
        }
    }
    if rest.is_empty() {
        crate::log(&format!("store: {read} documents read"));
        return;
    }
    crate::log(&format!(
        "store: {read} of {count} documents read, the rest on a thread"
    ));
    let rest: Vec<u64> = rest.iter().rev().copied().collect();
    std::thread::spawn(move || {
        // Whatever ends this thread, a panic included, the wait in editor_annotate is
        // released: a list that stopped short is shown short, never waited on forever.
        struct Whole(u64);
        impl Drop for Whole {
            fn drop(&mut self) {
                if STORE_GENERATION.load(Ordering::SeqCst) == self.0 {
                    set_store_loaded(true);
                }
            }
        }
        let _whole = Whole(generation);
        let started = Instant::now();
        let mut read = 0;
        for chunk in rest.chunks(256) {
            if STORE_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            let found = chunk
                .iter()
                .filter_map(|id| crate::store::read_one(*id))
                .collect();
            read += add_records(found);
        }
        if STORE_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        set_store_loaded(true);
        crate::log(&format!(
            "store: {read} more documents read in {} ms, the list is whole",
            started.elapsed().as_millis()
        ));
        if let Ok(app) = app() {
            let _ = app.emit("store-loaded", ());
        }
    });
}

/// A record read from disk as the list holds it; the number counter moves past it.
fn managed_from_record(record: crate::store::Record, source: std::path::PathBuf) -> Managed {
    LAST_ID.fetch_max(record.id, Ordering::SeqCst);
    Managed {
        id: record.id,
        source: match record.source {
            crate::store::SourceRecord::Capture => Source::Capture,
            crate::store::SourceRecord::File {
                path,
                frame,
                modified_ms,
                len,
            } => Source::File {
                path: std::path::PathBuf::from(path),
                frame,
                modified: modified_ms.map(crate::store::from_millis),
                len,
            },
        },
        width: record.width,
        height: record.height,
        preserved: Preserved::OnDisk(source),
        created: crate::store::from_millis(record.created_ms),
        annotated: crate::store::from_millis(record.modified_ms),
        notes: record.notes,
        save_error: None,
    }
}

/// The trash as the timeline shows it (S2.7): each trashed document with its days there.
#[derive(serde::Serialize)]
pub struct TrashedLine {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub file: String,
    pub days: u64,
}

#[tauri::command]
pub fn editor_trash_documents() -> Vec<TrashedLine> {
    let Ok(trash) = crate::store::trash_root() else {
        return Vec::new();
    };
    let mut lines: Vec<TrashedLine> = crate::store::trash_list()
        .into_iter()
        .filter_map(|(id, days)| {
            let record = crate::store::read_record(&trash.join(id.to_string())).ok()?;
            Some(TrashedLine {
                id,
                width: record.width,
                height: record.height,
                file: match record.source {
                    crate::store::SourceRecord::File { path, .. } => std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    crate::store::SourceRecord::Capture => String::new(),
                },
                days,
            })
        })
        .collect();
    lines.sort_by_key(|l| l.id);
    lines
}

/// A trashed document brought back (S2.7): its folder moves back the way it went, it
/// rejoins the list, and it is shown.
#[tauri::command]
pub fn editor_trash_restore(id: u64) -> Result<ImageInfo, String> {
    trash_rejoin(id)?;
    show_document(id)?;
    editor_image_info()
}

/// The first half of a restore, on its own (S2.9): the folder back out of the trash and
/// the document back in the list, nothing shown. Undo of a deletion uses it, so a document
/// that was not on screen when deleted returns to the timeline without taking the screen.
#[tauri::command]
pub fn editor_trash_rejoin(id: u64) -> Result<(), String> {
    trash_rejoin(id)
}

/// How long a restore waits for an image still being encoded (review 4, T2): a capture
/// deleted and brought back inside its own encode has its record but not yet its PNG, and
/// the thread writes it into the folder wherever it is.
const REJOIN_WAIT: std::time::Duration = std::time::Duration::from_millis(1500);

fn trash_rejoin(id: u64) -> Result<(), String> {
    // A pointer taken off the timeline in this session goes back as it was.
    let removed = REMOVED_LINKS.lock().ok().and_then(|mut removed| {
        let at = removed.iter().position(|l| l.id == id)?;
        Some(removed.remove(at))
    });
    if let Some(link) = removed {
        if let Ok(mut links) = LINKS.lock() {
            links.push(link);
            save_links(&links);
        }
        return Ok(());
    }
    let dir = crate::store::restore(id)?;
    // A folder that cannot rejoin goes back into the trash before the error is returned,
    // so the trash entry survives and a Restore later still finds it (review 4, T2).
    let back = |err: String| -> String {
        match crate::store::trash(id) {
            Ok(_) => err,
            Err(undo) => format!("{err}; and it could not go back to the trash: {undo}"),
        }
    };
    let record = match crate::store::read_record(&dir) {
        Ok(record) => record,
        Err(err) => return Err(back(err)),
    };
    let source = dir.join("source.png");
    let started = Instant::now();
    while !source.is_file() && started.elapsed() < REJOIN_WAIT {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !source.is_file() {
        return Err(back(format!("document {id} came back without its image")));
    }
    {
        let mut documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
        if !documents.iter().any(|d| d.id == id) {
            documents.push(managed_from_record(record, source));
        }
    }
    crate::log(&format!("document {id} restored from the trash"));
    Ok(())
}

/// Puts a managed document on screen from its own preserved image, as the resume of
/// Annotate does, without a file behind it: the document stands on its own (§3.3).
fn show_document(id: u64) -> Result<(), String> {
    if let Some(link) = link(id) {
        return show_link(&link);
    }
    // The handle under the lock, the decode outside it (review T7): a large PNG takes a
    // tenth of a second to decode, and the timeline, a thumbnail and a save in flight
    // would all wait on it.
    let (handle, name) = {
        let documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
        let document = documents
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("document {id} is not in the list"))?;
        (
            document.preserved_handle(),
            match document.source {
                Source::Capture => "capture",
                Source::File { .. } => "document",
            },
        )
    };
    let (id, preserved, width, height) = handle;
    let frame = preserved_frame(id, &preserved, width, height, name)?;
    state().set_document(None);
    if let Ok(mut slot) = state().folder.lock() {
        *slot = None;
    }
    // Selecting a document from Recon history activates history navigation (§3.4).
    state().history_active.store(true, Ordering::SeqCst);
    CURRENT_ID.store(id, Ordering::SeqCst);
    state().set_frame(frame);
    if let Ok(app) = app() {
        if let Ok(info) = editor_image_info() {
            set_title(app, &info);
        }
    }
    Ok(())
}

/// The page's notes for one document, saved: the record is rewritten whole, atomically. A
/// failure is returned to the page, which shows it and keeps the work; the document keeps
/// the notes in memory either way, so the next save carries them.
#[tauri::command]
pub fn editor_save_notes(document_id: u64, notes: serde_json::Value) -> Result<(), String> {
    let mut documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
    let document = documents
        .iter_mut()
        .find(|d| d.id == document_id)
        .ok_or_else(|| {
            format!("document {document_id} is not managed, so there is nothing to save into")
        })?;
    document.notes = notes;
    document.annotated = std::time::SystemTime::now();
    let result = write_record_of(document);
    if let Err(err) = &result {
        crate::log(&format!("document {document_id} NOT saved: {err}"));
    }
    result
}

/// The notes as the host holds them, for a page that has no copy of its own: after a
/// restart, or for a document read from disk.
#[tauri::command]
pub fn editor_notes(document_id: u64) -> Option<serde_json::Value> {
    DOCUMENTS.lock().ok().and_then(|documents| {
        documents
            .iter()
            .find(|d| d.id == document_id)
            .map(|d| d.notes.clone())
    })
}

/// The timeline's tabs as the page last saved them (Rotem, 2026-09-16): the page's own
/// object, kept whole in `tabs.json` beside the documents and never interpreted here, like a
/// document's notes. None when no tab was ever made. A file that does not parse is set aside
/// under a dated name and answered as none, so the page starts with Main alone and the next
/// save never writes over what was there.
#[tauri::command]
pub fn editor_tabs() -> Option<serde_json::Value> {
    let path = crate::store::tabs_path().ok()?;
    let text = std::fs::read(&path).ok()?;
    match serde_json::from_slice(&text) {
        Ok(value) => Some(value),
        Err(err) => {
            let aside = path.with_extension(format!(
                "broken-{}.json",
                crate::store::millis(std::time::SystemTime::now())
            ));
            let moved = std::fs::rename(&path, &aside);
            crate::log(&format!(
                "{}: {err}; set aside as {}: {}",
                path.display(),
                aside.display(),
                match moved {
                    Ok(()) => "moved".to_string(),
                    Err(err) => format!("NOT moved, {err}"),
                }
            ));
            None
        }
    }
}

/// Writes the tabs whole, through a temporary file and a rename like every record (QA §5).
#[tauri::command]
pub fn editor_save_tabs(tabs: serde_json::Value) -> Result<(), String> {
    let path = crate::store::tabs_path()?;
    let text = serde_json::to_vec_pretty(&tabs).map_err(|err| err.to_string())?;
    let result = crate::store::write_atomic(&path, &text);
    if let Err(err) = &result {
        crate::log(&format!("tabs NOT saved: {err}"));
    }
    result
}

/// The page's answer to "save now": it has saved what was pending.
static FLUSHES: AtomicU64 = AtomicU64::new(0);

/// The page has saved what was pending. If a quit is waiting on that, it goes now.
#[tauri::command]
pub fn editor_saves_flushed(app: AppHandle) {
    FLUSHES.fetch_add(1, Ordering::SeqCst);
    if QUIT_PENDING.swap(false, Ordering::SeqCst) {
        crate::log("quit: the page saved, exiting");
        app.exit(0);
    }
}

static QUIT_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Asks the page to save what is pending, without waiting: the page's commands run on the
/// thread this is called from, so a wait here would only hold them up (the two seconds a
/// close took on 2026-09-14). A close hides at once and the save lands behind it.
pub fn ask_to_save(app: &AppHandle) {
    let _ = app.emit("save-now", ());
}

/// Quit after the page has saved (§3.8): asked now, exited when the page answers, or after
/// a second and a half if it does not, so a stuck page can never hold Recon open.
pub fn quit_after_saves(app: &AppHandle) {
    QUIT_PENDING.store(true, Ordering::SeqCst);
    if app.emit("save-now", ()).is_err() {
        app.exit(0);
        return;
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        if QUIT_PENDING.swap(false, Ordering::SeqCst) {
            crate::log("quit: the page did not answer within a second and a half, exiting");
            handle.exit(0);
        }
    });
}

/// The timeline's thumbnail of one document (S2.1): at most 320 by 200 since S2.8 (160 by
/// 100 before it, when the strip could not grow), made once from the document's own image
/// and kept as `thumb.png` beside it, so old history costs a small file each and never a
/// decoded image in memory. A document whose folder is not on disk yet is thumbnailed from
/// what it holds and kept in memory only.
const THUMB_W: u32 = 320;
const THUMB_H: u32 = 200;

/// A thumbnail kept before S2.8 fits 160 by 100 and is blurry in a grown strip. It is
/// outgrown when the document holds more pixels than it and it does not touch the box a
/// thumbnail is made in now; then it is made again, once. A thumbnail the size of its
/// document, or one that already touches the box, is never remade.
fn outgrown(png: &[u8], id: u64) -> bool {
    let Some((w, h)) = image::ImageReader::new(std::io::Cursor::new(png))
        .with_guessed_format()
        .ok()
        .and_then(|reader| reader.into_dimensions().ok())
    else {
        return false;
    };
    let Some((dw, dh)) = DOCUMENTS.lock().ok().and_then(|documents| {
        documents
            .iter()
            .find(|d| d.id == id)
            .map(|d| (d.width, d.height))
    }) else {
        return false;
    };
    outgrown_by((w, h), (dw, dh))
}

/// The rule of `outgrown`, on sizes alone: the kept thumbnail's, then the document's.
fn outgrown_by((w, h): (u32, u32), (dw, dh): (u32, u32)) -> bool {
    (w < dw || h < dh) && w < THUMB_W && h < THUMB_H
}

#[cfg(test)]
mod thumbnail_tests {
    use super::outgrown_by;

    #[test]
    fn a_thumbnail_from_before_s2_8_is_remade_once_and_a_current_one_never() {
        // Kept at 160 by 100 from a 4K capture: remade.
        assert!(outgrown_by((160, 90), (3840, 2160)));
        // Made at 320 by 200 from the same: it touches the box, kept.
        assert!(!outgrown_by((320, 180), (3840, 2160)));
        // A tall picture kept at 25 by 100, then at 50 by 200: remade, then kept.
        assert!(outgrown_by((25, 100), (500, 2000)));
        assert!(!outgrown_by((50, 200), (500, 2000)));
        // A picture smaller than the box is its own thumbnail: kept.
        assert!(!outgrown_by((100, 60), (100, 60)));
    }
}

fn thumbnail(id: u64) -> Result<Vec<u8>, String> {
    if let Some(link) = link(id) {
        return link_thumbnail(&link);
    }
    let cached = crate::store::folder(id)
        .ok()
        .map(|dir| dir.join("thumb.png"))
        .filter(|path| path.is_file())
        .or_else(|| {
            crate::store::trash_root()
                .ok()
                .map(|trash| trash.join(id.to_string()).join("thumb.png"))
                .filter(|path| path.is_file())
        });
    if let Some(path) = cached {
        let png = std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        if !outgrown(&png, id) {
            return Ok(png);
        }
    }
    // The handle under the lock, the decode outside it (review 4, T6).
    let (hid, preserved, width, height) = {
        let documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
        let document = documents
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("document {id} is not in the list"))?;
        document.preserved_handle()
    };
    let frame = preserved_frame(hid, &preserved, width, height, "thumbnail")?;
    let (w, h) = (frame.width(), frame.height());
    let (tw, th) = thumb_size(w, h);
    let small = recon_pixels::resample(&frame.rgba, w, h, tw, th)
        .ok_or_else(|| format!("document {id}: the thumbnail did not resample"))?;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&small, tw, th, image::ExtendedColorType::Rgba8)
        .map_err(|err| format!("document {id}: the thumbnail did not encode: {err}"))?;
    // Beside the document's files, never into a folder made for it: a delete that moved the
    // folder meanwhile must not find it recreated with a thumbnail alone (review 4, T3).
    if let Ok(dir) = crate::store::folder(id) {
        if dir.is_dir() {
            if let Err(err) = crate::store::write_beside(&dir.join("thumb.png"), &png) {
                crate::log(&format!("document {id}: thumbnail not kept: {err}"));
            }
        }
    }
    Ok(png)
}

/// One document as the timeline lists it.
#[derive(serde::Serialize)]
pub struct DocumentLine {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    /// The file's name for an annotated file, empty for a capture.
    pub file: String,
    pub created_ms: u64,
    pub current: bool,
    /// A pointer to an opened file, not a document: removing it keeps nothing in the trash
    /// and never touches the file.
    pub linked: bool,
}

/// Every document and every opened file's pointer, oldest first, the one on screen marked
/// (S2.1). A pointer whose file is gone leaves the list here.
#[tauri::command]
pub fn editor_documents() -> Vec<DocumentLine> {
    let current = current_document_id();
    let links: Vec<Link> = match LINKS.lock() {
        Ok(mut links) => {
            let before = links.len();
            links.retain(|l| l.path.is_file());
            if links.len() != before {
                save_links(&links);
            }
            links.clone()
        }
        Err(_) => Vec::new(),
    };
    let ids = history_ids();
    let documents = match DOCUMENTS.lock() {
        Ok(documents) => documents,
        Err(_) => return Vec::new(),
    };
    let name = |path: &std::path::Path| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    ids.iter()
        .filter_map(|id| {
            if let Some(d) = documents.iter().find(|d| d.id == *id) {
                return Some(DocumentLine {
                    id: d.id,
                    width: d.width,
                    height: d.height,
                    file: match &d.source {
                        Source::File { path, .. } => name(path),
                        Source::Capture => String::new(),
                    },
                    created_ms: crate::store::millis(d.created),
                    current: d.id == current,
                    linked: false,
                });
            }
            links.iter().find(|l| l.id == *id).map(|l| DocumentLine {
                id: l.id,
                width: l.width,
                height: l.height,
                file: name(&l.path),
                created_ms: l.created_ms,
                current: l.id == current,
                linked: true,
            })
        })
        .collect()
}

/// What the store holds, for the HUD (§3.8: visible storage usage): the documents and the
/// bytes of every file under them, read from the file sizes, never from the images.
#[derive(serde::Serialize)]
pub struct Storage {
    pub documents: u32,
    pub bytes: u64,
}

#[tauri::command]
pub fn editor_storage() -> Storage {
    // From the store's ledger (S2.8), never a walk: a folder is measured when it is read
    // at startup and again by whatever writes, moves or removes it.
    let (documents, bytes) = crate::store::storage();
    Storage { documents, bytes }
}

/// Deletes a document (§3.8, S2.5): out of the list, its folder into Recon's trash for
/// thirty days, never an external original and never an export. If it was the one on
/// screen, the newer neighbour shows, else the older, else nothing, and the page gets the
/// image info or none. Returns what was moved, for the log and the notice.
#[tauri::command]
pub fn editor_delete_document(id: u64) -> Result<Option<ImageInfo>, String> {
    let ids = history_ids();
    let at = ids.iter().position(|d| *d == id);
    let was_current = id == current_document_id();
    // A pointer to an opened file: off the list, kept for this session's Ctrl+Z, and the
    // file itself is never touched (rule 11).
    let pointer = LINKS.lock().ok().and_then(|mut links| {
        let at = links.iter().position(|l| l.id == id)?;
        let link = links.remove(at);
        save_links(&links);
        Some(link)
    });
    if let Some(link) = &pointer {
        crate::log(&format!(
            "{} taken off the timeline; the file is untouched",
            link.path.display()
        ));
        if let Ok(mut removed) = REMOVED_LINKS.lock() {
            removed.push(link.clone());
        }
    }
    if pointer.is_none() {
        let mut documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
        if !documents.iter().any(|d| d.id == id) {
            return Err(format!("document {id} is not in the list"));
        }
        // The move first, under the lock, so an image still encoding lands where the folder
        // is (review T2); a move that fails keeps the document listed and is the page's to
        // show (review T10).
        match crate::store::trash(id) {
            Ok(Some(to)) => crate::log(&format!(
                "document {id} deleted, into the trash at {}",
                to.display()
            )),
            Ok(None) => crate::log(&format!("document {id} deleted; it had no folder yet")),
            Err(err) => {
                crate::log(&format!("document {id} NOT deleted: {err}"));
                return Err(format!("not deleted: {err}"));
            }
        }
        documents.retain(|d| d.id != id);
    }
    if !was_current {
        return Ok(Some(editor_image_info()?));
    }
    // The neighbour: the newer one, else the older, else the empty state.
    let next = at.and_then(|i| {
        ids.get(i + 1)
            .or_else(|| i.checked_sub(1).and_then(|j| ids.get(j)))
            .copied()
    });
    match next {
        Some(next) => match show_document(next) {
            Ok(()) => Ok(Some(editor_image_info()?)),
            // The delete is done either way; a neighbour that cannot be shown leaves the
            // editor empty rather than reporting a document that is gone as not deleted
            // (review 4, T7).
            Err(err) => {
                crate::log(&format!(
                    "document {id} deleted; its neighbour {next} could not be shown: {err}"
                ));
                show_nothing();
                Ok(None)
            }
        },
        None => {
            show_nothing();
            Ok(None)
        }
    }
}

/// The empty state: no image, no document, the plain title.
fn show_nothing() {
    if let Ok(mut slot) = state().image.lock() {
        *slot = None;
    }
    state().set_document(None);
    CURRENT_ID.store(0, Ordering::SeqCst);
    if let Ok(app) = app() {
        if let Some(window) = app.get_webview_window("editor") {
            let _ = window.set_title("Recon");
        }
    }
}

/// A document chosen from the timeline: shown from its own image, history active (§3.4).
#[tauri::command]
pub fn editor_show_document(id: u64) -> Result<ImageInfo, String> {
    if id != current_document_id() {
        show_document(id)?;
    }
    editor_image_info()
}

/// What the page needs about the document on screen against the managed list (§3.3):
/// whether it IS a managed document, when a document that exists for this file was last
/// annotated (the route to its saved edit, shown only while the file itself is on screen),
/// and whether the file has changed on disk since the document preserved it.
struct Standing {
    managed: bool,
    edited_ago_s: Option<u64>,
    source_changed: bool,
}

fn standing() -> Standing {
    let id = current_document_id();
    let key = state()
        .document
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|d| (d.opened.path.clone(), d.index)));
    let Some((path, frame)) = key else {
        // A capture is a managed document from the moment it exists (§3.3), and since
        // S1.8 it is in the list from that moment; a probe image the checks load is not.
        let managed = DOCUMENTS
            .lock()
            .map(|documents| documents.iter().any(|d| d.id == id))
            .unwrap_or(false);
        return Standing {
            managed,
            edited_ago_s: None,
            source_changed: false,
        };
    };
    let Ok(documents) = DOCUMENTS.lock() else {
        return Standing {
            managed: false,
            edited_ago_s: None,
            source_changed: false,
        };
    };
    let now = std::time::SystemTime::now();
    let ago = |at: std::time::SystemTime| now.duration_since(at).map(|d| d.as_secs()).unwrap_or(0);
    match documents.iter().find(|d| d.id == id) {
        Some(own) => {
            let source_changed = match &own.source {
                Source::File { modified, len, .. } => file_stamp(&path) != (*modified, *len),
                Source::Capture => false,
            };
            Standing {
                managed: true,
                edited_ago_s: None,
                source_changed,
            }
        }
        None => Standing {
            managed: false,
            edited_ago_s: documents
                .iter()
                .find(|d| d.is_for(&path, frame))
                .map(|d| ago(d.annotated)),
            source_changed: false,
        },
    }
}

/// Annotate (§3.3): a capture is managed already, so nothing happens; a file on screen
/// gets its managed document, created with the decoded image preserved, or resumed when
/// one exists for that path and frame, never a second one. Resuming shows the document's
/// own preserved pixels, not the file as it is now on disk. The mode itself is the page's;
/// this returns the image info the page loads.
#[tauri::command]
pub fn editor_annotate() -> Result<ImageInfo, String> {
    let image = state().image().ok_or("no image is loaded")?;
    let id = current_document_id();
    let key = {
        let slot = state()
            .document
            .lock()
            .map_err(|_| "the document is poisoned")?;
        slot.as_ref()
            .map(|d| (d.opened.path.clone(), d.index, d.opened.format.name()))
    };
    let Some((path, frame, name)) = key else {
        return editor_image_info();
    };
    // One document per file (§3.3): the lookup below must see every document, so it waits
    // for the startup read when Annotate comes within its first seconds (S2.8).
    wait_store_loaded();
    // The list is read under its lock and the lock is released here, before the image info
    // is built: that reads the list too, and a lock held across it would wait on itself.
    let (own, existing) = {
        let documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
        let own = documents.iter().any(|d| d.id == id);
        // The handle only: the decode runs after the lock is released (review 4, T6).
        let existing = (!own)
            .then(|| {
                documents
                    .iter()
                    .find(|d| d.is_for(&path, frame))
                    .map(|d| d.preserved_handle())
            })
            .flatten();
        (own, existing)
    };
    if own {
        // Annotate, View, Annotate inside the same document: nothing to create.
        return editor_image_info();
    }
    match existing {
        Some((existing_id, preserved, width, height)) => {
            let resumed = preserved_frame(existing_id, &preserved, width, height, name)?;
            CURRENT_ID.store(existing_id, Ordering::SeqCst);
            state().set_frame(resumed);
            if let Ok(mut documents) = DOCUMENTS.lock() {
                if let Some(document) = documents.iter_mut().find(|d| d.id == existing_id) {
                    document.annotated = std::time::SystemTime::now();
                }
            }
            crate::log(&format!(
                "annotate: document {existing_id} resumed for {} frame {frame}",
                path.display()
            ));
        }
        None => {
            let (modified, len) = file_stamp(&path);
            preserve(
                image,
                id,
                Source::File {
                    path: path.clone(),
                    frame,
                    modified,
                    len,
                },
            );
            crate::log(&format!(
                "annotate: document {id} created for {} frame {frame}",
                path.display()
            ));
        }
    }
    editor_image_info()
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
    // No frame of Windows' own: the page draws the top bar, with the title, the controls
    // and the three window buttons (Rotem's call, 2026-09-14). The edges still resize.
    WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .title("Recon")
        // 1600 wide and 1160 tall when it opens (Rotem, 2026-09-15); it was 1280 by 800.
        .inner_size(1600.0, 1160.0)
        // Centred in the main display's work area, across and in height (Rotem, 2026-09-15);
        // it opened at Windows' default place, near the left edge. On a display smaller than
        // the window, it is shrunk to the work area first, so the top bar is never above it.
        .prevent_overflow()
        .center()
        .decorations(false)
        .visible(false)
        .build()?;
    Ok(())
}

/// Shows the editor and reports how long the show call itself took.
///
/// A minimized window is restored first (Rotem, 2026-09-16): show changes nothing on a
/// window that is already visible, and the focus call skips a minimized one, so a capture
/// taken with the editor minimized left it in the taskbar. Restored, it comes to the front
/// the way a click on its taskbar button brings it.
/// The open shortcut, pressed: shows the window, or minimizes it when it is already the
/// one in front (Rotem, 2026-09-18). Returns what it did, for the log.
pub fn toggle(app: &AppHandle) -> Result<&'static str, String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    let visible = window.is_visible().map_err(|err| err.to_string())?;
    let minimized = window.is_minimized().map_err(|err| err.to_string())?;
    let focused = window.is_focused().map_err(|err| err.to_string())?;
    match crate::settings::open_press(visible, minimized, focused) {
        crate::settings::OpenPress::Minimize => {
            window.minimize().map_err(|err| err.to_string())?;
            Ok("minimized")
        }
        crate::settings::OpenPress::Show => show(app).map(|_| "shown"),
    }
}

pub fn show(app: &AppHandle) -> Result<u128, String> {
    let started = Instant::now();
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    if window.is_minimized().map_err(|err| err.to_string())? {
        window.unminimize().map_err(|err| err.to_string())?;
    }
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
    // Taking a capture activates history navigation (§3.4).
    state().history_active.store(true, Ordering::SeqCst);
    present_frame(frame, None)
}

/// Hides the editor and returns the focus to the application the capture began in (§3.1).
pub fn hide(app: &AppHandle) -> Result<&'static str, String> {
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    window.hide().map_err(|err| err.to_string())?;
    let returned = crate::focus::return_to_target();
    crate::log(&format!("editor hidden; {}", returned.line()));
    if let Ok(mut last) = LAST_RETURN.lock() {
        *last = returned.line();
    }
    Ok(returned.line())
}

/// What the last hide's focus return said, for the S1.10 checks.
static LAST_RETURN: Mutex<&'static str> = Mutex::new("");

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

fn present_frame(frame: Frame, reuse: Option<u64>) -> Result<u128, String> {
    let app = app()?;
    let is_capture = frame.source == "capture";
    let id = match reuse {
        Some(id) => {
            CURRENT_ID.store(id, Ordering::SeqCst);
            id
        }
        None => next_document_id(),
    };
    state().set_frame(frame);
    // A capture is a document from its first moment (§3.8: new captures save
    // automatically), so the previous one is never overwritten by the next (§3.3).
    if is_capture {
        if let Some(image) = state().image() {
            preserve(image, id, Source::Capture);
        }
    }
    let info = editor_image_info()?;
    set_title(app, &info);
    app.emit("capture-ready", &info)
        .map_err(|err| err.to_string())?;
    show(app)
}

/// Opens a file through the one image source and shows its first frame or page on the
/// same canvas a capture uses. The file is read once and never written (rule 11).
pub fn open_path(path: &std::path::Path) -> Result<u128, String> {
    open_path_as(path, false)
}

/// A file opened by name, from the picker, "Open with" or the command line: it joins the
/// timeline as a pointer to where it lives (Rotem, 2026-09-18), unless a managed document
/// for it is there already. A file stepped onto in its folder goes through `open_path` and
/// joins nothing.
pub fn open_file(path: &std::path::Path) -> Result<u128, String> {
    open_path_as(path, true)
}

fn open_path_as(path: &std::path::Path, joins: bool) -> Result<u128, String> {
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
    // The folder context: kept when the file is in the folder already listed, built once
    // otherwise (§3.4: read once per navigation context, never per keystroke).
    if let Ok(mut slot) = state().folder.lock() {
        let position = slot.as_ref().and_then(|ctx| {
            (path.parent() == Some(ctx.dir.as_path()))
                .then(|| ctx.contains(path))
                .flatten()
        });
        match (slot.as_mut(), position) {
            (Some(ctx), Some(index)) => ctx.index = index,
            _ => *slot = crate::folder::build(path),
        }
    }
    // Opening an external file activates folder navigation (§3.4).
    state().history_active.store(false, Ordering::SeqCst);
    let has_document = DOCUMENTS
        .lock()
        .map(|documents| documents.iter().any(|d| d.is_for(path, 0)))
        .unwrap_or(true);
    let reuse = (joins && !has_document).then(|| {
        let (width, height) = (decoded.width, decoded.height);
        let mut links = LINKS.lock().ok()?;
        if let Some(known) = links.iter().find(|l| same_path(&l.path, path)) {
            return Some(known.id);
        }
        let id = next_document_id();
        links.push(Link {
            id,
            path: path.to_path_buf(),
            width,
            height,
            created_ms: id,
        });
        save_links(&links);
        Some(id)
    });
    present_frame(frame_of(decoded, name), reuse.flatten())
}

/// Previous, next, first or last in the folder (§3.4). A file gone since the listing is
/// skipped with a log line; an end stays where it is and returns the image as it is.
#[tauri::command]
pub fn editor_navigate(step: String) -> Result<ImageInfo, String> {
    let step = match step.as_str() {
        "previous" => crate::folder::Step::Previous,
        "next" => crate::folder::Step::Next,
        "first" => crate::folder::Step::First,
        "last" => crate::folder::Step::Last,
        other => return Err(format!("{other:?} is not a step")),
    };
    if state().history_active.load(Ordering::SeqCst) {
        return navigate_history(step);
    }
    loop {
        let target = {
            let slot = state()
                .folder
                .lock()
                .map_err(|_| "the folder context is poisoned")?;
            let ctx = slot
                .as_ref()
                .ok_or("no folder is being walked: the image on screen did not come from a file")?;
            ctx.target(step).map(|i| (i, ctx.files[i].clone()))
        };
        let Some((index, path)) = target else {
            return editor_image_info();
        };
        match open_path(&path) {
            Ok(_) => return editor_image_info(),
            Err(err) => {
                crate::log(&format!(
                    "skipped {}: {err}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                if let Ok(mut slot) = state().folder.lock() {
                    if let Some(ctx) = slot.as_mut() {
                        ctx.forget(index);
                    }
                }
            }
        }
    }
}

/// Recon history, in creation order, oldest first: every managed document, captures and
/// annotated files alike (§3.3). Previous is older, next is newer, the ends stop.
fn history_ids() -> Vec<u64> {
    let mut ids: Vec<(u64, std::time::SystemTime)> = DOCUMENTS
        .lock()
        .map(|documents| documents.iter().map(|d| (d.id, d.created)).collect())
        .unwrap_or_default();
    if let Ok(links) = LINKS.lock() {
        ids.extend(links.iter().map(|l| {
            (
                l.id,
                std::time::UNIX_EPOCH + std::time::Duration::from_millis(l.created_ms),
            )
        }));
    }
    ids.sort_by_key(|(id, created)| (*created, *id));
    ids.into_iter().map(|(id, _)| id).collect()
}

/// Where the document on screen sits in history, one-based, and how many there are.
fn history_position() -> (u32, u32) {
    let ids = history_ids();
    let at = ids.iter().position(|id| *id == current_document_id());
    (at.map(|i| i as u32 + 1).unwrap_or(0), ids.len() as u32)
}

fn navigate_history(step: crate::folder::Step) -> Result<ImageInfo, String> {
    let ids = history_ids();
    let Some(at) = ids.iter().position(|id| *id == current_document_id()) else {
        return editor_image_info();
    };
    let last = ids.len() - 1;
    let target = match step {
        crate::folder::Step::Previous => at.checked_sub(1),
        crate::folder::Step::Next => (at < last).then_some(at + 1),
        crate::folder::Step::First => (at != 0).then_some(0),
        crate::folder::Step::Last => (at != last).then_some(last),
    };
    if let Some(index) = target {
        show_document(ids[index])?;
    }
    editor_image_info()
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
        crate::settings::editor_settings,
        crate::settings::editor_settings_set,
        crate::settings::editor_settings_recording,
        editor_open_dialog,
        editor_fullscreen,
        editor_navigate,
        editor_annotate,
        editor_save_notes,
        editor_notes,
        editor_saves_flushed,
        editor_save_as,
        editor_documents,
        editor_show_document,
        editor_thumbnail,
        editor_storage,
        editor_delete_document,
        editor_trash_documents,
        editor_trash_restore,
        editor_trash_rejoin,
        editor_tabs,
        editor_save_tabs,
        checks::editor_trash_list,
        checks::editor_trash_age,
        checks::editor_trash_break,
        checks::editor_sweep_trash,
        checks::editor_save_as_outcome,
        checks::editor_file_kind,
        checks::editor_save_as_plan,
        checks::editor_save_as_write,
        checks::editor_window_title,
        checks::editor_store_reset,
        checks::editor_store_list,
        checks::editor_store_read,
        checks::editor_store_reload,
        checks::editor_store_verify,
        checks::editor_store_break,
        checks::editor_store_remove_source,
        checks::editor_window_visible,
        checks::editor_in_front,
        checks::editor_hold_clipboard,
        checks::editor_stand_in,
        checks::editor_last_return,
        checks::editor_managed,
        checks::editor_replace_in_folder,
        checks::editor_make_folder,
        checks::editor_open_path,
        checks::editor_open_file,
        checks::editor_file_print,
        checks::editor_folder_names,
        checks::editor_opened_reload,
        checks::editor_remove_from_folder,
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
        crate::settings::editor_settings_pretend_taken,
        crate::settings::editor_settings_file,
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
        checks::editor_screen_compare,
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
        crate::settings::editor_settings,
        crate::settings::editor_settings_set,
        crate::settings::editor_settings_recording,
        editor_open_dialog,
        editor_fullscreen,
        editor_annotate,
        editor_navigate,
        editor_save_notes,
        editor_notes,
        editor_saves_flushed,
        editor_save_as,
        editor_documents,
        editor_show_document,
        editor_thumbnail,
        editor_storage,
        editor_delete_document,
        editor_trash_documents,
        editor_trash_restore,
        editor_trash_rejoin,
        editor_tabs,
        editor_save_tabs,
    ]);
    builder.register_asynchronous_uri_scheme_protocol("region", |_ctx, request, responder| {
        let query = request.uri().query().unwrap_or_default().to_string();
        // The timeline's thumbnails (S2.1) share the scheme and nothing else: each one is
        // made or read on a thread of its own, never through the one region worker, so a
        // strip of thirty thumbnails cannot delay the view the user is looking at.
        if let Some(id) = query
            .strip_prefix("thumb=")
            .and_then(|s| s.parse::<u64>().ok())
        {
            std::thread::spawn(move || {
                let response = match thumbnail(id) {
                    Ok(png) => tauri::http::Response::builder()
                        .header("Content-Type", "image/png")
                        .header("Access-Control-Allow-Origin", APP_ORIGIN)
                        .body(png),
                    Err(err) => tauri::http::Response::builder()
                        .status(404)
                        .header("Access-Control-Allow-Origin", APP_ORIGIN)
                        .body(err.into_bytes()),
                };
                responder.respond(response.expect("a thumbnail response"));
            });
            return;
        }
        serve_region(query, responder);
    })
}

// ---------------------------------------------------------------- the product's commands

/// Composes the page's annotation layer over the current image, in canvas space.
///
/// Returns the composite, the decoded layer (the checks count from it) and the time.
/// The blur regions the page sends in a header (S3.4): none when the header is absent.
fn blur_header(request: &tauri::ipc::Request<'_>) -> Result<Vec<crate::compose::BlurRect>, String> {
    request
        .headers()
        .get("blur")
        .and_then(|v| v.to_str().ok())
        .map(crate::compose::parse_blurs)
        .transpose()
        .map(|b| b.unwrap_or_default())
}

/// The crop the page sends in a header (S3.6): none when the header is absent or empty.
fn crop_header(
    request: &tauri::ipc::Request<'_>,
) -> Result<Option<crate::compose::CropRect>, String> {
    request
        .headers()
        .get("crop")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.trim().is_empty())
        .map(crate::compose::parse_crop)
        .transpose()
}

fn compose_layer(
    png: &[u8],
    margin: crate::compose::Margin,
    blurs: &[crate::compose::BlurRect],
    crop: Option<crate::compose::CropRect>,
) -> Result<(crate::compose::Composite, Vec<u8>, u128), String> {
    let image = state().image().ok_or("no image is loaded")?;
    // The blur and the crop are applied to a copy, never to the document's own pixels
    // (S3.4, S3.6).
    let (source, w, h) = crate::compose::prepare_source(
        &image.frame.rgba,
        image.frame.width(),
        image.frame.height(),
        blurs,
        crop,
    )?;
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
    let composite = crate::compose::compose(&source, w, h, &layer, margin, crate::compose::MAT)?;
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
    let blurs = blur_header(&request)?;
    let crop = crop_header(&request)?;
    let (composite, _, compose_ms) = compose_layer(&bytes, margin, &blurs, crop)?;
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

/// The timeline's thumbnail with the notes on it (Rotem, 2026-09-16). After every save of
/// the notes the page sends its layer already at the thumbnail's scale, the composition
/// fitted into the thumbnail box, with the margin in image pixels in a header as a copy
/// sends it; the host resamples the document's own image to that scale, composes the layer
/// over it with the margin around, and writes `thumb.png` anew, so the strip shows what the
/// export would. The blur rects are applied to a copy at full size before the resample, as
/// a copy applies them; the document's pixels are never touched. Only the picture on screen
/// can be thumbnailed this way, since the layer is the page's scene.
#[tauri::command]
pub fn editor_thumbnail(request: tauri::ipc::Request<'_>) -> Result<(), String> {
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
    let blurs = blur_header(&request)?;
    // The page names the document its layer belongs to: a picture shown between the layer
    // and its arrival would otherwise get the other picture's notes on its thumbnail.
    let id = current_document_id();
    let named: u64 = request
        .headers()
        .get("document")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok())
        .ok_or("the layer names no document")?;
    if named != id {
        return Err(format!(
            "the layer is document {named}'s and document {id} is on screen; nothing written"
        ));
    }
    let dir = crate::store::folder(id)?;
    if !dir.is_dir() {
        return Err(format!("document {id} is not on disk yet"));
    }
    let image = state().image().ok_or("no image is loaded")?;
    // The blur and the crop are applied to a copy at full size before the resample, as a
    // copy applies them; the document's pixels are never touched (S3.4, S3.6).
    let crop = crop_header(&request)?;
    let (source, w, h) = crate::compose::prepare_source(
        &image.frame.rgba,
        image.frame.width(),
        image.frame.height(),
        &blurs,
        crop,
    )?;
    let (cw, ch) = margin.canvas(w, h);
    let layer = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .map_err(|err| format!("the layer did not decode: {err}"))?
        .into_rgba8();
    let (tw, th) = layer.dimensions();
    let (ew, eh) = thumb_size(cw, ch);
    if (tw, th) != (ew, eh) {
        return Err(format!(
            "the layer is {tw}x{th} and the thumbnail of a {cw}x{ch} composition is {ew}x{eh}"
        ));
    }
    // The picture and the margin at the same scale, the rounding taken up by the margin's
    // far sides so the canvas is exactly the layer's size.
    let scale = (tw as f64 / cw as f64).min(th as f64 / ch as f64);
    let at = |n: u32| (n as f64 * scale).round() as u32;
    let left = at(margin.left).min(tw - 1);
    let top = at(margin.top).min(th - 1);
    let sw = at(w).clamp(1, tw - left);
    let sh = at(h).clamp(1, th - top);
    let small_margin = crate::compose::Margin {
        left,
        top,
        right: tw - left - sw,
        bottom: th - top - sh,
    };
    let small = recon_pixels::resample(&source, w, h, sw, sh)
        .ok_or_else(|| format!("document {id}: the thumbnail did not resample"))?;
    let composite = crate::compose::compose(
        &small,
        sw,
        sh,
        layer.as_raw(),
        small_margin,
        crate::compose::MAT,
    )?;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &composite.rgba,
            composite.width,
            composite.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|err| format!("document {id}: the thumbnail did not encode: {err}"))?;
    crate::store::write_beside(&dir.join("thumb.png"), &png)
}

/// The size a thumbnail of a `w` by `h` picture or composition takes: fitted into the box,
/// never enlarged, never empty. One rule for the host's own thumbnail and for the page's
/// layer, which must land on the same size.
fn thumb_size(w: u32, h: u32) -> (u32, u32) {
    let scale = (THUMB_W as f64 / w as f64)
        .min(THUMB_H as f64 / h as f64)
        .min(1.0);
    (
        ((w as f64 * scale).round() as u32).max(1),
        ((h as f64 * scale).round() as u32).max(1),
    )
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
    /// Where the file sits in its folder, one-based, and how many supported files the folder
    /// holds; 0 of 0 for a capture. The context names the list being walked (§3.4).
    pub position: u32,
    pub total: u32,
    pub context: String,
    /// Whether the image on screen is a managed document: a capture always, a file once
    /// Annotate created or resumed its document (§3.3). The page's mode follows it.
    pub managed: bool,
    /// The image on screen is an opened file listed in the timeline as a pointer.
    pub linked: bool,
    /// The route to a saved edit: seconds since a document that exists for this file and
    /// frame was last annotated, while the file itself is on screen; null otherwise.
    pub edited_ago_s: Option<u64>,
    /// A resumed document whose file has changed on disk since it preserved its image.
    pub source_changed: bool,
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
    // A document read back from disk, or resumed, has no file open behind it; its name and
    // frame come from the document itself.
    let (file, index) = if file.is_empty() {
        DOCUMENTS
            .lock()
            .ok()
            .and_then(|documents| {
                documents
                    .iter()
                    .find(|d| d.id == current_document_id())
                    .and_then(|d| match &d.source {
                        Source::File { path, frame, .. } => Some((
                            path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            *frame,
                        )),
                        Source::Capture => None,
                    })
            })
            .unwrap_or((file, index))
    } else {
        (file, index)
    };
    let (position, total, context) = if state().history_active.load(Ordering::SeqCst) {
        let (p, t) = history_position();
        if t > 0 {
            (p, t, "history".to_string())
        } else {
            (0, 0, String::new())
        }
    } else {
        match state().folder.lock() {
            Ok(slot) => match slot.as_ref() {
                Some(ctx) if !file.is_empty() => {
                    let (p, t) = ctx.position();
                    (p, t, "folder".to_string())
                }
                _ => (0, 0, String::new()),
            },
            Err(_) => (0, 0, String::new()),
        }
    };
    let standing = standing();
    Ok(ImageInfo {
        width: image.frame.width(),
        height: image.frame.height(),
        source: image.frame.source.to_string(),
        file,
        kind,
        index,
        count,
        document_id: current_document_id(),
        position,
        total,
        context,
        managed: standing.managed,
        linked: link(current_document_id()).is_some(),
        edited_ago_s: standing.edited_ago_s,
        source_changed: standing.source_changed,
    })
}

#[tauri::command]
pub fn editor_show(app: AppHandle) -> Result<u128, String> {
    show(&app)
}

/// Escape in the idle editor hides it and returns the focus (§3.6, §3.1).
#[tauri::command]
pub fn editor_hide(app: AppHandle) -> Result<String, String> {
    hide(&app).map(|line| line.to_string())
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
            Ok(Some(path)) => match open_file(&path) {
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

/// Where the document on screen came from, for the suggested name: its file, or none for
/// a capture.
fn source_file() -> Option<std::path::PathBuf> {
    let from_open = state()
        .document
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|d| d.opened.path.clone()));
    from_open.or_else(|| {
        DOCUMENTS.lock().ok().and_then(|documents| {
            documents
                .iter()
                .find(|d| d.id == current_document_id())
                .and_then(|d| match &d.source {
                    Source::File { path, .. } => Some(path.clone()),
                    Source::Capture => None,
                })
        })
    })
}

/// Where Save As opens, and the one file it may write over (Rotem, 2026-09-18): for an
/// annotated document that came from a PNG or a JPEG still there, the file's own folder and
/// name, and that file; for everything else the last export folder, a name marked as
/// annotated, and nothing.
fn save_as_plan() -> (std::path::PathBuf, String, Option<std::path::PathBuf>) {
    let id = current_document_id();
    let original = DOCUMENTS.lock().ok().and_then(|documents| {
        documents
            .iter()
            .find(|d| d.id == id)
            .and_then(|d| match &d.source {
                Source::File { path, .. } => Some(path.clone()),
                Source::Capture => None,
            })
    });
    if let Some(original) = original.filter(|path| crate::export::replaceable(path)) {
        if let (Some(folder), Some(name)) = (original.parent(), original.file_name()) {
            return (
                folder.to_path_buf(),
                name.to_string_lossy().to_string(),
                Some(original.clone()),
            );
        }
    }
    (
        crate::export::folder(),
        crate::export::suggested_name(source_file().as_deref()),
        None,
    )
}

/// After Save As wrote over a document's own file: the document's note of that file's size
/// and time follows, so the document does not report its own save as a change on disk.
fn note_own_save(id: u64, path: &std::path::Path) {
    let (new_modified, new_len) = file_stamp(path);
    if let Ok(mut documents) = DOCUMENTS.lock() {
        if let Some(document) = documents.iter_mut().find(|d| d.id == id) {
            if let Source::File { modified, len, .. } = &mut document.source {
                *modified = new_modified;
                *len = new_len;
            }
            if let Err(err) = write_record_of(document) {
                crate::log(&format!(
                    "document {id}: the saved file's stamp NOT kept: {err}"
                ));
            }
        }
    }
}

/// Save As (§3.6, S1.11): the composition is taken NOW, as the copy takes it, and the
/// dialog runs on its own thread with the last export folder and a name derived from the
/// source, marked as annotated. A chosen name that exists is never written over: the
/// dialog comes back with an available name filled in, until a free one is chosen or the
/// user cancels. The outcome reaches the page as an event, and stays readable for the
/// checks.
#[tauri::command]
pub fn editor_save_as(app: AppHandle, request: tauri::ipc::Request<'_>) -> Result<(), String> {
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
    let blurs = blur_header(&request)?;
    let crop = crop_header(&request)?;
    let (composite, _, compose_ms) = compose_layer(&bytes, margin, &blurs, crop)?;
    let window = app
        .get_webview_window("editor")
        .ok_or("there is no editor window")?;
    let owner = window.hwnd().map_err(|err| err.to_string())?.0 as isize;
    let (folder, name, replaces) = save_as_plan();
    let document_id = current_document_id();
    let title = if replaces.is_some() {
        "Save As"
    } else {
        "Save As, a new file"
    };
    set_save_as_outcome(SaveAsOutcome::open());
    std::thread::spawn(move || {
        let started = Instant::now();
        let outcome = {
            {
                let owner = windows::Win32::Foundation::HWND(owner as *mut _);
                let mut folder = folder;
                let mut name = name;
                loop {
                    match crate::dialog::save_as(Some(owner), &folder, &name, title) {
                        Ok(None) => break SaveAsOutcome::cancelled(),
                        Err(err) => break SaveAsOutcome::failed(err),
                        Ok(Some(chosen)) => match crate::export::encode(
                            &composite.rgba,
                            composite.width,
                            composite.height,
                            &chosen,
                        )
                        .and_then(|bytes| {
                            crate::export::write(&chosen, &bytes, replaces.as_deref())
                        }) {
                            Ok(crate::export::Written::New(path)) => {
                                break SaveAsOutcome::saved(path, composite.width, composite.height)
                            }
                            Ok(crate::export::Written::Replaced(path)) => {
                                note_own_save(document_id, &path);
                                break SaveAsOutcome::saved_over(
                                    path,
                                    composite.width,
                                    composite.height,
                                );
                            }
                            Ok(crate::export::Written::Exists { chosen, offered }) => {
                                crate::log(&format!(
                                    "Save As: {} exists, offering {}",
                                    chosen.display(),
                                    offered.display()
                                ));
                                folder =
                                    offered.parent().map(|p| p.to_path_buf()).unwrap_or(folder);
                                name = offered
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or(name);
                                set_save_as_outcome(SaveAsOutcome::offered(&chosen, &offered));
                                let _ = app.emit(
                                    "save-as-offered",
                                    SaveAsOutcome::offered(&chosen, &offered),
                                );
                            }
                            Err(err) => break SaveAsOutcome::failed(err),
                        },
                    }
                }
            }
        };
        crate::log(&format!(
            "Save As: {} (composed in {compose_ms} ms, {} ms in all)",
            outcome.line,
            started.elapsed().as_millis()
        ));
        set_save_as_outcome(outcome.clone());
        let _ = app.emit("save-as-done", outcome);
    });
    Ok(())
}

/// What a Save As came to, for the page's notice and for the checks.
#[derive(serde::Serialize, Clone, Default)]
pub struct SaveAsOutcome {
    /// "open", "saved", "cancelled", "failed" or "offered".
    pub state: String,
    pub path: String,
    pub offered: String,
    pub width: u32,
    pub height: u32,
    pub line: String,
}

impl SaveAsOutcome {
    fn open() -> Self {
        Self {
            state: "open".into(),
            line: "Save As is open".into(),
            ..Default::default()
        }
    }
    fn saved(path: std::path::PathBuf, width: u32, height: u32) -> Self {
        Self {
            state: "saved".into(),
            line: format!("saved {}x{} to {}", width, height, path.display()),
            path: path.display().to_string(),
            width,
            height,
            ..Default::default()
        }
    }
    fn saved_over(path: std::path::PathBuf, width: u32, height: u32) -> Self {
        Self {
            state: "saved".into(),
            line: format!("saved {}x{} over {}", width, height, path.display()),
            path: path.display().to_string(),
            width,
            height,
            ..Default::default()
        }
    }
    fn cancelled() -> Self {
        Self {
            state: "cancelled".into(),
            line: "nothing saved".into(),
            ..Default::default()
        }
    }
    fn failed(err: String) -> Self {
        Self {
            state: "failed".into(),
            line: format!("NOT SAVED: {err}"),
            ..Default::default()
        }
    }
    fn offered(chosen: &std::path::Path, offered: &std::path::Path) -> Self {
        Self {
            state: "offered".into(),
            line: format!(
                "{} exists and is not written over; {} is offered",
                chosen
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                offered
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            ),
            path: chosen.display().to_string(),
            offered: offered.display().to_string(),
            ..Default::default()
        }
    }
}

static SAVE_AS_OUTCOME: Mutex<Option<SaveAsOutcome>> = Mutex::new(None);

fn set_save_as_outcome(outcome: SaveAsOutcome) {
    if let Ok(mut slot) = SAVE_AS_OUTCOME.lock() {
        *slot = Some(outcome);
    }
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

    // A vector seen closer than it was rasterized is drawn from the file itself, at the
    // zoom: enlarging the scale-one render would show that render's pixels, not the
    // drawing's edges. A managed document keeps its preserved pixels, which are what it
    // copies and saves.
    if plan.target_w > region.width || plan.target_h > region.height {
        if let Some(drawing) = viewed_drawing() {
            let drawn = drawing
                .region(
                    region.x,
                    region.y,
                    plan.target_w as f32 / region.width as f32,
                    plan.target_h as f32 / region.height as f32,
                    plan.target_w,
                    plan.target_h,
                )
                .map_err(|err| err.to_string())?;
            return Ok((
                with_size_prefix(drawn.rgba, plan.target_w, plan.target_h),
                plan.target_w,
                plan.target_h,
            ));
        }
    }

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

/// The vector behind the picture on screen, while that picture is the file being viewed
/// and not a managed document's preserved image.
fn viewed_drawing() -> Option<source::svg::Drawing> {
    let drawing = state()
        .document
        .lock()
        .ok()?
        .as_ref()
        .and_then(|document| document.opened.drawing())?;
    let id = current_document_id();
    let managed = DOCUMENTS
        .lock()
        .map(|documents| documents.iter().any(|d| d.id == id))
        .unwrap_or(true);
    (!managed).then_some(drawing)
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

    /// The screen against the pixels the page says its canvas holds, at a rectangle the
    /// page names on the desktop in physical pixels. A picture with transparency is the
    /// case: the page can hold the right pixels while the screen still shows an earlier
    /// paint through them, and only the screen can say so.
    #[tauri::command]
    pub fn editor_screen_compare(request: tauri::ipc::Request<'_>) -> Result<SvgVerdict, String> {
        let header = |key: &str| -> Result<i64, String> {
            request
                .headers()
                .get(key)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or(format!("no {key} header"))
        };
        let (x, y) = (header("x")? as i32, header("y")? as i32);
        let (width, height) = (header("width")? as u32, header("height")? as u32);
        let mut expected = match request.body() {
            tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
            tauri::ipc::InvokeBody::Json(_) => return Err("the pixels arrived as JSON".into()),
        };
        if expected.len() != (width * height * 4) as usize {
            return Err(format!(
                "the page sent {} bytes for {width}x{height}",
                expected.len()
            ));
        }
        let live = screen::copy_rect(DesktopRect {
            x,
            y,
            width,
            height,
        })
        .map_err(|err| err.to_string())?;
        // The screen is opaque, so the comparison is on colour alone.
        for px in expected.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        let (differing, bbox) = compare(&live, &expected, width, height);
        let name = "svg-screen.png";
        let path = std::env::current_exe()
            .map(|exe| exe.with_file_name(name))
            .unwrap_or_else(|_| name.into());
        if let Some(img) = image::RgbaImage::from_raw(width, height, live) {
            let _ = img.save(&path);
        }
        Ok(SvgVerdict {
            name: name.to_string(),
            pixels: (width * height) as usize,
            differing,
            percent: differing as f64 * 100.0 / (width * height).max(1) as f64,
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

        let blurs = blur_header(&request)?;
        let crop = crop_header(&request)?;
        let (composite, layer, compose_ms) = compose_layer(&bytes, margin, &blurs, crop)?;
        let image = state().image().ok_or("no image is loaded")?;
        // The source the output is checked against is the one it was cut from: the crop of
        // the frame when one is named, unblurred, so a blur still counts as a difference and
        // a crop's copy is exact inside its own pixels.
        let (source, sw, sh) = crate::compose::prepare_source(
            &image.frame.rgba,
            image.frame.width(),
            image.frame.height(),
            &[],
            crop,
        )?;
        let source_mismatches =
            crate::compose::source_mismatches(&composite, &source, sw, sh, &layer, margin);
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

    /// One managed document as the S1.5 check sees it.
    #[derive(serde::Serialize)]
    pub struct ManagedLine {
        pub id: u64,
        pub source: String,
        pub path: String,
        pub frame: u32,
        pub width: u32,
        pub height: u32,
        /// The PNG's size, zero while the encode thread is still running.
        pub bytes: usize,
    }

    /// The managed documents, for the S1.5 check: never two for one file and frame.
    #[tauri::command]
    pub fn editor_managed() -> Vec<ManagedLine> {
        DOCUMENTS
            .lock()
            .map(|documents| {
                documents
                    .iter()
                    .map(|d| {
                        let (source, path, frame) = match &d.source {
                            Source::Capture => ("capture".to_string(), String::new(), 0),
                            Source::File { path, frame, .. } => {
                                ("file".to_string(), path.display().to_string(), *frame)
                            }
                        };
                        ManagedLine {
                            id: d.id,
                            source,
                            path,
                            frame,
                            width: d.width,
                            height: d.height,
                            bytes: d.bytes(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Overwrites one file of the check folder with the reference scene, a different
    /// picture of a different size, so the check can see that a resumed document keeps its
    /// own preserved image while the file on disk has moved on (§3.3).
    #[tauri::command]
    pub fn editor_replace_in_folder(name: String) -> Result<(), String> {
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err("a name, not a path".into());
        }
        std::fs::copy(
            fixture_dir().join("reference-scene.png"),
            folder_dir().join(name),
        )
        .map(|_| ())
        .map_err(|err| err.to_string())
    }

    fn folder_dir() -> std::path::PathBuf {
        std::env::current_exe()
            .map(|exe| exe.with_file_name("s14-folder"))
            .unwrap_or_else(|_| "s14-folder".into())
    }

    /// A folder for the S1.4 check: five supported files whose names sort one way as text
    /// and another way logically, and one that is not an image.
    #[tauri::command]
    pub fn editor_make_folder() -> Result<String, String> {
        let from = fixture_dir();
        if !from.join("gif-three-frames.gif").exists() {
            source::fixtures::make_export_set(&from).map_err(|err| err.to_string())?;
        }
        let dir = folder_dir();
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        for (name, source) in [
            ("img10.png", "png-alpha-and-swapped-profile.png"),
            ("img2.png", "png-alpha-and-swapped-profile.png"),
            ("IMG1.png", "png-alpha-and-swapped-profile.png"),
            ("b.jpg", "jpeg-orientation-6.jpg"),
            ("a.gif", "gif-three-frames.gif"),
        ] {
            std::fs::copy(from.join(source), dir.join(name)).map_err(|err| err.to_string())?;
        }
        std::fs::write(
            dir.join("notes.txt"),
            b"not an image
",
        )
        .map_err(|err| err.to_string())?;
        Ok(dir.display().to_string())
    }

    /// Opens a file through the product's own path, folder context included.
    #[tauri::command]
    pub fn editor_open_path(path: String) -> Result<ImageInfo, String> {
        open_path(std::path::Path::new(&path))?;
        editor_image_info()
    }

    /// A file opened by name, the way the picker and "Open with" open one: it joins the
    /// timeline as a pointer.
    #[tauri::command]
    pub fn editor_open_file(path: String) -> Result<ImageInfo, String> {
        open_file(std::path::Path::new(&path))?;
        editor_image_info()
    }

    /// The names of the files in a folder, so a check can say nothing was left beside one.
    #[tauri::command]
    pub fn editor_folder_names(dir: String) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&dir).map_err(|err| err.to_string())?;
        Ok(entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect())
    }

    /// A file's length and a hash of its bytes, so a check can say the file was not
    /// touched (rule 11).
    #[tauri::command]
    pub fn editor_file_print(path: String) -> Result<String, String> {
        let bytes = std::fs::read(&path).map_err(|err| err.to_string())?;
        let mut hash = 0xcbf29ce484222325u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
        Ok(format!("{} bytes, {hash:016x}", bytes.len()))
    }

    /// The list of opened files as it is on disk, and the startup read of it again.
    #[tauri::command]
    pub fn editor_opened_reload() -> Result<String, String> {
        let path = crate::store::opened_path()?;
        let text = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
        load_links();
        Ok(text)
    }

    /// Removes one file of the check folder, so the walk meets a file that has gone.
    #[tauri::command]
    pub fn editor_remove_from_folder(name: String) -> Result<(), String> {
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err("a name, not a path".into());
        }
        std::fs::remove_file(folder_dir().join(name)).map_err(|err| err.to_string())
    }

    /// The window's title, for the S1.3 check.
    #[tauri::command]
    pub fn editor_window_title(app: AppHandle) -> Result<String, String> {
        app.get_webview_window("editor")
            .ok_or("there is no editor window")?
            .title()
            .map_err(|err| err.to_string())
    }

    fn store_dir() -> std::path::PathBuf {
        std::env::current_exe()
            .map(|exe| exe.with_file_name("s18-store"))
            .unwrap_or_else(|_| "s18-store".into())
    }

    /// Empties the checks' store and the list, so a section starts from nothing on disk.
    #[tauri::command]
    pub fn editor_store_reset() -> Result<String, String> {
        let dir = store_dir();
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        crate::store::set_root(dir.clone());
        // The checks' trash sits beside their store, and starts empty with it.
        if let Ok(trash) = crate::store::trash_root() {
            let _ = std::fs::remove_dir_all(trash);
        }
        // A startup read still running belongs to the store that was; it stops (S2.8).
        STORE_GENERATION.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut documents) = DOCUMENTS.lock() {
            documents.clear();
        }
        // The opened files' pointers start empty with the store.
        if let Ok(path) = crate::store::opened_path() {
            let _ = std::fs::remove_file(path);
        }
        load_links();
        set_store_loaded(true);
        Ok(dir.display().to_string())
    }

    #[derive(serde::Serialize)]
    pub struct StoreLine {
        pub id: u64,
        pub source_png: bool,
        pub source_bytes: u64,
        pub json: bool,
        pub modified_ms: u64,
        pub notes: String,
        pub leftovers: u32,
        pub thumb: bool,
        /// Every file in the folder together, as a walk counts it: what the ledger must say.
        pub bytes: u64,
    }

    /// What is on disk, folder by folder, as the S1.8 check reads it.
    #[tauri::command]
    pub fn editor_store_list() -> Result<Vec<StoreLine>, String> {
        let mut lines = Vec::new();
        for entry in std::fs::read_dir(store_dir())
            .map_err(|err| err.to_string())?
            .flatten()
        {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let record = crate::store::read_record(&dir).ok();
            let source = std::fs::metadata(dir.join("source.png")).ok();
            let leftovers = std::fs::read_dir(&dir)
                .map(|d| {
                    d.flatten()
                        .filter(|e| e.path().to_string_lossy().ends_with(".tmp"))
                        .count()
                })
                .unwrap_or(0) as u32;
            lines.push(StoreLine {
                id: record
                    .as_ref()
                    .map(|r| r.id)
                    .or_else(|| dir.file_name()?.to_string_lossy().parse().ok())
                    .unwrap_or(0),
                source_png: source.is_some(),
                source_bytes: source.map(|m| m.len()).unwrap_or(0),
                json: record.is_some(),
                modified_ms: record.as_ref().map(|r| r.modified_ms).unwrap_or(0),
                notes: record.map(|r| r.notes.to_string()).unwrap_or_default(),
                leftovers,
                thumb: dir.join("thumb.png").is_file(),
                bytes: std::fs::read_dir(&dir)
                    .map(|d| {
                        d.flatten()
                            .filter_map(|e| e.metadata().ok())
                            .filter(|m| m.is_file())
                            .map(|m| m.len())
                            .sum()
                    })
                    .unwrap_or(0),
            });
        }
        lines.sort_by_key(|l| l.id);
        Ok(lines)
    }

    /// One document's JSON, as written.
    #[tauri::command]
    pub fn editor_store_read(id: u64) -> Result<String, String> {
        std::fs::read_to_string(store_dir().join(id.to_string()).join("document.json"))
            .map_err(|err| err.to_string())
    }

    /// A restart, without the process: the list and the image on screen are dropped, the
    /// store is read again from disk, and the latest document reopens as at startup.
    #[tauri::command]
    pub fn editor_store_reload() -> Result<ImageInfo, String> {
        STORE_GENERATION.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut documents) = DOCUMENTS.lock() {
            documents.clear();
        }
        if let Ok(mut slot) = state().image.lock() {
            *slot = None;
        }
        CURRENT_ID.store(0, Ordering::SeqCst);
        crate::store::set_root(store_dir());
        load_store();
        editor_image_info()
    }

    /// Whether a file document's source.png is, pixel for pixel, the frame the file gives
    /// when decoded afresh: the S0.5 leg carried here, the frame asked for, upright,
    /// converted, at the raster size fixed at open. False when the file changed meanwhile.
    #[tauri::command]
    pub fn editor_store_verify(id: u64) -> Result<bool, String> {
        let (path, frame_index) = {
            let documents = DOCUMENTS.lock().map_err(|_| "the documents are poisoned")?;
            let document = documents
                .iter()
                .find(|d| d.id == id)
                .ok_or_else(|| format!("document {id} is not in the list"))?;
            match &document.source {
                Source::File { path, frame, .. } => (path.clone(), *frame),
                Source::Capture => return Err("a capture has no file to compare with".into()),
            }
        };
        let preserved = std::fs::read(store_dir().join(id.to_string()).join("source.png"))
            .map_err(|err| err.to_string())?;
        let preserved = image::load_from_memory_with_format(&preserved, image::ImageFormat::Png)
            .map_err(|err| err.to_string())?
            .into_rgba8();
        let mut opened = source::open(&path).map_err(|err| err.to_string())?;
        let fresh = opened.frame(frame_index).map_err(|err| err.to_string())?;
        Ok(preserved.dimensions() == (fresh.width, fresh.height)
            && preserved.as_raw() == &fresh.rgba)
    }

    /// Takes one document's image off the checks' store, so a show of it fails the way a
    /// document whose PNG went unreadable would (review 4, T7).
    #[tauri::command]
    pub fn editor_store_remove_source(id: u64) -> Result<(), String> {
        std::fs::remove_file(store_dir().join(id.to_string()).join("source.png"))
            .map_err(|err| err.to_string())
    }

    /// Points the store at a file instead of a folder, so every write fails, and back.
    #[tauri::command]
    pub fn editor_store_break(on: bool) -> Result<(), String> {
        if on {
            let blocker = store_dir().join("blocked");
            std::fs::write(&blocker, b"a file where the store expects a folder")
                .map_err(|err| err.to_string())?;
            crate::store::set_root(blocker);
        } else {
            crate::store::set_root(store_dir());
        }
        Ok(())
    }

    static CLIPBOARD_HOLDER: Mutex<Option<std::process::Child>> = Mutex::new(None);

    /// Holds the clipboard open from ANOTHER PROCESS, our own executable started with
    /// `--hold-clipboard`, so a publish meets what it meets when another application is
    /// holding the clipboard, which is the real failure. A thread of this process would
    /// not do: the clipboard is open per task, and a second open from the same process
    /// succeeds. Returns whether it is held, so the check never assumes it.
    #[tauri::command]
    pub fn editor_hold_clipboard(on: bool) -> Result<bool, String> {
        let mut slot = CLIPBOARD_HOLDER
            .lock()
            .map_err(|_| "the clipboard holder slot is poisoned")?;
        if on {
            if slot.is_none() {
                *slot = Some(crate::selftest::open_clipboard_holder()?);
            }
            Ok(true)
        } else {
            if let Some(mut child) = slot.take() {
                let _ = child.kill();
                let _ = child.wait();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Ok(false)
        }
    }

    static STAND_IN: Mutex<Option<std::process::Child>> = Mutex::new(None);

    /// A stand-in application as the return target: opened and remembered, or closed so
    /// the target is gone, as the self test's section G does with the same window.
    #[tauri::command]
    pub fn editor_stand_in(on: bool) -> Result<(), String> {
        let mut slot = STAND_IN
            .lock()
            .map_err(|_| "the stand-in slot is poisoned")?;
        if on {
            if slot.is_none() {
                let (child, hwnd) = crate::selftest::open_stand_in()?;
                crate::focus::remember(hwnd);
                *slot = Some(child);
            }
        } else if let Some(mut child) = slot.take() {
            let _ = child.kill();
            let _ = child.wait();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    }

    #[tauri::command]
    pub fn editor_last_return() -> String {
        LAST_RETURN
            .lock()
            .map(|l| l.to_string())
            .unwrap_or_default()
    }

    /// The trash's contents, number and age in days.
    #[tauri::command]
    pub fn editor_trash_list() -> Vec<(u64, u64)> {
        crate::store::trash_list()
    }

    /// Backdates a trashed document by so many days, so the sweep can be seen to work.
    #[tauri::command]
    pub fn editor_trash_age(id: u64, days: u64) -> Result<(), String> {
        let dir = crate::store::trash_root()?.join(id.to_string());
        let then =
            crate::store::millis(std::time::SystemTime::now()).saturating_sub(days * 86_400_000);
        crate::store::write_atomic(&dir.join("trashed"), then.to_string().as_bytes())
    }

    /// Puts a file where the checks' trash folder goes, so a move into the trash fails,
    /// and takes it away again (review T10).
    #[tauri::command]
    pub fn editor_trash_break(on: bool) -> Result<(), String> {
        let trash = crate::store::trash_root()?;
        if on {
            let _ = std::fs::remove_dir_all(&trash);
            std::fs::write(&trash, b"a file where the trash expects a folder")
                .map_err(|err| err.to_string())
        } else {
            match std::fs::remove_file(&trash) {
                Ok(()) => Ok(()),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(err) => Err(err.to_string()),
            }
        }
    }

    #[tauri::command]
    pub fn editor_sweep_trash() -> Vec<u64> {
        crate::store::sweep_trash(crate::store::TRASH_DAYS)
    }

    #[derive(serde::Serialize)]
    pub struct FileKind {
        pub format: String,
        pub width: u32,
        pub height: u32,
        pub bytes: u64,
    }

    /// What a file on disk is, by its bytes: the S2.6 check reads a JPEG back.
    #[tauri::command]
    pub fn editor_file_kind(path: String) -> Result<FileKind, String> {
        let bytes = std::fs::read(&path).map_err(|err| err.to_string())?;
        let format = image::guess_format(&bytes).map_err(|err| err.to_string())?;
        let decoded =
            image::load_from_memory_with_format(&bytes, format).map_err(|err| err.to_string())?;
        Ok(FileKind {
            format: format!("{format:?}").to_ascii_lowercase(),
            width: decoded.width(),
            height: decoded.height(),
            bytes: bytes.len() as u64,
        })
    }

    /// The last Save As, as the checks read it.
    #[tauri::command]
    pub fn editor_save_as_outcome() -> SaveAsOutcome {
        SAVE_AS_OUTCOME
            .lock()
            .ok()
            .and_then(|slot| slot.clone())
            .unwrap_or_default()
    }

    #[derive(serde::Serialize)]
    pub struct SaveAsPlan {
        pub folder: String,
        pub name: String,
        pub source: String,
        /// The one file this Save As may write over, empty when there is none.
        pub replaces: String,
    }

    /// The folder and the name the dialog would open with, for the image on screen.
    #[tauri::command]
    pub fn editor_save_as_plan() -> SaveAsPlan {
        let source = source_file();
        let (folder, name, replaces) = save_as_plan();
        SaveAsPlan {
            folder: folder.display().to_string(),
            name,
            source: source.map(|p| p.display().to_string()).unwrap_or_default(),
            replaces: replaces
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        }
    }

    /// The write itself, without the dialog: the layer composed as Save As composes it and
    /// written to the path in the header through the one never-overwrite path.
    #[tauri::command]
    pub fn editor_save_as_write(
        request: tauri::ipc::Request<'_>,
    ) -> Result<crate::export::Written, String> {
        let header = |name: &str| {
            request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };
        let margin = header("margin")
            .map(|m| crate::compose::Margin::parse(&m))
            .transpose()?
            .unwrap_or_default();
        let path = header("path").ok_or("no path header")?;
        let bytes = match request.body() {
            tauri::ipc::InvokeBody::Raw(bytes) => bytes.clone(),
            tauri::ipc::InvokeBody::Json(_) => {
                return Err("the layer arrived as JSON, not as bytes".into())
            }
        };
        let blurs = blur_header(&request)?;
        let crop = crop_header(&request)?;
        let (composite, _, _) = compose_layer(&bytes, margin, &blurs, crop)?;
        let target = std::path::Path::new(&path);
        let encoded =
            crate::export::encode(&composite.rgba, composite.width, composite.height, target)?;
        let (_, _, replaces) = save_as_plan();
        let written = crate::export::write(target, &encoded, replaces.as_deref())?;
        if let crate::export::Written::Replaced(path) = &written {
            note_own_save(current_document_id(), path);
        }
        Ok(written)
    }

    /// Whether the window is shown, for the S1.7 Copy and Return check.
    #[tauri::command]
    pub fn editor_window_visible(app: AppHandle) -> Result<bool, String> {
        app.get_webview_window("editor")
            .ok_or("there is no editor window")?
            .is_visible()
            .map_err(|err| err.to_string())
    }

    /// What the last Ctrl+O did, for the S1.2 check.
    #[tauri::command]
    pub fn editor_dialog_outcome() -> String {
        DIALOG_OUTCOME.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Whether the window in front belongs to this process: the screen compares read the
    /// screen where our window is, and with another window over it they would read that.
    #[tauri::command]
    pub fn editor_in_front() -> bool {
        let owner = crate::platform::foreground_owner();
        owner.exists && owner.pid == std::process::id()
    }

    /// Escape, pressed for real, so the picker under test closes the way a person closes it.
    #[tauri::command]
    pub fn editor_press_escape() -> bool {
        // Only into our own window: a key pressed for real lands wherever the focus is, and
        // with someone else's application in front it would land there (S0.8's lesson).
        let owner = crate::platform::foreground_owner();
        if !owner.exists || owner.pid != std::process::id() {
            crate::log(&format!(
                "escape NOT sent: the foreground window is {:?}, pid {}",
                owner.title, owner.pid
            ));
            return false;
        }
        crate::selftest::press_escape();
        true
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
    fn small_image(width: u32, height: u32, fill: u8) -> Arc<Image> {
        Arc::new(Image::new(Frame {
            geometry: FrameGeometry {
                origin_x: 0,
                origin_y: 0,
                width,
                height,
            },
            rgba: vec![fill; (width * height * 4) as usize],
            source: "test",
        }))
    }

    #[test]
    fn a_document_is_preserved_once_and_found_by_its_file_and_frame_case_blind() {
        let path = std::path::PathBuf::from("C:\\Pictures\\Client\\Shot.PNG");
        let source = Source::File {
            path: path.clone(),
            frame: 2,
            modified: None,
            len: 0,
        };
        // Numbers no other test uses, since the registry is process-wide.
        preserve(small_image(4, 3, 7), 900_001, source.clone());
        preserve(small_image(4, 3, 9), 900_001, source);
        let documents = DOCUMENTS.lock().unwrap();
        let mine: Vec<&Managed> = documents.iter().filter(|d| d.id == 900_001).collect();
        assert_eq!(
            mine.len(),
            1,
            "a second preserve under the same number is ignored"
        );
        let document = mine[0];
        assert!(document.is_for(std::path::Path::new("c:\\pictures\\client\\shot.png"), 2));
        assert!(
            !document.is_for(&path, 0),
            "another frame of the file is another document"
        );
        let (hid, preserved, width, height) = document.preserved_handle();
        let frame = preserved_frame(hid, &preserved, width, height, "test").unwrap();
        assert_eq!((frame.width(), frame.height()), (4, 3));
        assert_eq!(frame.rgba[0], 7, "the first image is the one preserved");
    }
}
