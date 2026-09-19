//! The magnifier: the panel with the zoom circle and the text above it. ONE unit (Rotem,
//! 2026-09-19): the capture overlay, the editor's Ruler and its Color picker all show the
//! picture this module draws, so a number or a colour changed here changes in all three.
//!
//! A caller hands over the pixels around its point, one per cell, and the text for the top
//! of the panel; it gets the finished panel back, and places it with `place_panel`. The
//! overlay blends the panel onto its window; the editor's page asks for it over the region
//! scheme and paints the bytes as they come.

use std::ffi::c_void;

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, SIZE};
use windows::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
    DeleteObject, GdiFlush, GetGlyphOutlineW, GetTextExtentPoint32W, GetTextMetricsW, SelectObject,
    SetBkMode, SetTextColor, TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, FF_DONTCARE, FIXED,
    FW_MEDIUM, GDI_ERROR, GGO_METRICS, GLYPHMETRICS, HBITMAP, HDC, HFONT, HGDIOBJ, MAT2,
    OUT_DEFAULT_PRECIS, TEXTMETRICW, TRANSPARENT,
};

/// The text above the circle (Rotem, 2026-09-17, as the capture's size label): 14 px on the
/// app's background colour, 8 px corners, 8 px of padding at the sides and above (Rotem,
/// 2026-09-19, from 4 above). These are the numbers at 100%; every caller scales them by its display's own
/// scale, as the cursor scales. The text's colour and face are not stated: the ruler's
/// white, in the page's UI font, until they are.
const TEXT_PX: i32 = 14;
const RADIUS_PX: i32 = 8;
const PAD_X_PX: i32 = 8;
const PAD_Y_PX: i32 = 10;
/// Between the text and the circle (Rotem, 2026-09-19, from the 4 px above the text).
const TEXT_GAP_PX: i32 = 10;
/// The app background, `project-os/Design.md`; the page carries the same value as `--paper`.
pub const FILL_RGB: (u8, u8, u8) = (0x0D, 0x0E, 0x12);
const TEXT_RGB: (u8, u8, u8) = (0xF2, 0xF2, 0xF2);

/// The circle (Rotem, 2026-09-18, with a picture): 112 px across, showing the pixels
/// around the pointer large enough to tell apart. What the picture shows and his words do
/// not, provisional until he states it: the circle in a panel of the label's fill and
/// corners, 8 px around it (Rotem, 2026-09-19, from 4); the panel below the pointer with its left edge 8 px right of
/// it, and on the pointer's other side where that would leave its ground; 8 px to a source
/// pixel; a 1 px grid between them; #2554FB lines on the centre pixel's top and left edges
/// (his, the same day); a 2 px ring in the text's white.
const DIAMETER_PX: i32 = 112;
const CELL_PX: i32 = 8;
const INSET_PX: i32 = 8;
const GAP_PX: i32 = 8;
const RING_PX: i32 = 2;
const RING_RGB: (u8, u8, u8) = (0xF2, 0xF2, 0xF2);
/// The two lines inside the circle, Rotem's #2554FB (2026-09-18, from the frame's blue).
pub const LINE_RGB: (u8, u8, u8) = (0x25, 0x54, 0xFB);
/// How much of the lines' colour the pixel on each side of them takes: 1.64 px wide (after
/// 1.44), less the whole pixel in the middle, halved.
pub const LINE_SIDE: f64 = 0.32;
/// The grid between the pixels, Rotem's #808080 (2026-09-18, after #A8A8A8, #8C8C8C and
/// #707070, from the pixel darkened by half).
const GRID_RGB: (u8, u8, u8) = (0x80, 0x80, 0x80);
/// How much of the grid's colour lies over the pixel under it: his 48%, the same day, after 64%.
const GRID_STRENGTH: f64 = 0.48;

/// A size at 100%, as it is on a display at this scale.
pub fn scaled(px: i32, scale_percent: u32) -> i32 {
    (px * scale_percent as i32 + 50) / 100
}

