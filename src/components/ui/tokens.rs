//! Design tokens as data, plus the OKLCH contrast maths.
//!
//! `input.css` is what the browser uses; [`TOKENS`] mirrors its colour
//! tokens so the gallery can print swatches and contrast ratios, and so the
//! tests below can check that both agree and that every contrast pair listed
//! in `docs/DESIGN.md` meets its minimum.

/// One colour token: Tailwind name (`--color-<name>`), OKLCH value, role.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorToken {
    pub name: &'static str,
    pub l: f64,
    pub c: f64,
    pub h: f64,
    pub group: TokenGroup,
    pub role: &'static str,
}

/// Where a colour token may be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenGroup {
    Ground,
    Line,
    Text,
    Accent,
    Domain,
    Status,
    Terrain,
}

impl TokenGroup {
    pub fn label(self) -> &'static str {
        match self {
            TokenGroup::Ground => "Ground: green-slate h200",
            TokenGroup::Line => "Line",
            TokenGroup::Text => "Text",
            TokenGroup::Accent => "Accent: survey chalk",
            TokenGroup::Domain => "Domain: sides and results",
            TokenGroup::Status => "Status",
            TokenGroup::Terrain => "Terrain: canvases only",
        }
    }
}

const fn t(
    name: &'static str,
    l: f64,
    c: f64,
    h: f64,
    group: TokenGroup,
    role: &'static str,
) -> ColorToken {
    ColorToken {
        name,
        l,
        c,
        h,
        group,
        role,
    }
}

/// Every opaque colour token in `input.css`, in display order.
pub const TOKENS: &[ColorToken] = &[
    t(
        "page",
        0.180,
        0.018,
        200.0,
        TokenGroup::Ground,
        "Workspace behind everything",
    ),
    t(
        "ink-1",
        0.205,
        0.020,
        200.0,
        TokenGroup::Ground,
        "App chrome, side rails, raised layer",
    ),
    t(
        "ink-2",
        0.235,
        0.020,
        200.0,
        TokenGroup::Ground,
        "Fields, selected rows, second layer",
    ),
    t(
        "ink-3",
        0.275,
        0.020,
        200.0,
        TokenGroup::Ground,
        "Hover and pressed",
    ),
    t(
        "canvas",
        0.190,
        0.022,
        195.0,
        TokenGroup::Ground,
        "Map and graph ground",
    ),
    t(
        "rule",
        0.340,
        0.016,
        200.0,
        TokenGroup::Line,
        "Hairlines for structure (decorative)",
    ),
    t(
        "outline",
        0.580,
        0.020,
        200.0,
        TokenGroup::Line,
        "Boundary of anything interactive",
    ),
    t(
        "primary",
        0.950,
        0.008,
        190.0,
        TokenGroup::Text,
        "Primary text",
    ),
    t(
        "secondary",
        0.820,
        0.012,
        195.0,
        TokenGroup::Text,
        "Secondary text",
    ),
    t(
        "muted",
        0.700,
        0.014,
        200.0,
        TokenGroup::Text,
        "Muted text, labels, overlines",
    ),
    t(
        "accent",
        0.910,
        0.100,
        108.0,
        TokenGroup::Accent,
        "Focus, selection, primary action",
    ),
    t(
        "accent-hover",
        0.950,
        0.070,
        108.0,
        TokenGroup::Accent,
        "Primary action hover",
    ),
    t(
        "accent-ink",
        0.240,
        0.030,
        110.0,
        TokenGroup::Accent,
        "Text on accent fills",
    ),
    t(
        "side-blue",
        0.700,
        0.120,
        245.0,
        TokenGroup::Domain,
        "Blue side: circle marker",
    ),
    t(
        "side-red",
        0.700,
        0.120,
        28.0,
        TokenGroup::Domain,
        "Red side: square marker",
    ),
    t(
        "side-blue-deep",
        0.300,
        0.050,
        245.0,
        TokenGroup::Domain,
        "Blue token fill",
    ),
    t(
        "side-red-deep",
        0.300,
        0.050,
        28.0,
        TokenGroup::Domain,
        "Red token fill",
    ),
    t(
        "win",
        0.800,
        0.110,
        175.0,
        TokenGroup::Domain,
        "Win: filled W glyph",
    ),
    t(
        "win-ink",
        0.220,
        0.030,
        175.0,
        TokenGroup::Domain,
        "Text on win fill",
    ),
    t(
        "loss",
        0.700,
        0.014,
        200.0,
        TokenGroup::Domain,
        "Loss: hollow L glyph, no hue",
    ),
    t(
        "success",
        0.790,
        0.120,
        155.0,
        TokenGroup::Status,
        "Done, saved, ready",
    ),
    t(
        "warning",
        0.800,
        0.140,
        78.0,
        TokenGroup::Status,
        "Needs attention; always with text",
    ),
    t(
        "danger",
        0.720,
        0.160,
        12.0,
        TokenGroup::Status,
        "Errors, destructive actions",
    ),
    t(
        "danger-deep",
        0.420,
        0.130,
        14.0,
        TokenGroup::Status,
        "Danger button hover fill (with primary text)",
    ),
    t(
        "info",
        0.790,
        0.080,
        215.0,
        TokenGroup::Status,
        "Neutral information",
    ),
    t(
        "terrain-jungle",
        0.215,
        0.024,
        170.0,
        TokenGroup::Terrain,
        "Jungle fill",
    ),
    t(
        "terrain-river",
        0.255,
        0.032,
        225.0,
        TokenGroup::Terrain,
        "River",
    ),
    t(
        "terrain-lane",
        0.300,
        0.014,
        200.0,
        TokenGroup::Terrain,
        "Lanes",
    ),
    t(
        "terrain-texture",
        0.240,
        0.022,
        195.0,
        TokenGroup::Terrain,
        "Contours and grid, 1.1:1",
    ),
];

