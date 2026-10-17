//! RiftSketch (M152): the drawn Rift. An original schematic built from
//! measured coordinates (`scripts/rift-geometry/`, `docs/rift-schematic.md`),
//! never traced or vectorised from a picture of the map, at two levels of
//! detail (recipe cards A and B, `.planning/missions/M152/recipes-r1.md`):
//!
//! - **Sketch** (zoomed out, the hero surfaces): frame, three lanes, the
//!   river as two banks on a two-cubic centreline, the two bases as their
//!   side shapes, impassable terrain as axis-aligned hatch (no outlines),
//!   Baron and Dragon diamonds with their words, ten bush clusters.
//! - **Survey** (zoomed in, `/design` only this round): exact wall outlines
//!   with 45-degree hatch held at a 6 px screen pitch, bushes as outlined
//!   stipple, river and ground outlines, and the structures and camps as ink
//!   glyphs at their positions.
//!
//! One ink (`primary` at 70 %), one stroke weight (1 px, 1.25 px from `md`,
//! non-scaling), radius 0, nothing moves. The only chalk is the optional
//! surveyed label and its leader.

use leptos::prelude::*;

use super::rift_geometry as g;

/// Level of detail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiftLevel {
    /// Smoothed, hatch-only terrain: heroes, Match Day, share strip.
    #[default]
    Sketch,
    /// Exact outlines, hatched walls, stippled bushes, structure glyphs.
    Survey,
}

/// Zoom stop of the survey level (its hatch pitch and glyph sizes are set
/// for a 560 px sheet shown at this scale).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiftZoom {
    /// 1120 px sheet.
    #[default]
    X2,
    /// 2240 px sheet.
    X4,
}

impl RiftZoom {
    /// Drawing units per screen pixel at this stop (560 px base sheet).
    fn unit_per_px(self) -> f32 {
        match self {
            RiftZoom::X2 => 1.0 / 11.2,
            RiftZoom::X4 => 1.0 / 22.4,
        }
    }

    fn id(self) -> &'static str {
        match self {
            RiftZoom::X2 => "2",
            RiftZoom::X4 => "4",
        }
    }
}

/// A named point the chalk label can be surveyed onto. A base is named by
/// its shape on the sheet, never by an invented side: the caller anchors a
/// label at a base only when the data says which one it is (card D).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiftAnchor {
    /// The circle (bottom left).
    BlueBase,
    /// The square (top right).
    RedBase,
}

impl RiftAnchor {
    fn leader(self) -> ((f32, f32), (f32, f32)) {
        match self {
            RiftAnchor::BlueBase => g::LEADER_BLUE,
            RiftAnchor::RedBase => g::LEADER_RED,
        }
    }

    fn nexus(self) -> (f32, f32) {
        match self {
            RiftAnchor::BlueBase => g::NEXUS_BLUE,
            RiftAnchor::RedBase => g::NEXUS_RED,
        }
    }
}

/// Which part of the sheet to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiftCrop {
    /// The whole square sheet.
    #[default]
    Full,
    /// Match Day's 88 px inset: lanes, river (gap at mid), bases only.
    Inset,
    /// Share strip centred on the river-mid bridge: 40 x 10.7 units below
    /// `md`, 50 x 9.4 from `md`; no hatch, no bushes (R1-b).
    ShareStrip,
    /// Share strip centred on one base, for an artefact that carries a side.
    ShareStripAt(RiftAnchor),
}

/// Ground the sheet sits on (the survey level erases hatch near outlines
/// in this colour).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiftGround {
    /// Landing, share: the page ground.
    #[default]
    Page,
    /// Inside a Match Day card.
    Ink1,
}

impl RiftGround {
    fn erase(self) -> &'static str {
        match self {
            RiftGround::Page => "fill-none stroke-page",
            RiftGround::Ink1 => "fill-none stroke-ink-1",
        }
    }
}

/// The surveyed label: chalk text set on the sheet with a 1 px leader to a
/// named anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiftLabel {
    pub text: String,
    pub anchor: RiftAnchor,
}

impl RiftLabel {
    pub fn new(text: impl Into<String>, anchor: RiftAnchor) -> Self {
        Self {
            text: text.into(),
            anchor,
        }
    }
}