pub fn solid_header(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
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
    }
}

/// Google Sans, medium, the page's own bundled face unpacked from its web format (Rotem,
/// 2026-09-19, from Segoe UI): Windows' text drawing reads a .ttf, never a .woff2. Given to
/// this process alone, once, from memory; nothing is installed on the machine.
static FACE: &[u8] = include_bytes!("../../editor/fonts/google-sans-medium-latin.ttf");
static FACE_LOADED: std::sync::Once = std::sync::Once::new();

/// The text's face at a scale: a negative height is the em size, so 14 px here is the
/// 14 px the page means. The caller deletes it. A face that cannot be made is invalid,
/// and leaves the text unmeasured and undrawn.
pub fn font(scale_percent: u32) -> HFONT {
    FACE_LOADED.call_once(|| {
        let count = 0u32;
        let _ = unsafe {
            AddFontMemResourceEx(
                FACE.as_ptr() as *const c_void,
                FACE.len() as u32,
                None,
                &count,
            )
        };
    });
    unsafe {
        CreateFontW(
            -scaled(TEXT_PX, scale_percent),
            0,
            0,
            0,
            FW_MEDIUM.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Google Sans 18pt Medium"),
        )
    }
}

/// A size as the panel writes it: 160x100.
pub fn size_text(width: u32, height: u32) -> String {
    format!("{width}x{height}")
}

/// How far the panel stands from the pointer, at a scale.
pub fn gap(scale_percent: u32) -> i32 {
    scaled(GAP_PX, scale_percent)
}

/// How many source pixels the circle shows across at a scale, and one's width on it.
pub fn cells_at(scale_percent: u32) -> (i32, i32) {
    let cell = scaled(CELL_PX, scale_percent).max(1);
    (cell_count(scaled(DIAMETER_PX, scale_percent), cell), cell)
}

/// The pixels around the point, one per cell, BGRA, `count` across and down, the point's
/// own pixel in the middle. An alpha of 0 marks a cell with no pixel behind it, past the
/// picture's or the display's edge, where the panel's own fill shows.
pub struct Cells {
    pub count: i32,
    pub bgra: Vec<u8>,
}

impl Cells {
    /// Every cell empty.
    pub fn empty(count: i32) -> Self {
        Cells {
            count,
            bgra: vec![0u8; (count * count * 4) as usize],
        }
    }
}

/// The panel as laid out at one scale, with or without its text.
pub struct Layout {
    pub width: i32,
    pub height: i32,
    /// Its corners.
    radius: i32,
    /// The circle's centre inside the panel, and its radius.
    pub circle: (i32, i32, i32),
    ring: i32,
    /// One source pixel's width on the circle, and how many across, an odd count so the
    /// point's own pixel sits in the middle.
    pub cell: i32,
    pub cells: i32,
    /// The text above the circle, and where it sits inside the panel.
    text: Option<(Vec<u16>, (i32, i32))>,
}

