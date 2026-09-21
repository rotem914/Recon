//! The stitcher of a scrolling capture: frames of one area of the screen, taken while the
//! person scrolls it, in; one long picture out.
//!
//! Every frame is cut into upright strips, and every row of every strip is reduced to one
//! number. How far the page moved between two frames is then a question about numbers: the
//! distance at which the rows of the new frame are the rows of the last one. The strips
//! answer it apart and the answer most of them give wins, so a column that does not move
//! with the page, a side menu that stays, a scrollbar, a video, is outvoted rather than
//! obeyed. Rows that stand still while the rest moves, a bar stuck to the top or the bottom,
//! are left out of the count, and the ones at the bottom are kept out of the middle of the
//! picture: the long picture ends with the last frame's bottom and holds it once.
//!
//! The lowest rows of a frame are never kept from any frame but the last. A window's round
//! bottom corners are blended by Windows with whatever scrolls under them, so those few
//! pixels are neither the page nor still, and a picture that kept them would carry a dark
//! notch at both sides wherever two frames meet.
//!
//! The picture is whole after every frame, so it can be handed over at any moment. A frame
//! that cannot be placed, because the page moved further than the two frames share, changes
//! nothing: the last placed frame stays the one to match, and the person scrolls back to it.
//!
//! Bytes in, bytes out, four to a pixel, top-down; which channel is which does not matter
//! here, only that the fourth is the alpha, which a screen copy leaves unset and is ignored.

use std::collections::HashMap;

/// A strip is about this wide; an area gets at most `MAX_STRIPS` of them.
const STRIP_PX: usize = 48;
const MAX_STRIPS: usize = 12;
/// Rows a strip needs in the shared part before its answer counts.
const MIN_ROWS: u32 = 8;
/// The part of a strip's counted rows that has to agree with a distance.
const AGREE: f64 = 0.8;
/// At most this many rows of a strip are looked up to propose distances.
const ANCHORS: usize = 96;
/// A row number that repeats more often than this in one strip proposes nothing: a table
/// of identical lines would propose every distance there is.
const REPEATS: usize = 6;
/// How many proposed distances are looked at closely.
const CANDIDATES: usize = 8;
/// A frame with this part of its telling rows where they were is the same frame again.
const SAME: f64 = 0.97;

/// What one frame did to the picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The first frame: it is the picture.
    First,
    /// Nothing moved since the last placed frame.
    Unchanged,
    /// Placed inside what the picture already holds, after a scroll back up.
    Placed,
    /// Placed, and the picture is this many rows longer.
    Grew(u32),
    /// Not placed: it shares too little with the last placed frame.
    Lost,
    /// Placed, and the picture has reached the longest it may be.
    Full,
}

/// One frame as numbers: for every strip, one number per row, and whether the row tells
/// anything, which a row equal to the one above it does not.
struct Signature {
    rows: Vec<Vec<u64>>,
    telling: Vec<Vec<bool>>,
}

pub struct Stitcher {
    width: usize,
    height: usize,
    strips: Vec<(usize, usize)>,
    max_rows: usize,
    /// How far in from the left and right edges a scrollbar is looked for.
    edge_band: usize,
    /// How many rows at a frame's bottom may hold a window's round corners.
    corner_rows: usize,
    canvas: Vec<u8>,
    /// The last placed frame, and the row of the picture its top row is.
    reference: Option<(Signature, Vec<u8>, i64)>,
    last_distance: i64,
    /// For every column of the two edge bands, in how many placed frames it held a pixel
    /// that neither stood still nor moved with the page: a scrollbar's thumb.
    rogue: Vec<u32>,
    /// For the same columns, whether each has been one colour from top to bottom in every
    /// placed frame: the flat track a scrollbar's thumb runs in, and its border.
    flat: Vec<bool>,
    full: bool,
}

