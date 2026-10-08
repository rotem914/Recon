//! The timing log (2026-10-07): the lines that time a capture on its way to the clipboard,
//! kept in a file, because a product run's standard output goes nowhere. A slow day then
//! shows which step was slow: the S0.7 marks, each image's encode, the page's showing and
//! its copy with the layer's export, and the host's compose, encode and publish, each line
//! stamped with the time it was made, in milliseconds since 1970, the clock a document's
//! number is read on.
//!
//! Only those lines: the rest of the host's log, and the page's other lines and errors,
//! name the window a capture began in and the files opened, and stay on the standard
//! output. The lines are handed to one writer thread of their own, so no capture thread,
//! and not the main one, ever waits on the disk. Past its cap the file is renamed to the one
//! older file kept beside it, replacing it, and a new one begins; a file deleted or moved
//! while Recon runs is begun again at its place. Nothing here ever stops Recon: a line that
//! cannot be written is dropped, the writer tries again a little later, and the file says
//! how many lines it lost when it is back. Lines still on their way when Recon quits are
//! lost without a count: a few milliseconds' worth.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// How large the file grows before it is renamed to the older one: about a month of
/// captures at a hundred a day, at about a dozen lines each.
pub const CAP: u64 = 4 * 1024 * 1024;

/// How long the writer waits, after a line it could not write, before it tries again.
const RETRY: Duration = Duration::from_secs(30);

/// The product's file: `%LOCALAPPDATA%\Recon\timing.log`, beside the documents.
#[cfg(not(feature = "stage0-checks"))]
pub fn product_path() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .map(|dir| PathBuf::from(dir).join("Recon").join("timing.log"))
        .ok_or_else(|| "LOCALAPPDATA is not set".to_string())
}

/// A checks build's file, beside its executable, so no check run writes into the user's
/// folder.
#[cfg(feature = "stage0-checks")]
pub fn checks_path() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map(|exe| exe.with_file_name("s-timing.log"))
        .map_err(|err| format!("the executable's path is unknown: {err}"))
}

/// The older file kept beside `path`: `timing.old.log` for `timing.log`.
pub fn older(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "timing".into());
    path.with_file_name(format!("{stem}.old.log"))
}

fn open_append(path: &Path) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("{}: {err}", path.display()))
}

/// Whether the file's last byte is not a line break: a write that failed halfway left part
/// of a line there.
fn ends_mid_line(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    if file.seek(SeekFrom::End(-1)).is_err() {
        return false;
    }
    let mut last = [0u8; 1];
    file.read_exact(&mut last).is_ok() && last[0] != b'\n'
}

/// One log: where it lives, the file while it is open, how much it holds, and its cap.
pub struct Sink {
    path: PathBuf,
    file: Option<std::fs::File>,
    len: u64,
    cap: u64,
}

impl Sink {
    /// Opens the file for appending, made with its folder when missing; a file already past
    /// the cap is renamed to the older one first, and a line a failed write left half done
    /// is ended, so the next entry starts a line of its own.
    pub fn open(path: PathBuf, cap: u64) -> Result<Sink, String> {
        let mut sink = Sink {
            path,
            file: None,
            len: 0,
            cap,
        };
        sink.reopen()?;
        Ok(sink)
    }

    fn reopen(&mut self) -> Result<(), String> {
        self.file = None;
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
        }
        let len = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let mut len = if len >= self.cap {
            Self::retire(&self.path, self.cap)
        } else {
            len
        };
        let mut file = open_append(&self.path)?;
        if ends_mid_line(&self.path) {
            file.write_all(b"\n")
                .map_err(|err| format!("{}: {err}", self.path.display()))?;
            len += 1;
        }
        self.file = Some(file);
        self.len = len;
        Ok(())
    }

    /// Renames the file to the older one, which the rename replaces, and says what the count
    /// starts again from. Nothing when the rename went, or when there was no file to rename.
    /// When another program holding the file refused it, both files stay as they were and the
    /// count sits a sixteenth below the cap, so the next try comes soon rather than a whole
    /// cap later.
    fn retire(path: &Path, cap: u64) -> u64 {
        match std::fs::rename(path, older(path)) {
            Ok(()) => 0,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => 0,
            Err(_) => cap - cap / 16,
        }
    }

    /// Appends one line, a line break inside it folded to a space so every line of the file
    /// is one stamped entry. A file gone from its place, deleted or moved meanwhile, is
    /// begun again there first. Past the cap, the file is renamed to the older one and a new
    /// one opened; a new one that will not open is tried again at the next line, this line
    /// being in the older file already.
    pub fn write(&mut self, line: &str) -> Result<(), String> {
        if self.file.is_none() || !self.path.exists() {
            self.reopen()?;
        }
        let Some(file) = self.file.as_mut() else {
            return Err(format!("{}: not open", self.path.display()));
        };
        let mut text: String = line
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .collect();
        text.push('\n');
        file.write_all(text.as_bytes())
            .map_err(|err| format!("{}: {err}", self.path.display()))?;
        self.len += text.len() as u64;
        if self.len >= self.cap {
            self.len = Self::retire(&self.path, self.cap);
            if self.len == 0 {
                self.file = open_append(&self.path).ok();
            }
        }
        Ok(())
    }
}

