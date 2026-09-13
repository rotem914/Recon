//! S0.7: captures through the product path, the marks of each, and memory as a floor and a
//! slope. Run with `--measure <runs> [--walk <folder>]` on a release build.
//!
//! The process runs as it does in the tray: hotkey registered, editor created hidden. This
//! thread presses the configured hotkey with synthesized input, waits for the overlay to be
//! usable, drags the whole primary display, waits for the editor to be usable, looks at the
//! screen, samples memory, hides the editor, and goes again. The drag is inside neither
//! interval, because both are read from the marks, never from this thread's own waiting.
//!
//! Every step that would send input to another application is guarded: nothing is pressed
//! unless the hotkey is registered, and nothing is dragged unless the overlay said it is on
//! screen and taking input. A run that cannot continue stops the whole measurement.
//!
//! An attempt someone else's input touched is not counted. Before each one the harness waits
//! for ten seconds of quiet on the keyboard and mouse if anyone used them, and it goes on
//! until it has enough clean runs. The first attempt at this found why: a person at the
//! machine moved the pointer mid-drag and took the focus, and five runs lost their marks.

use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, GetLastInputInfo, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, LASTINPUTINFO, VK_CONTROL,
    VK_ESCAPE, VK_MENU, VK_SHIFT,
};

use crate::capture::coords::DesktopRect;
use crate::capture::screen::copy_rect;
use crate::marks;

const MB: f64 = 1024.0 * 1024.0;

/// This process and the web view processes it started, in bytes.
#[derive(Clone, Copy, Default)]
struct Memory {
    host_private: usize,
    host_working: usize,
    web_private: usize,
    web_working: usize,
    web_processes: usize,
    image: usize,
    levels: usize,
}

impl Memory {
    fn total_private(&self) -> usize {
        self.host_private + self.web_private
    }

    fn csv(&self) -> String {
        format!(
            "{:.1},{:.1},{:.1},{:.1},{},{:.1},{:.1}",
            self.host_private as f64 / MB,
            self.host_working as f64 / MB,
            self.web_private as f64 / MB,
            self.web_working as f64 / MB,
            self.web_processes,
            self.image as f64 / MB,
            self.levels as f64 / MB
        )
    }

    fn line(&self) -> String {
        format!(
            "private {:.1} MB (host {:.1}, web view {:.1} in {} processes), working set host {:.1} MB and web view {:.1} MB; the live image {:.1} MB plus {:.1} MB of levels",
            self.total_private() as f64 / MB,
            self.host_private as f64 / MB,
            self.web_private as f64 / MB,
            self.web_processes,
            self.host_working as f64 / MB,
            self.web_working as f64 / MB,
            self.image as f64 / MB,
            self.levels as f64 / MB
        )
    }
}

const MEMORY_HEADER: &str =
    "host_private_mb,host_working_mb,web_private_mb,web_working_mb,web_processes,image_mb,levels_mb";

fn counters(pid: u32) -> Option<(usize, usize)> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut c = PROCESS_MEMORY_COUNTERS_EX {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            ..Default::default()
        };
        let ok = GetProcessMemoryInfo(
            handle,
            &mut c as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
            c.cb,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        ok.then_some((c.PrivateUsage, c.WorkingSetSize))
    }
}

/// Every process descended from `root`: for this host, the web view's browser process and
/// the renderer, GPU and utility processes it starts.
fn descendants(root: u32) -> Vec<u32> {
    let mut pairs = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return Vec::new();
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                pairs.push((entry.th32ProcessID, entry.th32ParentProcessID));
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    let mut found = vec![root];
    let mut i = 0;
    while i < found.len() {
        let parent = found[i];
        for &(pid, ppid) in &pairs {
            if ppid == parent && pid != parent && !found.contains(&pid) {
                found.push(pid);
            }
        }
        i += 1;
    }
    found.remove(0);
    found
}

fn memory() -> Memory {
    let me = std::process::id();
    let (host_private, host_working) = counters(me).unwrap_or_default();
    let (image, levels) = crate::editor::live_bytes();
    let mut m = Memory {
        host_private,
        host_working,
        image,
        levels,
        ..Default::default()
    };
    for pid in descendants(me) {
        if let Some((private, working)) = counters(pid) {
            m.web_private += private;
            m.web_working += working;
            m.web_processes += 1;
        }
    }
    m
}

