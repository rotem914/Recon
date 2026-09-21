//! The little drawing a scrolling capture needs: a bitmap with its own transparency, smooth
//! round shapes and lines built pixel by pixel as the magnifier's panel is, and text.
//!
//! A bitmap here is BGRA, top-down, every colour already multiplied by its pixel's alpha,
//! which is what both ways of showing it want: blended onto the overlay's window, or handed
//! to Windows as a layered window's whole picture.

use std::ffi::c_void;

use windows::Win32::Foundation::{COLORREF, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetTextExtentPoint32W,
    SelectObject, SetBkMode, SetTextColor, TextOutW, DIB_RGB_COLORS, HBITMAP, HDC, HFONT, HGDIOBJ,
    TRANSPARENT,
};

use crate::magnifier::{scaled, solid_header, LINE_RGB};

/// The round button on the capture frame (Rotem, 2026-09-22): 56 px across, an arrow
/// pointing down, 24 px up from the frame's bottom edge. Not stated, so provisional: in the
/// middle of the frame's width; the magnifier's lines' blue as its fill, lighter under the
/// pointer; the arrow and a 2 px ring in the magnifier's white; a 24 px arrow, a 2 px line.
pub const BUTTON_PX: i32 = 56;
pub const BUTTON_RISE_PX: i32 = 24;
pub const BLUE_RGB: (u8, u8, u8) = LINE_RGB;
pub const BLUE_HOT_RGB: (u8, u8, u8) = (0x4B, 0x74, 0xFF);
pub const WHITE_RGB: (u8, u8, u8) = (0xF2, 0xF2, 0xF2);
const RING_PX: i32 = 2;
const ARROW_PX: i32 = 24;
const ARROW_LINE_PX: i32 = 2;

/// A straight line: where it starts and where it ends.
pub type Line = ((f64, f64), (f64, f64));

pub struct Canvas {
    pub dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut c_void,
    pub width: i32,
    pub height: i32,
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}

