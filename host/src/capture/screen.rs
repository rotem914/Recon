//! The chosen capture path: one copy of the whole virtual screen.
//!
//! Part 6a picked this because a single synchronous call already spans every display and
//! every negative coordinate, so the foundation risk is retired at the lowest cost. Part 8
//! says the modern per-display capture API is earned only by a demonstrated failure here:
//! wrong on DPI, missing window content, or diagnosed as the cause of a missed latency
//! target.
//!
//! CAPTUREBLT is included so layered windows are in the freeze, which is what makes a menu
//! or a tooltip appear in a capture. It costs time and can flicker on some machines; that
//! trade is measured in S0.7, not assumed here.

use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP,
    HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use super::coords::FrameGeometry;
use super::{CaptureError, CaptureSource, Frame};

pub struct WholeVirtualScreen;

impl CaptureSource for WholeVirtualScreen {
    fn name(&self) -> &'static str {
        "whole virtual screen, GDI BitBlt with CAPTUREBLT"
    }

    fn freeze(&self) -> Result<Frame, CaptureError> {
        let geometry = virtual_screen()?;
        let rgba = unsafe { copy_screen(geometry)? };
        Ok(Frame {
            geometry,
            rgba,
            source: self.name(),
        })
    }
}

/// The virtual desktop's bounds in physical pixels. The origin is negative whenever a
/// display is arranged left of or above the primary one, which is the case the coordinate
/// module exists for.
pub fn virtual_screen() -> Result<FrameGeometry, CaptureError> {
    let (x, y, w, h) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };
    if w <= 0 || h <= 0 {
        return Err(CaptureError::ImpossibleBounds {
            width: w,
            height: h,
        });
    }
    Ok(FrameGeometry {
        origin_x: x,
        origin_y: y,
        width: w as u32,
        height: h as u32,
    })
}

/// Copies an arbitrary desktop rectangle straight off the screen.
///
/// Used by the self test to ask Windows for the same rectangle a crop claims to hold. Two
/// answers that disagree mean the coordinate conversion is wrong, which is the one thing
/// this step cannot afford to get away with.
pub fn copy_rect(rect: super::coords::DesktopRect) -> Result<Vec<u8>, CaptureError> {
    let geometry = FrameGeometry {
        origin_x: rect.x,
        origin_y: rect.y,
        width: rect.width,
        height: rect.height,
    };
    unsafe { copy_screen(geometry) }
}

/// The one GDI sequence: screen DC, memory DC, top-down 32bpp DIB, one blit, read back.
unsafe fn copy_screen(geometry: FrameGeometry) -> Result<Vec<u8>, CaptureError> {
    let width = geometry.width as i32;
    let height = geometry.height as i32;

    let screen_dc = unsafe { GetDC(None) };
    if screen_dc.is_invalid() {
        return Err(CaptureError::Gdi("GetDC"));
    }

    // From here on every early return has to release what it took, so the work is done in
    // a closure and the teardown runs once, after it.
    let result = (|| unsafe {
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.is_invalid() {
            return Err(CaptureError::Gdi("CreateCompatibleDC"));
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Negative height asks for a top-down bitmap, so row 0 is the top row and
                // nothing downstream has to remember that GDI is bottom-up by default.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut c_void = std::ptr::null_mut();
        let bitmap: HBITMAP =
            match CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
                _ => {
                    let _ = DeleteDC(mem_dc);
                    return Err(CaptureError::Gdi("CreateDIBSection"));
                }
            };

        let previous = SelectObject(mem_dc, HGDIOBJ(bitmap.0));

        let blit = BitBlt(
            mem_dc,
            0,
            0,
            width,
            height,
            Some(screen_dc),
            geometry.origin_x,
            geometry.origin_y,
            SRCCOPY | CAPTUREBLT,
        );

        let out = if blit.is_ok() {
            // The DIB is BGRA in memory, packed and top-down. The contract in §3.7 says one
            // predictable shape comes out of a source, so this is the only place that knows
            // about the byte order, and alpha is forced opaque because a screen copy has no
            // transparency to preserve.
            let count = (width as usize) * (height as usize);
            let src = std::slice::from_raw_parts(bits as *const u8, count * 4);
            let mut rgba = vec![0u8; count * 4];
            for i in 0..count {
                rgba[i * 4] = src[i * 4 + 2];
                rgba[i * 4 + 1] = src[i * 4 + 1];
                rgba[i * 4 + 2] = src[i * 4];
                rgba[i * 4 + 3] = 255;
            }
            Ok(rgba)
        } else {
            Err(CaptureError::Gdi("BitBlt"))
        };

        SelectObject(mem_dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(mem_dc);
        // Touch the header after the blit so the compiler cannot decide the struct was
        // unused and reorder its lifetime; CreateDIBSection borrowed it.
        std::hint::black_box(&mut info);
        out
    })();

    unsafe { ReleaseDC(None, screen_dc) };
    result
}
