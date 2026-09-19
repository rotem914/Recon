//! The per-pixel work the host does on the interactive path, in a crate of its own.
//!
//! Why a separate crate: a generic function such as `image::imageops::resize` is compiled in
//! the crate that CALLS it, at that crate's optimisation level. Optimising the `image` crate
//! in the dev profile therefore did nothing for the resample, which is why a fit view of a
//! 4K image cost 1.1 s in a debug build (F43). Everything here is non-generic, so the
//! instantiation happens in this crate, and this crate is optimised in every profile
//! (`[profile.dev.package.recon-pixels]` in the host's Cargo.toml).
//!
//! Nothing here knows about the screen, a window, or a file. Bytes in, bytes out, RGBA,
//! 4 bytes per pixel, top-down, tightly packed.

use image::{ImageBuffer, Rgba};

/// Copies one rectangle out of an image, row by row. `None` if it does not fit.
pub fn crop(
    rgba: &[u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Option<Vec<u8>> {
    if w == 0 || h == 0 || x.checked_add(w)? > width || y.checked_add(h)? > height {
        return None;
    }
    if rgba.len() < (width as usize) * (height as usize) * 4 {
        return None;
    }
    let stride = width as usize * 4;
    let row_bytes = w as usize * 4;
    let mut out = Vec::with_capacity(row_bytes * h as usize);
    for row in 0..h as usize {
        let start = (y as usize + row) * stride + x as usize * 4;
        out.extend_from_slice(&rgba[start..start + row_bytes]);
    }
    Some(out)
}

/// Resamples an image to a new size with a triangle filter.
///
/// Triangle rather than Lanczos because this runs while the user zooms and pans; at an
/// exact 2:1 it is a 2x2 box average, which is what the pyramid levels are built with.
pub fn resample(
    rgba: &[u8],
    width: u32,
    height: u32,
    target_w: u32,
    target_h: u32,
) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || target_w == 0 || target_h == 0 {
        return None;
    }
    let source: ImageBuffer<Rgba<u8>, &[u8]> = ImageBuffer::from_raw(width, height, rgba)?;
    let out = image::imageops::resize(
        &source,
        target_w,
        target_h,
        image::imageops::FilterType::Triangle,
    );
    Some(out.into_raw())
}

/// Half the size in each dimension, rounded down, never below one pixel.
pub fn half(width: u32, height: u32) -> (u32, u32) {
    ((width / 2).max(1), (height / 2).max(1))
}

/// Whether every pixel is solid, so nothing of the image is see-through.
pub fn is_opaque(rgba: &[u8]) -> bool {
    rgba.as_chunks::<4>().0.iter().all(|px| px[3] == 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_see_through_pixel_is_not_opaque() {
        let mut img = gradient(4, 4);
        assert!(is_opaque(&img));
        img[4 * 9 + 3] = 254;
        assert!(!is_opaque(&img));
    }

    fn gradient(w: u32, h: u32) -> Vec<u8> {
        let mut v = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                v[i] = (x % 256) as u8;
                v[i + 1] = (y % 256) as u8;
                v[i + 2] = 7;
                v[i + 3] = 255;
            }
        }
        v
    }

    #[test]
    fn crop_takes_the_right_rows() {
        let img = gradient(10, 10);
        let out = crop(&img, 10, 10, 3, 4, 2, 2).unwrap();
        assert_eq!(out.len(), 16);
        // pixel (3,4) then (4,4) then (3,5) then (4,5)
        assert_eq!(&out[0..2], &[3, 4]);
        assert_eq!(&out[4..6], &[4, 4]);
        assert_eq!(&out[8..10], &[3, 5]);
        assert_eq!(&out[12..14], &[4, 5]);
    }

    #[test]
    fn crop_refuses_what_does_not_fit() {
        let img = gradient(10, 10);
        assert!(crop(&img, 10, 10, 9, 0, 2, 1).is_none());
        assert!(crop(&img, 10, 10, 0, 0, 0, 1).is_none());
        assert!(crop(&img, 10, 10, 0, 0, 1, 1).is_some());
    }

    #[test]
    fn a_checkerboard_halves_to_flat_grey() {
        let w = 8;
        let h = 8;
        let mut v = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let on = (x + y) % 2 == 0;
                let value = if on { 255 } else { 0 };
                v[i] = value;
                v[i + 1] = value;
                v[i + 2] = value;
                v[i + 3] = 255;
            }
        }
        let out = resample(&v, w, h, 4, 4).unwrap();
        for px in out.chunks(4) {
            assert!((100..=155).contains(&px[0]), "not averaged: {}", px[0]);
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn resample_at_the_same_size_is_the_identity() {
        let img = gradient(6, 5);
        let out = resample(&img, 6, 5, 6, 5).unwrap();
        assert_eq!(out, img);
    }
}
