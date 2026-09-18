//! The clipboard, published from the host and never from the web view (part 5).
//!
//! Three formats go up together, so the two paste destinations the plan names and the
//! ordinary Windows applications all find one they read: `PNG`, the registered format a
//! Chromium-based page reads first and the only one that carries the image losslessly;
//! `CF_DIBV5`, the system bitmap with an alpha channel; and `CF_DIB`, the plain bitmap
//! that every old application reads, flattened over white because those readers ignore
//! alpha. Which of the three each destination actually took is what S0.6 records.

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BITMAPV5HEADER, BI_BITFIELDS, BI_RGB};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{CF_DIB, CF_DIBV5, CF_UNICODETEXT};

/// What went up, for the log and the S0.6 record.
#[derive(Debug, serde::Serialize)]
pub struct Published {
    pub formats: Vec<&'static str>,
    pub png_bytes: usize,
    pub encode_ms: u128,
    pub publish_ms: u128,
}

/// 'sRGB' as the V5 header wants it, the one colour space every output is in (§3.7).
const LCS_SRGB: u32 = 0x7352_4742;

/// The registered name Chromium, Office and the .NET clipboard all use for a PNG.
fn png_format() -> u32 {
    let name: Vec<u16> = "PNG\0".encode_utf16().collect();
    unsafe { RegisterClipboardFormatW(windows::core::PCWSTR(name.as_ptr())) }
}

/// The clipboard held open for the length of one publish, released on every exit path.
struct Open;

impl Open {
    fn acquire() -> Result<Open, String> {
        // Another application can hold the clipboard for a few milliseconds after its own
        // copy; ten short retries cover that without ever waiting on a stuck one for long.
        let mut last = None;
        for _ in 0..10 {
            match unsafe { OpenClipboard(None) } {
                Ok(()) => return Ok(Open),
                Err(err) => {
                    last = Some(err);
                    std::thread::sleep(std::time::Duration::from_millis(15));
                }
            }
        }
        Err(format!(
            "the clipboard could not be opened: {}",
            last.map(|e| e.to_string()).unwrap_or_default()
        ))
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Copies bytes into a movable global block and hands it to the clipboard, which owns it
/// from then on. On a failed handover the block is freed here.
fn set(format: u32, bytes: &[u8]) -> Result<(), String> {
    unsafe {
        let block: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).map_err(|e| e.to_string())?;
        let target = GlobalLock(block);
        if target.is_null() {
            let _ = GlobalFree(Some(block));
            return Err("the clipboard block could not be locked".into());
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, bytes.len());
        let _ = GlobalUnlock(block);
        if let Err(err) = SetClipboardData(format, Some(HANDLE(block.0))) {
            let _ = GlobalFree(Some(block));
            return Err(format!("the clipboard refused format {format}: {err}"));
        }
    }
    Ok(())
}

/// Bottom-up BGRA rows, which is the only pixel order a DIB has.
fn bottom_up_bgra(rgba: &[u8], width: u32, height: u32, flatten_over_white: bool) -> Vec<u8> {
    let row = width as usize * 4;
    let mut out = Vec::with_capacity(rgba.len());
    for y in (0..height as usize).rev() {
        for px in rgba[y * row..(y + 1) * row].as_chunks::<4>().0 {
            let (r, g, b, a) = (px[0] as u32, px[1] as u32, px[2] as u32, px[3] as u32);
            if flatten_over_white && a < 255 {
                let over = |c: u32| ((c * a + 255 * (255 - a) + 127) / 255) as u8;
                out.extend_from_slice(&[over(b), over(g), over(r), 255]);
            } else {
                out.extend_from_slice(&[b as u8, g as u8, r as u8, a as u8]);
            }
        }
    }
    out
}

fn dib_v5(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixels = bottom_up_bgra(rgba, width, height, false);
    let header = BITMAPV5HEADER {
        bV5Size: std::mem::size_of::<BITMAPV5HEADER>() as u32,
        bV5Width: width as i32,
        bV5Height: height as i32,
        bV5Planes: 1,
        bV5BitCount: 32,
        bV5Compression: BI_BITFIELDS,
        bV5SizeImage: pixels.len() as u32,
        bV5RedMask: 0x00FF_0000,
        bV5GreenMask: 0x0000_FF00,
        bV5BlueMask: 0x0000_00FF,
        bV5AlphaMask: 0xFF00_0000,
        bV5CSType: LCS_SRGB,
        ..Default::default()
    };
    let mut out = Vec::with_capacity(std::mem::size_of::<BITMAPV5HEADER>() + pixels.len());
    out.extend_from_slice(unsafe { as_bytes(&header) });
    out.extend_from_slice(&pixels);
    out
}

fn dib_plain(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixels = bottom_up_bgra(rgba, width, height, true);
    let header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: height as i32,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        biSizeImage: pixels.len() as u32,
        ..Default::default()
    };
    let mut out = Vec::with_capacity(std::mem::size_of::<BITMAPINFOHEADER>() + pixels.len());
    out.extend_from_slice(unsafe { as_bytes(&header) });
    out.extend_from_slice(&pixels);
    out
}

