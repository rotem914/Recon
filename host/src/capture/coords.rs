//! The one conversion between desktop coordinates and image space.
//!
//! Part 5 of the plan names four coordinate spaces and says the conversion happens once,
//! at the edge, and that naming the spaces is what makes a wrong conversion findable.
//! This module is that edge, and it is the only place allowed to subtract a frame origin.
//!
//! Desktop space: physical pixels across every display. Its origin is the top-left of the
//! PRIMARY display, so a display arranged to the left or above the primary one has
//! NEGATIVE coordinates. That is the case this arithmetic exists for, and the tests below
//! are written against it because this machine currently has one display at the origin and
//! cannot exercise it live.
//!
//! Image space: pixels inside a captured frame. Origin at the frame's top-left, never
//! negative, and nothing downstream may hold a desktop coordinate.

/// A rectangle in desktop space: physical pixels, origin at the primary display's
/// top-left, so `x` and `y` may be negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl DesktopRect {
    /// Builds a rectangle from two points in any order, which is what a drag produces.
    pub fn from_points(ax: i32, ay: i32, bx: i32, by: i32) -> Self {
        let (left, right) = if ax <= bx { (ax, bx) } else { (bx, ax) };
        let (top, bottom) = if ay <= by { (ay, by) } else { (by, ay) };
        Self {
            x: left,
            y: top,
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// A rectangle inside a captured frame. Always non-negative, always inside the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Where a captured frame sits in desktop space, and how big it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameGeometry {
    /// Desktop coordinate of the frame's pixel (0, 0). Negative when a display is
    /// arranged left of or above the primary one.
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
}

/// Why a desktop rectangle could not become an image rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    /// Zero width or height. A click without a drag.
    Empty,
    /// The rectangle is not entirely inside the frame.
    ///
    /// Deliberately not clamped. Clamping would hand back a rectangle that looks valid and
    /// contains the wrong pixels, which is the failure this whole module exists to prevent:
    /// a caller that asked for the wrong thing should hear about it, not receive a guess.
    Outside,
}

/// Desktop space to image space. The only conversion, called once per capture.
pub fn desktop_to_image(
    frame: FrameGeometry,
    rect: DesktopRect,
) -> Result<ImageRect, ConvertError> {
    if rect.is_empty() {
        return Err(ConvertError::Empty);
    }

    // i64 throughout: a desktop coordinate can be negative and a width can be large, and
    // the subtraction below must not be allowed to wrap on the way to a u32.
    let left = i64::from(rect.x) - i64::from(frame.origin_x);
    let top = i64::from(rect.y) - i64::from(frame.origin_y);
    let right = left + i64::from(rect.width);
    let bottom = top + i64::from(rect.height);

    if left < 0 || top < 0 || right > i64::from(frame.width) || bottom > i64::from(frame.height) {
        return Err(ConvertError::Outside);
    }

    Ok(ImageRect {
        x: left as u32,
        y: top as u32,
        width: rect.width,
        height: rect.height,
    })
}

