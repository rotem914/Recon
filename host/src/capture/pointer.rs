//! The mouse pointer as it stood at the freeze (Rotem, 2026-09-21).
//!
//! A screen copy never holds the pointer, so it is read beside the freeze: which pointer
//! Windows was showing, where, and its picture with its own transparency. It travels to the
//! editor apart from the frozen pixels and lands there as an element on top of the capture,
//! so it can be moved or deleted and the capture under it is whole.
//!
//! Windows hands a pointer over as something to draw, never as pixels with alpha, and an
//! old one-colour pointer has no alpha at all. So it is drawn twice, on black and on white,
//! and the alpha is what the two drawings disagree by. A pixel that INVERTS what is under it,
//! the text pointer's way, comes out lighter on black than on white, which no alpha explains;
//! it is kept as solid black, which is how such a pointer reads over a light page.
//!
//! On a scaled display Windows hands over the pointer at its 100% size, 32 px, while it
//! draws a larger one (measured here on a 225% display: 32 by 32 handed over). So the
//! pointer is asked for again at the size Windows gives a pointer on that display, from the
//! pointer's own file, which holds it sharp at several sizes. That the size Windows names
//! is the size it draws is inferred, not measured. Not established here: a pointer enlarged
//! in Accessibility, and an application's own pointer, which has no file to be read from
//! again and is stretched instead.

use std::ffi::c_void;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetDC, GetObjectW,
    MonitorFromPoint, ReleaseDC, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HBITMAP, HGDIOBJ, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, GetSystemMetricsForDpi, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    CopyImage, DestroyIcon, DrawIconEx, GetCursorInfo, GetIconInfo, CURSORINFO, CURSOR_SHOWING,
    DI_NORMAL, HICON, ICONINFO, IMAGE_CURSOR, LR_COPYFROMRESOURCE, SM_CXCURSOR,
};

use super::coords::DesktopRect;

/// No pointer Windows ships is near this; a size past it is a bitmap not worth trusting.
const LARGEST: i32 = 512;

pub struct Pointer {
    /// Where its picture sits in desktop space: the pointer's position less its hot spot.
    pub at: DesktopRect,
    /// Tightly packed, top-down, straight alpha.
    pub rgba: Vec<u8>,
}

/// The pointer Windows is showing right now, or None when it shows none or will not say.
/// Called before the overlay goes up, which swaps the pointer for its own crosshair.
pub fn grab() -> Option<Pointer> {
    unsafe {
        let mut info = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        GetCursorInfo(&mut info).ok()?;
        #[cfg(test)]
        println!(
            "Windows says: flags {}, a pointer handle {}, at {},{}",
            info.flags.0,
            !info.hCursor.is_invalid(),
            info.ptScreenPos.x,
            info.ptScreenPos.y
        );
        if info.flags.0 & CURSOR_SHOWING.0 == 0 || info.hCursor.is_invalid() {
            return None;
        }
        // The pointer at the size of the display it stands on, read again from its own file;
        // the copy is this function's to destroy. At 100%, or when Windows will not make
        // one, the pointer as handed over.
        let wanted = size_on_display(info.ptScreenPos);
        let sized = wanted.and_then(|side| {
            CopyImage(
                HANDLE(info.hCursor.0),
                IMAGE_CURSOR,
                side,
                side,
                LR_COPYFROMRESOURCE,
            )
            .ok()
            .map(|copy| HICON(copy.0))
        });
        let resized = sized.and_then(|copy| {
            let pointer = read(copy, info.ptScreenPos);
            let _ = DestroyIcon(copy);
            pointer
        });
        resized.or_else(|| read(HICON(info.hCursor.0), info.ptScreenPos))
    }
}

/// The side Windows gives a pointer on the display under `at`, or None at 100%.
unsafe fn size_on_display(at: windows::Win32::Foundation::POINT) -> Option<i32> {
    let monitor = unsafe { MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST) };
    let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
    unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.ok()?;
    let side = unsafe { GetSystemMetricsForDpi(SM_CXCURSOR, dpi_x) };
    (dpi_x != 96 && side > 0 && side <= LARGEST).then_some(side)
}

/// One pointer's picture and its place, its hot spot on `at`.
unsafe fn read(icon: HICON, at: windows::Win32::Foundation::POINT) -> Option<Pointer> {
    unsafe {
        let mut parts = ICONINFO::default();
        GetIconInfo(icon, &mut parts).ok()?;
        // A one-colour pointer has no colour bitmap, and its mask is two pictures stacked.
        let size = if parts.hbmColor.is_invalid() {
            bitmap_size(parts.hbmMask).map(|(w, h)| (w, h / 2))
        } else {
            bitmap_size(parts.hbmColor)
        };
        // GetIconInfo makes copies of both bitmaps, and they are the caller's to delete.
        for bitmap in [parts.hbmMask, parts.hbmColor] {
            if !bitmap.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
            }
        }
        let (width, height) = size?;
        if width <= 0 || height <= 0 || width > LARGEST || height > LARGEST {
            return None;
        }
        let on_black = draw(icon, width, height, 0x00)?;
        let on_white = draw(icon, width, height, 0xFF)?;
        let rgba = recover(&on_black, &on_white);
        // A picture with nothing in it is no pointer to show.
        if rgba.iter().skip(3).step_by(4).all(|&alpha| alpha == 0) {
            return None;
        }
        Some(Pointer {
            at: DesktopRect {
                x: at.x - parts.xHotspot as i32,
                y: at.y - parts.yHotspot as i32,
                width: width as u32,
                height: height as u32,
            },
            rgba,
        })
    }
}