impl Canvas {
    /// A bitmap of this size, every pixel see-through.
    pub fn new(width: i32, height: i32) -> Option<Canvas> {
        if width <= 0 || height <= 0 {
            return None;
        }
        unsafe {
            let dc = CreateCompatibleDC(None);
            if dc.is_invalid() {
                return None;
            }
            let mut info = solid_header(width, height);
            let mut bits: *mut c_void = std::ptr::null_mut();
            let bitmap = match CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
                _ => {
                    let _ = DeleteDC(dc);
                    return None;
                }
            };
            std::hint::black_box(&mut info);
            let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
            Some(Canvas {
                dc,
                bitmap,
                previous,
                bits,
                width,
                height,
            })
        }
    }

    pub fn pixels(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.bits as *mut u8,
                (self.width * self.height * 4) as usize,
            )
        }
    }

    /// Lays a colour over one pixel by how much of the pixel the shape covers.
    fn over(&mut self, x: i32, y: i32, (r, g, b): (u8, u8, u8), coverage: f64) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || coverage <= 0.0 {
            return;
        }
        let at = ((y * self.width + x) * 4) as usize;
        let c = coverage.min(1.0);
        let pixels = self.pixels();
        for (offset, colour) in [(0, b), (1, g), (2, r), (3, 255u8)] {
            let under = pixels[at + offset] as f64;
            pixels[at + offset] = (colour as f64 * c + under * (1.0 - c)).round() as u8;
        }
    }

    /// A rectangle with round corners, smooth at the corners. A radius of half its height
    /// makes a pill, and of half its side a disc.
    pub fn round_rect(&mut self, x: i32, y: i32, w: i32, h: i32, radius: i32, rgb: (u8, u8, u8)) {
        for py in 0..h {
            for px in 0..w {
                let coverage = crate::magnifier::corner_coverage(w, h, radius, px, py);
                self.over(x + px, y + py, rgb, coverage);
            }
        }
    }

    /// Straight lines with round ends, this wide, smooth at their edges.
    pub fn lines(&mut self, lines: &[Line], width: f64, rgb: (u8, u8, u8)) {
        let half = width / 2.0;
        for y in 0..self.height {
            for x in 0..self.width {
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                let nearest = lines
                    .iter()
                    .map(|&((ax, ay), (bx, by))| {
                        let (dx, dy) = (bx - ax, by - ay);
                        let length = dx * dx + dy * dy;
                        let along = if length == 0.0 {
                            0.0
                        } else {
                            (((px - ax) * dx + (py - ay) * dy) / length).clamp(0.0, 1.0)
                        };
                        ((px - ax - along * dx).powi(2) + (py - ay - along * dy).powi(2)).sqrt()
                    })
                    .fold(f64::MAX, f64::min);
                self.over(x, y, rgb, (half + 0.5 - nearest).clamp(0.0, 1.0));
            }
        }
    }

    /// Text, its top left at x,y. It has to lie wholly on pixels already filled solid: the
    /// text call clears the alpha of what it touches, and `solid` is the rectangle, x, y,
    /// width, height, whose alpha is put back to full after it.
    pub fn text(
        &mut self,
        face: HFONT,
        x: i32,
        y: i32,
        text: &str,
        (r, g, b): (u8, u8, u8),
        solid: (i32, i32, i32, i32),
    ) {
        if face.is_invalid() {
            return;
        }
        let text: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            let previous = SelectObject(self.dc, HGDIOBJ(face.0));
            SetBkMode(self.dc, TRANSPARENT);
            SetTextColor(
                self.dc,
                COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16),
            );
            let _ = TextOutW(self.dc, x, y, &text);
            let _ = GdiFlush();
            SelectObject(self.dc, previous);
        }
        let (width, height) = (self.width, self.height);
        let pixels = self.pixels();
        for py in solid.1.max(0)..(solid.1 + solid.3).min(height) {
            for px in solid.0.max(0)..(solid.0 + solid.2).min(width) {
                pixels[((py * width + px) * 4 + 3) as usize] = 255;
            }
        }
    }
}

/// How wide and tall a text is in a face.
pub fn measure(face: HFONT, text: &str) -> (i32, i32) {
    if face.is_invalid() {
        return (0, 0);
    }
    let text: Vec<u16> = text.encode_utf16().collect();
    let mut extent = SIZE::default();
    unsafe {
        let dc = CreateCompatibleDC(None);
        let previous = SelectObject(dc, HGDIOBJ(face.0));
        let _ = GetTextExtentPoint32W(dc, &text, &mut extent);
        SelectObject(dc, previous);
        let _ = DeleteDC(dc);
    }
    (extent.cx, extent.cy)
}

/// The round button at a display's scale, as it is or as it is under the pointer.
pub fn scroll_button(scale_percent: u32, hot: bool) -> Option<Canvas> {
    let side = scaled(BUTTON_PX, scale_percent);
    let mut canvas = Canvas::new(side, side)?;
    let ring = scaled(RING_PX, scale_percent);
    canvas.round_rect(0, 0, side, side, side / 2, WHITE_RGB);
    canvas.round_rect(
        ring,
        ring,
        side - 2 * ring,
        side - 2 * ring,
        side / 2,
        if hot { BLUE_HOT_RGB } else { BLUE_RGB },
    );
    // The arrow, drawn in a 24 px box in the button's middle: a shaft, and an open head.
    let unit = scaled(ARROW_PX, scale_percent) as f64 / 24.0;
    let origin = (side as f64 - 24.0 * unit) / 2.0;
    let at = |x: f64, y: f64| (origin + x * unit, origin + y * unit);
    canvas.lines(
        &[
            (at(12.0, 5.0), at(12.0, 19.0)),
            (at(6.0, 13.0), at(12.0, 19.0)),
            (at(18.0, 13.0), at(12.0, 19.0)),
        ],
        scaled(ARROW_LINE_PX, scale_percent) as f64,
        WHITE_RGB,
    );
    Some(canvas)
}