impl Stitcher {
    /// A stitcher for frames of this size. `max_rows` is the longest the picture may get,
    /// never shorter than one frame; `edge_band` is how many columns in from the left and
    /// the right edge a scrollbar is looked for, 0 for never; `corner_rows` is how many rows
    /// at a frame's top and bottom a window's round corners may reach into.
    pub fn new(
        width: u32,
        height: u32,
        max_rows: u32,
        edge_band: u32,
        corner_rows: u32,
    ) -> Option<Stitcher> {
        let (width, height) = (width as usize, height as usize);
        if width == 0 || height == 0 {
            return None;
        }
        let count = (width / STRIP_PX).clamp(1, MAX_STRIPS);
        let strips = (0..count)
            .map(|n| (n * width / count, (n + 1) * width / count))
            .collect();
        let edge_band = (edge_band as usize).min(width / 4);
        Some(Stitcher {
            width,
            height,
            strips,
            max_rows: (max_rows as usize).max(height),
            edge_band,
            corner_rows: (corner_rows as usize).min(height / 4),
            canvas: Vec::new(),
            reference: None,
            last_distance: 0,
            rogue: vec![0; edge_band * 2],
            flat: vec![true; edge_band * 2],
            full: false,
        })
    }

    /// The picture's size as it stands: the frame's width, and every row placed so far.
    pub fn size(&self) -> (u32, u32) {
        (
            self.width as u32,
            (self.canvas.len() / (self.width * 4)) as u32,
        )
    }

    fn rows(&self) -> usize {
        self.canvas.len() / (self.width * 4)
    }

    /// Takes the next frame. One of the wrong size changes nothing and is `Lost`.
    pub fn push(&mut self, frame: &[u8]) -> Step {
        if frame.len() != self.width * self.height * 4 {
            return Step::Lost;
        }
        if self.full {
            return Step::Full;
        }
        let signature = self.signature(frame);
        // Taken out while it is compared, and put back as it was unless this frame is placed.
        let Some((reference, reference_pixels, top)) = self.reference.take() else {
            self.canvas.extend_from_slice(frame);
            self.reference = Some((signature, frame.to_vec(), 0));
            return Step::First;
        };
        if same(&reference, &signature) {
            self.reference = Some((reference, reference_pixels, top));
            return Step::Unchanged;
        }
        let Some((distance, agreeing)) = self.distance(&reference, &signature) else {
            self.reference = Some((reference, reference_pixels, top));
            return Step::Lost;
        };
        let still_rows = still_rows(&reference, &signature, &agreeing, self.height);
        self.note_rogue(&reference_pixels, frame, distance, &still_rows);
        let top = top + distance;
        let step = if top + self.height as i64 > self.rows() as i64 {
            // The rows that stand still at the bottom were put at the picture's end with the
            // frame that was the lowest so far; they come off, and the new frame goes on
            // from there down, its own bottom included.
            let still = still_at_bottom(&reference, &signature, &agreeing, self.height)
                .max(self.corner_rows);
            let keep = (self.rows() - still.min(self.rows())).max((top.max(0)) as usize);
            let from = (keep as i64 - top).clamp(0, self.height as i64) as usize;
            let room = self.max_rows.saturating_sub(keep);
            let take = (self.height - from).min(room);
            let before = self.rows();
            self.canvas.truncate(keep * self.width * 4);
            let bytes = self.width * 4;
            self.canvas
                .extend_from_slice(&frame[from * bytes..(from + take) * bytes]);
            if take < self.height - from {
                self.full = true;
                Step::Full
            } else {
                Step::Grew((self.rows() as i64 - before as i64).max(0) as u32)
            }
        } else {
            Step::Placed
        };
        self.last_distance = distance;
        self.reference = Some((signature, frame.to_vec(), top));
        step
    }

    /// The picture: its bytes, its width and its height. A scrollbar found at the left or
    /// the right edge is cut off, since its thumb stood somewhere else in every frame.
    pub fn finish(self) -> (Vec<u8>, u32, u32) {
        let rows = self.rows();
        let (left, right) = self.scrollbar();
        if left + right == 0 || left + right >= self.width {
            return (self.canvas, self.width as u32, rows as u32);
        }
        let width = self.width - left - right;
        let mut out = Vec::with_capacity(width * rows * 4);
        for row in 0..rows {
            let start = (row * self.width + left) * 4;
            out.extend_from_slice(&self.canvas[start..start + width * 4]);
        }
        (out, width as u32, rows as u32)
    }

    fn signature(&self, frame: &[u8]) -> Signature {
        let mut rows = Vec::with_capacity(self.strips.len());
        let mut telling = Vec::with_capacity(self.strips.len());
        for &(from, to) in &self.strips {
            let mut numbers = Vec::with_capacity(self.height);
            for y in 0..self.height {
                let start = (y * self.width + from) * 4;
                let end = (y * self.width + to) * 4;
                let mut number: u64 = 0xCBF2_9CE4_8422_2325;
                for pixel in frame[start..end].as_chunks::<4>().0 {
                    let colour = pixel[0] as u64 | (pixel[1] as u64) << 8 | (pixel[2] as u64) << 16;
                    number = (number ^ colour).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    number = number.rotate_left(23);
                }
                numbers.push(number);
            }
            let tells = (0..self.height)
                .map(|y| y > 0 && numbers[y] != numbers[y - 1])
                .collect();
            rows.push(numbers);
            telling.push(tells);
        }
        Signature { rows, telling }
    }

