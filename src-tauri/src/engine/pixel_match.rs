//! Pure colour-matching logic for the pixel trigger.
//!
//! Nothing in here touches Win32, so the matching rules can be unit-tested on
//! any platform. Screen capture and the run loop live in `pixel_trigger.rs`.

/// Horizontal search distance (in pixels, each side) around the anchor column
/// when checking the second line. Absorbs anti-aliasing and darker sprite
/// details (e.g. a fish silhouette inside the green zone).
pub const CHECK_SPREAD_PX: usize = 2;

/// An anchor must cover at least this many consecutive pixels. Filters single
/// stray pixels that happen to have the anchor colour.
pub const MIN_ANCHOR_RUN_PX: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// A pixel from a 32-bit top-down GDI DIB (`0x00RRGGBB` as a little-endian u32).
    #[inline]
    pub const fn from_bgrx(px: u32) -> Self {
        Self {
            r: (px >> 16) as u8,
            g: (px >> 8) as u8,
            b: px as u8,
        }
    }

    /// Parses `#rrggbb` (case-insensitive, leading `#` optional).
    pub fn parse_hex(value: &str) -> Option<Self> {
        let hex = value.trim().trim_start_matches('#');
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Self::new(channel(0)?, channel(2)?, channel(4)?))
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// "This pixel has colour X, give or take `tolerance` per channel."
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorRule {
    pub color: Rgb,
    pub tolerance: u8,
}

impl ColorRule {
    #[inline]
    pub fn matches(&self, c: Rgb) -> bool {
        let t = self.tolerance;
        self.color.r.abs_diff(c.r) <= t
            && self.color.g.abs_diff(c.g) <= t
            && self.color.b.abs_diff(c.b) <= t
    }
}

/// Line mode: find the anchor colour on line A, then require the check colour
/// on line B in the same column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineRule {
    /// Left edge (inclusive), virtual-screen coordinates. Always `<= x2`.
    pub x1: i32,
    /// Right edge (inclusive).
    pub x2: i32,
    pub anchor_y: i32,
    pub check_y: i32,
    pub anchor: ColorRule,
    pub check: ColorRule,
}

impl LineRule {
    pub fn new(
        x1: i32,
        x2: i32,
        anchor_y: i32,
        check_y: i32,
        anchor: ColorRule,
        check: ColorRule,
    ) -> Self {
        Self {
            x1: x1.min(x2),
            x2: x1.max(x2),
            anchor_y,
            check_y,
            anchor,
            check,
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        (self.x2 - self.x1) as usize + 1
    }

    /// Top row of the capture rectangle that covers both lines.
    #[inline]
    pub fn top(&self) -> i32 {
        self.anchor_y.min(self.check_y)
    }

    #[inline]
    pub fn height(&self) -> usize {
        (self.anchor_y - self.check_y).unsigned_abs() as usize + 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LineEvaluation {
    /// Column of the anchor centre, relative to `x1`.
    pub anchor_offset: Option<usize>,
    /// Colour on line B directly at the anchor column.
    pub check_color: Option<Rgb>,
    pub matched: bool,
}

/// Centre of the longest run of pixels matching `rule`, if that run is at
/// least `min_run` long.
pub fn find_anchor_center(row: &[Rgb], rule: &ColorRule, min_run: usize) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None; // (start, len)
    let mut run_start = 0usize;
    let mut run_len = 0usize;

    for (i, px) in row.iter().enumerate() {
        if rule.matches(*px) {
            if run_len == 0 {
                run_start = i;
            }
            run_len += 1;
            if best.is_none_or(|(_, len)| run_len > len) {
                best = Some((run_start, run_len));
            }
        } else {
            run_len = 0;
        }
    }

    best.filter(|&(_, len)| len >= min_run.max(1))
        .map(|(start, len)| start + (len - 1) / 2)
}

pub fn evaluate_line(
    anchor_row: &[Rgb],
    check_row: &[Rgb],
    anchor: &ColorRule,
    check: &ColorRule,
) -> LineEvaluation {
    let Some(center) = find_anchor_center(anchor_row, anchor, MIN_ANCHOR_RUN_PX) else {
        return LineEvaluation::default();
    };
    if check_row.is_empty() {
        return LineEvaluation {
            anchor_offset: Some(center),
            ..LineEvaluation::default()
        };
    }
    let center_in_check = center.min(check_row.len() - 1);
    let lo = center_in_check.saturating_sub(CHECK_SPREAD_PX);
    let hi = (center_in_check + CHECK_SPREAD_PX).min(check_row.len() - 1);
    let matched = check_row[lo..=hi].iter().any(|c| check.matches(*c));

    LineEvaluation {
        anchor_offset: Some(center),
        check_color: Some(check_row[center_in_check]),
        matched,
    }
}

/// Point mode: a single pixel that must (or must not) have a colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointRule {
    pub x: i32,
    pub y: i32,
    pub rule: ColorRule,
    pub negate: bool,
}

impl PointRule {
    #[inline]
    pub fn matches(&self, c: Rgb) -> bool {
        self.rule.matches(c) != self.negate
    }
}

/// A wait/hold time in milliseconds, optionally randomized within `min..=max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MsRange {
    pub min: u64,
    pub max: u64,
}

impl MsRange {
    pub const fn fixed(ms: u64) -> Self {
        Self { min: ms, max: ms }
    }

    /// `randomize = false` ignores `max`; a `max` below `min` collapses to `min`.
    pub fn new(min: u64, max: u64, randomize: bool) -> Self {
        if randomize {
            Self {
                min,
                max: max.max(min),
            }
        } else {
            Self::fixed(min)
        }
    }

