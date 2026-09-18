//! SVG through `resvg`.
//!
//! A vector has no pixel size of its own. For viewing it is rendered at whatever size the
//! window asks for; for annotation the size is fixed once, at the displayed size in physical
//! pixels when Annotate is pressed (§3.5, part 5's one exception), and that decision lives
//! with the editor, not here. This provider renders at a requested pixel size, and the
//! intrinsic size travels in `Kind::Vector` so the caller can choose.
//!
//! What `resvg` renders and what it drops is S0.5's content test, compared against the web
//! view's own rendering of the same file; the report names any element that went missing.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use resvg::{tiny_skia, usvg};

use super::{DecodedFrame, Format, FrameSource, Kind, Notes, OpenError, Opened};

/// The system fonts, loaded once: an SVG with `<text>` needs them, and loading them costs
/// tens of milliseconds that should not be paid per file.
fn options() -> &'static usvg::Options<'static> {
    static OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        opt
    })
}

pub fn open(path: &Path, bytes: Vec<u8>) -> Result<Opened, OpenError> {
    let started = std::time::Instant::now();
    let tree = usvg::Tree::from_data(&bytes, options()).map_err(|err| OpenError::Corrupt {
        format: Format::Svg,
        detail: err.to_string(),
    })?;
    let size = tree.size();
    let (intrinsic_w, intrinsic_h) = (size.width(), size.height());
    // The default render is the intrinsic size, rounded up, so a viewer that has not yet
    // chosen a size still gets a picture. Capped so a pathological viewBox cannot ask for
    // gigabytes.
    let (width, height) = render_size(intrinsic_w, intrinsic_h, 1.0);
    let notes = Notes {
        orientation_applied: None,
        color: "sRGB, as SVG colours are".into(),
        alpha: "alpha preserved".into(),
        provider: "resvg",
        remarks: vec![
            format!("intrinsic size {intrinsic_w}x{intrinsic_h}"),
            format!("parsed in {} ms", started.elapsed().as_millis()),
        ],
    };
    Ok(Opened {
        path: path.to_path_buf(),
        format: Format::Svg,
        kind: Kind::Vector {
            intrinsic_w,
            intrinsic_h,
        },
        width,
        height,
        notes,
        source: Box::new(Vector {
            tree: Arc::new(tree),
            scale: 1.0,
        }),
    })
}

/// The pixel size for a vector at a scale, at least one pixel and at most 16384 a side.
pub fn render_size(intrinsic_w: f32, intrinsic_h: f32, scale: f32) -> (u32, u32) {
    let w = (intrinsic_w * scale).ceil().clamp(1.0, 16384.0) as u32;
    let h = (intrinsic_h * scale).ceil().clamp(1.0, 16384.0) as u32;
    (w, h)
}

/// Rasterizes a tree to straight-alpha RGBA at a scale.
pub fn rasterize(tree: &usvg::Tree, scale: f32) -> Result<DecodedFrame, OpenError> {
    let size = tree.size();
    let (width, height) = render_size(size.width(), size.height(), scale);
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(OpenError::Corrupt {
        format: Format::Svg,
        detail: format!("no pixmap of {width}x{height}"),
    })?;
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    // tiny-skia holds premultiplied pixels; the contract wants straight alpha.
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Ok(DecodedFrame {
        width,
        height,
        rgba,
    })
}

/// The vector itself, for a view closer than the size it was rasterized at: the region
/// service draws the part on screen from it, at the zoom, so an enlarged SVG has its own
/// edges and not the pixels of a smaller render. Shared, so a draw never holds the
/// document's lock.
#[derive(Clone)]
pub struct Drawing(Arc<usvg::Tree>);

impl Drawing {
    /// The rectangle at `x`, `y` of the scale-one render, drawn `width` by `height` pixels
    /// at `scale_x` by `scale_y`, straight alpha.
    pub fn region(
        &self,
        x: u32,
        y: u32,
        scale_x: f32,
        scale_y: f32,
        width: u32,
        height: u32,
    ) -> Result<DecodedFrame, OpenError> {
        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(OpenError::Corrupt {
            format: Format::Svg,
            detail: format!("no pixmap of {width}x{height}"),
        })?;
        let transform = tiny_skia::Transform::from_scale(scale_x, scale_y)
            .post_translate(-(x as f32) * scale_x, -(y as f32) * scale_y);
        resvg::render(&self.0, transform, &mut pixmap.as_mut());
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for px in pixmap.pixels() {
            let c = px.demultiply();
            rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
        }
        Ok(DecodedFrame {
            width,
            height,
            rgba,
        })
    }
}

struct Vector {
    tree: Arc<usvg::Tree>,
    scale: f32,
}

impl FrameSource for Vector {
    fn frame(&mut self, _index: u32) -> Result<DecodedFrame, OpenError> {
        rasterize(&self.tree, self.scale)
    }

    fn drawing(&self) -> Option<Drawing> {
        Some(Drawing(self.tree.clone()))
    }
}
