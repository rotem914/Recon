//! The image source: a path in, a decoded frame and its metadata out.
//!
//! Part 5 gives this one interface and one native implementation for every format, so past
//! this boundary the editor cannot tell a capture from a file. The contract it hands back is
//! §3.7's decoded-image contract: orientation applied exactly once here, dimensions after
//! orientation, sRGB, and the frame or page addressable by index.
//!
//! What lives behind it: the `image` crate for PNG, JPEG, BMP, GIF and WebP (`raster.rs`),
//! `resvg` for SVG (`svg.rs`), and the Windows imaging stack for TIFF, HEIC and AVIF
//! (`wic.rs`). The last one depends on codecs installed on the machine, and a missing codec
//! is reported as a missing codec, never as a corrupt file.
//!
//! The invariant this module is measured against (CLAUDE.md rule 11): an external file is
//! only ever READ. Nothing here opens a file for writing, writes beside it, or copies it.
//! The decode report proves that by hashing every file before and after.

use std::path::{Path, PathBuf};

pub mod raster;
pub mod svg;
pub mod wic;

#[cfg(feature = "stage0-checks")]
pub mod fixtures;
#[cfg(feature = "stage0-checks")]
pub mod report;

/// The formats Recon promises in §3.7, by what the file actually is, not by its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpeg,
    Bmp,
    Gif,
    WebP,
    Tiff,
    Heic,
    Avif,
    Svg,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Png => "PNG",
            Format::Jpeg => "JPEG",
            Format::Bmp => "BMP",
            Format::Gif => "GIF",
            Format::WebP => "WebP",
            Format::Tiff => "TIFF",
            Format::Heic => "HEIC",
            Format::Avif => "AVIF",
            Format::Svg => "SVG",
        }
    }
}

/// What kind of thing the file is, which decides what "the displayed frame" means.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// One picture.
    Still,
    /// Frames in time, each with a delay in milliseconds. Annotation takes a still of one.
    Animation { delays_ms: Vec<u32> },
    /// Pages, as in a multipage TIFF. Annotation operates on one.
    Pages { count: u32 },
    /// A vector, rendered at a size chosen at open time; part 5's one exception.
    Vector { intrinsic_w: f32, intrinsic_h: f32 },
}

impl Kind {
    /// How many frames or pages can be asked for by index.
    pub fn count(&self) -> u32 {
        match self {
            Kind::Still | Kind::Vector { .. } => 1,
            Kind::Animation { delays_ms } => delays_ms.len().max(1) as u32,
            Kind::Pages { count } => (*count).max(1),
        }
    }
}

/// One decoded frame in the contract's shape: RGBA, straight alpha, top-down, sRGB,
/// oriented.
pub struct DecodedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// What happened at decode, in words, for the evidence line and the S0.8 report. Three of
/// the fields are read only by the decode report, which the product build does not carry.
#[derive(Debug, Clone, Default)]
#[cfg_attr(not(feature = "stage0-checks"), allow(dead_code))]
pub struct Notes {
    /// The EXIF orientation that was applied, 1 to 8, if the file carried one.
    pub orientation_applied: Option<u8>,
    /// Where the colour came from: "sRGB, untagged", "converted from an embedded profile
    /// of N bytes", or a reason it could not be.
    pub color: String,
    /// Whether the format's transparency survived: "alpha preserved" or "opaque".
    pub alpha: String,
    /// The provider that answered, for the report's chain-of-custody column.
    pub provider: &'static str,
    /// Anything else worth a reader's attention.
    pub remarks: Vec<String>,
}

/// An opened file: its metadata, and a way to decode any frame or page by index.
pub struct Opened {
    pub path: PathBuf,
    pub format: Format,
    pub kind: Kind,
    /// Dimensions after orientation, of frame 0.
    pub width: u32,
    pub height: u32,
    pub notes: Notes,
    source: Box<dyn FrameSource>,
}

impl Opened {
    /// The frame or page at `index`, decoded on demand. Index 0 is always valid.
    pub fn frame(&mut self, index: u32) -> Result<DecodedFrame, OpenError> {
        if index >= self.kind.count() {
            return Err(OpenError::NoSuchFrame {
                index,
                count: self.kind.count(),
            });
        }
        self.source.frame(index)
    }
}

/// What a provider has to offer past open.
pub trait FrameSource: Send {
    fn frame(&mut self, index: u32) -> Result<DecodedFrame, OpenError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The file could not be read at all: missing, unreadable, a folder.
    Unreadable(String),
    /// The bytes are not a format Recon supports. Carries what the sniff saw.
    Unsupported(String),
    /// A supported format that this machine has no decoder for, with the way to get one.
    MissingCodec {
        format: Format,
        how: String,
    },
    /// A supported format whose bytes did not decode. Carries the decoder's own words.
    Corrupt {
        format: Format,
        detail: String,
    },
    NoSuchFrame {
        index: u32,
        count: u32,
    },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::Unreadable(why) => write!(f, "the file could not be read: {why}"),
            OpenError::Unsupported(seen) => write!(f, "not an image format Recon opens ({seen})"),
            OpenError::MissingCodec { format, how } => {
                write!(
                    f,
                    "{} needs a codec this machine does not have. {how}",
                    format.name()
                )
            }
            OpenError::Corrupt { format, detail } => {
                write!(f, "the {} file did not decode: {detail}", format.name())
            }
            OpenError::NoSuchFrame { index, count } => {
                write!(f, "frame {index} of {count} does not exist")
            }
        }
    }
}

