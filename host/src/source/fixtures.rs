//! The files S0.5's evidence is produced on, generated here so the report can say what each
//! one holds and check the decode against it. Run with `--make-fixtures <dir>`.
//!
//! Every fixture carries a known mark: a red block at a known corner, a frame whose colour
//! says which frame it is, a page that names its number, a profile that swaps two
//! primaries. A decode that quietly rotates twice, hands back the first frame, or ignores
//! the profile changes the mark, which is what makes each line of the report a check that
//! can fail rather than a decode that returned something.
//!
//! What cannot be generated here, and has to come from a real file: HEIC and AVIF, whose
//! encoders are not in this toolchain. The report names them as pending inputs.

use std::io::{Cursor, Write};
use std::path::Path;

use image::{ExtendedColorType, ImageEncoder, RgbaImage};

/// The colour a frame or page carries, by index, so a report can tell which one came back.
pub fn index_color(index: u32) -> [u8; 4] {
    match index % 3 {
        0 => [220, 30, 30, 255],
        1 => [30, 180, 60, 255],
        _ => [40, 70, 220, 255],
    }
}

fn solid(width: u32, height: u32, rgba: [u8; 4]) -> RgbaImage {
    RgbaImage::from_fn(width, height, |_, _| image::Rgba(rgba))
}

/// A picture with a red block in its top-left corner and a blue one bottom-right, so a
/// rotation or a flip is visible in the pixels.
fn corner_marked(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        if x < 40 && y < 40 {
            image::Rgba([255, 0, 0, 255])
        } else if x >= width - 40 && y >= height - 40 {
            image::Rgba([0, 0, 255, 255])
        } else {
            image::Rgba([200, 200, 200, 255])
        }
    })
}

/// An ICC v2 matrix/TRC display profile whose red and green primaries are SWAPPED. A pixel
/// stored as pure red in this profile is green in sRGB, so honouring the profile is visible
/// in one pixel and ignoring it is too.
pub fn swapped_primaries_icc() -> Vec<u8> {
    fn s15(v: f64) -> [u8; 4] {
        ((v * 65536.0).round() as i32).to_be_bytes()
    }
    fn xyz_tag(x: f64, y: f64, z: f64) -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(b"XYZ ");
        t.extend_from_slice(&[0; 4]);
        t.extend_from_slice(&s15(x));
        t.extend_from_slice(&s15(y));
        t.extend_from_slice(&s15(z));
        t
    }
    fn curv_gamma22() -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(b"curv");
        t.extend_from_slice(&[0; 4]);
        t.extend_from_slice(&1u32.to_be_bytes());
        t.extend_from_slice(&0x0233u16.to_be_bytes()); // 2.2 in u8Fixed8
        t.extend_from_slice(&[0; 2]);
        t
    }
    // sRGB primaries adapted to D50, with red and green exchanged.
    let tags: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"wtpt", xyz_tag(0.9642, 1.0, 0.8249)),
        (b"rXYZ", xyz_tag(0.3851, 0.7169, 0.0971)),
        (b"gXYZ", xyz_tag(0.4361, 0.2225, 0.0139)),
        (b"bXYZ", xyz_tag(0.1431, 0.0606, 0.7141)),
        (b"rTRC", curv_gamma22()),
        (b"gTRC", curv_gamma22()),
        (b"bTRC", curv_gamma22()),
    ];
    let table_len = 4 + tags.len() * 12;
    let mut data_offset = 128 + table_len;
    let mut table = Vec::new();
    let mut data = Vec::new();
    table.extend_from_slice(&(tags.len() as u32).to_be_bytes());
    for (sig, body) in &tags {
        table.extend_from_slice(*sig);
        table.extend_from_slice(&(data_offset as u32).to_be_bytes());
        table.extend_from_slice(&(body.len() as u32).to_be_bytes());
        data.extend_from_slice(body);
        // Four-byte alignment between tags.
        while data.len() % 4 != 0 {
            data.push(0);
        }
        data_offset = 128 + table_len + data.len();
    }
    let total = 128 + table_len + data.len();
    let mut header = Vec::with_capacity(128);
    header.extend_from_slice(&(total as u32).to_be_bytes());
    header.extend_from_slice(b"none");
    header.extend_from_slice(&0x0210_0000u32.to_be_bytes());
    header.extend_from_slice(b"mntr");
    header.extend_from_slice(b"RGB ");
    header.extend_from_slice(b"XYZ ");
    header.extend_from_slice(&[0; 12]); // date
    header.extend_from_slice(b"acsp");
    header.extend_from_slice(&[0; 4]); // platform
    header.extend_from_slice(&[0; 4]); // flags
    header.extend_from_slice(&[0; 4]); // manufacturer
    header.extend_from_slice(&[0; 4]); // model
    header.extend_from_slice(&[0; 8]); // attributes
    header.extend_from_slice(&[0; 4]); // intent
    header.extend_from_slice(&s15(0.9642));
    header.extend_from_slice(&s15(1.0));
    header.extend_from_slice(&s15(0.8249));
    header.extend_from_slice(&[0; 4]); // creator
    header.extend_from_slice(&[0; 16]); // id
    header.extend_from_slice(&[0; 28]); // reserved
    assert_eq!(header.len(), 128);
    let mut out = header;
    out.extend_from_slice(&table);
    out.extend_from_slice(&data);
    out
}