/// Lays the panel out: the text row first, with its padding, then the circle with its
/// inset (Rotem, 2026-09-18, the size above the circle, from under it). The text is
/// measured with the given face on the given DC; one that cannot be measured is left out.
pub fn layout(hdc: HDC, face: HFONT, scale_percent: u32, text: Option<&str>) -> Layout {
    let diameter = scaled(DIAMETER_PX, scale_percent);
    let (cells, cell) = cells_at(scale_percent);
    let inset = scaled(INSET_PX, scale_percent);
    let text = text.and_then(|text| {
        if face.is_invalid() {
            return None;
        }
        let text: Vec<u16> = text.encode_utf16().collect();
        let mut extent = SIZE::default();
        // The room the letters really take (Rotem, 2026-09-19): this face's line is far
        // taller than its digits, so the padding above and the gap to the circle are
        // measured from a digit's own top and from the line it stands on, not from the
        // line's box. Where the digit cannot be measured, the box stands in.
        let mut ink = None;
        let measured = unsafe {
            let previous = SelectObject(hdc, HGDIOBJ(face.0));
            let ok = GetTextExtentPoint32W(hdc, &text, &mut extent).as_bool();
            let mut metrics = TEXTMETRICW::default();
            let mut glyph = GLYPHMETRICS::default();
            let identity = MAT2 {
                eM11: FIXED { fract: 0, value: 1 },
                eM12: FIXED::default(),
                eM21: FIXED::default(),
                eM22: FIXED { fract: 0, value: 1 },
            };
            if GetTextMetricsW(hdc, &mut metrics).as_bool()
                && GetGlyphOutlineW(hdc, '0' as u32, GGO_METRICS, &mut glyph, 0, None, &identity)
                    != GDI_ERROR as u32
                && glyph.gmptGlyphOrigin.y > 0
            {
                ink = Some((
                    metrics.tmAscent - glyph.gmptGlyphOrigin.y,
                    glyph.gmptGlyphOrigin.y,
                ));
            }
            SelectObject(hdc, previous);
            ok
        };
        if !measured || extent.cx <= 0 || extent.cy <= 0 {
            return None;
        }
        Some((text, extent, ink.unwrap_or((0, extent.cy))))
    });
    let pad = (
        scaled(PAD_X_PX, scale_percent),
        scaled(PAD_Y_PX, scale_percent),
    );
    let width = (diameter + 2 * inset).max(text.as_ref().map_or(0, |(_, e, _)| e.cx + 2 * pad.0));
    let above = text.as_ref().map_or(inset, |(_, _, (_, tall))| {
        pad.1 + tall + scaled(TEXT_GAP_PX, scale_percent)
    });
    let height = above + diameter + inset;
    let text = text.map(|(t, e, (top, _))| (t, ((width - e.cx) / 2, pad.1 - top)));
    Layout {
        width,
        height,
        radius: scaled(RADIUS_PX, scale_percent),
        circle: (width / 2, above + diameter / 2, diameter / 2),
        ring: scaled(RING_PX, scale_percent),
        cell,
        cells,
        text,
    }
}

/// Where the panel goes: below the pointer by the gap, its left edge the gap right of the
/// pointer, and on the pointer's other side where that would leave the display, so it is
/// never cut off at an edge.
pub fn place_panel(
    pointer: (i32, i32),
    size: (i32, i32),
    display: (i32, i32),
    gap: i32,
) -> (i32, i32) {
    let mut left = pointer.0 + gap;
    if left + size.0 > display.0 {
        left = pointer.0 - gap - size.0;
    }
    let mut top = pointer.1 + gap;
    if top + size.1 > display.1 {
        top = pointer.1 - gap - size.1;
    }
    (left.max(0), top.max(0))
}

/// How many source pixels the circle shows across: enough cells to cover its diameter,
/// and an odd count so the pointer's own pixel is the middle one.
pub fn cell_count(diameter: i32, cell: i32) -> i32 {
    let n = (diameter + cell - 1) / cell.max(1);
    if n % 2 == 0 {
        n + 1
    } else {
        n
    }
}

