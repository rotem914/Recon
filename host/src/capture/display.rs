//! What the displays actually are, at the moment of a capture.
//!
//! Everything here is read fresh each time rather than cached. A display can be added,
//! removed, moved or rescaled between two captures, and a cached layout is how a capture
//! ends up reading the wrong rectangle after someone unplugs a monitor.

use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{
    GetAwarenessFromDpiAwarenessContext, GetDpiForMonitor, GetThreadDpiAwarenessContext,
    SetProcessDpiAwarenessContext, DPI_AWARENESS, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    DPI_AWARENESS_INVALID, DPI_AWARENESS_PER_MONITOR_AWARE, DPI_AWARENESS_SYSTEM_AWARE,
    DPI_AWARENESS_UNAWARE, MDT_EFFECTIVE_DPI,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

use super::coords::DesktopRect;

/// One display, in the only units that matter here: physical desktop pixels.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub rect: DesktopRect,
    /// 100 for an unscaled display, 150 for 150%, and so on. Rounded from the effective DPI.
    pub scale_percent: u32,
    pub primary: bool,
    pub device: String,
}

/// Declares this process per-monitor DPI aware, and must be the first thing it does.
///
/// Found by S0.2's own self test: run without this, the process is UNAWARE, Windows
/// virtualises every coordinate, and a 2560x1440 display at 150% reports itself as
/// 1707x960. Every rectangle, every blit and every comparison downstream is then quietly
/// wrong, and the pictures still look plausible. Relying on the application runtime to set
/// it later is not enough either, because the self test never builds one.
///
/// A second call, or a manifest that already declared it, makes this fail; that is not an
/// error, and the return value says which happened so the log can carry it.
pub fn make_per_monitor_aware() -> &'static str {
    let already = dpi_awareness();
    if already == "per-monitor aware" {
        return "per-monitor aware already, by manifest or by an earlier call";
    }
    let set = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    match set {
        Ok(()) => "per-monitor aware, set at startup",
        Err(_) => "COULD NOT BE SET: coordinates are virtualised and every capture is wrong",
    }
}

/// How this process sees coordinates. If this is not per-monitor aware, every number in a
/// capture is a lie told by Windows on the process's behalf, so it is logged at startup and
/// again in the self test rather than assumed.
pub fn dpi_awareness() -> &'static str {
    unsafe {
        let ctx = GetThreadDpiAwarenessContext();
        let awareness: DPI_AWARENESS = GetAwarenessFromDpiAwarenessContext(ctx);
        match awareness {
            DPI_AWARENESS_UNAWARE => {
                "unaware (coordinates would be virtualised: wrong for capture)"
            }
            DPI_AWARENESS_SYSTEM_AWARE => "system aware (wrong on a mixed-scale desktop)",
            DPI_AWARENESS_PER_MONITOR_AWARE => "per-monitor aware",
            DPI_AWARENESS_INVALID => "invalid",
            _ => "unrecognised",
        }
    }
}

/// Every display, left to right in whatever order Windows reports them.
pub fn monitors() -> Vec<MonitorInfo> {
    let mut found: Vec<MonitorInfo> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut found as *mut Vec<MonitorInfo> as isize),
        );
    }
    found
}

unsafe extern "system" fn collect(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let out = unsafe { &mut *(lparam.0 as *mut Vec<MonitorInfo>) };

    let mut info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let ok = unsafe {
        GetMonitorInfoW(
            monitor,
            &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };
    if !ok.as_bool() {
        // One unreadable monitor must not silently shrink the list: say so and keep going.
        eprintln!("GetMonitorInfoW failed for one monitor; it is missing from the layout");
        return BOOL(1);
    }

    let r = info.monitorInfo.rcMonitor;
    let mut dpi_x = 96u32;
    let mut dpi_y = 96u32;
    let _ = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };

    let device = String::from_utf16_lossy(&info.szDevice)
        .trim_end_matches('\0')
        .to_string();

    out.push(MonitorInfo {
        rect: DesktopRect {
            x: r.left,
            y: r.top,
            width: (r.right - r.left).max(0) as u32,
            height: (r.bottom - r.top).max(0) as u32,
        },
        scale_percent: (dpi_x * 100 + 48) / 96,
        primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        device,
    });
    BOOL(1)
}
