//! Save As (§3.6, S1.11): a PNG of the composition to a NEW file, every time. The suggested
//! name is derived from the source and marked as annotated, the folder is the last one
//! exported to, and a name that exists is never written over: an available one is offered
//! instead. There is no overwrite path here, and rule 11 is why.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The last folder a file was saved into, for the next dialog; the user's Pictures folder
/// until then.
static LAST_FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn folder() -> PathBuf {
    if let Some(last) = LAST_FOLDER.lock().ok().and_then(|slot| slot.clone()) {
        if last.is_dir() {
            return last;
        }
    }
    default_folder()
}

pub fn remember_folder(folder: &Path) {
    if let Ok(mut slot) = LAST_FOLDER.lock() {
        *slot = Some(folder.to_path_buf());
    }
}

fn default_folder() -> PathBuf {
    let profile = std::env::var_os("USERPROFILE").map(PathBuf::from);
    match profile {
        Some(profile) if profile.join("Pictures").is_dir() => profile.join("Pictures"),
        Some(profile) => profile,
        None => PathBuf::from("."),
    }
}

/// The local wall clock as `YYYY-MM-DD HH-MM-SS`, for a capture's name.
fn local_stamp() -> String {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!(
        "{:04}-{:02}-{:02} {:02}-{:02}-{:02}",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
    )
}

/// The suggested name: the source file's stem marked as annotated, or the capture's time
/// marked the same way. Never the source's own name, so an original is never the default
/// target.
pub fn suggested_name(source_file: Option<&Path>) -> String {
    match source_file.and_then(|p| p.file_stem()) {
        Some(stem) => format!("{} annotated.png", stem.to_string_lossy()),
        None => format!("capture {} annotated.png", local_stamp()),
    }
}

/// The first of `name`, `name (2)`, `name (3)`... that does not exist in `folder`.
pub fn available(folder: &Path, name: &str) -> PathBuf {
    let first = folder.join(name);
    if !first.exists() {
        return first;
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 2..10_000 {
        let candidate = folder.join(format!("{stem} ({n}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    folder.join(format!(
        "{stem} ({}){ext}",
        crate::store::millis(std::time::SystemTime::now())
    ))
}

/// What a write attempt came to.
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub enum Written {
    /// The file was created with these bytes.
    New(PathBuf),
    /// That name exists; nothing was written, and this one is free.
    Exists { chosen: PathBuf, offered: PathBuf },
}

/// Writes `png` to `path` only if nothing is there: the file is created new, never
/// truncated, and a write that fails midway is removed so no half file is left.
pub fn write_new(path: &Path, png: &[u8]) -> Result<Written, String> {
    use std::io::Write;
    let folder = path.parent().unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    std::fs::create_dir_all(folder)
        .map_err(|err| format!("{} could not be created: {err}", folder.display()))?;
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(Written::Exists {
                chosen: path.to_path_buf(),
                offered: available(folder, &name),
            });
        }
        Err(err) => return Err(format!("{} could not be created: {err}", path.display())),
    };
    if let Err(err) = file.write_all(png).and_then(|_| file.flush()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(format!("{} could not be written: {err}", path.display()));
    }
    remember_folder(folder);
    Ok(Written::New(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_is_derived_and_marked_and_never_the_source() {
        let source = Path::new("C:\\Client\\Header photo.png");
        assert_eq!(suggested_name(Some(source)), "Header photo annotated.png");
        let capture = suggested_name(None);
        assert!(capture.starts_with("capture ") && capture.ends_with(" annotated.png"));
    }

    #[test]
    fn an_existing_name_is_never_written_and_a_free_one_is_offered() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("export-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a annotated.png");
        assert_eq!(
            write_new(&path, b"first").unwrap(),
            Written::New(path.clone())
        );
        let second = write_new(&path, b"second").unwrap();
        assert_eq!(
            second,
            Written::Exists {
                chosen: path.clone(),
                offered: dir.join("a annotated (2).png")
            }
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        std::fs::write(dir.join("a annotated (2).png"), b"x").unwrap();
        assert_eq!(
            available(&dir, "a annotated.png"),
            dir.join("a annotated (3).png")
        );
        assert_eq!(folder(), dir);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
