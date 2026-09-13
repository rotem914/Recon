//! The composer: the untouched source, the annotation layer over it, the margin around it.
//!
//! One function makes every output, the clipboard and the file alike, so the two can never
//! disagree with each other (§3.6). The source goes into the output byte for byte at the
//! margin offset, alpha included: a pixel the layer did not touch is the source pixel, which
//! is the exact-equality half of S0.6's first check and the export half of rule 11. The
//! layer arrives straight-alpha from the page's canvas encoder and is composited "over" in
//! straight alpha, so a semi-transparent source under a semi-transparent note comes out as
//! the standard blend rather than as a premultiplied surprise.
//!
//! Canvas space (part 5): the output is the source plus the stored margins, and every layer
//! coordinate is already in canvas space, so the page and the host agree on one offset.

/// The annotation margin, in image pixels, one value per edge (§3.5). Stored as four edges
/// so a growing margin never rewrites an annotation coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Margin {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Margin {
    /// "left,top,right,bottom", as the page sends it in a request header.
    pub fn parse(text: &str) -> Result<Margin, String> {
        let parts: Vec<u32> = text
            .split(',')
            .map(|p| p.trim().parse::<u32>())
            .collect::<Result<_, _>>()
            .map_err(|_| format!("a margin is four numbers, not {text:?}"))?;
        if parts.len() != 4 {
            return Err(format!("a margin is four numbers, not {text:?}"));
        }
        if parts.iter().any(|&edge| edge > 16_384) {
            return Err(format!(
                "a margin edge of {} pixels is not one",
                parts.iter().max().unwrap()
            ));
        }
        Ok(Margin {
            left: parts[0],
            top: parts[1],
            right: parts[2],
            bottom: parts[3],
        })
    }

    /// The canvas size for a source of this size.
    pub fn canvas(&self, width: u32, height: u32) -> (u32, u32) {
        (
            width + self.left + self.right,
            height + self.top + self.bottom,
        )
    }
}

/// The colour of the margin. Neutral, light, and opaque: a note that needed room should
/// read as sitting beside the picture, not floating in a void. Rotem's to change.
pub const MAT: [u8; 4] = [0xE9, 0xEA, 0xEC, 0xFF];