/// The cells, each one a cell wide, BGRA: no smoothing, the grid's line on the last pixel
/// of every cell across and down, and the #2554FB lines on the centre cell's top and left
/// edges, the grid's line there, so they cross at the point's pixel's top left corner and
/// the pixel itself is whole (Rotem, 2026-09-18, from another tool's picture).
fn magnified(cells: &Cells, cell: i32) -> Vec<u8> {
    let count = cells.count;
    let side = count * cell;
    let mut bytes = vec![0u8; (side * side * 4) as usize];
    let centre = (count / 2) * cell - 1;
    for y in 0..side {
        for x in 0..side {
            let at = ((y * side + x) * 4) as usize;
            let from = (((y / cell) * count + x / cell) * 4) as usize;
            let (b, g, r) = if cells.bgra[from + 3] == 0 {
                (FILL_RGB.2, FILL_RGB.1, FILL_RGB.0)
            } else {
                (cells.bgra[from], cells.bgra[from + 1], cells.bgra[from + 2])
            };
            bytes[at] = b;
            bytes[at + 1] = g;
            bytes[at + 2] = r;
            if x == centre || y == centre {
                bytes[at] = LINE_RGB.2;
                bytes[at + 1] = LINE_RGB.1;
                bytes[at + 2] = LINE_RGB.0;
            } else if x % cell == cell - 1 || y % cell == cell - 1 {
                let over = |under: u8, grid: u8| {
                    (under as f64 * (1.0 - GRID_STRENGTH) + grid as f64 * GRID_STRENGTH).round()
                        as u8
                };
                bytes[at] = over(bytes[at], GRID_RGB.2);
                bytes[at + 1] = over(bytes[at + 1], GRID_RGB.1);
                bytes[at + 2] = over(bytes[at + 2], GRID_RGB.0);
            }
            // The lines are 1.64 px wide, centred on the grid's line (Rotem, 2026-09-18, so
            // the eye finds them sooner): the pixel on each side takes 0.32 of their colour.
            if x != centre && y != centre && ((x - centre).abs() == 1 || (y - centre).abs() == 1) {
                let side_of = |under: u8, line: u8| {
                    (under as f64 * (1.0 - LINE_SIDE) + line as f64 * LINE_SIDE).round() as u8
                };
                bytes[at] = side_of(bytes[at], LINE_RGB.2);
                bytes[at + 1] = side_of(bytes[at + 1], LINE_RGB.1);
                bytes[at + 2] = side_of(bytes[at + 2], LINE_RGB.0);
            }
            bytes[at + 3] = 255;
        }
    }
    bytes
}

/// How much of a pixel a rounded rectangle of this size covers: 1 inside, 0 outside, and
/// the fraction of it inside the arc at the corners, so the corners are smooth. The radius
/// is cut down to half the shorter side, as a page's corners are.
pub fn corner_coverage(w: i32, h: i32, radius: i32, x: i32, y: i32) -> f64 {
    let r = radius.min(w / 2).min(h / 2).max(0) as f64;
    let px = x as f64 + 0.5;
    let py = y as f64 + 0.5;
    let cx = if px < r {
        r
    } else if px > w as f64 - r {
        w as f64 - r
    } else {
        return 1.0;
    };
    let cy = if py < r {
        r
    } else if py > h as f64 - r {
        h as f64 - r
    } else {
        return 1.0;
    };
    let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
    (r + 0.5 - d).clamp(0.0, 1.0)
}

/// A drawn panel: a bitmap of the layout's size in a DC of its own, BGRA premultiplied by
/// each pixel's coverage, ready to be blended onto a window or read. Freed when dropped.
pub struct Rendered {
    pub dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut c_void,
    pub width: i32,
    pub height: i32,
}

impl Rendered {
    pub fn pixels(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.bits as *const u8,
                (self.width * self.height * 4) as usize,
            )
        }
    }
}

impl Drop for Rendered {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}

