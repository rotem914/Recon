//! The marks of S0.7, and the startup marks beside them (project-os/Plan.md, S0.7).
//!
//! One clock: every mark is an `Instant` taken in this process, the two the page reports
//! included, which are stamped when the host hears of them. A page mark is therefore late by
//! one message crossing and never early, so an interval read from it can only overstate.
//!
//! A run begins at the hotkey, and every mark in it is read as milliseconds from that
//! moment. The product keeps the marks of the latest run and logs each one; only the
//! measurement run reads them back.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const HOTKEY_RECEIVED: &str = "hotkey received";
pub const FREEZE_DONE: &str = "freeze done";
pub const OVERLAY_DISPLAYED: &str = "overlay displayed";
pub const OVERLAY_ACCEPTING: &str = "overlay accepting input";
pub const SELECTION_COMPLETED: &str = "selection completed";
pub const EDITOR_SHOW_RETURNED: &str = "editor show returned";
pub const PAGE_PAINTED: &str = "page painted";
pub const PAGE_FOCUSED: &str = "page focused";

pub const READY: &str = "ready";
pub const PAGE_BOOTED: &str = "page booted";

struct Run {
    origin: Instant,
    marks: Vec<(&'static str, Instant)>,
}

static RUN: Mutex<Option<Run>> = Mutex::new(None);
static PROCESS_START: OnceLock<Instant> = OnceLock::new();
static STARTUP: Mutex<Vec<(&'static str, Instant)>> = Mutex::new(Vec::new());

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// The origin of every startup mark. Called first thing in `main`; the time the operating
/// system spent before `main` is not in it.
pub fn process_started() {
    let _ = PROCESS_START.set(Instant::now());
}

/// Starts a run at the moment the hotkey was received, replacing the previous run's marks.
pub fn begin(received: Instant) {
    if let Ok(mut slot) = RUN.lock() {
        *slot = Some(Run {
            origin: received,
            marks: vec![(HOTKEY_RECEIVED, received)],
        });
    }
}

/// Records a mark in the current run, if there is one, and logs it.
pub fn mark(name: &'static str) {
    let now = Instant::now();
    let since = match RUN.lock() {
        Ok(mut slot) => slot.as_mut().map(|run| {
            run.marks.push((name, now));
            now.duration_since(run.origin)
        }),
        Err(_) => None,
    };
    if let Some(since) = since {
        crate::log(&format!("mark: {name} +{:.1} ms", ms(since)));
    }
}

/// Records a startup mark, measured from `process_started`, and logs it.
pub fn startup(name: &'static str) {
    let now = Instant::now();
    let Some(start) = PROCESS_START.get() else {
        return;
    };
    if let Ok(mut list) = STARTUP.lock() {
        list.push((name, now));
    }
    crate::log(&format!(
        "startup: {name} {:.1} ms after main began",
        ms(now.duration_since(*start))
    ));
}

/// The current run's marks, milliseconds from the hotkey, in the order they arrived.
#[cfg(feature = "stage0-checks")]
pub fn snapshot() -> Vec<(&'static str, f64)> {
    RUN.lock()
        .ok()
        .and_then(|slot| {
            slot.as_ref().map(|run| {
                run.marks
                    .iter()
                    .map(|(name, at)| (*name, ms(at.duration_since(run.origin))))
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// The startup marks, milliseconds from the start of `main`.
#[cfg(feature = "stage0-checks")]
pub fn startup_snapshot() -> Vec<(&'static str, f64)> {
    let Some(start) = PROCESS_START.get() else {
        return Vec::new();
    };
    STARTUP
        .lock()
        .map(|list| {
            list.iter()
                .map(|(name, at)| (*name, ms(at.duration_since(*start))))
                .collect()
        })
        .unwrap_or_default()
}