/// The writer's side: lines arrive here and are written in order. With no file, or after a
/// line that could not be written, the lines are counted as lost until `retry` has passed,
/// and the file is opened afresh then; the first line written after it is a note of how
/// many were lost.
fn writer(path: PathBuf, sink: Option<Sink>, lines: Receiver<String>, retry: Duration, cap: u64) {
    let mut sink = sink;
    let mut lost: u64 = 0;
    let mut resume_at: Option<Instant> = None;
    for line in lines {
        if let Some(at) = resume_at {
            if Instant::now() < at {
                lost += 1;
                continue;
            }
            resume_at = None;
        }
        if sink.is_none() {
            sink = Sink::open(path.clone(), cap).ok();
        }
        let Some(open) = sink.as_mut() else {
            lost += 1;
            resume_at = Some(Instant::now() + retry);
            continue;
        };
        if lost > 0 {
            let note = format!(
                "{} | the timing log lost {lost} lines before this one",
                stamp()
            );
            if open.write(&note).is_err() {
                lost += 1;
                sink = None;
                resume_at = Some(Instant::now() + retry);
                continue;
            }
            lost = 0;
        }
        if open.write(&line).is_err() {
            lost += 1;
            sink = None;
            resume_at = Some(Instant::now() + retry);
        }
    }
}

fn stamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

static LINES: OnceLock<Sender<String>> = OnceLock::new();

