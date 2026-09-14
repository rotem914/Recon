//! The document store (S1.8, part 5): one folder per document under the store root, holding
//! the preserved image as `source.png`, written once and never rewritten, and
//! `document.json` with a schema version, rewritten on every save. Every write goes to a
//! temporary file beside its target and is renamed into place (QA §5), so a document on
//! disk is always whole: the old one or the new one, never half of either.
//!
//! The root is set once at startup: the user's local application data in the product,
//! since documents are large and machine-local, and a folder beside the executable for the
//! checks, so a check never writes into the user's own documents. There is no index; the
//! folders are the index, and a scan reads every `document.json` it can parse.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA: u32 = 1;

static ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Where the product keeps its documents: `%LOCALAPPDATA%\Recon\documents`.
pub fn product_root() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("Recon").join("documents"))
}

pub fn set_root(path: PathBuf) {
    if let Ok(mut slot) = ROOT.lock() {
        *slot = Some(path);
    }
}

pub fn root() -> Result<PathBuf, String> {
    ROOT.lock()
        .ok()
        .and_then(|slot| slot.clone())
        .ok_or_else(|| "the store has no root".to_string())
}

pub fn folder(id: u64) -> Result<PathBuf, String> {
    Ok(root()?.join(id.to_string()))
}

/// Recon's own trash, beside the documents: a deleted document's folder moves here whole
/// and is swept after thirty days (S2.5, Rotem's call on 2026-09-14). Never the system's
/// recycle bin, whose emptying is not ours to time.
pub fn trash_root() -> Result<PathBuf, String> {
    let root = root()?;
    Ok(root
        .parent()
        .map(|p| p.join("trash"))
        .unwrap_or_else(|| root.join("trash")))
}

pub const TRASH_DAYS: u64 = 30;

/// Moves a document's folder into the trash, whole, with the time it was trashed written
/// beside it. A folder that is not on disk yet is nothing to move.
pub fn trash(id: u64) -> Result<Option<PathBuf>, String> {
    let from = folder(id)?;
    if !from.is_dir() {
        return Ok(None);
    }
    let trash = trash_root()?;
    std::fs::create_dir_all(&trash)
        .map_err(|err| format!("{} could not be created: {err}", trash.display()))?;
    let to = trash.join(id.to_string());
    if to.exists() {
        let _ = std::fs::remove_dir_all(&to);
    }
    std::fs::rename(&from, &to)
        .map_err(|err| format!("{} could not be moved to the trash: {err}", from.display()))?;
    let stamp = format!("{}", millis(SystemTime::now()));
    write_atomic(&to.join("trashed"), stamp.as_bytes())?;
    Ok(Some(to))
}

/// Moves a trashed document's folder back among the documents, the stamp removed (S2.7).
pub fn restore(id: u64) -> Result<PathBuf, String> {
    let from = trash_root()?.join(id.to_string());
    if !from.is_dir() {
        return Err(format!("document {id} is not in the trash"));
    }
    let to = folder(id)?;
    if to.exists() {
        return Err(format!("document {id} is already among the documents"));
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("{} could not be created: {err}", parent.display()))?;
    }
    std::fs::rename(&from, &to)
        .map_err(|err| format!("{} could not be moved back: {err}", from.display()))?;
    let _ = std::fs::remove_file(to.join("trashed"));
    Ok(to)
}

/// What the trash holds: each folder's number and how many days it has been there.
pub fn trash_list() -> Vec<(u64, u64)> {
    let Ok(trash) = trash_root() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&trash) else {
        return Vec::new();
    };
    let now = millis(SystemTime::now());
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        let Some(id) = dir
            .file_name()
            .and_then(|n| n.to_string_lossy().parse::<u64>().ok())
        else {
            continue;
        };
        let trashed = std::fs::read_to_string(dir.join("trashed"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .or_else(|| {
                std::fs::metadata(&dir)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(millis)
            })
            .unwrap_or(now);
        found.push((id, now.saturating_sub(trashed) / 86_400_000));
    }
    found
}

/// Removes, for good, every trashed document older than the limit. Returns the numbers
/// removed. Run at startup, so a day of use costs nothing here.
pub fn sweep_trash(days: u64) -> Vec<u64> {
    let Ok(trash) = trash_root() else {
        return Vec::new();
    };
    let mut removed = Vec::new();
    for (id, age) in trash_list() {
        if age >= days && std::fs::remove_dir_all(trash.join(id.to_string())).is_ok() {
            removed.push(id);
        }
    }
    removed
}

pub fn millis(at: SystemTime) -> u64 {
    at.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn from_millis(ms: u64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::from_millis(ms)
}

/// Where a document's image came from, as it is written. `path` is the file's full path as
/// it was opened; `frame` the frame or page that was on screen at Annotate (§3.8).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SourceRecord {
    Capture,
    File {
        path: String,
        frame: u32,
        modified_ms: Option<u64>,
        len: u64,
    },
}