/// One composed output, straight-alpha RGBA in canvas space.
pub struct Composite {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The source at the margin offset, the margin around it, and the layer over both.
///
/// `layer` is canvas-sized straight RGBA. Where its alpha is zero the output is the source
/// (or the margin) byte for byte; where it is 255 the output is the layer; between, the
/// straight-alpha "over" blend, with the output alpha the union of the two.
pub fn compose(
    source: &[u8],
    source_w: u32,
    source_h: u32,
    layer: &[u8],
    margin: Margin,
    mat: [u8; 4],
) -> Result<Composite, String> {
    let (w, h) = margin.canvas(source_w, source_h);
    let count = w as usize * h as usize;
    if source.len() != source_w as usize * source_h as usize * 4 {
        return Err(format!(
            "the source is {} bytes for {source_w}x{source_h}",
            source.len()
        ));
    }
    if layer.len() != count * 4 {
        return Err(format!(
            "the layer is {} bytes and the canvas is {w}x{h}",
            layer.len()
        ));
    }

    // The margin first, then the source rows dropped in whole at the offset.
    let mut out = Vec::with_capacity(count * 4);
    for _ in 0..count {
        out.extend_from_slice(&mat);
    }
    let row_bytes = source_w as usize * 4;
    for y in 0..source_h as usize {
        let from = y * row_bytes;
        let to = ((y + margin.top as usize) * w as usize + margin.left as usize) * 4;
        out[to..to + row_bytes].copy_from_slice(&source[from..from + row_bytes]);
    }

    // Then the layer, straight alpha over.
    for i in 0..count {
        let la = layer[i * 4 + 3];
        if la == 0 {
            continue;
        }
        let o = &mut out[i * 4..i * 4 + 4];
        if la == 255 {
            o.copy_from_slice(&layer[i * 4..i * 4 + 4]);
            continue;
        }
        let la = la as f32 / 255.0;
        let sa = o[3] as f32 / 255.0;
        let oa = la + sa * (1.0 - la);
        for c in 0..3 {
            let lc = layer[i * 4 + c] as f32;
            let sc = o[c] as f32;
            o[c] = ((lc * la + sc * sa * (1.0 - la)) / oa)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        o[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    Ok(Composite {
        width: w,
        height: h,
        rgba: out,
    })
}

/// How many source pixels the layer left alone came out different from the source. The
/// number S0.6's first check reads, and it has to be zero, exactly, alpha included.
#[cfg(any(feature = "stage0-checks", test))]
pub fn source_mismatches(
    composite: &Composite,
    source: &[u8],
    source_w: u32,
    source_h: u32,
    layer: &[u8],
    margin: Margin,
) -> usize {
    let w = composite.width as usize;
    let mut mismatches = 0;
    for y in 0..source_h as usize {
        for x in 0..source_w as usize {
            let s = (y * source_w as usize + x) * 4;
            let o = ((y + margin.top as usize) * w + x + margin.left as usize) * 4;
            if layer[o + 3] == 0 && composite.rgba[o..o + 4] != source[s..s + 4] {
                mismatches += 1;
            }
        }
    }
    mismatches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::new();
        for y in 0..h {
            for x in 0..w {
                // Varied colours and varied alpha, so an exact comparison means something.
                v.extend_from_slice(&[
                    (x * 37 % 256) as u8,
                    (y * 91 % 256) as u8,
                    ((x + y) * 13 % 256) as u8,
                    ((x * y) % 256) as u8,
                ]);
            }
        }
        v
    }

    #[test]
    fn an_untouched_semi_transparent_source_survives_byte_for_byte() {
        let (w, h) = (16, 12);
        let source = checker(w, h);
        let layer = vec![0u8; (w * h * 4) as usize];
        let out = compose(&source, w, h, &layer, Margin::default(), MAT).unwrap();
        assert_eq!(out.rgba, source);
        assert_eq!(
            source_mismatches(&out, &source, w, h, &layer, Margin::default()),
            0
        );
    }

    #[test]
    fn the_source_lands_at_the_margin_offset_and_the_margin_is_the_mat() {
        let (w, h) = (4, 3);
        let source = checker(w, h);
        let margin = Margin {
            left: 2,
            top: 1,
            right: 3,
            bottom: 2,
        };
        let (cw, ch) = margin.canvas(w, h);
        assert_eq!((cw, ch), (9, 6));
        let layer = vec![0u8; (cw * ch * 4) as usize];
        let out = compose(&source, w, h, &layer, margin, MAT).unwrap();
        // Corners are mat.
        assert_eq!(&out.rgba[0..4], &MAT);
        let last = ((ch * cw - 1) * 4) as usize;
        assert_eq!(&out.rgba[last..last + 4], &MAT);
        // Source pixel (1, 2) sits at canvas (3, 3).
        let s = ((2 * w + 1) * 4) as usize;
        let o = ((3 * cw + 3) * 4) as usize;
        assert_eq!(&out.rgba[o..o + 4], &source[s..s + 4]);
        assert_eq!(source_mismatches(&out, &source, w, h, &layer, margin), 0);
    }

    #[test]
    fn an_opaque_layer_pixel_replaces_and_a_half_one_blends() {
        let source = vec![0, 0, 255, 128];
        let mut layer = vec![255, 0, 0, 255];
        let out = compose(&source, 1, 1, &layer, Margin::default(), MAT).unwrap();
        assert_eq!(out.rgba, [255, 0, 0, 255]);

        layer[3] = 128;
        let out = compose(&source, 1, 1, &layer, Margin::default(), MAT).unwrap();
        // la = sa = 0.502: oa = 0.752, r = 255 * 0.502 / 0.752, b = 255 * 0.502 * 0.498 / 0.752.
        let [r, g, b, a] = <[u8; 4]>::try_from(&out.rgba[..]).unwrap();
        assert!((r as i32 - 170).abs() <= 1, "r {r}");
        assert_eq!(g, 0);
        assert!((b as i32 - 85).abs() <= 1, "b {b}");
        assert!((a as i32 - 192).abs() <= 1, "a {a}");
    }

    #[test]
    fn a_touched_pixel_is_counted_as_a_mismatch_only_when_the_layer_missed_it() {
        let (w, h) = (3, 1);
        let source = checker(w, h);
        let mut layer = vec![0u8; 12];
        layer[4..8].copy_from_slice(&[9, 9, 9, 255]);
        let mut out = compose(&source, w, h, &layer, Margin::default(), MAT).unwrap();
        assert_eq!(
            source_mismatches(&out, &source, w, h, &layer, Margin::default()),
            0
        );
        // Damage an untouched pixel: exactly one mismatch.
        out.rgba[0] ^= 1;
        assert_eq!(
            source_mismatches(&out, &source, w, h, &layer, Margin::default()),
            1
        );
    }

    #[test]
    fn a_margin_string_parses_and_a_bad_one_is_refused() {
        assert_eq!(
            Margin::parse("1, 2,3,4").unwrap(),
            Margin {
                left: 1,
                top: 2,
                right: 3,
                bottom: 4
            }
        );
        assert!(Margin::parse("1,2,3").is_err());
        assert!(Margin::parse("a,b,c,d").is_err());
        assert!(Margin::parse("1,2,3,99999").is_err());
    }

    #[test]
    fn a_wrong_sized_layer_is_refused() {
        let source = vec![0u8; 16];
        assert!(compose(&source, 2, 2, &[0u8; 8], Margin::default(), MAT).is_err());
    }
}