    /// Value for a uniform sample `unit` in `[0, 1)`.
    pub fn at(&self, unit: f64) -> u64 {
        if self.max <= self.min {
            return self.min;
        }
        let span = (self.max - self.min + 1) as f64;
        let offset = (span * unit.clamp(0.0, 0.999_999_999)) as u64;
        self.min + offset.min(self.max - self.min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_range_fixed_and_random() {
        let fixed = MsRange::new(50, 100, false);
        assert_eq!((fixed.at(0.0), fixed.at(0.99)), (50, 50));
        let random = MsRange::new(50, 100, true);
        assert_eq!(random.at(0.0), 50);
        assert_eq!(random.at(0.999_999), 100);
        assert!((50..=100).contains(&random.at(0.5)));
        let inverted = MsRange::new(80, 10, true);
        assert_eq!((inverted.min, inverted.max), (80, 80));
        assert_eq!(MsRange::fixed(0).at(0.7), 0);
    }

    const WHITE: Rgb = Rgb::new(255, 255, 255);
    const GREEN: Rgb = Rgb::new(60, 210, 30);
    const DARK: Rgb = Rgb::new(20, 45, 50);
    const FISH: Rgb = Rgb::new(40, 120, 20);

    fn white_rule() -> ColorRule {
        ColorRule {
            color: WHITE,
            tolerance: 30,
        }
    }
    fn green_rule() -> ColorRule {
        ColorRule {
            color: GREEN,
            tolerance: 40,
        }
    }

    fn row(len: usize, fill: Rgb, spans: &[(usize, usize, Rgb)]) -> Vec<Rgb> {
        let mut r = vec![fill; len];
        for &(start, end, c) in spans {
            for px in &mut r[start..end] {
                *px = c;
            }
        }
        r
    }

    #[test]
    fn hex_roundtrip_and_rejects_garbage() {
        assert_eq!(Rgb::parse_hex("#3CD21E"), Some(Rgb::new(0x3c, 0xd2, 0x1e)));
        assert_eq!(Rgb::parse_hex("ffffff"), Some(WHITE));
        assert_eq!(Rgb::new(1, 2, 255).to_hex(), "#0102ff");
        assert_eq!(Rgb::parse_hex("#fff"), None);
        assert_eq!(Rgb::parse_hex("#gggggg"), None);
    }

    #[test]
    fn bgrx_decodes_gdi_layout() {
        assert_eq!(Rgb::from_bgrx(0x00_3c_d2_1e), Rgb::new(0x3c, 0xd2, 0x1e));
    }

    #[test]
    fn tolerance_is_per_channel() {
        let rule = ColorRule {
            color: Rgb::new(100, 100, 100),
            tolerance: 10,
        };
        assert!(rule.matches(Rgb::new(110, 90, 100)));
        assert!(!rule.matches(Rgb::new(111, 100, 100)));
    }

    #[test]
    fn anchor_center_uses_longest_run_and_ignores_single_pixels() {
        let r = row(100, DARK, &[(5, 6, WHITE), (40, 51, WHITE)]);
        assert_eq!(find_anchor_center(&r, &white_rule(), 2), Some(45));
        let lone = row(100, DARK, &[(5, 6, WHITE)]);
        assert_eq!(find_anchor_center(&lone, &white_rule(), 2), None);
    }

    #[test]
    fn line_matches_when_arrow_is_under_green_zone() {
        let arrow = row(400, DARK, &[(150, 162, WHITE)]);
        let bar = row(400, DARK, &[(120, 230, GREEN)]);
        let eval = evaluate_line(&arrow, &bar, &white_rule(), &green_rule());
        assert_eq!(eval.anchor_offset, Some(155));
        assert!(eval.matched);
    }

    #[test]
    fn line_does_not_match_when_arrow_is_outside_green_zone() {
        let arrow = row(400, DARK, &[(300, 312, WHITE)]);
        let bar = row(400, DARK, &[(120, 230, GREEN)]);
        let eval = evaluate_line(&arrow, &bar, &white_rule(), &green_rule());
        assert_eq!(eval.anchor_offset, Some(305));
        assert!(!eval.matched);
        assert_eq!(eval.check_color, Some(DARK));
    }

    #[test]
    fn line_tolerates_dark_sprite_pixel_next_to_green() {
        let arrow = row(400, DARK, &[(150, 162, WHITE)]);
        // Column 155 is a fish pixel, neighbours are green.
        let bar = row(400, DARK, &[(120, 230, GREEN), (154, 157, FISH)]);
        let eval = evaluate_line(&arrow, &bar, &white_rule(), &green_rule());
        assert!(eval.matched);
    }

    #[test]
    fn no_anchor_means_no_match() {
        let arrow = row(400, DARK, &[]);
        let bar = row(400, GREEN, &[]);
        assert_eq!(
            evaluate_line(&arrow, &bar, &white_rule(), &green_rule()),
            LineEvaluation::default()
        );
    }

    #[test]
    fn line_rule_normalizes_edges_and_capture_box() {
        let rule = LineRule::new(500, 100, 320, 280, white_rule(), green_rule());
        assert_eq!((rule.x1, rule.x2, rule.width()), (100, 500, 401));
        assert_eq!((rule.top(), rule.height()), (280, 41));
    }

    #[test]
    fn negated_point_inverts_match() {
        let p = PointRule {
            x: 0,
            y: 0,
            rule: green_rule(),
            negate: true,
        };
        assert!(!p.matches(GREEN));
        assert!(p.matches(DARK));
    }
}