/// A foreground/background pair with the minimum contrast it must meet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastPair {
    pub fg: &'static str,
    pub bg: &'static str,
    pub min: f64,
    pub what: &'static str,
}

const fn p(fg: &'static str, bg: &'static str, min: f64, what: &'static str) -> ContrastPair {
    ContrastPair { fg, bg, min, what }
}

/// The pairs the gallery prints. `docs/DESIGN.md` lists the full set the
/// tests enforce (between the `contrast-pairs` markers).
pub const GALLERY_PAIRS: &[ContrastPair] = &[
    p("primary", "page", 4.5, "text"),
    p("secondary", "ink-1", 4.5, "text"),
    p("muted", "ink-3", 4.5, "text"),
    p("accent-ink", "accent", 4.5, "text"),
    p("accent", "ink-2", 4.5, "focus ring, selected text"),
    p("outline", "ink-2", 3.0, "control edge"),
    p("side-blue", "ink-1", 3.0, "marker"),
    p("side-red", "ink-1", 3.0, "marker"),
    p("win-ink", "win", 4.5, "text"),
    p("loss", "ink-1", 3.0, "glyph"),
    p("warning", "ink-1", 4.5, "text"),
    p("danger", "ink-1", 4.5, "text"),
    p("primary", "danger-deep", 4.5, "danger button hover"),
    p("success", "ink-1", 4.5, "text"),
    p("info", "ink-1", 4.5, "text"),
    p("primary", "side-blue-deep", 4.5, "token label"),
    p("primary", "side-red-deep", 4.5, "token label"),
];

/// Look a token up by name.
pub fn token(name: &str) -> Option<&'static ColorToken> {
    TOKENS.iter().find(|t| t.name == name)
}

/// OKLCH to linear sRGB (clamped to the gamut) using Björn Ottosson's matrices.
pub fn oklch_to_linear_srgb(l: f64, c: f64, h: f64) -> [f64; 3] {
    let (sin, cos) = h.to_radians().sin_cos();
    let a = c * cos;
    let b = c * sin;
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let g = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let bl = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;
    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), bl.clamp(0.0, 1.0)]
}