/// One `document.json`. The notes are the page's own object, carried whole and never
/// interpreted by the host, so the document format is not coupled to anything here.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Record {
    pub schema: u32,
    pub id: u64,
    pub created_ms: u64,
    pub modified_ms: u64,
    pub width: u32,
    pub height: u32,
    pub source: SourceRecord,
    pub notes: serde_json::Value,
}

/// Writes the bytes to a temporary file beside the target and renames it into place.
pub fn write_atomic(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("{} has no folder", target.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|err| format!("{} could not be created: {err}", parent.display()))?;
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let temporary = parent.join(format!("{name}.{}.tmp", std::process::id()));
    std::fs::write(&temporary, bytes)
        .map_err(|err| format!("{} could not be written: {err}", temporary.display()))?;
    std::fs::rename(&temporary, target).map_err(|err| {
        let _ = std::fs::remove_file(&temporary);
        format!("{} could not be put in place: {err}", target.display())
    })
}

/// The preserved image, written once. A second call for the same document changes nothing
/// and returns the existing path: the source is never rewritten.
pub fn write_source(id: u64, png: &[u8]) -> Result<PathBuf, String> {
    let path = folder(id)?.join("source.png");
    if path.is_file() {
        return Ok(path);
    }
    write_atomic(&path, png)?;
    Ok(path)
}

pub fn write_record(record: &Record) -> Result<(), String> {
    let path = folder(record.id)?.join("document.json");
    let text = serde_json::to_vec_pretty(record).map_err(|err| err.to_string())?;
    write_atomic(&path, &text)
}

pub fn read_record(dir: &Path) -> Result<Record, String> {
    let path = dir.join("document.json");
    let text = std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
    let record: Record =
        serde_json::from_slice(&text).map_err(|err| format!("{}: {err}", path.display()))?;
    if record.schema > SCHEMA {
        return Err(format!(
            "{}: schema {} is newer than this build's {SCHEMA}",
            path.display(),
            record.schema
        ));
    }
    Ok(record)
}

/// Every document on disk that parses, with its image's path; one that does not parse is
/// named on the log and left where it is, never deleted.
pub fn scan() -> Vec<(Record, PathBuf)> {
    let Ok(root) = root() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        match read_record(&dir) {
            Ok(record) => {
                let source = dir.join("source.png");
                if source.is_file() {
                    found.push((record, source));
                } else {
                    println!("store: {} has no source.png, left alone", dir.display());
                }
            }
            Err(err) => println!("store: skipped {err}"),
        }
    }
    found.sort_by_key(|(record, _)| record.modified_ms);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_write_is_whole_and_the_source_is_never_rewritten() {
        // Under the crate's own build folder, never the system's temporary one (rule 12).
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("store-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        set_root(dir.clone());
        let first = write_source(42, b"first").unwrap();
        let second = write_source(42, b"second").unwrap();
        assert_eq!(first, second);
        assert_eq!(std::fs::read(&first).unwrap(), b"first");
        let record = Record {
            schema: SCHEMA,
            id: 42,
            created_ms: 1,
            modified_ms: 2,
            width: 4,
            height: 3,
            source: SourceRecord::Capture,
            notes: serde_json::json!({ "callouts": [] }),
        };
        write_record(&record).unwrap();
        let mut again = record.clone();
        again.modified_ms = 3;
        write_record(&again).unwrap();
        let back = read_record(&folder(42).unwrap()).unwrap();
        assert_eq!(back.modified_ms, 3);
        assert!(std::fs::read_dir(folder(42).unwrap())
            .unwrap()
            .flatten()
            .all(|e| !e.path().to_string_lossy().ends_with(".tmp")));
        assert_eq!(scan().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// S2.8's measurement, run by hand: what the startup scan costs at a year of
    /// thousands of captures a month. Ignored because it writes sixty thousand files.
    /// `cargo test --features stage0-checks -- --ignored --nocapture thirty_thousand`
    #[test]
    #[ignore]
    fn thirty_thousand_documents_scan_in() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("store-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        set_root(dir.clone());
        let started = std::time::Instant::now();
        for id in 1..=30_000u64 {
            write_source(id, b"not a png, the scan only asks whether it exists").unwrap();
            write_record(&Record {
                schema: SCHEMA,
                id,
                created_ms: id,
                modified_ms: id,
                width: 1920,
                height: 1080,
                source: SourceRecord::Capture,
                notes: serde_json::json!({ "callouts": [{ "id": "c1", "text": "a note of ordinary length" }] }),
            })
            .unwrap();
        }
        println!("written in {:?}", started.elapsed());
        for pass in 1..=3 {
            let started = std::time::Instant::now();
            let found = scan();
            println!(
                "scan {pass}: {} documents in {:?}",
                found.len(),
                started.elapsed()
            );
            assert_eq!(found.len(), 30_000);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