/// The configured hotkey as virtual keys: modifiers, then the one key.
fn hotkey_keys(text: &str) -> Option<(Vec<u8>, u8)> {
    let mut modifiers = Vec::new();
    let mut key = None;
    for part in text.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers.push(VK_CONTROL.0 as u8),
            "shift" => modifiers.push(VK_SHIFT.0 as u8),
            "alt" => modifiers.push(VK_MENU.0 as u8),
            other if other.len() == 1 && other.chars().all(|c| c.is_ascii_alphanumeric()) => {
                key = Some(other.to_ascii_uppercase().as_bytes()[0]);
            }
            _ => return None,
        }
    }
    Some((modifiers, key?))
}

fn press_hotkey(modifiers: &[u8], key: u8) {
    unsafe {
        for &m in modifiers {
            keybd_event(m, 0, KEYBD_EVENT_FLAGS(0), 0);
        }
        keybd_event(key, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(key, 0, KEYEVENTF_KEYUP, 0);
        for &m in modifiers.iter().rev() {
            keybd_event(m, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

/// The system's tick of the last keyboard or mouse input from anyone, this run's included.
fn last_input() -> u32 {
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        let _ = GetLastInputInfo(&mut info);
    }
    info.dwTime
}

/// Waits until nobody has touched the keyboard or mouse for `quiet`, giving up after
/// `limit`. This harness sends nothing while it waits, so any change is a person.
pub(crate) fn wait_for_quiet(quiet: Duration, limit: Duration) -> bool {
    let started = Instant::now();
    let mut seen = last_input();
    let mut still_since = Instant::now();
    while started.elapsed() < limit {
        std::thread::sleep(Duration::from_millis(250));
        let now = last_input();
        if now != seen {
            seen = now;
            still_since = Instant::now();
        }
        if still_since.elapsed() >= quiet {
            return true;
        }
    }
    false
}

fn wait_for(names: &[&str], timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        let got = marks::snapshot();
        if names.iter().all(|n| got.iter().any(|(m, _)| m == n)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    false
}

fn at(list: &[(&'static str, f64)], name: &str) -> Option<f64> {
    list.iter().find(|(m, _)| *m == name).map(|(_, t)| *t)
}

fn later(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    Some(a?.max(b?))
}

/// The editor window's pixels on the screen, sampled: how many are pure black, and how
/// many distinct coarse colours there are. A surface that never painted is uniform.
fn look_at_editor(app: &AppHandle) -> Option<(f64, usize)> {
    let window = app.get_webview_window("editor")?;
    let position = window.inner_position().ok()?;
    let size = window.inner_size().ok()?;
    let inset = 40;
    let rect = DesktopRect {
        x: position.x + inset,
        y: position.y + inset,
        width: size.width.saturating_sub(80),
        height: size.height.saturating_sub(80),
    };
    let pixels = copy_rect(rect).ok()?;
    let mut black = 0usize;
    let mut seen = 0usize;
    let mut colours = std::collections::HashSet::new();
    for px in pixels.as_chunks::<4>().0.iter().step_by(37) {
        seen += 1;
        if px[0] < 8 && px[1] < 8 && px[2] < 8 {
            black += 1;
        }
        colours.insert((px[0] >> 4, px[1] >> 4, px[2] >> 4));
    }
    Some((black as f64 / seen.max(1) as f64, colours.len()))
}

fn hide_editor(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("editor") {
        let _ = window.hide();
    }
}

/// The nearest-rank percentile: the value at rank ceil(p/100 * n) in ascending order. With
/// thirty samples the 95th is the 29th value, the second-worst.
fn nearest_rank(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted.get(rank.max(1) - 1).copied()
}

struct Report {
    lines: Vec<String>,
}

impl Report {
    fn say(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        self.lines.push(line);
    }
}

fn beside_exe(name: &str) -> std::path::PathBuf {
    std::env::current_exe()
        .map(|exe| exe.with_file_name(name))
        .unwrap_or_else(|_| name.into())
}

fn interval_summary(report: &mut Report, label: &str, values: &[f64], target: f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let raw: Vec<String> = values.iter().map(|v| format!("{v:.1}")).collect();
    report.say(format!("{label}, {} runs, ms, in run order:", values.len()));
    report.say(format!("  {}", raw.join(" ")));
    if let (Some(max), Some(p95), Some(median)) = (
        sorted.last().copied(),
        nearest_rank(&sorted, 95.0),
        nearest_rank(&sorted, 50.0),
    ) {
        report.say(format!(
            "  maximum {max:.1}, 95th percentile by nearest rank {p95:.1}, median by nearest rank {median:.1}; target {target:.0}: {}",
            if max <= target { "every run within it" } else { "NOT every run within it" }
        ));
    }
}

pub fn run(app: AppHandle, runs: usize, walk: Option<String>) {
    let mut report = Report { lines: Vec::new() };
    report.say("=== S0.7: the marks, the two intervals, and memory ===");

    // ---- startup, and the floor
    let booted = Instant::now();
    while booted.elapsed() < Duration::from_secs(30)
        && !marks::startup_snapshot()
            .iter()
            .any(|(n, _)| *n == marks::PAGE_BOOTED)
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    let startup = marks::startup_snapshot();
    report.say(format!(
        "startup, from the start of main: ready {} ms, editor page booted {} ms",
        at(&startup, marks::READY)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "never".into()),
        at(&startup, marks::PAGE_BOOTED)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "never".into()),
    ));
    std::thread::sleep(Duration::from_secs(3));
    let floor = memory();
    report.say(format!(
        "floor, 3 s idle after the page booted, editor hidden, nothing captured: {}",
        floor.line()
    ));
    let mut memory_rows = vec![format!("moment,{MEMORY_HEADER}")];
    memory_rows.push(format!("floor,{}", floor.csv()));

    let mut run_rows = vec![format!(
        "run,clean,freeze_done,overlay_accepting,overlay_displayed,selection_completed,editor_show_returned,page_painted,page_focused,overlay_usable_ms,editor_usable_ms,screen_black,screen_colours,{MEMORY_HEADER}"
    )];
    let mut overlay_usable = Vec::new();
    let mut editor_usable = Vec::new();
    let mut stopped = None;
    let mut attempts = 0usize;
    let mut counted = 0usize;
    let mut not_counted: Vec<String> = Vec::new();

    if runs > 0 {
        let hotkey = crate::config::load().hotkey;
        let keys = hotkey_keys(&hotkey);
        let primary = crate::capture::display::monitors()
            .into_iter()
            .find(|m| m.primary);
        if !crate::HOTKEY_REGISTERED.load(std::sync::atomic::Ordering::SeqCst) {
            stopped = Some(format!(
                "{hotkey} is not registered, so pressing it would reach another application"
            ));
        } else if keys.is_none() {
            stopped = Some(format!("{hotkey} is not a key this run knows how to press"));
        } else if primary.is_none() {
            stopped = Some("there is no primary display to drag across".into());
        }
        if let (None, Some((modifiers, key)), Some(primary)) = (&stopped, keys, primary) {
            let r = primary.rect;
            report.say(format!(
                "{runs} captures of the primary display, {}x{} at {}%, with {hotkey}",
                r.width, r.height, primary.scale_percent
            ));
            let mut ours = last_input();
            while counted < runs && attempts < runs + 15 {
                attempts += 1;
                let n = attempts;
                if last_input() != ours {
                    println!(
                        "attempt {n}: someone is using the keyboard or mouse; waiting for 10 s of quiet"
                    );
                    if !wait_for_quiet(Duration::from_secs(10), Duration::from_secs(300)) {
                        stopped = Some(format!(
                            "attempt {n}: the machine was in use for five minutes, so the measurement stopped"
                        ));
                        break;
                    }
                }
                let fires = crate::FIRES.load(std::sync::atomic::Ordering::SeqCst);
                press_hotkey(&modifiers, key);
                let pressed = Instant::now();
                while crate::FIRES.load(std::sync::atomic::Ordering::SeqCst) == fires
                    && pressed.elapsed() < Duration::from_secs(2)
                {
                    std::thread::sleep(Duration::from_millis(2));
                }
                if crate::FIRES.load(std::sync::atomic::Ordering::SeqCst) == fires {
                    stopped = Some(format!("run {n}: the hotkey press never arrived"));
                    break;
                }
                if !wait_for(
                    &[marks::OVERLAY_DISPLAYED, marks::OVERLAY_ACCEPTING],
                    Duration::from_secs(5),
                ) {
                    if at(&marks::snapshot(), marks::OVERLAY_DISPLAYED).is_some() {
                        unsafe {
                            keybd_event(VK_ESCAPE.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
                            keybd_event(VK_ESCAPE.0 as u8, 0, KEYEVENTF_KEYUP, 0);
                        }
                    }
                    stopped = Some(format!(
                        "run {n}: the overlay did not report itself usable, so nothing was dragged"
                    ));
                    break;
                }
                // The user's drag, which neither interval contains.
                std::thread::sleep(Duration::from_millis(120));
                crate::selftest::move_to(r.x, r.y);
                crate::selftest::press_left();
                crate::selftest::move_to(r.x + r.width as i32 / 2, r.y + r.height as i32 / 2);
                crate::selftest::move_to(r.x + r.width as i32 - 1, r.y + r.height as i32 - 1);
                crate::selftest::release_left();
                let released = last_input();

                let complete = wait_for(
                    &[
                        marks::SELECTION_COMPLETED,
                        marks::EDITOR_SHOW_RETURNED,
                        marks::PAGE_PAINTED,
                        marks::PAGE_FOCUSED,
                    ],
                    Duration::from_secs(8),
                );
                let list = marks::snapshot();
                let look = look_at_editor(&app);
                let mem = memory();
                let overlay = later(
                    at(&list, marks::OVERLAY_DISPLAYED),
                    at(&list, marks::OVERLAY_ACCEPTING),
                );
                let editor = later(
                    later(
                        at(&list, marks::EDITOR_SHOW_RETURNED),
                        at(&list, marks::PAGE_PAINTED),
                    ),
                    at(&list, marks::PAGE_FOCUSED),
                )
                .zip(at(&list, marks::SELECTION_COMPLETED))
                .map(|(end, start)| end - start);
                let size = crate::editor::editor_image_info()
                    .ok()
                    .map(|i| (i.width, i.height));
                let whole = size == Some((r.width - 1, r.height - 1));
                let touched = last_input() != released;
                let clean = complete && whole && !touched;
                let reason = if !complete {
                    "a mark never arrived"
                } else if !whole {
                    "the selection was not the dragged rectangle"
                } else if touched {
                    "someone used the keyboard or mouse during it"
                } else {
                    ""
                };
                if clean {
                    counted += 1;
                    if let Some(v) = overlay {
                        overlay_usable.push(v);
                    }
                    if let Some(v) = editor {
                        editor_usable.push(v);
                    }
                } else {
                    not_counted.push(format!("attempt {n}: {reason}"));
                }
                let cell = |v: Option<f64>| v.map(|v| format!("{v:.1}")).unwrap_or_default();
                run_rows.push(format!(
                    "{n},{clean},{},{},{},{},{},{},{},{},{},{},{},{}",
                    cell(at(&list, marks::FREEZE_DONE)),
                    cell(at(&list, marks::OVERLAY_ACCEPTING)),
                    cell(at(&list, marks::OVERLAY_DISPLAYED)),
                    cell(at(&list, marks::SELECTION_COMPLETED)),
                    cell(at(&list, marks::EDITOR_SHOW_RETURNED)),
                    cell(at(&list, marks::PAGE_PAINTED)),
                    cell(at(&list, marks::PAGE_FOCUSED)),
                    cell(overlay),
                    cell(editor),
                    look.map(|(b, _)| format!("{b:.3}")).unwrap_or_default(),
                    look.map(|(_, c)| c.to_string()).unwrap_or_default(),
                    mem.csv()
                ));
                memory_rows.push(format!("after capture {n},{}", mem.csv()));
                println!(
                    "run {n}: overlay usable {} ms, editor usable {} ms{}",
                    cell(overlay),
                    cell(editor),
                    if clean {
                        String::new()
                    } else {
                        format!(", NOT COUNTED: {reason}")
                    }
                );
                hide_editor(&app);
                std::thread::sleep(Duration::from_millis(800));
                ours = released;
            }
            if stopped.is_none() && counted < runs {
                stopped = Some(format!("only {counted} clean runs in {attempts} attempts"));
            }
        }
    }

    if let Some(reason) = &stopped {
        report.say(format!("STOPPED: {reason}"));
    }

    if runs > 0 {
        report.say("");
        report.say("intervals, read from the marks on one clock, the host's:");
        report.say("  overlay usable = the later of 'overlay displayed' (every display's overlay painted once) and 'overlay accepting input' (its own posted message dispatched), from 'hotkey received'");
        report.say("  editor usable = the latest of 'editor show returned', 'page painted' (second animation frame after the new pixels) and 'page focused' (image loaded and keyboard focus), from 'selection completed'");
        report.say(format!(
            "{counted} runs counted of {attempts} attempts; an attempt is not counted when a mark never arrived, the selection was not the dragged rectangle, or someone used the keyboard or mouse during it"
        ));
        for line in &not_counted {
            report.say(format!("  {line}"));
        }
        interval_summary(
            &mut report,
            "hotkey received to overlay usable",
            &overlay_usable,
            250.0,
        );
        interval_summary(
            &mut report,
            "selection completed to editor usable",
            &editor_usable,
            500.0,
        );

        std::thread::sleep(Duration::from_secs(2));
        let after = memory();
        memory_rows.push(format!("after all captures, 2 s idle,{}", after.csv()));
        report.say("");
        report.say(format!(
            "after {attempts} captures, editor hidden, 2 s idle: {}",
            after.line()
        ));
        report.say(format!(
            "  delta from the floor: {:+.1} MB private",
            (after.total_private() as f64 - floor.total_private() as f64) / MB
        ));
        let first = run_rows.get(1).map(|_| ());
        if first.is_some() {
            let privates: Vec<f64> = memory_rows
                .iter()
                .filter(|r| r.starts_with("after capture "))
                .filter_map(|r| {
                    let cols: Vec<&str> = r.split(',').collect();
                    Some(cols.get(1)?.parse::<f64>().ok()? + cols.get(3)?.parse::<f64>().ok()?)
                })
                .collect();
            if privates.len() >= 2 {
                let tail = &privates[privates.len() / 2..];
                let head = &privates[..privates.len() / 2];
                let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
                report.say(format!(
                    "  slope across the captures: first capture {:.1} MB, last {:.1} MB, mean of the first half {:.1} MB, of the second half {:.1} MB",
                    privates[0],
                    privates[privates.len() - 1],
                    mean(head),
                    mean(tail)
                ));
            }
        }
    }

    // ---- the folder walk
    if let Some(dir) = walk.filter(|_| stopped.is_none()) {
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        let before = memory();
        memory_rows.push(format!("walk start,{}", before.csv()));
        report.say("");
        report.say(format!(
            "folder walk of {} files in {dir}, one open at a time through the product's open path",
            files.len()
        ));
        let mut peak = before;
        for path in &files {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let result = crate::editor::open_path(path);
            std::thread::sleep(Duration::from_millis(500));
            let mem = memory();
            if mem.total_private() > peak.total_private() {
                peak = mem;
            }
            memory_rows.push(format!("walk {name},{}", mem.csv()));
            println!(
                "walk {name}: {}, {:.1} MB private",
                match result {
                    Ok(_) => "opened".to_string(),
                    Err(err) => format!("not opened, {err}"),
                },
                mem.total_private() as f64 / MB
            );
        }
        hide_editor(&app);
        std::thread::sleep(Duration::from_secs(2));
        let after = memory();
        memory_rows.push(format!("walk end, hidden, 2 s idle,{}", after.csv()));
        report.say(format!("  before the walk: {}", before.line()));
        report.say(format!("  peak during it:  {}", peak.line()));
        report.say(format!("  after it, hidden, 2 s idle: {}", after.line()));
        report.say(format!(
            "  delta across the walk: {:+.1} MB private",
            (after.total_private() as f64 - before.total_private() as f64) / MB
        ));
    }

    let write = |name: &str, rows: &[String]| {
        let path = beside_exe(name);
        match std::fs::write(&path, rows.join("\n") + "\n") {
            Ok(()) => println!("written: {}", path.display()),
            Err(err) => println!("{name} not written: {err}"),
        }
    };
    if runs > 0 {
        write("s07-runs.csv", &run_rows);
    }
    write("s07-memory.csv", &memory_rows);
    write("s07-summary.txt", &report.lines);
    app.exit(if stopped.is_some() { 1 } else { 0 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_thirty_samples_the_95th_by_nearest_rank_is_the_second_worst() {
        let sorted: Vec<f64> = (1..=30).map(f64::from).collect();
        assert_eq!(nearest_rank(&sorted, 95.0), Some(29.0));
        assert_eq!(nearest_rank(&sorted, 100.0), Some(30.0));
        assert_eq!(nearest_rank(&sorted, 50.0), Some(15.0));
        assert_eq!(nearest_rank(&[], 95.0), None);
    }

    #[test]
    fn the_default_hotkey_parses_to_its_keys() {
        let (modifiers, key) = hotkey_keys("Ctrl+Shift+4").unwrap();
        assert_eq!(modifiers, vec![VK_CONTROL.0 as u8, VK_SHIFT.0 as u8]);
        assert_eq!(key, b'4');
        assert!(hotkey_keys("Ctrl+F12").is_none());
    }
}