/// The bytes of a plain-old-data header. Both headers are `repr(C)` with no padding.
unsafe fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>()) }
}

/// Encodes the composite once as PNG and publishes all three formats in one clipboard
/// transaction. Every format is built before the clipboard is touched, so the likely
/// failure, an encode that fails, leaves the previous contents in place; a handover that
/// fails after the clipboard was emptied leaves it empty and is reported as an error, and
/// the page shows success only on success (§3.6).
pub fn publish(rgba: &[u8], width: u32, height: u32) -> Result<Published, String> {
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(format!("{} bytes is not {width}x{height} RGBA", rgba.len()));
    }
    let started = std::time::Instant::now();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|err| format!("the PNG did not encode: {err}"))?;
    let v5 = dib_v5(rgba, width, height);
    let plain = dib_plain(rgba, width, height);
    let encode_ms = started.elapsed().as_millis();

    let started = std::time::Instant::now();
    let _open = Open::acquire()?;
    unsafe { EmptyClipboard() }.map_err(|e| format!("the clipboard could not be emptied: {e}"))?;
    set(png_format(), &png)?;
    set(CF_DIBV5.0 as u32, &v5)?;
    set(CF_DIB.0 as u32, &plain)?;
    Ok(Published {
        formats: vec!["PNG", "CF_DIBV5", "CF_DIB"],
        png_bytes: png.len(),
        encode_ms,
        publish_ms: started.elapsed().as_millis(),
    })
}

/// The colour picker's HEX (Rotem, 2026-09-18): one short line of text, the only text the
/// host ever publishes. Built before the clipboard is touched, as the image is.
pub fn publish_text(text: &str) -> Result<(), String> {
    let bytes: Vec<u8> = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect();
    let _open = Open::acquire()?;
    unsafe { EmptyClipboard() }.map_err(|e| format!("the clipboard could not be emptied: {e}"))?;
    set(CF_UNICODETEXT.0 as u32, &bytes)
}

use image::ImageEncoder;

/// The text the clipboard holds now, as a paste would get it. Checks only.
#[cfg(feature = "stage0-checks")]
pub fn read_text() -> Result<String, String> {
    use windows::Win32::System::DataExchange::GetClipboardData;
    let _open = Open::acquire()?;
    unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32)
            .map_err(|e| format!("the clipboard holds no text: {e}"))?;
        let block = HGLOBAL(handle.0);
        let source = GlobalLock(block) as *const u16;
        if source.is_null() {
            return Err("the clipboard's text could not be locked".into());
        }
        let mut len = 0;
        while *source.add(len) != 0 {
            len += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(source, len));
        let _ = GlobalUnlock(block);
        Ok(text)
    }
}

/// What the clipboard holds now, read back the way a destination would. Checks only: the
/// product never reads the clipboard.
#[cfg(feature = "stage0-checks")]
pub struct ReadBack {
    pub png: Option<Vec<u8>>,
    pub dib_v5: bool,
    pub dib: bool,
}

#[cfg(feature = "stage0-checks")]
pub fn read_back() -> Result<ReadBack, String> {
    use windows::Win32::System::DataExchange::{GetClipboardData, IsClipboardFormatAvailable};
    use windows::Win32::System::Memory::GlobalSize;
    let _open = Open::acquire()?;
    let png = unsafe {
        match GetClipboardData(png_format()) {
            Ok(handle) => {
                let block = HGLOBAL(handle.0);
                let size = GlobalSize(block);
                let from = GlobalLock(block);
                if from.is_null() {
                    None
                } else {
                    let bytes = std::slice::from_raw_parts(from as *const u8, size).to_vec();
                    let _ = GlobalUnlock(block);
                    Some(bytes)
                }
            }
            Err(_) => None,
        }
    };
    Ok(ReadBack {
        png,
        dib_v5: unsafe { IsClipboardFormatAvailable(CF_DIBV5.0 as u32) }.is_ok(),
        dib: unsafe { IsClipboardFormatAvailable(CF_DIB.0 as u32) }.is_ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dib_is_bottom_up_bgra_and_the_plain_one_is_flattened() {
        // Two rows: top row red half-transparent, bottom row blue opaque.
        let rgba = [255, 0, 0, 128, 0, 0, 255, 255];
        let v5 = dib_v5(&rgba, 1, 2);
        assert_eq!(v5.len(), 124 + 8);
        assert_eq!(
            &v5[124..128],
            &[255, 0, 0, 255],
            "bottom row first, as BGRA"
        );
        assert_eq!(&v5[128..132], &[0, 0, 255, 128], "alpha kept");
        let plain = dib_plain(&rgba, 1, 2);
        assert_eq!(plain.len(), 40 + 8);
        assert_eq!(&plain[40..44], &[255, 0, 0, 255]);
        // 255 * 128/255 + 255 * 127/255 for the untouched channels: white shows through.
        assert_eq!(&plain[44..48], &[127, 127, 255, 255]);
    }
}