/// Ink group: one colour, one weight, round caps and joins, non-scaling.
const INK: &str = "fill-none stroke-primary/70 stroke-1 md:stroke-[1.25] [stroke-linecap:round] [stroke-linejoin:round] [&_path]:[vector-effect:non-scaling-stroke]";
/// The Match Day inset keeps 1 px at every width.
const INK_THIN: &str = "fill-none stroke-primary/70 stroke-1 [stroke-linecap:round] [stroke-linejoin:round] [&_path]:[vector-effect:non-scaling-stroke]";
const CHALK: &str = "fill-none stroke-accent stroke-1 [stroke-linecap:round] [&_path]:[vector-effect:non-scaling-stroke]";
const TICKS: &str = "hidden fill-none stroke-muted stroke-1 md:inline [stroke-linecap:round] [&_path]:[vector-effect:non-scaling-stroke]";
const LABEL: &str =
    "eyebrow pointer-events-none absolute whitespace-nowrap text-accent md:text-label";
const WORD: &str = "eyebrow pointer-events-none absolute whitespace-nowrap";

fn f(v: f32) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

fn circle(c: (f32, f32), r: f32) -> String {
    format!(
        "M{},{}a{r},{r} 0 1 0 {d},0a{r},{r} 0 1 0 -{d},0",
        f(c.0 - r),
        f(c.1),
        r = f(r),
        d = f(2.0 * r)
    )
}

fn diamond(c: (f32, f32), half: f32) -> String {
    format!(
        "M{},{}l{h},{h}l-{h},{h}l-{h},-{h}Z",
        f(c.0),
        f(c.1 - half),
        h = f(half)
    )
}

fn square(c: (f32, f32), half: f32) -> String {
    format!(
        "M{},{}h{s}v{s}h-{s}Z",
        f(c.0 - half),
        f(c.1 - half),
        s = f(2.0 * half)
    )
}

/// Survey glyphs at one zoom stop (card B sizes, in screen pixels):
/// returns (filled, stroked) path data.
pub fn survey_glyphs(zoom: RiftZoom) -> (String, String) {
    let k = zoom.unit_per_px();
    let mut fill = String::new();
    let mut stroke = String::new();
    for t in g::TOWERS {
        fill.push_str(&circle(t, 2.5 * k));
    }
    for n in [g::NEXUS_BLUE, g::NEXUS_RED] {
        fill.push_str(&circle(n, 2.0 * k));
    }
    for c in g::BUFF_CAMPS {
        fill.push_str(&circle(c, 1.0 * k));
        stroke.push_str(&circle(c, 3.5 * k));
    }
    for c in g::SMALL_CAMPS {
        stroke.push_str(&circle(c, 2.0 * k));
    }
    for i in g::INHIBITORS {
        stroke.push_str(&diamond(i, 4.5 * k));
    }
    stroke.push_str(&circle(g::NEXUS_BLUE, 8.0 * k));
    stroke.push_str(&square(g::NEXUS_RED, 8.0 * k));
    stroke.push_str(&diamond(g::PIT_BARON, 7.0 * k));
    stroke.push_str(&diamond(g::PIT_DRAGON, 7.0 * k));
    (fill, stroke)
}

/// A position on the sheet as CSS percentages of a view box.
fn pct(v: f32, origin: f32, size: f32) -> String {
    format!("{}%", f((v - origin) / size * 100.0))
}

/// The chalk label and its leader over one view box.
fn label_view(label: &RiftLabel, vb: (f32, f32, f32, f32)) -> (impl IntoView, impl IntoView) {
    let ((x0, y0), (x1, y1)) = label.anchor.leader();
    let (vx, vy, vw, vh) = vb;
    let top = pct(y1, vy, vh);
    // The text starts one unit past the leader's end, away from the base.
    let style = match label.anchor {
        RiftAnchor::BlueBase => format!(
            "left:{};top:{top};transform:translateY(-50%)",
            pct(x1 + 1.0, vx, vw)
        ),
        RiftAnchor::RedBase => format!(
            "right:{}%;top:{top};transform:translateY(-50%)",
            f((vx + vw - (x1 - 1.0)) / vw * 100.0)
        ),
    };
    let leader = format!("M{},{}L{},{}", f(x0), f(y0), f(x1), f(y1));
    let dot = format!("M{},{}h0", f(x0), f(y0));
    let text = label.text.clone();
    (
        view! {
            <g class=CHALK data-rift-chalk="leader">
                <path d=leader />
                <path d=dot />
            </g>
        },
        view! { <span class=LABEL style=style data-rift-chalk="label">{text}</span> },
    )
}