/// WCAG relative luminance of an OKLCH colour.
pub fn relative_luminance(l: f64, c: f64, h: f64) -> f64 {
    let [r, g, b] = oklch_to_linear_srgb(l, c, h);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG 2 contrast ratio between two tokens.
pub fn contrast(a: &ColorToken, b: &ColorToken) -> f64 {
    let la = relative_luminance(a.l, a.c, a.h);
    let lb = relative_luminance(b.l, b.c, b.h);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Contrast between two named tokens; `None` if either name is unknown.
pub fn contrast_by_name(fg: &str, bg: &str) -> Option<f64> {
    Some(contrast(token(fg)?, token(bg)?))
}

/// The CSS value of a token, as written in `input.css`.
pub fn css_value(t: &ColorToken) -> String {
    format!("oklch({:.3} {:.3} {})", t.l, t.c, t.h)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT_CSS: &str = include_str!("../../../input.css");
    const DESIGN_MD: &str = include_str!("../../../docs/DESIGN.md");

    /// `--color-name: oklch(L C H);` lines from the `@theme` block.
    fn css_tokens() -> Vec<(String, f64, f64, f64, bool)> {
        let mut out = Vec::new();
        for line in INPUT_CSS.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix("--color-") else {
                continue;
            };
            let Some((name, value)) = rest.split_once(':') else {
                continue;
            };
            let value = value.trim().trim_end_matches(';').trim();
            let Some(inner) = value
                .strip_prefix("oklch(")
                .and_then(|v| v.strip_suffix(')'))
            else {
                continue;
            };
            let (nums, alpha) = match inner.split_once('/') {
                Some((n, _)) => (n, true),
                None => (inner, false),
            };
            let parts: Vec<f64> = nums
                .split_whitespace()
                .map(|n| n.parse().expect("numeric oklch component"))
                .collect();
            assert_eq!(parts.len(), 3, "oklch needs L C H: {line}");
            out.push((name.trim().to_string(), parts[0], parts[1], parts[2], alpha));
        }
        out
    }

    #[test]
    fn rust_table_matches_input_css() {
        let css: Vec<_> = css_tokens().into_iter().filter(|t| !t.4).collect();
        for tok in TOKENS {
            let found = css
                .iter()
                .find(|c| c.0 == tok.name)
                .unwrap_or_else(|| panic!("--color-{} missing from input.css", tok.name));
            assert!(
                (found.1 - tok.l).abs() < 1e-9
                    && (found.2 - tok.c).abs() < 1e-9
                    && (found.3 - tok.h).abs() < 1e-9,
                "--color-{} differs between input.css and TOKENS",
                tok.name
            );
        }
        for c in &css {
            assert!(
                token(&c.0).is_some(),
                "--color-{} is in input.css but not in TOKENS",
                c.0
            );
        }
    }

    #[test]
    fn every_token_is_inside_the_srgb_gamut() {
        for tok in TOKENS {
            let (sin, cos) = tok.h.to_radians().sin_cos();
            let (a, b) = (tok.c * cos, tok.c * sin);
            let l_ = tok.l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
            let m_ = tok.l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
            let s_ = tok.l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
            let (l3, m3, s3) = (l_.powi(3), m_.powi(3), s_.powi(3));
            let rgb = [
                4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
                -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
                -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
            ];
            for v in rgb {
                assert!(
                    (-1e-3..=1.0 + 1e-3).contains(&v),
                    "{} out of gamut",
                    tok.name
                );
            }
        }
    }

    #[test]
    fn known_values() {
        // White on black is 21:1; a colour against itself is 1:1.
        let white = t("w", 1.0, 0.0, 0.0, TokenGroup::Text, "");
        let black = t("b", 0.0, 0.0, 0.0, TokenGroup::Ground, "");
        assert!((contrast(&white, &black) - 21.0).abs() < 0.01);
        assert!((contrast(&white, &white) - 1.0).abs() < 1e-9);
    }

    /// Rows `| fg | bg | min | ... |` between the contrast-pairs markers.
    fn design_pairs() -> Vec<(String, String, f64)> {
        let start = DESIGN_MD
            .find("<!-- contrast-pairs:start -->")
            .expect("DESIGN.md has the contrast-pairs:start marker");
        let end = DESIGN_MD
            .find("<!-- contrast-pairs:end -->")
            .expect("DESIGN.md has the contrast-pairs:end marker");
        DESIGN_MD[start..end]
            .lines()
            .filter(|l| l.starts_with("| `"))
            .map(|l| {
                let cells: Vec<&str> = l.split('|').map(|c| c.trim()).collect();
                let fg = cells[1].trim_matches('`').to_string();
                let bg = cells[2].trim_matches('`').to_string();
                let min: f64 = cells[3].parse().expect("minimum is a number");
                (fg, bg, min)
            })
            .collect()
    }

    #[test]
    fn design_md_contrast_pairs_meet_their_minimum() {
        let pairs = design_pairs();
        assert!(pairs.len() >= 30, "DESIGN.md lists the full pair table");
        let mut failures = Vec::new();
        for (fg, bg, min) in &pairs {
            let ratio = contrast_by_name(fg, bg)
                .unwrap_or_else(|| panic!("unknown token in DESIGN.md pair {fg} on {bg}"));
            if ratio + 1e-9 < *min {
                failures.push(format!("{fg} on {bg}: {ratio:.2} < {min}"));
            }
        }
        assert!(failures.is_empty(), "contrast failures: {failures:#?}");
    }

    #[test]
    fn gallery_pairs_are_in_design_md() {
        let pairs = design_pairs();
        for gp in GALLERY_PAIRS {
            assert!(
                pairs
                    .iter()
                    .any(|(f, b, m)| f == gp.fg && b == gp.bg && (*m - gp.min).abs() < 1e-9),
                "gallery pair {} on {} is not in DESIGN.md",
                gp.fg,
                gp.bg
            );
        }
    }

    #[test]
    fn sides_are_a_matched_pair_and_results_do_not_reuse_them() {
        let blue = token("side-blue").unwrap();
        let red = token("side-red").unwrap();
        assert_eq!((blue.l, blue.c), (red.l, red.c));
        for result in ["win", "loss"] {
            let r = token(result).unwrap();
            for side in [blue, red] {
                let dh = (r.h - side.h).abs() % 360.0;
                let dh = dh.min(360.0 - dh);
                assert!(
                    dh > 40.0 || r.c < 0.03,
                    "{result} is too close to {} in hue",
                    side.name
                );
            }
        }
    }

    #[test]
    fn no_sub_12px_text_size_tokens() {
        for line in INPUT_CSS.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("--text-") {
                if rest.contains("--line-height") {
                    continue;
                }
                if let Some((_, v)) = rest.split_once(':') {
                    let v = v.trim().trim_end_matches(';');
                    if let Some(rem) = v.strip_suffix("rem") {
                        let px = rem.parse::<f64>().unwrap() * 16.0;
                        assert!(px >= 12.0, "{line} is below 12px");
                    }
                }
            }
        }
    }
}
