//! The geometry of "Select an area of the page" in the feedback dialog
//! (M149 B): a selection drawn in CSS pixels over the captured still, the
//! keyboard moves and resizes, and the crop in the still's own pixels.
//! Pure and platform-independent, so it is unit tested on the server build.

/// Selections smaller than this (CSS pixels, either side) are ignored.
pub const MIN_AREA_PX: f64 = 24.0;

/// A rectangle in pixels; `w` and `h` are never negative once normalised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A crop in the still's pixels (whole numbers, inside the image).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

fn finite_pos(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

/// A non-finite value as 0, a negative size as 0 (`f64::clamp` panics on
/// NaN bounds, and a panic stops the whole WASM runtime).
fn size(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

fn coord(v: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    /// From the drag's start and its current point, in any direction.
    pub fn from_corners(ax: f64, ay: f64, bx: f64, by: f64) -> Self {
        Rect {
            x: ax.min(bx),
            y: ay.min(by),
            w: (ax - bx).abs(),
            h: (ay - by).abs(),
        }
    }

    /// A rectangle with negative sizes (a drag up or left) turned round.
    pub fn normalised(self) -> Self {
        Rect::from_corners(self.x, self.y, self.x + self.w, self.y + self.h)
    }

    /// The starting selection: centred, half the viewport each way.
    pub fn centred(view_w: f64, view_h: f64) -> Self {
        let (vw, vh) = (size(view_w), size(view_h));
        let (w, h) = (vw / 2.0, vh / 2.0);
        Rect {
            x: (vw - w) / 2.0,
            y: (vh - h) / 2.0,
            w,
            h,
        }
    }

    /// Big enough to keep (at least [`MIN_AREA_PX`] each way).
    pub fn is_usable(self) -> bool {
        let r = self.normalised();
        r.w >= MIN_AREA_PX && r.h >= MIN_AREA_PX
    }

    /// Clamped inside a `view_w` x `view_h` box (normalised; a side larger
    /// than the box shrinks to it).
    pub fn clamped(self, view_w: f64, view_h: f64) -> Self {
        let r = self.normalised();
        let (vw, vh) = (size(view_w), size(view_h));
        let w = size(r.w).min(vw);
        let h = size(r.h).min(vh);
        let r = Rect {
            x: coord(r.x),
            y: coord(r.y),
            ..r
        };
        Rect {
            x: r.x.clamp(0.0, vw - w),
            y: r.y.clamp(0.0, vh - h),
            w,
            h,
        }
    }

    /// Moved by (`dx`, `dy`), staying inside the box (arrow keys).
    pub fn moved(self, dx: f64, dy: f64, view_w: f64, view_h: f64) -> Self {
        let r = self.clamped(view_w, view_h);
        Rect {
            x: r.x + coord(dx),
            y: r.y + coord(dy),
            ..r
        }
        .clamped(view_w, view_h)
    }

    /// Resized by (`dw`, `dh`) from the top-left corner, never under
    /// [`MIN_AREA_PX`] and never past the box (Alt + arrow keys).
    pub fn resized(self, dw: f64, dh: f64, view_w: f64, view_h: f64) -> Self {
        let r = self.clamped(view_w, view_h);
        let max_w = size(size(view_w) - r.x);
        let max_h = size(size(view_h) - r.y);
        Rect {
            w: size(r.w + coord(dw)).max(MIN_AREA_PX).min(max_w),
            h: size(r.h + coord(dh)).max(MIN_AREA_PX).min(max_h),
            ..r
        }
    }
}

/// Map a selection in CSS pixels over the displayed still to the still's
/// own pixels. The still (`natural_w` x `natural_h`) is drawn inside a
/// `display_w` x `display_h` box scaled to fit (`object-contain`: centred,
/// bars on two sides). The selection is normalised, clamped to the drawn
/// picture and rounded outwards to whole pixels. `None` when a size is not
/// positive, or when what is left on the picture is under [`MIN_AREA_PX`].
pub fn crop_to_source(
    sel: Rect,
    display_w: f64,
    display_h: f64,
    natural_w: f64,
    natural_h: f64,
) -> Option<Crop> {
    if !(finite_pos(display_w) && finite_pos(display_h))
        || !(finite_pos(natural_w) && finite_pos(natural_h))
    {
        return None;
    }
    let s = sel.normalised();
    if !(s.x.is_finite() && s.y.is_finite() && s.w.is_finite() && s.h.is_finite()) {
        return None;
    }
    let scale = (display_w / natural_w).min(display_h / natural_h);
    let (drawn_w, drawn_h) = (natural_w * scale, natural_h * scale);
    let (ox, oy) = ((display_w - drawn_w) / 2.0, (display_h - drawn_h) / 2.0);
    let x0 = s.x.max(ox);
    let y0 = s.y.max(oy);
    let x1 = (s.x + s.w).min(ox + drawn_w);
    let y1 = (s.y + s.h).min(oy + drawn_h);
    if x1 - x0 < MIN_AREA_PX || y1 - y0 < MIN_AREA_PX {
        return None;
    }
    let sx0 = ((x0 - ox) / scale).floor().clamp(0.0, natural_w);
    let sy0 = ((y0 - oy) / scale).floor().clamp(0.0, natural_h);
    let sx1 = ((x1 - ox) / scale).ceil().clamp(0.0, natural_w);
    let sy1 = ((y1 - oy) / scale).ceil().clamp(0.0, natural_h);
    let (w, h) = (sx1 - sx0, sy1 - sy0);
    if w < 1.0 || h < 1.0 {
        return None;
    }
    Some(Crop {
        x: sx0 as u32,
        y: sy0 as u32,
        w: w as u32,
        h: h as u32,
    })
}

/// The size readout under the selection: "640 x 360".
pub fn size_readout(sel: Rect) -> String {
    let r = sel.normalised();
    format!("{} x {}", r.w.round() as i64, r.h.round() as i64)
}

/// What the polite live region reads after a move or resize.
pub fn announcement(sel: Rect) -> String {
    let r = sel.normalised();
    format!(
        "Area {} by {} pixels, {} from the left, {} from the top.",
        r.w.round() as i64,
        r.h.round() as i64,
        r.x.round() as i64,
        r.y.round() as i64
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_size_maps_one_to_one() {
        let c = crop_to_source(
            Rect::new(100.0, 50.0, 640.0, 360.0),
            1280.0,
            720.0,
            1280.0,
            720.0,
        );
        assert_eq!(
            c,
            Some(Crop {
                x: 100,
                y: 50,
                w: 640,
                h: 360
            })
        );
    }

    #[test]
    fn device_pixels_scale_up() {
        // A 2x screen: the still has twice the CSS pixels.
        let c = crop_to_source(
            Rect::new(10.0, 20.0, 100.0, 50.0),
            1280.0,
            720.0,
            2560.0,
            1440.0,
        )
        .unwrap();
        assert_eq!((c.x, c.y, c.w, c.h), (20, 40, 200, 100));
    }

    #[test]
    fn letterboxed_still_accounts_for_the_bars() {
        // A 1000 x 500 still in a 1000 x 1000 box: scale 1, bars of 250 on top and bottom.
        let c = crop_to_source(
            Rect::new(0.0, 250.0, 1000.0, 500.0),
            1000.0,
            1000.0,
            1000.0,
            500.0,
        )
        .unwrap();
        assert_eq!((c.x, c.y, c.w, c.h), (0, 0, 1000, 500));
        // A selection that reaches into the bar is clamped to the picture.
        let c = crop_to_source(
            Rect::new(100.0, 100.0, 200.0, 300.0),
            1000.0,
            1000.0,
            1000.0,
            500.0,
        )
        .unwrap();
        assert_eq!((c.x, c.y, c.w, c.h), (100, 0, 200, 150));
        // Entirely in the bar: nothing.
        assert_eq!(
            crop_to_source(
                Rect::new(100.0, 10.0, 200.0, 200.0),
                1000.0,
                1000.0,
                1000.0,
                500.0
            ),
            None
        );
    }

    #[test]
    fn negative_drags_are_normalised() {
        let up_left = Rect::new(740.0, 410.0, -640.0, -360.0);
        assert_eq!(
            crop_to_source(up_left, 1280.0, 720.0, 1280.0, 720.0),
            crop_to_source(
                Rect::new(100.0, 50.0, 640.0, 360.0),
                1280.0,
                720.0,
                1280.0,
                720.0
            )
        );
        assert_eq!(
            Rect::from_corners(50.0, 60.0, 10.0, 20.0),
            Rect::new(10.0, 20.0, 40.0, 40.0)
        );
    }

    #[test]
    fn selections_past_the_edges_are_clamped() {
        let c = crop_to_source(
            Rect::new(-50.0, -50.0, 2000.0, 2000.0),
            1280.0,
            720.0,
            1280.0,
            720.0,
        )
        .unwrap();
        assert_eq!((c.x, c.y, c.w, c.h), (0, 0, 1280, 720));
    }

    #[test]
    fn small_and_degenerate_selections_are_refused() {
        assert_eq!(
            crop_to_source(
                Rect::new(10.0, 10.0, 23.0, 200.0),
                1280.0,
                720.0,
                1280.0,
                720.0
            ),
            None
        );
        assert!(crop_to_source(
            Rect::new(10.0, 10.0, 24.0, 24.0),
            1280.0,
            720.0,
            1280.0,
            720.0
        )
        .is_some());
        assert_eq!(
            crop_to_source(Rect::new(0.0, 0.0, 100.0, 100.0), 0.0, 720.0, 1280.0, 720.0),
            None
        );
        assert_eq!(
            crop_to_source(Rect::new(0.0, 0.0, 100.0, 100.0), 1280.0, 720.0, 0.0, 720.0),
            None
        );
        assert_eq!(
            crop_to_source(
                Rect::new(f64::NAN, 0.0, 100.0, 100.0),
                1280.0,
                720.0,
                1280.0,
                720.0
            ),
            None
        );
        assert!(!Rect::new(0.0, 0.0, 30.0, 10.0).is_usable());
        assert!(Rect::new(0.0, 0.0, -30.0, -30.0).is_usable());
    }

    #[test]
    fn fractional_selections_round_outwards() {
        let c = crop_to_source(
            Rect::new(10.4, 10.6, 30.2, 30.2),
            100.0,
            100.0,
            100.0,
            100.0,
        )
        .unwrap();
        assert_eq!((c.x, c.y, c.w, c.h), (10, 10, 31, 31));
    }

    #[test]
    fn keyboard_moves_and_resizes_stay_inside() {
        let start = Rect::centred(1280.0, 720.0);
        assert_eq!(start, Rect::new(320.0, 180.0, 640.0, 360.0));
        assert_eq!(start.moved(8.0, 0.0, 1280.0, 720.0).x, 328.0);
        assert_eq!(start.moved(0.0, -1.0, 1280.0, 720.0).y, 179.0);
        // Cannot leave the viewport.
        let far = start.moved(10_000.0, 10_000.0, 1280.0, 720.0);
        assert_eq!((far.x, far.y), (640.0, 360.0));
        let r = start.resized(8.0, -8.0, 1280.0, 720.0);
        assert_eq!((r.w, r.h), (648.0, 352.0));
        // Never under the minimum, never past the right or bottom edge.
        let tiny = Rect::new(0.0, 0.0, 30.0, 30.0).resized(-100.0, -100.0, 1280.0, 720.0);
        assert_eq!((tiny.w, tiny.h), (MIN_AREA_PX, MIN_AREA_PX));
        let big = start.resized(10_000.0, 10_000.0, 1280.0, 720.0);
        assert_eq!((big.w, big.h), (960.0, 540.0));
    }

    #[test]
    fn readout_and_announcement() {
        let r = Rect::new(320.0, 180.0, 640.0, 360.0);
        assert_eq!(size_readout(r), "640 x 360");
        assert_eq!(
            announcement(r),
            "Area 640 by 360 pixels, 320 from the left, 180 from the top."
        );
    }
}