/// Starts the run's timing log at `path`, once, with its writer thread. The file is opened
/// here, so a failure is said by the caller; the writer then starts without it and tries
/// again every `RETRY` while lines come.
pub fn open(path: PathBuf) -> Result<PathBuf, String> {
    if LINES.get().is_some() {
        return Err("the timing log is open already".into());
    }
    let (sink, failed) = match Sink::open(path.clone(), CAP) {
        Ok(sink) => (Some(sink), None),
        Err(err) => (None, Some(err)),
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    let shown = path.clone();
    std::thread::Builder::new()
        .name("timing log".into())
        .spawn(move || writer(path, sink, receiver, RETRY, CAP))
        .map_err(|err| format!("the timing log's writer did not start: {err}"))?;
    let _ = LINES.set(sender);
    match failed {
        None => Ok(shown),
        Some(err) => Err(format!(
            "{err}; tried again every {} s while lines come",
            RETRY.as_secs()
        )),
    }
}

/// Hands one stamped line to the writer, if the run has a timing log. Never waits on the
/// disk.
pub fn write(line: &str) {
    if let Some(lines) = LINES.get() {
        let _ = lines.send(line.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{older, writer, Sink};

    /// A folder of the test's own, under the test executable's, inside the project, named
    /// by this process too, so two test runs at once never share one.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::current_exe()
            .expect("the test executable")
            .with_file_name(format!("logfile-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn lines_are_appended_and_kept_across_a_reopen() {
        let dir = scratch("append");
        let path = dir.join("timing.log");
        let mut sink = Sink::open(path.clone(), 1_000_000).expect("opened");
        sink.write("1 | first").expect("written");
        sink.write("2 | second\nwith a break").expect("written");
        drop(sink);
        let mut sink = Sink::open(path.clone(), 1_000_000).expect("opened again");
        sink.write("3 | third").expect("written");
        drop(sink);
        let text = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(text, "1 | first\n2 | second with a break\n3 | third\n");
        assert!(!older(&path).exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn past_the_cap_the_file_becomes_the_older_one_and_a_new_one_begins() {
        let dir = scratch("cap");
        let path = dir.join("timing.log");
        let mut sink = Sink::open(path.clone(), 20).expect("opened");
        sink.write("0123456789").expect("written"); // 11 bytes
        sink.write("abcdefghij").expect("written"); // 22 bytes: past the cap, retired
        sink.write("after").expect("written");
        drop(sink);
        assert_eq!(
            std::fs::read_to_string(older(&path)).expect("the older file"),
            "0123456789\nabcdefghij\n"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("the new file"),
            "after\n"
        );
        // A second retirement replaces the older file, so two files at most.
        let mut sink = Sink::open(path.clone(), 20).expect("opened");
        sink.write("0123456789abcdefghij").expect("written");
        drop(sink);
        assert_eq!(
            std::fs::read_to_string(older(&path)).expect("the older file"),
            "after\n0123456789abcdefghij\n"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("the new file"), "");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_already_past_the_cap_is_retired_at_open() {
        let dir = scratch("open");
        let path = dir.join("timing.log");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "x".repeat(50)).unwrap();
        let mut sink = Sink::open(path.clone(), 20).expect("opened");
        sink.write("fresh").expect("written");
        drop(sink);
        assert_eq!(
            std::fs::read_to_string(older(&path)).unwrap(),
            "x".repeat(50)
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "fresh\n");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Another program holding the file without letting it be renamed, as a log viewer or a
    /// backup can: the older file is kept, writing goes on, and the next try comes a
    /// sixteenth of the cap later, not a whole cap.
    #[test]
    fn a_rename_refused_keeps_both_files_and_tries_again_soon() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = scratch("held");
        let path = dir.join("timing.log");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(older(&path), "the older one\n").unwrap();
        let mut sink = Sink::open(path.clone(), 160).expect("opened");
        // Read and write shared, delete not: a rename of the file is refused while held.
        let holder = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(&path)
            .expect("held");
        sink.write(&"a".repeat(170)).expect("written past the cap");
        assert_eq!(
            std::fs::read_to_string(older(&path)).unwrap(),
            "the older one\n",
            "the older file survives a refused rename"
        );
        sink.write("still written").expect("written while held");
        drop(holder);
        // The next line still past the cap tries the rename again, and it goes now.
        sink.write(&"b".repeat(15)).expect("written");
        assert!(std::fs::read_to_string(older(&path))
            .unwrap()
            .contains("still written"));
        sink.write("after").expect("written");
        drop(sink);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "after\n");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// The file deleted while Recon writes to it, as a person cleaning the folder would:
    /// the next line begins it again at its place.
    #[test]
    fn a_file_deleted_meanwhile_is_begun_again() {
        let dir = scratch("deleted");
        let path = dir.join("timing.log");
        let mut sink = Sink::open(path.clone(), 1_000_000).expect("opened");
        sink.write("before").expect("written");
        std::fs::remove_file(&path).expect("deleted while open");
        sink.write("after").expect("written");
        drop(sink);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "after\n");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// A line a failed write left half done is ended at the next open, so the next entry
    /// is a line of its own.
    #[test]
    fn a_half_line_left_by_a_failed_write_is_ended_at_open() {
        let dir = scratch("half");
        let path = dir.join("timing.log");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "1 | whole\n2 | ha").unwrap();
        let mut sink = Sink::open(path.clone(), 1_000_000).expect("opened");
        sink.write("3 | next").expect("written");
        drop(sink);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "1 | whole\n2 | ha\n3 | next\n"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    /// The writer started with no file, its folder blocked by a file of that name: the lines
    /// are counted as lost; once the place is free, the next line is written after a note of
    /// how many were lost, and the count starts again.
    #[test]
    fn the_writer_counts_lost_lines_and_says_so_when_the_file_is_back() {
        let dir = scratch("writer");
        std::fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("logs");
        std::fs::write(&blocker, "a file where the folder should be").unwrap();
        let path = blocker.join("timing.log");
        assert!(Sink::open(path.clone(), 1_000_000).is_err());
        let (send, receive) = std::sync::mpsc::channel();
        let target = path.clone();
        let thread = std::thread::spawn(move || {
            writer(target, None, receive, std::time::Duration::ZERO, 1_000_000)
        });
        send.send("1 | lost".to_string()).unwrap();
        send.send("2 | lost".to_string()).unwrap();
        // The two lines are taken in order; wait until both have been tried and failed.
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::fs::remove_file(&blocker).unwrap();
        send.send("3 | kept".to_string()).unwrap();
        send.send("4 | kept".to_string()).unwrap();
        drop(send);
        thread.join().unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "{text}");
        assert!(lines[0].ends_with(" | the timing log lost 2 lines before this one"));
        assert_eq!(lines[1], "3 | kept");
        assert_eq!(lines[2], "4 | kept");
        let _ = std::fs::remove_dir_all(dir);
    }
}
