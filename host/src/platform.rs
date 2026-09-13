//! What the platform actually is, read fresh each time (project-os/Plan.md, S0.8).
//!
//! Two things the product needs to know rather than assume. Each active display's advanced
//! colour state, because a display with HDR on hands GDI an SDR rendering of itself and a
//! capture taken there is not what the screen shows: v1 does not capture HDR, and the rule
//! is "detected rather than silently wrong", so the state is logged at startup and named at
//! every freeze. And who owns the window that was in front before the overlay, elevation
//! included, because the hotkey and the focus return against an elevated window are facts
//! S0.8 records from the log rather than guesses.

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
    DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
    DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SDR_WHITE_LEVEL,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Graphics::Gdi::{
    DISPLAYCONFIG_COLOR_ENCODING_INTENSITY, DISPLAYCONFIG_COLOR_ENCODING_RGB,
    DISPLAYCONFIG_COLOR_ENCODING_YCBCR420, DISPLAYCONFIG_COLOR_ENCODING_YCBCR422,
    DISPLAYCONFIG_COLOR_ENCODING_YCBCR444,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindow,
};

/// One active display's colour facts, as Windows reports them.
#[derive(Debug, Clone)]
pub struct DisplayFacts {
    /// The GDI name, `\\.\DISPLAY1`, which is how the capture path names a display too.
    pub device: String,
    /// The monitor's own name from its EDID, when it has one.
    pub name: String,
    pub hdr_supported: bool,
    /// Windows calls this "advanced color enabled": HDR is on for this display.
    pub hdr_enabled: bool,
    pub wide_color_enforced: bool,
    pub bits_per_channel: u32,
    pub encoding: &'static str,
    /// Where SDR white sits when HDR is on, in nits; 80 is the SDR baseline.
    pub sdr_white_nits: f32,
}

impl DisplayFacts {
    pub fn line(&self) -> String {
        format!(
            "{} {:?}: HDR {}{}, {} bits per channel, {}, SDR white {:.0} nits",
            self.device,
            self.name,
            if self.hdr_enabled {
                "ON"
            } else if self.hdr_supported {
                "off (supported)"
            } else {
                "not supported"
            },
            if self.wide_color_enforced {
                ", wide colour enforced"
            } else {
                ""
            },
            self.bits_per_channel,
            self.encoding,
            self.sdr_white_nits
        )
    }
}

fn wide(text: &[u16]) -> String {
    let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
    String::from_utf16_lossy(&text[..end])
}

