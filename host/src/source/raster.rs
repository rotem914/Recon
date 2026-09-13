//! PNG, JPEG, BMP, GIF and WebP through the `image` crate.
//!
//! Three things happen here that must happen exactly once (§3.7): the EXIF orientation is
//! applied, an embedded colour profile is converted to sRGB, and the frames of an animation
//! are made addressable by index. The frame that comes out is already upright and sRGB, and
//! nothing downstream rotates or converts anything.

use std::io::Cursor;
use std::path::Path;

use image::{AnimationDecoder, ImageDecoder};

use super::{to_srgb, DecodedFrame, Format, FrameSource, Kind, Notes, OpenError, Opened};

pub fn open(path: &Path, format: Format, bytes: Vec<u8>) -> Result<Opened, OpenError> {
    let corrupt = |err: &dyn std::fmt::Display| OpenError::Corrupt {
        format,
        detail: err.to_string(),
    };

    // Animations first: GIF always, WebP when its container says so. Every frame is decoded
    // now, because the crate's animation decoders are forward-only and a frame asked for by
    // index has to be there when the user steps to it.
    let animated = match format {
        Format::Gif => true,
        Format::WebP => {
            let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(&bytes))
                .map_err(|e| corrupt(&e))?;
            decoder.has_animation()
        }
        _ => false,
    };
    if animated {
        return open_animation(path, format, bytes);
    }

    let (frame, notes) = decode_still(&bytes, format)?;
    let (width, height) = (frame.width, frame.height);
    Ok(Opened {
        path: path.to_path_buf(),
        format,
        kind: Kind::Still,
        width,
        height,
        notes,
        source: Box::new(Still {
            frame: Some(frame),
            bytes,
            format,
        }),
    })
}

/// One still, decoded from the file's bytes: orientation applied once, sRGB once.
fn decode_still(bytes: &[u8], format: Format) -> Result<(DecodedFrame, Notes), OpenError> {
    let corrupt = |err: &dyn std::fmt::Display| OpenError::Corrupt {
        format,
        detail: err.to_string(),
    };
    let mut decoder = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| corrupt(&e))?
        .into_decoder()
        .map_err(|e| corrupt(&e))?;
    let orientation = decoder.orientation().map_err(|e| corrupt(&e))?;
    let icc = decoder.icc_profile().map_err(|e| corrupt(&e))?;
    let has_alpha = decoder.color_type().has_alpha();
    let mut image = image::DynamicImage::from_decoder(decoder).map_err(|e| corrupt(&e))?;

    // Orientation, once. The dimensions the document sees are the ones after this.
    image.apply_orientation(orientation);
    let mut rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    let color = to_srgb(rgba.as_mut(), icc.as_deref());

    let notes = Notes {
        orientation_applied: match orientation {
            image::metadata::Orientation::NoTransforms => None,
            other => Some(other.to_exif()),
        },
        color,
        alpha: if has_alpha {
            "alpha preserved".into()
        } else {
            "opaque".into()
        },
        provider: "image crate",
        remarks: Vec::new(),
    };
    Ok((
        DecodedFrame {
            width,
            height,
            rgba: rgba.into_raw(),
        },
        notes,
    ))
}

fn open_animation(path: &Path, format: Format, bytes: Vec<u8>) -> Result<Opened, OpenError> {
    let corrupt = |err: &dyn std::fmt::Display| OpenError::Corrupt {
        format,
        detail: err.to_string(),
    };
    let (frames, icc): (Vec<image::Frame>, Option<Vec<u8>>) = match format {
        Format::Gif => {
            let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(&bytes))
                .map_err(|e| corrupt(&e))?;
            let frames = decoder
                .into_frames()
                .collect_frames()
                .map_err(|e| corrupt(&e))?;
            (frames, None)
        }
        Format::WebP => {
            let mut decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(&bytes))
                .map_err(|e| corrupt(&e))?;
            let icc = decoder.icc_profile().map_err(|e| corrupt(&e))?;
            let frames = decoder
                .into_frames()
                .collect_frames()
                .map_err(|e| corrupt(&e))?;
            (frames, icc)
        }
        _ => unreachable!("only GIF and WebP animate here"),
    };
    if frames.is_empty() {
        return Err(corrupt(&"the animation has no frames"));
    }
    let delays_ms: Vec<u32> = frames
        .iter()
        .map(|f| {
            let (num, den) = f.delay().numer_denom_ms();
            num.checked_div(den).unwrap_or(0)
        })
        .collect();
    let first = frames[0].buffer();
    let (width, height) = first.dimensions();
    let color = match icc.as_deref() {
        Some(profile) => {
            // Applied to every frame as it is decoded below; the note is made once.
            let mut probe = first.as_raw().clone();
            to_srgb(&mut probe, Some(profile))
        }
        None => "sRGB, untagged".to_string(),
    };
    let notes = Notes {
        orientation_applied: None,
        color,
        alpha: "alpha preserved".into(),
        provider: "image crate",
        remarks: vec![format!(
            "{} frames, decoded and held in memory",
            frames.len()
        )],
    };
    Ok(Opened {
        path: path.to_path_buf(),
        format,
        kind: Kind::Animation { delays_ms },
        width,
        height,
        notes,
        source: Box::new(Animation { frames, icc }),
    })
}

/// A still: decoded once at open and handed over on the first ask; a second ask decodes it
/// again from the file's bytes. The first version kept a second copy of the pixels for that
/// case, which doubled every opened still in memory, 192 MB twice for a 48-megapixel PNG,
/// for an ask nothing makes today (review of 2026-09-13, R4).
struct Still {
    frame: Option<DecodedFrame>,
    bytes: Vec<u8>,
    format: Format,
}

impl FrameSource for Still {
    fn frame(&mut self, _index: u32) -> Result<DecodedFrame, OpenError> {
        match self.frame.take() {
            Some(frame) => Ok(frame),
            None => decode_still(&self.bytes, self.format).map(|(frame, _)| frame),
        }
    }
}

struct Animation {
    frames: Vec<image::Frame>,
    icc: Option<Vec<u8>>,
}

impl FrameSource for Animation {
    fn frame(&mut self, index: u32) -> Result<DecodedFrame, OpenError> {
        let frame = self
            .frames
            .get(index as usize)
            .ok_or(OpenError::NoSuchFrame {
                index,
                count: self.frames.len() as u32,
            })?;
        let buffer = frame.buffer();
        let (width, height) = buffer.dimensions();
        let mut rgba = buffer.as_raw().clone();
        if let Some(profile) = self.icc.as_deref() {
            to_srgb(&mut rgba, Some(profile));
        }
        Ok(DecodedFrame {
            width,
            height,
            rgba,
        })
    }
}