/// Where something that sat on the desktop, the mouse pointer's picture, lands in the image a
/// selection became: its top left in that image's pixels, or None when no part of it is
/// inside. Signed, since a pointer at the selection's edge hangs over it.
pub fn desktop_box_in_selection(selection: DesktopRect, boxed: DesktopRect) -> Option<(i32, i32)> {
    let left = i64::from(boxed.x) - i64::from(selection.x);
    let top = i64::from(boxed.y) - i64::from(selection.y);
    let inside = left < i64::from(selection.width)
        && top < i64::from(selection.height)
        && left + i64::from(boxed.width) > 0
        && top + i64::from(boxed.height) > 0;
    inside.then_some((left as i32, top as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pointer inside the selection lands at its offset, one hanging over the left edge at
    /// a negative one, and one wholly outside nowhere; on a display left of the primary too.
    #[test]
    fn a_box_on_the_desktop_lands_in_the_selection() {
        let selection = DesktopRect {
            x: -1900,
            y: 100,
            width: 400,
            height: 300,
        };
        let boxed = |x, y| DesktopRect {
            x,
            y,
            width: 32,
            height: 32,
        };
        assert_eq!(
            desktop_box_in_selection(selection, boxed(-1800, 150)),
            Some((100, 50))
        );
        assert_eq!(
            desktop_box_in_selection(selection, boxed(-1910, 100)),
            Some((-10, 0))
        );
        assert_eq!(desktop_box_in_selection(selection, boxed(-1932, 100)), None);
        assert_eq!(desktop_box_in_selection(selection, boxed(-1500, 100)), None);
    }

    /// One display at the origin: this machine's current arrangement.
    fn single_display() -> FrameGeometry {
        FrameGeometry {
            origin_x: 0,
            origin_y: 0,
            width: 5120,
            height: 1440,
        }
    }

    /// The arrangement S0.2's evidence asks for and this machine cannot show live: a
    /// secondary display placed LEFT of the primary, so the virtual desktop starts at a
    /// negative x. Primary 3840x2160 at (0,0), secondary 1920x1080 at (-1920, 0).
    fn secondary_on_the_left() -> FrameGeometry {
        FrameGeometry {
            origin_x: -1920,
            origin_y: 0,
            width: 3840 + 1920,
            height: 2160,
        }
    }

    /// A display placed ABOVE the primary, so y is negative too.
    fn secondary_above() -> FrameGeometry {
        FrameGeometry {
            origin_x: 0,
            origin_y: -1080,
            width: 3840,
            height: 2160 + 1080,
        }
    }

    #[test]
    fn drag_in_any_direction_gives_the_same_rectangle() {
        let a = DesktopRect::from_points(100, 200, 300, 500);
        let b = DesktopRect::from_points(300, 500, 100, 200);
        let c = DesktopRect::from_points(100, 500, 300, 200);
        let d = DesktopRect::from_points(300, 200, 100, 500);
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, d);
        assert_eq!(
            a,
            DesktopRect {
                x: 100,
                y: 200,
                width: 200,
                height: 300
            }
        );
    }

    #[test]
    fn origin_at_zero_is_a_straight_copy() {
        let got = desktop_to_image(
            single_display(),
            DesktopRect {
                x: 40,
                y: 50,
                width: 10,
                height: 20,
            },
        )
        .unwrap();
        assert_eq!(
            got,
            ImageRect {
                x: 40,
                y: 50,
                width: 10,
                height: 20
            }
        );
    }

    #[test]
    fn a_negative_origin_shifts_the_rectangle_into_the_frame() {
        // A selection entirely on the left-hand display: desktop x is negative, image x is not.
        let got = desktop_to_image(
            secondary_on_the_left(),
            DesktopRect {
                x: -1900,
                y: 10,
                width: 100,
                height: 50,
            },
        )
        .unwrap();
        assert_eq!(
            got,
            ImageRect {
                x: 20,
                y: 10,
                width: 100,
                height: 50
            }
        );
    }

    #[test]
    fn the_primary_display_is_offset_when_a_display_sits_to_its_left() {
        // The same desktop rectangle that was a straight copy above is now shifted by 1920,
        // which is exactly the bug a missing origin subtraction produces.
        let got = desktop_to_image(
            secondary_on_the_left(),
            DesktopRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
        )
        .unwrap();
        assert_eq!(got.x, 1920);
        assert_eq!(got.y, 0);
    }

    #[test]
    fn a_negative_y_origin_works_the_same_way() {
        let got = desktop_to_image(
            secondary_above(),
            DesktopRect {
                x: 10,
                y: -1080,
                width: 30,
                height: 40,
            },
        )
        .unwrap();
        assert_eq!(
            got,
            ImageRect {
                x: 10,
                y: 0,
                width: 30,
                height: 40
            }
        );
    }

    #[test]
    fn the_far_corner_is_inside_and_one_pixel_past_it_is_not() {
        let frame = secondary_on_the_left();
        let flush = DesktopRect {
            x: 3840 - 10,
            y: 2160 - 10,
            width: 10,
            height: 10,
        };
        assert!(desktop_to_image(frame, flush).is_ok());

        let over = DesktopRect {
            x: 3840 - 10,
            y: 2160 - 10,
            width: 11,
            height: 10,
        };
        assert_eq!(desktop_to_image(frame, over), Err(ConvertError::Outside));
    }

    #[test]
    fn a_rectangle_starting_before_the_frame_is_refused_not_clamped() {
        let frame = secondary_on_the_left();
        let before = DesktopRect {
            x: -2000,
            y: 0,
            width: 200,
            height: 100,
        };
        assert_eq!(desktop_to_image(frame, before), Err(ConvertError::Outside));
    }

    #[test]
    fn a_click_without_a_drag_is_empty_not_a_one_pixel_capture() {
        let click = DesktopRect::from_points(500, 500, 500, 500);
        assert!(click.is_empty());
        assert_eq!(
            desktop_to_image(single_display(), click),
            Err(ConvertError::Empty)
        );
    }

    #[test]
    fn one_pixel_is_a_capture() {
        let one = DesktopRect::from_points(500, 500, 501, 501);
        assert_eq!(
            desktop_to_image(single_display(), one).unwrap(),
            ImageRect {
                x: 500,
                y: 500,
                width: 1,
                height: 1
            }
        );
    }

    #[test]
    fn a_huge_rectangle_at_a_negative_origin_does_not_wrap() {
        // The arithmetic runs in i64 precisely so this returns Outside rather than a u32
        // that wrapped into something plausible.
        let frame = secondary_on_the_left();
        let huge = DesktopRect {
            x: -1920,
            y: 0,
            width: u32::MAX,
            height: 10,
        };
        assert_eq!(desktop_to_image(frame, huge), Err(ConvertError::Outside));
    }
}