/// Draws the panel: the rounded fill built pixel by pixel, the pixels around the point
/// inside the circle with the ring around them, the corners' and the circle's edge pixels
/// covered by the fraction of them inside the arc so both are smooth, and the text above
/// the circle. Every pixel wholly inside the panel keeps a full alpha: a GDI text call
/// clears the alpha of the pixels it touches, and the overlay's pixels are shown by theirs.
/// `hdc` is the DC the bitmap is made compatible with, none for the screen's.
pub unsafe fn render(
    hdc: Option<HDC>,
    face: HFONT,
    layout: &Layout,
    cells: &Cells,
) -> Option<Rendered> {
    let (w, h) = (layout.width, layout.height);
    if w <= 0 || h <= 0 || cells.count != layout.cells {
        return None;
    }
    let pixels_around = magnified(cells, layout.cell);
    let dc = unsafe { CreateCompatibleDC(hdc) };
    if dc.is_invalid() {
        return None;
    }
    let mut info = solid_header(w, h);
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap = match unsafe { CreateDIBSection(hdc, &info, DIB_RGB_COLORS, &mut bits, None, 0) } {
        Ok(bitmap) if !bitmap.is_invalid() && !bits.is_null() => bitmap,
        _ => {
            let _ = unsafe { DeleteDC(dc) };
            return None;
        }
    };
    std::hint::black_box(&mut info);
    let len = (w * h * 4) as usize;

    // The fill, premultiplied by each pixel's coverage, BGRA as GDI wants it; then the
    // circle over it: the ring between its radius and the picture's, the pixels around
    // the point inside, each edge blended by its coverage.
    {
        let pixels = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, len) };
        let (r, g, b) = FILL_RGB;
        let (cx, cy, radius) = layout.circle;
        let side = layout.cells * layout.cell;
        let centre = (layout.cells / 2) * layout.cell - 1;
        let ring = (RING_RGB.2, RING_RGB.1, RING_RGB.0);
        for y in 0..h {
            for x in 0..w {
                let coverage = corner_coverage(w, h, layout.radius, x, y);
                let at = ((y * w + x) * 4) as usize;
                let over = |c: u8| (c as f64 * coverage).round() as u8;
                pixels[at] = over(b);
                pixels[at + 1] = over(g);
                pixels[at + 2] = over(r);
                pixels[at + 3] = (255.0 * coverage).round() as u8;

                let dx = x as f64 + 0.5 - cx as f64;
                let dy = y as f64 + 0.5 - cy as f64;
                let distance = (dx * dx + dy * dy).sqrt();
                let outer = (radius as f64 + 0.5 - distance).clamp(0.0, 1.0);
                if outer <= 0.0 {
                    continue;
                }
                let inner = ((radius - layout.ring) as f64 + 0.5 - distance).clamp(0.0, 1.0);
                let (mx, my) = (x - cx + centre, y - cy + centre);
                let around = if mx >= 0 && my >= 0 && mx < side && my < side {
                    let from = ((my * side + mx) * 4) as usize;
                    (
                        pixels_around[from],
                        pixels_around[from + 1],
                        pixels_around[from + 2],
                    )
                } else {
                    (b, g, r)
                };
                let blend = |fill: u8, ring: u8, inside: u8| {
                    (fill as f64 * (1.0 - outer)
                        + ring as f64 * (outer - inner)
                        + inside as f64 * inner)
                        .round() as u8
                };
                pixels[at] = blend(b, ring.0, around.0);
                pixels[at + 1] = blend(g, ring.1, around.1);
                pixels[at + 2] = blend(r, ring.2, around.2);
                pixels[at + 3] = 255;
            }
        }
    }

    let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    if let Some((text, at)) = &layout.text {
        let previous_font = unsafe { SelectObject(dc, HGDIOBJ(face.0)) };
        unsafe {
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(
                dc,
                COLORREF(TEXT_RGB.0 as u32 | (TEXT_RGB.1 as u32) << 8 | (TEXT_RGB.2 as u32) << 16),
            );
            let _ = TextOutW(dc, at.0, at.1, text);
            let _ = GdiFlush();
            SelectObject(dc, previous_font);
        }
        // The text sits in the flat part of the panel, where every pixel is wholly
        // covered; the text call cleared the alpha of the pixels it touched, and this
        // puts it back.
        let pixels = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, len) };
        for y in 0..h {
            for x in 0..w {
                if corner_coverage(w, h, layout.radius, x, y) >= 1.0 {
                    pixels[((y * w + x) * 4 + 3) as usize] = 255;
                }
            }
        }
    }
    Some(Rendered {
        dc,
        bitmap,
        previous,
        bits,
        width: w,
        height: h,
    })
}
