//! Folder navigation: previous and next through the supported files in the folder the
//! opened file came from (§3.4, S1.4).
//!
//! The listing is read once, when a file is opened from a folder that is not the current
//! context, and never rescanned on a keystroke. The order is Windows' own logical order,
//! `StrCmpLogicalW`, numeric-aware and case-insensitive, so `img2` sorts before `img10`
//! and the sequence matches Explorer's (part 5). A file that has disappeared since the
//! listing is skipped with a note, never an error.
//!
//! This is one of two lists the editor can walk. Recon's own history is the other (S1.9),
//! and the two never move each other; the context is named in the image info so the page
//! can say which one is active.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::Win32::UI::Shell::StrCmpLogicalW;

/// The folder being walked: its supported files in order, and where the opened one sits.
#[derive(Debug, Clone)]
pub struct Context {
    pub dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub index: usize,
}

impl Context {
    /// "12 of 240", as the indicator shows it: one-based.
    pub fn position(&self) -> (u32, u32) {
        (self.index as u32 + 1, self.files.len() as u32)
    }

    pub fn contains(&self, path: &Path) -> Option<usize> {
        self.files.iter().position(|p| same_file(p, path))
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .eq_ignore_ascii_case(&b.to_string_lossy())
}

fn supported(path: &Path) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    let ext = format!(".{}", ext.to_string_lossy().to_ascii_lowercase());
    crate::registration::EXTENSIONS.contains(&ext.as_str())
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Windows' logical order: numeric-aware and case-insensitive, the order Explorer shows.
pub fn logical_cmp(a: &str, b: &str) -> Ordering {
    let (wa, wb) = (wide(a), wide(b));
    let result = unsafe { StrCmpLogicalW(PCWSTR(wa.as_ptr()), PCWSTR(wb.as_ptr())) };
    result.cmp(&0)
}

/// The names in logical order, which is what the listing and the tests share.
pub fn sort_logical(names: &mut [PathBuf]) {
    names.sort_by(|a, b| {
        logical_cmp(
            &a.file_name().unwrap_or_default().to_string_lossy(),
            &b.file_name().unwrap_or_default().to_string_lossy(),
        )
    });
}

/// Reads the opened file's folder once and places the file in it. None when the folder
/// cannot be listed or the file is not among its supported files.
pub fn build(opened: &Path) -> Option<Context> {
    let dir = opened.parent()?.to_path_buf();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && supported(p))
        .collect();
    sort_logical(&mut files);
    let index = files.iter().position(|p| same_file(p, opened))?;
    Some(Context { dir, files, index })
}

/// Which neighbour to go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Previous,
    Next,
    First,
    Last,
}

impl Context {
    /// The index a step lands on, or None at an end: the ends stop, they do not wrap.
    pub fn target(&self, step: Step) -> Option<usize> {
        let last = self.files.len().checked_sub(1)?;
        match step {
            Step::Previous => self.index.checked_sub(1),
            Step::Next => (self.index < last).then_some(self.index + 1),
            Step::First => (self.index != 0).then_some(0),
            Step::Last => (self.index != last).then_some(last),
        }
    }

    /// Drops a file that has disappeared since the listing, keeping the index on the same
    /// neighbour. Returns whether anything was removed.
    pub fn forget(&mut self, at: usize) -> bool {
        if at >= self.files.len() {
            return false;
        }
        self.files.remove(at);
        if at < self.index || self.index >= self.files.len() {
            self.index = self.index.saturating_sub(1);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_order_is_numeric_aware_and_case_insensitive() {
        let mut names: Vec<PathBuf> = ["img10.png", "img2.png", "IMG1.jpg", "b.gif", "a.webp"]
            .iter()
            .map(PathBuf::from)
            .collect();
        sort_logical(&mut names);
        let got: Vec<String> = names
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            got,
            ["a.webp", "b.gif", "IMG1.jpg", "img2.png", "img10.png"]
        );
    }

    #[test]
    fn the_ends_stop_and_a_gone_file_is_forgotten_in_place() {
        let files: Vec<PathBuf> = ["a", "b", "c", "d"].iter().map(PathBuf::from).collect();
        let mut ctx = Context {
            dir: PathBuf::from("."),
            files,
            index: 0,
        };
        assert_eq!(ctx.target(Step::Previous), None);
        assert_eq!(ctx.target(Step::Next), Some(1));
        assert_eq!(ctx.target(Step::Last), Some(3));
        assert_eq!(ctx.position(), (1, 4));
        // "b" vanished while stepping from "a": forgetting it keeps "a" current and the
        // next step lands on "c".
        assert!(ctx.forget(1));
        assert_eq!(ctx.index, 0);
        assert_eq!(ctx.target(Step::Next), Some(1));
        assert_eq!(ctx.files[1], PathBuf::from("c"));
        // The current file itself vanishing moves the index back to a real file.
        ctx.index = 2;
        assert!(ctx.forget(2));
        assert_eq!(ctx.index, 1);
        assert_eq!(ctx.position(), (2, 2));
    }

    #[test]
    fn only_supported_types_count() {
        assert!(supported(Path::new("C:\\x\\A.PNG")));
        assert!(supported(Path::new("C:\\x\\a.heic")));
        assert!(!supported(Path::new("C:\\x\\notes.txt")));
        assert!(!supported(Path::new("C:\\x\\noext")));
    }
}