/// A TIFF structure carrying one tag: EXIF orientation 6, a quarter turn clockwise.
fn exif_orientation(value: u16) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend_from_slice(b"II*\0");
    t.extend_from_slice(&8u32.to_le_bytes());
    t.extend_from_slice(&1u16.to_le_bytes()); // one entry
    t.extend_from_slice(&0x0112u16.to_le_bytes()); // Orientation
    t.extend_from_slice(&3u16.to_le_bytes()); // SHORT
    t.extend_from_slice(&1u32.to_le_bytes());
    t.extend_from_slice(&value.to_le_bytes());
    t.extend_from_slice(&[0; 2]);
    t.extend_from_slice(&0u32.to_le_bytes()); // no next IFD
    t
}

/// The VP8L chunk out of a lossless WebP the `image` crate wrote, for wrapping in ANMF.
fn vp8l_chunk(webp: &[u8]) -> Option<&[u8]> {
    let mut at = 12;
    while at + 8 <= webp.len() {
        let size =
            u32::from_le_bytes([webp[at + 4], webp[at + 5], webp[at + 6], webp[at + 7]]) as usize;
        let end = at + 8 + size + (size & 1);
        if &webp[at..at + 4] == b"VP8L" {
            return webp.get(at..end.min(webp.len()));
        }
        at = end;
    }
    None
}

fn u24(v: u32) -> [u8; 3] {
    [
        (v & 0xFF) as u8,
        ((v >> 8) & 0xFF) as u8,
        ((v >> 16) & 0xFF) as u8,
    ]
}

/// An animated WebP, hand-wrapped: the container spec is small and the crate encodes only
/// stills, so each frame is a lossless still placed in an ANMF chunk.
fn animated_webp(frames: &[RgbaImage], delay_ms: u32) -> Vec<u8> {
    let (w, h) = frames[0].dimensions();
    let mut body = Vec::new();
    // VP8X: animation and alpha flags, canvas size minus one.
    body.extend_from_slice(b"VP8X");
    body.extend_from_slice(&10u32.to_le_bytes());
    body.extend_from_slice(&[0x12, 0, 0, 0]);
    body.extend_from_slice(&u24(w - 1));
    body.extend_from_slice(&u24(h - 1));
    // ANIM: background colour and loop count.
    body.extend_from_slice(b"ANIM");
    body.extend_from_slice(&6u32.to_le_bytes());
    body.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    for frame in frames {
        let mut still = Cursor::new(Vec::new());
        image::codecs::webp::WebPEncoder::new_lossless(&mut still)
            .encode(frame.as_raw(), w, h, ExtendedColorType::Rgba8)
            .expect("a lossless webp still");
        let still = still.into_inner();
        let chunk = vp8l_chunk(&still).expect("a VP8L chunk in the still");
        let mut anmf = Vec::new();
        anmf.extend_from_slice(&u24(0)); // x / 2
        anmf.extend_from_slice(&u24(0)); // y / 2
        anmf.extend_from_slice(&u24(w - 1));
        anmf.extend_from_slice(&u24(h - 1));
        anmf.extend_from_slice(&u24(delay_ms));
        anmf.push(0b10); // do not blend with the canvas; dispose to nothing
        anmf.extend_from_slice(chunk);
        body.extend_from_slice(b"ANMF");
        body.extend_from_slice(&(anmf.len() as u32).to_le_bytes());
        body.extend_from_slice(&anmf);
        if anmf.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
    out.extend_from_slice(b"WEBP");
    out.extend_from_slice(&body);
    out
}

fn write(dir: &Path, name: &str, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let bytes = bytes.as_ref();
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path)?;
    file.write_all(bytes)?;
    println!("  wrote {} ({} bytes)", path.display(), bytes.len());
    Ok(())
}