/// The drawn Rift (see the module doc for the two levels).
///
/// A11y: decorative artwork. The whole drawing, its chalk label and its
/// Baron/Dragon words are `aria-hidden` (the SVG is also
/// `focusable="false"`); the page states what the drawing shows in its own
/// text (the label repeats a name the page already prints). It carries no
/// controls and nothing in it takes focus; the zoomable preview on `/design`
/// puts its controls outside the drawing.
#[component]
pub fn RiftSketch(
    #[prop(optional)] level: RiftLevel,
    #[prop(optional)] crop: RiftCrop,
    /// Survey only: the zoom stop the sheet is drawn for.
    #[prop(optional)]
    zoom: RiftZoom,
    #[prop(optional)] ground: RiftGround,
    /// The surveyed label (chalk). Full and share-strip crops only.
    #[prop(optional)]
    label: Option<RiftLabel>,
    /// Sizing classes for the wrapper (for example `w-full max-w-[560px]`).
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    match crop {
        RiftCrop::Inset => view! {
            <div class=format!("relative {class}") aria-hidden="true" data-rift="inset">
                <svg viewBox="0 0 100 100" class="block h-auto w-full" focusable="false">
                    <g class=INK_THIN>
                        <path d=g::SKETCH_LANES />
                        <path d=g::INSET_RIVER />
                        <path d=g::SKETCH_BASE_BLUE />
                        <path d=g::SKETCH_BASE_RED />
                    </g>
                </svg>
            </div>
        }
        .into_any(),
        RiftCrop::ShareStrip | RiftCrop::ShareStripAt(_) => {
            let centre = match crop {
                RiftCrop::ShareStripAt(a) => a.nexus(),
                _ => (50.0, 50.0),
            };
            let label = match crop {
                RiftCrop::ShareStripAt(_) => label,
                _ => None,
            };
            let strip = move |w: f32, h: f32, shown: &'static str| {
                let x = (centre.0 - w / 2.0).clamp(0.0, 100.0 - w);
                let y = (centre.1 - h / 2.0).clamp(0.0, 100.0 - h);
                let vb = format!("{} {} {} {}", f(x), f(y), f(w), f(h));
                let (lead, text) = match &label {
                    Some(l) => {
                        let (a, b) = label_view(l, (x, y, w, h));
                        (Some(a), Some(b))
                    }
                    None => (None, None),
                };
                view! {
                    <div class=format!("relative {shown}") data-rift-strip=f(w)>
                        <svg viewBox=vb class="block h-auto w-full" focusable="false" preserveAspectRatio="xMidYMid slice">
                            <g class=INK>
                                <path d=g::SKETCH_LANES />
                                <path d=g::SKETCH_BANKS />
                                <path d=g::SKETCH_BASE_BLUE />
                                <path d=g::SKETCH_BASE_RED />
                            </g>
                            {lead}
                        </svg>
                        {text}
                    </div>
                }
            };
            view! {
                <div class=format!("relative {class}") aria-hidden="true" data-rift="share">
                    {strip(40.0, 10.7, "md:hidden")}
                    {strip(50.0, 9.4, "hidden md:block")}
                </div>
            }
            .into_any()
        }
        RiftCrop::Full => match level {
            RiftLevel::Sketch => {
                let (lead, text) = match &label {
                    Some(l) => {
                        let (a, b) = label_view(l, (0.0, 0.0, 100.0, 100.0));
                        (Some(a), Some(b))
                    }
                    None => (None, None),
                };
                let half = 1.45;
                let baron = format!(
                    "left:{};top:{};transform:translateY(-50%)",
                    pct(g::BARON_AT.0 + half + 0.8, 0.0, 100.0),
                    pct(g::BARON_AT.1, 0.0, 100.0)
                );
                let dragon = format!(
                    "right:{};top:{};transform:translateY(-50%)",
                    pct(100.0 - (g::DRAGON_AT.0 - half - 0.8), 0.0, 100.0),
                    pct(g::DRAGON_AT.1, 0.0, 100.0)
                );
                let cols = ["A", "B", "C", "D", "E", "F", "G", "H"];
                let ticks_x = cols
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let left = pct(2.0 + 96.0 * (i as f32 + 0.5) / 8.0, 0.0, 100.0);
                        let style = format!("left:{left};top:0;transform:translate(-50%,-110%)");
                        view! { <span class="figures pointer-events-none absolute hidden text-caption text-muted md:block" style=style>{*c}</span> }
                    })
                    .collect_view();
                let ticks_y = (1..=5)
                    .map(|i| {
                        let top = pct(2.0 + 96.0 * (i as f32 - 0.5) / 5.0, 0.0, 100.0);
                        let style = format!("left:0;top:{top};transform:translate(-140%,-50%)");
                        view! { <span class="figures pointer-events-none absolute hidden text-caption text-muted md:block" style=style>{i}</span> }
                    })
                    .collect_view();
                view! {
                    <div class=format!("relative {class}") aria-hidden="true" data-rift="sketch">
                        <svg viewBox="0 0 100 100" class="block h-auto w-full" focusable="false">
                            <g class=TICKS>
                                <path d=g::SKETCH_TICKS />
                            </g>
                            <g class=INK>
                                <path d=g::SKETCH_FRAME />
                                <path d=g::SKETCH_LANES />
                                <path d=g::SKETCH_BANKS />
                                <path d=g::SKETCH_BASE_BLUE />
                                <path d=g::SKETCH_BASE_RED />
                                <path d=g::SKETCH_MARKS />
                                <path d=g::SKETCH_HATCH class="[shape-rendering:crispEdges]" data-rift-layer="hatch" />
                                <path d=g::SKETCH_BUSH_DOTS class="stroke-2" data-rift-layer="bush" />
                            </g>
                            {lead}
                        </svg>
                        <span class=WORD style=baron>"Baron"</span>
                        <span class=WORD style=dragon>"Dragon"</span>
                        {ticks_x}
                        {ticks_y}
                        {text}
                    </div>
                }
                .into_any()
            }
            RiftLevel::Survey => {
                let k = zoom.unit_per_px();
                let id = zoom.id();
                let hatch_id = format!("rift-hatch-{id}");
                let stipple_id = format!("rift-stipple-{id}");
                let walls_id = format!("rift-walls-{id}");
                let bushes_id = format!("rift-bushes-{id}");
                // 6 px hatch pitch, 1 px line; 7 px hex stipple of 2 px dots.
                let p = 6.0 * k;
                let hatch_line = format!("M0,0V{}", f(p));
                let sw = f(k);
                let sx = 7.0 * k;
                let sy = 7.0 * 3f32.sqrt() * k;
                let r = k;
                let stipple = [
                    (0.0, 0.0),
                    (sx, 0.0),
                    (0.0, sy),
                    (sx, sy),
                    (sx / 2.0, sy / 2.0),
                ]
                .iter()
                .map(|&c| circle(c, r))
                .collect::<String>();
                let (glyph_fill, glyph_stroke) = survey_glyphs(zoom);
                let hatch_fill = format!("url(#{hatch_id})");
                let stipple_fill = format!("url(#{stipple_id})");
                let walls_clip = format!("url(#{walls_id})");
                let bushes_clip = format!("url(#{bushes_id})");
                view! {
                    <div class=format!("relative {class}") aria-hidden="true" data-rift="survey" data-rift-zoom=id>
                        <svg viewBox="0 0 100 100" class="block h-auto w-full" focusable="false">
                            <defs>
                                <pattern id=hatch_id.clone() patternUnits="userSpaceOnUse" width=f(p) height=f(p) patternTransform="rotate(45)">
                                    <path d=hatch_line class="fill-none stroke-primary/70" stroke-width=sw.clone() />
                                </pattern>
                                <pattern id=stipple_id.clone() patternUnits="userSpaceOnUse" width=f(sx) height=f(sy)>
                                    <path d=stipple class="fill-primary/70" />
                                </pattern>
                                <clipPath id=walls_id.clone()>
                                    <path d=g::SURVEY_WALLS />
                                </clipPath>
                                <clipPath id=bushes_id.clone()>
                                    <path d=g::SURVEY_BUSHES />
                                </clipPath>
                            </defs>
                            <rect x="0" y="0" width="100" height="100" fill=hatch_fill clip-path=walls_clip data-rift-layer="hatch" />
                            // Erase the hatch 4 px either side of each outline.
                            <path d=g::SURVEY_WALLS class=ground.erase() stroke-width="8" vector-effect="non-scaling-stroke" />
                            <rect x="0" y="0" width="100" height="100" fill=stipple_fill clip-path=bushes_clip data-rift-layer="bush" />
                            <g class=INK>
                                <path d=g::SURVEY_BOUNDARY />
                                <path d=g::SURVEY_RIVER />
                                <path d=g::SURVEY_WALLS data-rift-layer="walls" />
                                <path d=g::SURVEY_BUSHES />
                                <path d=glyph_stroke data-rift-layer="glyphs" />
                            </g>
                            <path d=glyph_fill class="fill-primary/70" />
                        </svg>
                    </div>
                }
                .into_any()
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_counts_match_the_geometry_json() {
        let json: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/rift/map11-geometry.json"
        )))
        .expect("geometry json");
        let counts = &json["counts"];
        assert_eq!(counts["tower"], 22);
        assert_eq!(counts["inhibitor"], 6);
        assert_eq!(counts["nexus"], 2);
        assert_eq!(counts["pit"], 2);
        assert_eq!(counts["tower"], g::TOWERS.len());
        assert_eq!(counts["inhibitor"], g::INHIBITORS.len());
        assert_eq!(counts["wall"], g::WALL_COUNT);
        assert_eq!(counts["bush"], g::BUSH_COUNT);
        assert_eq!(counts["camp"], g::BUFF_CAMPS.len() + g::SMALL_CAMPS.len());
        assert_eq!(g::CAMP_COUNT, g::BUFF_CAMPS.len() + g::SMALL_CAMPS.len());
        // One closed subpath per wall and per bush in the survey outlines.
        assert_eq!(g::SURVEY_WALLS.matches('Z').count(), g::WALL_COUNT);
        assert_eq!(g::SURVEY_BUSHES.matches('Z').count(), g::BUSH_COUNT);
    }

    #[test]
    fn every_feature_names_its_source() {
        let json: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/rift/map11-geometry.json"
        )))
        .expect("geometry json");
        let features = json["features"].as_array().expect("features");
        assert!(features.len() > 100);
        for feature in features {
            let id = feature["id"].as_str().unwrap_or("?");
            let source = feature["source"].as_str().unwrap_or("");
            assert!(
                ["fixture:", "measured:", "reported", "approximate"]
                    .iter()
                    .any(|k| source.starts_with(k)),
                "{id} has no recognised source: {source:?}"
            );
            if let Some(path) = source
                .strip_prefix("fixture:")
                .and_then(|s| s.split(':').next())
            {
                assert!(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join(path)
                        .exists(),
                    "{id}: fixture {path} is missing"
                );
            }
        }
        // The elevation stub: only the river is below the ground.
        let low: Vec<&str> = features
            .iter()
            .filter(|f| f.get("elev").is_some())
            .map(|f| f["id"].as_str().unwrap_or("?"))
            .collect();
        assert_eq!(low, vec!["river"]);
    }

    #[test]
    fn the_sketch_fits_the_inline_budget_and_stays_on_the_sheet() {
        let sketch = [
            g::SKETCH_FRAME,
            g::SKETCH_LANES,
            g::SKETCH_BANKS,
            g::SKETCH_BASE_BLUE,
            g::SKETCH_BASE_RED,
            g::SKETCH_HATCH,
            g::SKETCH_BUSH_DOTS,
            g::SKETCH_MARKS,
        ];
        let bytes: usize = sketch.iter().map(|s| s.len()).sum();
        assert!(bytes < 12_000, "sketch path data is {bytes} bytes");
        for (x, y) in g::TOWERS.iter().chain(g::INHIBITORS.iter()) {
            assert!((0.0..=100.0).contains(x) && (0.0..=100.0).contains(y));
        }
    }

    #[test]
    fn survey_glyphs_scale_with_the_zoom_stop() {
        let (fill2, stroke2) = survey_glyphs(RiftZoom::X2);
        let (fill4, _) = survey_glyphs(RiftZoom::X4);
        // A tower dot, two nexus centre dots and a dot per buff camp.
        let dots = g::TOWERS.len() + 2 + g::BUFF_CAMPS.len();
        assert_eq!(fill2.matches('M').count(), dots);
        assert_eq!(fill4.matches('M').count(), dots);
        assert_ne!(fill2, fill4);
        assert!(stroke2.contains('Z'));
    }
}