    /// How far the page moved from the reference to the new frame, in rows, down the page
    /// being positive, with the strips that agree on it. None when no distance has enough
    /// of the strips behind it.
    fn distance(&self, reference: &Signature, new: &Signature) -> Option<(i64, Vec<bool>)> {
        let height = self.height as i64;
        let mut proposed: HashMap<i64, u32> = HashMap::new();
        for strip in 0..self.strips.len() {
            let mut places: HashMap<u64, Vec<usize>> = HashMap::new();
            for y in 0..self.height {
                if reference.telling[strip][y] {
                    places.entry(reference.rows[strip][y]).or_default().push(y);
                }
            }
            let telling: Vec<usize> = (0..self.height)
                .filter(|&y| new.telling[strip][y])
                .collect();
            let every = telling.len().div_ceil(ANCHORS).max(1);
            for &y in telling.iter().step_by(every) {
                let Some(found) = places.get(&new.rows[strip][y]) else {
                    continue;
                };
                if found.len() > REPEATS {
                    continue;
                }
                for &at in found {
                    let distance = at as i64 - y as i64;
                    if distance != 0 {
                        *proposed.entry(distance).or_default() += 1;
                    }
                }
            }
        }
        let mut proposed: Vec<(i64, u32)> = proposed.into_iter().collect();
        proposed.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.abs().cmp(&b.0.abs())));
        proposed.truncate(CANDIDATES);

        let mut best: Option<(i64, Vec<bool>, u32, u64)> = None;
        for (distance, _) in proposed {
            let mut agreeing = vec![false; self.strips.len()];
            let (mut voters, mut agreed, mut matched_rows) = (0u32, 0u32, 0u64);
            for (strip, agrees) in agreeing.iter_mut().enumerate() {
                let (mut counted, mut matched) = (0u32, 0u32);
                for y in 0..height {
                    let at = y + distance;
                    if at < 0 || at >= height || !new.telling[strip][y as usize] {
                        continue;
                    }
                    let row = new.rows[strip][y as usize];
                    let moved = row == reference.rows[strip][at as usize];
                    // A row that stayed where it was and did not move with the page is a
                    // bar stuck in place, and so is the row it would have come from when
                    // that one stayed: neither says anything about the distance.
                    if !moved
                        && (row == reference.rows[strip][y as usize]
                            || new.rows[strip][at as usize] == reference.rows[strip][at as usize])
                    {
                        continue;
                    }
                    counted += 1;
                    matched += moved as u32;
                }
                if counted < MIN_ROWS {
                    continue;
                }
                voters += 1;
                if matched as f64 >= counted as f64 * AGREE {
                    agreed += 1;
                    *agrees = true;
                    matched_rows += matched as u64;
                }
            }
            if voters == 0 || agreed * 2 < voters || (voters > 1 && agreed < 2) {
                continue;
            }
            let better = match &best {
                None => true,
                Some((held, _, held_agreed, held_rows)) => {
                    (agreed, matched_rows) > (*held_agreed, *held_rows)
                        || ((agreed, matched_rows) == (*held_agreed, *held_rows)
                            && (distance - self.last_distance).abs()
                                < (held - self.last_distance).abs())
                }
            };
            if better {
                best = Some((distance, agreeing, agreed, matched_rows));
            }
        }
        best.map(|(distance, agreeing, _, _)| (distance, agreeing))
    }

    /// Counts, for the columns of the two edge bands, a pixel that neither stood still nor
    /// moved with the page between these two frames.
    fn note_rogue(&mut self, reference: &[u8], new: &[u8], distance: i64, still: &[bool]) {
        if self.edge_band == 0 {
            return;
        }
        let height = self.height as i64;
        let band = self.edge_band;
        let corner = self.corner_rows as i64;
        for slot in 0..band * 2 {
            let x = if slot < band {
                slot
            } else {
                self.width - band * 2 + slot
            };
            let mut rogue = false;
            for y in corner..height - corner {
                let at = y + distance;
                // A row of a bar stuck in place, here or where this row came from, holds
                // pixels that did not move with the page and are no scrollbar's; and so do
                // the rows a window's round corners reach into.
                if at < corner || at >= height - corner || still[y as usize] || still[at as usize] {
                    continue;
                }
                let here = (y as usize * self.width + x) * 4;
                let there = (at as usize * self.width + x) * 4;
                let pixel = &new[here..here + 3];
                if pixel != &reference[here..here + 3] && pixel != &reference[there..there + 3] {
                    rogue = true;
                    break;
                }
            }
            if rogue {
                self.rogue[slot] += 1;
            }
            if self.flat[slot] {
                let first = (self.corner_rows * self.width + x) * 4;
                self.flat[slot] = (self.corner_rows..self.height - self.corner_rows).all(|y| {
                    let here = (y * self.width + x) * 4;
                    new[here..here + 3] == new[first..first + 3]
                });
            }
        }
    }

    /// How many columns to cut at the left and at the right: from the innermost column of an
    /// edge band that held a rogue pixel in two placed frames or more, out to the edge, and
    /// inward over the columns beside it that were one colour from top to bottom all along,
    /// the track's own margin and border. A finding narrower than 4 columns is no scrollbar
    /// and cuts nothing.
    pub fn scrollbar(&self) -> (usize, usize) {
        let band = self.edge_band;
        if band == 0 {
            return (0, 0);
        }
        let left = (0..band)
            .rev()
            .find(|&slot| self.rogue[slot] >= 2)
            .map_or(0, |slot| {
                let mut last = slot;
                while last + 1 < band && self.flat[last + 1] {
                    last += 1;
                }
                last + 1
            });
        let right = (band..band * 2)
            .find(|&slot| self.rogue[slot] >= 2)
            .map_or(0, |slot| {
                let mut first = slot;
                while first > band && self.flat[first - 1] {
                    first -= 1;
                }
                band * 2 - first
            });
        (
            if left >= 4 { left } else { 0 },
            if right >= 4 { right } else { 0 },
        )
    }
}