/// What the first bytes say the file is. The extension is never consulted: a renamed file
/// is the everyday case in a folder of client material.
pub fn sniff(bytes: &[u8]) -> Option<Format> {
    let has = |prefix: &[u8]| bytes.len() >= prefix.len() && &bytes[..prefix.len()] == prefix;
    if has(b"\x89PNG\r\n\x1a\n") {
        return Some(Format::Png);
    }
    if has(b"\xFF\xD8\xFF") {
        return Some(Format::Jpeg);
    }
    if has(b"BM") {
        return Some(Format::Bmp);
    }
    if has(b"GIF87a") || has(b"GIF89a") {
        return Some(Format::Gif);
    }
    if has(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        return Some(Format::WebP);
    }
    if has(b"II*\0") || has(b"MM\0*") {
        return Some(Format::Tiff);
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        let brand = &bytes[8..12];
        if brand == b"avif" || brand == b"avis" {
            return Some(Format::Avif);
        }
        if brand == b"heic"
            || brand == b"heix"
            || brand == b"mif1"
            || brand == b"heif"
            || brand == b"msf1"
        {
            return Some(Format::Heic);
        }
    }
    // SVG is text: an XML declaration or the root element within the first kilobyte, with
    // a BOM or leading whitespace allowed.
    let head = &bytes[..bytes.len().min(1024)];
    let text = String::from_utf8_lossy(head);
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    if trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && text.contains("<svg")) {
        return Some(Format::Svg);
    }
    None
}

/// Opens a file for reading, decides what it is from its bytes, and hands it to the one
/// provider for that format. The whole file is read once, here; no provider touches the
/// path again, which is how "only ever read" stays easy to see.
pub fn open(path: &Path) -> Result<Opened, OpenError> {
    let bytes = std::fs::read(path).map_err(|err| OpenError::Unreadable(err.to_string()))?;
    let format = sniff(&bytes).ok_or_else(|| {
        let seen: String = bytes
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        OpenError::Unsupported(format!("first bytes {seen}"))
    })?;
    match format {
        Format::Png | Format::Jpeg | Format::Bmp | Format::Gif | Format::WebP => {
            raster::open(path, format, bytes)
        }
        Format::Svg => svg::open(path, bytes),
        Format::Tiff | Format::Heic | Format::Avif => wic::open(path, format, bytes),
    }
}

/// The eight-bit sRGB conversion of a decoded image carrying an embedded ICC profile.
///
/// One place, so orientation-once and sRGB-once (§3.7) each have exactly one place to be
/// wrong. Returns the colour note for the evidence line.
pub fn to_srgb(rgba: &mut [u8], icc: Option<&[u8]>) -> String {
    let Some(profile_bytes) = icc else {
        return "sRGB, untagged".to_string();
    };
    let Some(input) = qcms::Profile::new_from_slice(profile_bytes, false) else {
        return format!(
            "an embedded profile of {} bytes could not be parsed, so treated as sRGB",
            profile_bytes.len()
        );
    };
    if input.is_sRGB() {
        return format!(
            "an embedded profile of {} bytes that is sRGB, so nothing to convert",
            profile_bytes.len()
        );
    }
    let output = qcms::Profile::new_sRGB();
    let Some(transform) = qcms::Transform::new(
        &input,
        &output,
        qcms::DataType::RGBA8,
        qcms::Intent::Perceptual,
    ) else {
        return format!(
            "an embedded profile of {} bytes that no transform could be built for, so treated as sRGB",
            profile_bytes.len()
        );
    };
    transform.apply(rgba);
    format!(
        "converted to sRGB from an embedded profile of {} bytes",
        profile_bytes.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sniff_reads_bytes_not_names() {
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\n...."), Some(Format::Png));
        assert_eq!(sniff(b"\xFF\xD8\xFF\xE0"), Some(Format::Jpeg));
        assert_eq!(sniff(b"GIF89a"), Some(Format::Gif));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(Format::WebP));
        assert_eq!(sniff(b"II*\0"), Some(Format::Tiff));
        assert_eq!(sniff(b"\0\0\0\x18ftypavif"), Some(Format::Avif));
        assert_eq!(sniff(b"\0\0\0\x18ftypheic"), Some(Format::Heic));
        assert_eq!(sniff(b"\0\0\0\x18ftypmif1"), Some(Format::Heic));
        assert_eq!(
            sniff(b"<?xml version=\"1.0\"?>\n<svg xmlns=\"x\"/>"),
            Some(Format::Svg)
        );
        assert_eq!(sniff(b"\xEF\xBB\xBF  <svg/>"), Some(Format::Svg));
        assert_eq!(sniff(b"hello"), None);
        assert_eq!(sniff(b""), None);
    }

    #[test]
    fn a_frame_past_the_end_is_refused() {
        assert_eq!(Kind::Still.count(), 1);
        assert_eq!(
            Kind::Animation {
                delays_ms: vec![10, 10, 10]
            }
            .count(),
            3
        );
        assert_eq!(Kind::Pages { count: 0 }.count(), 1);
    }

    #[test]
    fn untagged_pixels_are_left_alone() {
        let mut px = vec![200, 10, 10, 255, 0, 0, 0, 0];
        let note = to_srgb(&mut px, None);
        assert_eq!(note, "sRGB, untagged");
        assert_eq!(px, vec![200, 10, 10, 255, 0, 0, 0, 0]);
    }
}