/// Every fixture, with what it holds said once in its name.
pub fn make(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;

    // PNG: transparency, and an embedded profile with red and green swapped. The left half
    // is stored as pure red at full alpha, the right half is fully transparent.
    {
        let img = RgbaImage::from_fn(200, 100, |x, _| {
            if x < 100 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut out = Cursor::new(Vec::new());
        let mut enc = image::codecs::png::PngEncoder::new(&mut out);
        enc.set_icc_profile(swapped_primaries_icc())
            .expect("png takes a profile");
        enc.write_image(img.as_raw(), 200, 100, ExtendedColorType::Rgba8)
            .expect("png");
        write(dir, "png-alpha-and-swapped-profile.png", out.into_inner())?;
    }

    // JPEG: 300x200 with the corner marks, tagged as orientation 6. Upright it is 200x300
    // with the red mark at the top RIGHT.
    {
        let img = image::DynamicImage::ImageRgba8(corner_marked(300, 200)).into_rgb8();
        let mut out = Cursor::new(Vec::new());
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 95);
        enc.set_exif_metadata(exif_orientation(6))
            .expect("jpeg takes exif");
        enc.write_image(img.as_raw(), 300, 200, ExtendedColorType::Rgb8)
            .expect("jpeg");
        write(dir, "jpeg-orientation-6.jpg", out.into_inner())?;
    }

    // BMP: the corner marks, nothing else.
    {
        let img = image::DynamicImage::ImageRgba8(corner_marked(160, 120)).into_rgb8();
        let mut out = Cursor::new(Vec::new());
        image::codecs::bmp::BmpEncoder::new(&mut out)
            .write_image(img.as_raw(), 160, 120, ExtendedColorType::Rgb8)
            .expect("bmp");
        write(dir, "bmp-corners.bmp", out.into_inner())?;
    }

    // GIF: three frames, red then green then blue, 80 ms each.
    {
        let mut out = Cursor::new(Vec::new());
        {
            let mut enc = image::codecs::gif::GifEncoder::new_with_speed(&mut out, 10);
            enc.set_repeat(image::codecs::gif::Repeat::Infinite)
                .expect("repeat");
            let frames = (0..3).map(|i| {
                image::Frame::from_parts(
                    solid(120, 90, index_color(i)),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(80, 1),
                )
            });
            enc.encode_frames(frames).expect("gif frames");
        }
        write(dir, "gif-three-frames.gif", out.into_inner())?;
    }

    // WebP: a still with transparency, and an animation of the same three colours.
    {
        let still = RgbaImage::from_fn(120, 90, |x, _| {
            if x < 60 {
                image::Rgba([220, 30, 30, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut out = Cursor::new(Vec::new());
        image::codecs::webp::WebPEncoder::new_lossless(&mut out)
            .encode(still.as_raw(), 120, 90, ExtendedColorType::Rgba8)
            .expect("webp still");
        write(dir, "webp-still-with-alpha.webp", out.into_inner())?;
        let frames: Vec<RgbaImage> = (0..3).map(|i| solid(120, 90, index_color(i))).collect();
        write(dir, "webp-three-frames.webp", animated_webp(&frames, 80))?;
    }

    // TIFF: three pages, red, green, blue, and a fourth-page-free count of three.
    {
        let mut out = Cursor::new(Vec::new());
        {
            let mut enc = tiff::encoder::TiffEncoder::new(&mut out).expect("tiff");
            for i in 0..3 {
                let page = solid(100, 80, index_color(i));
                enc.write_image::<tiff::encoder::colortype::RGBA8>(100, 80, page.as_raw())
                    .expect("tiff page");
            }
        }
        write(dir, "tiff-three-pages.tif", out.into_inner())?;
    }

    // A large PNG: 8000x6000, a gradient so it compresses in seconds and decodes in a
    // measurable time.
    {
        let img = RgbaImage::from_fn(8000, 6000, |x, y| {
            image::Rgba([(x / 32) as u8, (y / 24) as u8, ((x + y) / 55) as u8, 255])
        });
        let mut out = Cursor::new(Vec::new());
        image::codecs::png::PngEncoder::new_with_quality(
            &mut out,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::NoFilter,
        )
        .write_image(img.as_raw(), 8000, 6000, ExtendedColorType::Rgba8)
        .expect("large png");
        write(dir, "png-large-8000x6000.png", out.into_inner())?;
    }

    make_svgs(dir)?;

    // Not an image at all, so the report can show the refusal names the format.
    write(
        dir,
        "not-an-image.txt",
        b"this is a text file, not a picture
",
    )?;
    Ok(())
}

/// The three SVGs on their own, for the check that compares resvg against the web view:
/// text labels, fills that live in a CSS style block the way Illustrator exports them, and
/// a mask plus a clip the way Figma exports them.
pub fn make_svgs(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write(
        dir,
        "svg-text-labels.svg",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="120" viewBox="0 0 320 120">
  <rect x="10" y="10" width="300" height="100" rx="8" fill="#eef2ff" stroke="#3355aa" stroke-width="2"/>
  <text x="24" y="52" font-family="Arial, sans-serif" font-size="24" fill="#112244">Label one</text>
  <text x="296" y="92" font-family="Arial, sans-serif" font-size="20" fill="#aa3333" text-anchor="end">תווית שתיים</text>
</svg>"##,
    )?;
    write(
        dir,
        "svg-css-style-block.svg",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="240" height="160" viewBox="0 0 240 160">
  <defs>
    <style type="text/css">.cls-1{fill:#3366cc;}.cls-2{fill:#cc3333;stroke:#331111;stroke-width:4px;}.cls-3{fill:none;stroke:#118844;stroke-width:6px;}</style>
  </defs>
  <rect class="cls-1" x="20" y="20" width="120" height="80"/>
  <circle class="cls-2" cx="180" cy="60" r="36"/>
  <path class="cls-3" d="M20 130 L220 130"/>
</svg>"##,
    )?;
    write(
        dir,
        "svg-mask-and-clip.svg",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="240" height="160" viewBox="0 0 240 160">
  <defs>
    <mask id="hole" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="160">
      <rect x="0" y="0" width="240" height="160" fill="white"/>
      <circle cx="80" cy="80" r="40" fill="black"/>
    </mask>
    <clipPath id="half"><rect x="120" y="0" width="120" height="160"/></clipPath>
  </defs>
  <rect x="10" y="10" width="220" height="140" fill="#ff9900" mask="url(#hole)"/>
  <circle cx="160" cy="80" r="60" fill="#2244aa" clip-path="url(#half)"/>
</svg>"##,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_swapped_profile_parses_and_is_not_srgb() {
        let icc = swapped_primaries_icc();
        let profile = qcms::Profile::new_from_slice(&icc, false).expect("qcms parses it");
        assert!(!profile.is_sRGB());
    }

    #[test]
    fn the_swapped_profile_turns_red_into_green() {
        let icc = swapped_primaries_icc();
        let mut px = vec![255, 0, 0, 255];
        let note = crate::source::to_srgb(&mut px, Some(&icc));
        assert!(note.starts_with("converted"), "{note}");
        assert!(px[1] > px[0], "expected green to dominate, got {px:?}");
    }

    #[test]
    fn the_animated_webp_is_read_back_as_three_frames() {
        let frames: Vec<RgbaImage> = (0..3).map(|i| solid(16, 8, index_color(i))).collect();
        let bytes = animated_webp(&frames, 80);
        assert_eq!(
            crate::source::sniff(&bytes),
            Some(crate::source::Format::WebP)
        );
        let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(&bytes)).unwrap();
        assert!(decoder.has_animation());
        let got = image::AnimationDecoder::into_frames(decoder)
            .collect_frames()
            .unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(got[2].buffer().get_pixel(0, 0).0, index_color(2));
    }
}
