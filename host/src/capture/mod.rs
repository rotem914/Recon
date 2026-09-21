//! The capture interface, and the frame it produces.
//!
//! Part 6a of the plan chose one initial capture path and required it to sit behind a small
//! interface, so the implementation can be replaced, or joined by a second one, without the
//! rest of the application noticing. Part 8 says a second path is earned only by a
//! demonstrated failure of this one.
//!
//! The interface is deliberately "freeze once, hand back a frame". A future video source
//! implements the same trait and adds its own way of producing further frames; nothing here
//! assumes one frame per session, and nothing here brings video infrastructure into v1.

pub mod coords;
pub mod display;
pub mod pointer;
pub mod screen;

// Re-exported because every caller needs the conversion itself; the coordinate TYPES are
// reached through coords:: on purpose, so a reader always sees which space they are in.
pub use coords::desktop_to_image;

use coords::{FrameGeometry, ImageRect};

/// A frozen picture of everything the user could see, plus where it sits in desktop space.
///
/// `rgba` is tightly packed, 4 bytes per pixel, top-down. A screen copy carries no
/// meaningful transparency, so alpha is 255 everywhere: the decoded-image contract in §3.7
/// wants one predictable shape, not a per-source surprise.
pub struct Frame {
    pub geometry: FrameGeometry,
    pub rgba: Vec<u8>,
    /// Which implementation produced it, for the log and for the S0.8 report.
    pub source: &'static str,
}

impl Frame {
    pub fn width(&self) -> u32 {
        self.geometry.width
    }

    pub fn height(&self) -> u32 {
        self.geometry.height
    }

    /// Copies one image-space rectangle out of the frame, row by row.
    ///
    /// The rectangle must already have been through `desktop_to_image`, which is what
    /// guarantees it is inside the frame; this asserts that rather than trusting it, because
    /// a wrong rectangle here reads pixels from the wrong row and looks like a plausible
    /// picture of the wrong thing.
    pub fn crop(&self, rect: ImageRect) -> Option<Vec<u8>> {
        let (fw, fh) = (self.geometry.width, self.geometry.height);
        if rect.width == 0
            || rect.height == 0
            || rect.x.checked_add(rect.width)? > fw
            || rect.y.checked_add(rect.height)? > fh
        {
            return None;
        }

        let stride = fw as usize * 4;
        let row_bytes = rect.width as usize * 4;
        let mut out = Vec::with_capacity(row_bytes * rect.height as usize);
        for row in 0..rect.height {
            let start = (rect.y + row) as usize * stride + rect.x as usize * 4;
            out.extend_from_slice(&self.rgba[start..start + row_bytes]);
        }
        Some(out)
    }
}

#[derive(Debug)]
pub enum CaptureError {
    /// The virtual desktop reported a size Windows should never report.
    ImpossibleBounds { width: i32, height: i32 },
    /// A GDI call failed. Carries the call's name, because the failing one is the whole story.
    Gdi(&'static str),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::ImpossibleBounds { width, height } => {
                write!(f, "the virtual desktop measured {width}x{height}")
            }
            CaptureError::Gdi(call) => write!(f, "{call} failed"),
        }
    }
}

/// One way of freezing what the user can see.
pub trait CaptureSource {
    /// Freeze everything, once, and hand back the pixels with their place in desktop space.
    fn freeze(&self) -> Result<Frame, CaptureError>;

    /// Named in the log and in the S0.8 report, so a measurement can be attributed.
    fn name(&self) -> &'static str;
}