/// Every active display path, source name to target facts. Empty if the query fails.
pub fn displays() -> Vec<DisplayFacts> {
    let mut out = Vec::new();
    unsafe {
        let mut paths = 0u32;
        let mut modes = 0u32;
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut paths, &mut modes).0 != 0 {
            return out;
        }
        let mut path_list: Vec<DISPLAYCONFIG_PATH_INFO> = vec![Default::default(); paths as usize];
        let mut mode_list: Vec<DISPLAYCONFIG_MODE_INFO> = vec![Default::default(); modes as usize];
        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut paths,
            path_list.as_mut_ptr(),
            &mut modes,
            mode_list.as_mut_ptr(),
            None,
        )
        .0 != 0
        {
            return out;
        }
        for path in &path_list[..paths as usize] {
            let mut source: DISPLAYCONFIG_SOURCE_DEVICE_NAME = std::mem::zeroed();
            source.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            source.header.adapterId = path.sourceInfo.adapterId;
            source.header.id = path.sourceInfo.id;
            let device = if DisplayConfigGetDeviceInfo(&mut source.header) == 0 {
                wide(&source.viewGdiDeviceName)
            } else {
                String::from("(unnamed source)")
            };

            let mut target: DISPLAYCONFIG_TARGET_DEVICE_NAME = std::mem::zeroed();
            target.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            target.header.adapterId = path.targetInfo.adapterId;
            target.header.id = path.targetInfo.id;
            let name = if DisplayConfigGetDeviceInfo(&mut target.header) == 0 {
                wide(&target.monitorFriendlyDeviceName)
            } else {
                String::new()
            };

            let mut colour: DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO = std::mem::zeroed();
            colour.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO;
            colour.header.size =
                std::mem::size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO>() as u32;
            colour.header.adapterId = path.targetInfo.adapterId;
            colour.header.id = path.targetInfo.id;
            let (flags, bits, encoding) = if DisplayConfigGetDeviceInfo(&mut colour.header) == 0 {
                (
                    colour.Anonymous.value,
                    colour.bitsPerColorChannel,
                    match colour.colorEncoding {
                        DISPLAYCONFIG_COLOR_ENCODING_RGB => "RGB",
                        DISPLAYCONFIG_COLOR_ENCODING_YCBCR444 => "YCbCr 4:4:4",
                        DISPLAYCONFIG_COLOR_ENCODING_YCBCR422 => "YCbCr 4:2:2",
                        DISPLAYCONFIG_COLOR_ENCODING_YCBCR420 => "YCbCr 4:2:0",
                        DISPLAYCONFIG_COLOR_ENCODING_INTENSITY => "intensity",
                        _ => "unknown encoding",
                    },
                )
            } else {
                (0, 0, "colour info unavailable")
            };

            let mut white: DISPLAYCONFIG_SDR_WHITE_LEVEL = std::mem::zeroed();
            white.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL;
            white.header.size = std::mem::size_of::<DISPLAYCONFIG_SDR_WHITE_LEVEL>() as u32;
            white.header.adapterId = path.targetInfo.adapterId;
            white.header.id = path.targetInfo.id;
            let sdr_white_nits = if DisplayConfigGetDeviceInfo(&mut white.header) == 0 {
                // Documented as a multiplier of 80 nits in thousandths.
                white.SDRWhiteLevel as f32 * 80.0 / 1000.0
            } else {
                0.0
            };

            out.push(DisplayFacts {
                device,
                name,
                hdr_supported: flags & 0b1 != 0,
                hdr_enabled: flags & 0b10 != 0,
                wide_color_enforced: flags & 0b100 != 0,
                bits_per_channel: bits,
                encoding,
                sdr_white_nits,
            });
        }
    }
    out
}

/// The displays with HDR on right now, by name, for the freeze to warn about.
pub fn hdr_on() -> Vec<String> {
    displays()
        .into_iter()
        .filter(|d| d.hdr_enabled)
        .map(|d| format!("{} {:?}", d.device, d.name))
        .collect()
}

/// Who owns a window: its process, its title, and whether that process is elevated.
#[derive(Debug, Clone)]
pub struct WindowOwner {
    pub exists: bool,
    pub pid: u32,
    pub title: String,
    /// None when the process could not be asked, which a protected process refuses.
    pub elevated: Option<bool>,
}

impl WindowOwner {
    pub fn line(&self) -> String {
        if !self.exists {
            return "no window".into();
        }
        format!(
            "{:?} (pid {}, {})",
            self.title,
            self.pid,
            match self.elevated {
                Some(true) => "elevated",
                Some(false) => "not elevated",
                None => "elevation unknown",
            }
        )
    }
}

pub fn window_owner(hwnd: HWND) -> WindowOwner {
    unsafe {
        if hwnd.is_invalid() || !IsWindow(Some(hwnd)).as_bool() {
            return WindowOwner {
                exists: false,
                pid: 0,
                title: String::new(),
                elevated: None,
            };
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut buffer = [0u16; 256];
        let n = GetWindowTextW(hwnd, &mut buffer).max(0) as usize;
        WindowOwner {
            exists: true,
            pid,
            title: String::from_utf16_lossy(&buffer[..n]),
            elevated: process_is_elevated(pid),
        }
    }
}

pub fn foreground_owner() -> WindowOwner {
    window_owner(unsafe { GetForegroundWindow() })
}

/// Whether a process runs elevated, asked through its token. A process this one may not
/// open, such as a protected system process, answers None.
pub fn process_is_elevated(pid: u32) -> Option<bool> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut token = HANDLE::default();
        let opened = OpenProcessToken(process, TOKEN_QUERY, &mut token);
        let _ = CloseHandle(process);
        opened.ok()?;
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let asked = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut core::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);
        asked.ok()?;
        Some(elevation.TokenIsElevated != 0)
    }
}

/// This process's own elevation, for the report.
pub fn this_process_is_elevated() -> Option<bool> {
    process_is_elevated(std::process::id())
}