/// Whether the new frame is the reference again: nearly every telling row where it was.
fn same(reference: &Signature, new: &Signature) -> bool {
    let (mut counted, mut still) = (0u64, 0u64);
    for strip in 0..new.rows.len() {
        for y in 0..new.rows[strip].len() {
            if new.telling[strip][y] || reference.telling[strip][y] {
                counted += 1;
                still += (new.rows[strip][y] == reference.rows[strip][y]) as u64;
            }
        }
    }
    counted == 0 || still as f64 >= counted as f64 * SAME
}

/// For every row, whether it stood still between the two frames in three quarters of the
/// strips that moved with the page.
fn still_rows(
    reference: &Signature,
    new: &Signature,
    agreeing: &[bool],
    height: usize,
) -> Vec<bool> {
    let moving: Vec<usize> = (0..agreeing.len()).filter(|&s| agreeing[s]).collect();
    (0..height)
        .map(|y| {
            let equal = moving
                .iter()
                .filter(|&&s| new.rows[s][y] == reference.rows[s][y])
                .count();
            !moving.is_empty() && equal * 4 >= moving.len() * 3
        })
        .collect()
}

/// How many rows at the bottom stood still between the two frames, in three quarters of the
/// strips that moved with the page; never more than half the frame.
fn still_at_bottom(
    reference: &Signature,
    new: &Signature,
    agreeing: &[bool],
    height: usize,
) -> usize {
    let moving: Vec<usize> = (0..agreeing.len()).filter(|&s| agreeing[s]).collect();
    if moving.is_empty() {
        return 0;
    }
    let mut still = 0;
    for y in (height / 2..height).rev() {
        let equal = moving
            .iter()
            .filter(|&&s| new.rows[s][y] == reference.rows[s][y])
            .count();
        if equal * 4 < moving.len() * 3 {
            break;
        }
        still += 1;
    }
    still
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 480;
    const H: usize = 300;

    /// A page: every pixel its own function of its place, in blocks of text-like lines with
    /// blank rows between them, so rows repeat inside a block's gap and nowhere else.
    fn page(rows: usize) -> Vec<u8> {
        let mut out = vec![255u8; W * rows * 4];
        for y in 0..rows {
            if y % 23 < 5 {
                continue;
            }
            for x in 0..W {
                let n = (x as u32 / 7).wrapping_mul(2654435761) ^ (y as u32).wrapping_mul(40503);
                let at = (y * W + x) * 4;
                out[at] = (n >> 8) as u8;
                out[at + 1] = (n >> 16) as u8;
                out[at + 2] = (n >> 24) as u8;
            }
        }
        out
    }

    fn view(page: &[u8], top: usize) -> Vec<u8> {
        page[top * W * 4..(top + H) * W * 4].to_vec()
    }

    fn stitcher() -> Stitcher {
        Stitcher::new(W as u32, H as u32, 100_000, 24, 0).unwrap()
    }

    #[test]
    fn a_page_scrolled_in_steps_comes_back_whole() {
        let page = page(1500);
        let mut s = stitcher();
        assert_eq!(s.push(&view(&page, 0)), Step::First);
        let mut top = 0;
        for step in [40, 120, 7, 200, 1, 150, 90, 250, 100, 242] {
            top += step;
            assert_eq!(
                s.push(&view(&page, top)),
                Step::Grew(step as u32),
                "at {top}"
            );
        }
        assert_eq!(top + H, 1500);
        let (pixels, w, h) = s.finish();
        assert_eq!((w, h), (W as u32, 1500));
        assert!(pixels == page, "the stitched picture is not the page");
    }

    #[test]
    fn the_same_frame_again_changes_nothing() {
        let page = page(900);
        let mut s = stitcher();
        s.push(&view(&page, 0));
        assert_eq!(s.push(&view(&page, 0)), Step::Unchanged);
        assert_eq!(s.size(), (W as u32, H as u32));
    }

    #[test]
    fn a_jump_past_what_two_frames_share_is_lost_and_found_again_on_the_way_back() {
        let page = page(2000);
        let mut s = stitcher();
        s.push(&view(&page, 0));
        assert_eq!(s.push(&view(&page, 100)), Step::Grew(100));
        // Further than a frame: nothing shared, nothing placed, nothing added.
        assert_eq!(s.push(&view(&page, 900)), Step::Lost);
        assert_eq!(s.size().1, (H + 100) as u32);
        // Back within reach of the last placed frame, and on from there.
        assert_eq!(s.push(&view(&page, 250)), Step::Grew(150));
        assert_eq!(s.push(&view(&page, 400)), Step::Grew(150));
        let (pixels, _, h) = s.finish();
        assert_eq!(h, 700);
        assert!(pixels == page[..700 * W * 4]);
    }

    #[test]
    fn a_scroll_back_up_adds_nothing_and_the_way_down_again_goes_on() {
        let page = page(1200);
        let mut s = stitcher();
        s.push(&view(&page, 0));
        assert_eq!(s.push(&view(&page, 200)), Step::Grew(200));
        assert_eq!(s.push(&view(&page, 60)), Step::Placed);
        assert_eq!(s.push(&view(&page, 0)), Step::Placed);
        assert_eq!(s.push(&view(&page, 180)), Step::Placed);
        assert_eq!(s.push(&view(&page, 380)), Step::Grew(180));
        let (pixels, _, h) = s.finish();
        assert_eq!(h, 680);
        assert!(pixels == page[..680 * W * 4]);
    }

    /// A frame of the page with a bar stuck to its top, one stuck to its bottom, a side
    /// menu that does not move, and a scrollbar whose thumb follows the scroll.
    fn dressed(page: &[u8], top: usize, total: usize) -> Vec<u8> {
        let mut frame = view(page, top);
        let mut paint =
            |x0: usize, x1: usize, y0: usize, y1: usize, f: &dyn Fn(usize, usize) -> [u8; 3]| {
                for y in y0..y1 {
                    for x in x0..x1 {
                        let at = (y * W + x) * 4;
                        frame[at..at + 3].copy_from_slice(&f(x, y));
                    }
                }
            };
        paint(0, W, 0, 30, &|x, y| [(x * 3 + y) as u8, 20, (y * 9) as u8]);
        paint(0, W, H - 24, H, &|x, y| [200, (x + y * 5) as u8, 40]);
        paint(0, 90, 30, H - 24, &|x, y| {
            [(y * 7) as u8, (x * 2) as u8, 90]
        });
        paint(W - 16, W, 0, H, &|_, _| [230, 230, 230]);
        let thumb = top * (H - 40) / (total - H);
        paint(W - 14, W - 2, thumb, thumb + 40, &|_, _| [120, 120, 120]);
        frame
    }

    #[test]
    fn bars_that_stay_a_side_menu_and_a_scrollbar_do_not_break_the_picture() {
        let total = 1600;
        let page = page(total);
        let mut s = stitcher();
        let mut top = 0;
        assert_eq!(s.push(&dressed(&page, 0, total)), Step::First);
        for step in [80, 130, 60, 170, 110, 140, 95, 215, 150, 150] {
            top += step;
            let got = s.push(&dressed(&page, top, total));
            assert_eq!(got, Step::Grew(step as u32), "at {top}");
        }
        assert_eq!(top + H, total);
        let (pixels, w, h) = s.finish();
        // The scrollbar's 16 columns are cut off; the rest is as wide as the frame.
        assert_eq!((w as usize, h as usize), (W - 16, total));
        let last = dressed(&page, top, total);
        let first = dressed(&page, 0, total);
        let out = |x: usize, y: usize| &pixels[(y * (W - 16) + x) * 4..(y * (W - 16) + x) * 4 + 3];
        // The moving part is the page, row for row, under the first frame's top bar.
        for y in (30..total - 24).step_by(11) {
            for x in (100..W - 20).step_by(13) {
                let at = (y * W + x) * 4;
                assert_eq!(out(x, y), &page[at..at + 3], "{x},{y}");
            }
        }
        // The top bar once, at the top; the bottom bar once, at the very end.
        for x in (0..W - 16).step_by(17) {
            let a = (5 * W + x) * 4;
            assert_eq!(out(x, 5), &first[a..a + 3]);
            let b = ((H - 10) * W + x) * 4;
            assert_eq!(out(x, total - 10), &last[b..b + 3]);
            // And not in the middle: where the first frame's bottom bar was, the page is.
            if x >= 100 {
                let c = ((H - 10) * W + x) * 4;
                assert_eq!(out(x, H - 10), &page[c..c + 3]);
            }
        }
    }

    #[test]
    fn the_picture_stops_at_the_longest_it_may_be() {
        let page = page(2000);
        let mut s = Stitcher::new(W as u32, H as u32, 500, 0, 0).unwrap();
        s.push(&view(&page, 0));
        assert_eq!(s.push(&view(&page, 150)), Step::Grew(150));
        assert_eq!(s.push(&view(&page, 300)), Step::Full);
        assert_eq!(s.push(&view(&page, 400)), Step::Full);
        let (pixels, _, h) = s.finish();
        assert_eq!(h, 500);
        assert!(pixels == page[..500 * W * 4]);
    }

    /// A frame of the page as a window with round bottom corners shows it: in both bottom
    /// corners, pixels that are part page and part whatever lies behind the window.
    fn cornered(page: &[u8], top: usize) -> Vec<u8> {
        let mut frame = view(page, top);
        for y in H - 8..H {
            for x in (0..8).chain(W - 8..W) {
                let edge = if x < 8 { 7 - x } else { x - (W - 8) };
                if edge + (y - (H - 8)) >= 8 {
                    let at = (y * W + x) * 4;
                    for channel in 0..3 {
                        frame[at + channel] /= 3;
                    }
                }
            }
        }
        frame
    }

    #[test]
    fn a_windows_round_corners_are_in_the_picture_once_at_its_end_and_cut_nothing() {
        let page = page(1400);
        let mut s = Stitcher::new(W as u32, H as u32, 100_000, 24, 12).unwrap();
        let mut top = 0;
        s.push(&cornered(&page, 0));
        for step in [90, 90, 5, 180, 90, 210, 90, 90, 165, 90] {
            top += step;
            assert_eq!(
                s.push(&cornered(&page, top)),
                Step::Grew(step as u32),
                "at {top}"
            );
        }
        assert_eq!(top + H, 1400);
        let (pixels, w, h) = s.finish();
        assert_eq!((w as usize, h as usize), (W, 1400));
        // Everything above the last frame's corner rows is the page; the last rows are the
        // last frame's own, corners included.
        assert!(pixels[..(1400 - 8) * W * 4] == page[..(1400 - 8) * W * 4]);
        let last = cornered(&page, top);
        assert!(pixels[(1400 - 8) * W * 4..] == last[(H - 8) * W * 4..]);
    }

    #[test]
    fn a_frame_of_another_size_is_refused() {
        let mut s = stitcher();
        assert_eq!(s.push(&vec![0u8; 16]), Step::Lost);
        assert_eq!(s.size(), (W as u32, 0));
    }
}