unsafe fn bitmap_size(bitmap: HBITMAP) -> Option<(i32, i32)> {
    let mut described = BITMAP::default();
    let wrote = unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut described as *mut BITMAP as *mut c_void),
        )
    };
    (wrote != 0).then_some((described.bmWidth, described.bmHeight))
}

/// The pointer drawn at its own size over one flat ground, as BGRA bytes, top-down.
unsafe fn draw(icon: HICON, width: i32, height: i32, ground: u8) -> Option<Vec<u8>> {
    let screen_dc = unsafe { GetDC(None) };
    if screen_dc.is_invalid() {
        return None;
    }
    let result = (|| unsafe {
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.is_invalid() {
            return None;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let bitmap =
            match CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
                _ => {
                    let _ = DeleteDC(mem_dc);
                    return None;
                }
            };
        let bytes = width as usize * height as usize * 4;
        std::ptr::write_bytes(bits as *mut u8, ground, bytes);
        let previous = SelectObject(mem_dc, HGDIOBJ(bitmap.0));
        let drawn = DrawIconEx(mem_dc, 0, 0, icon, width, height, 0, None, DI_NORMAL);
        let _ = GdiFlush();
        let out = drawn
            .is_ok()
            .then(|| std::slice::from_raw_parts(bits as *const u8, bytes).to_vec());
        SelectObject(mem_dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(mem_dc);
        out
    })();
    unsafe { ReleaseDC(None, screen_dc) };
    result
}

/// Two BGRA drawings of one picture, on black and on white, into straight-alpha RGBA.
///
/// On black a channel is colour times alpha; on white it is that plus what the white shows
/// through. So white less black is 255 less the alpha, and the colour is the black drawing
/// divided by the alpha. A pixel lighter on black than on white inverts, and is solid black.
fn recover(on_black: &[u8], on_white: &[u8]) -> Vec<u8> {
    let mut rgba = vec![0u8; on_black.len()];
    for i in (0..on_black.len().min(on_white.len()) / 4).map(|pixel| pixel * 4) {
        let (b, w) = (&on_black[i..i + 3], &on_white[i..i + 3]);
        if (0..3).any(|c| w[c] < b[c]) {
            rgba[i + 3] = 255;
            continue;
        }
        // The channel that disagrees least is the alpha's floor; the three agree unless a
        // rounding step came between them.
        let through = (0..3).map(|c| w[c] - b[c]).min().unwrap_or(255);
        let alpha = 255 - through;
        if alpha == 0 {
            continue;
        }
        for c in 0..3 {
            let colour = (u32::from(b[2 - c]) * 255 + u32::from(alpha) / 2) / u32::from(alpha);
            rgba[i + c] = colour.min(255) as u8;
        }
        rgba[i + 3] = alpha;
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::recover;

    #[test]
    fn a_solid_pixel_keeps_its_colour() {
        // BGRA in, RGBA out: a solid red reads the same on both grounds.
        assert_eq!(
            recover(&[0, 0, 255, 0], &[0, 0, 255, 0]),
            vec![255, 0, 0, 255]
        );
    }

    #[test]
    fn a_clear_pixel_is_clear() {
        assert_eq!(
            recover(&[0, 0, 0, 0], &[255, 255, 255, 0]),
            vec![0, 0, 0, 0]
        );
    }

    #[test]
    fn a_half_clear_white_comes_back_white_at_half() {
        assert_eq!(
            recover(&[128, 128, 128, 0], &[255, 255, 255, 0]),
            vec![255, 255, 255, 128]
        );
    }

    /// The real thing, on a desktop with a pointer showing, so not part of the ordinary run:
    /// `cargo test --all-features pointer -- --ignored`. The picture is written beside the
    /// test executable, over a grey ground, to be looked at.
    #[test]
    #[ignore]
    fn the_pointer_on_this_desktop_is_read_with_its_transparency() {
        let started = std::time::Instant::now();
        let pointer = super::grab().expect("Windows shows a pointer and hands it over");
        // The read sits inside the hotkey-to-overlay interval, so its cost is said.
        println!("read in {} us", started.elapsed().as_micros());
        let (w, h) = (pointer.at.width, pointer.at.height);
        let alphas: Vec<u8> = pointer.rgba.iter().skip(3).step_by(4).copied().collect();
        assert!(alphas.contains(&255), "some of it is solid");
        assert!(alphas.contains(&0), "and some of its box is clear");
        let mut seen = image::RgbaImage::from_pixel(w, h, image::Rgba([128, 128, 128, 255]));
        for (i, pixel) in seen.pixels_mut().enumerate() {
            let p = &pointer.rgba[i * 4..i * 4 + 4];
            let a = u32::from(p[3]);
            for c in 0..3 {
                pixel[c] = ((u32::from(p[c]) * a + 128 * (255 - a)) / 255) as u8;
            }
        }
        let path = std::env::current_exe()
            .unwrap()
            .with_file_name("pointer-grab.png");
        seen.save(&path).unwrap();
        println!(
            "{w}x{h} at {},{} written to {}",
            pointer.at.x,
            pointer.at.y,
            path.display()
        );
    }

    #[test]
    fn a_pixel_that_inverts_is_solid_black() {
        assert_eq!(
            recover(&[255, 255, 255, 0], &[0, 0, 0, 0]),
            vec![0, 0, 0, 255]
        );
    }
}
