//! Canvas primitives (docs/DESIGN.md section 5): LegendBlock, CanvasFrame
//! with ScaleBar, and Annotation. Reserved for the Sand Table map and the
//! Draft Graph (M4); nothing outside a canvas uses a frame or a note.

use leptos::prelude::*;

use super::draft::DraftSide;

/// Ink of a legend swatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwatchInk {
    #[default]
    Primary,
    Muted,
    SideBlue,
    SideRed,
    Accent,
}

impl SwatchInk {
    fn stroke(self) -> &'static str {
        match self {
            SwatchInk::Primary => "stroke-primary",
            SwatchInk::Muted => "stroke-secondary",
            SwatchInk::SideBlue => "stroke-side-blue",
            SwatchInk::SideRed => "stroke-side-red",
            SwatchInk::Accent => "stroke-accent",
        }
    }

    fn fill(self) -> &'static str {
        match self {
            SwatchInk::Primary => "fill-primary",
            SwatchInk::Muted => "fill-secondary",
            SwatchInk::SideBlue => "fill-side-blue",
            SwatchInk::SideRed => "fill-side-red",
            SwatchInk::Accent => "fill-accent",
        }
    }
}

/// Line style: solid for exact or strong, dashed for estimated or
/// counters, dotted for weak.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Line weight (encodes strength).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineWeight {
    Thin,
    #[default]
    Medium,
    Thick,
}

/// What a legend entry draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swatch {
    /// A line in a style and weight, optionally with an arrowhead.
    Line {
        style: LineStyle,
        weight: LineWeight,
        ink: SwatchInk,
        arrow: bool,
    },
    /// Thin over thick: "line width = strength".
    Widths(SwatchInk),
    /// An area: solid edge (known) or soft dashed edge (approximate).
    Area { soft: bool, ink: SwatchInk },
    /// An exact event: a solid ring anchor.
    Anchor(SwatchInk),
    /// The side tokens: blue circle and red square.
    Sides,
    /// One side's token.
    Side(DraftSide),
    /// Nested contour rings (elevation, fit).
    Contours,
}

/// One row of a [`LegendBlock`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegendEntry {
    pub swatch: Swatch,
    pub label: String,
}

impl LegendEntry {
    pub fn new(swatch: Swatch, label: impl Into<String>) -> Self {
        Self {
            swatch,
            label: label.into(),
        }
    }
}

fn swatch_view(s: Swatch) -> AnyView {
    match s {
        Swatch::Line {
            style,
            weight,
            ink,
            arrow,
        } => {
            let w = match weight {
                LineWeight::Thin => "1.5",
                LineWeight::Medium => "2.5",
                LineWeight::Thick => "4",
            };
            let dash = match style {
                LineStyle::Solid => None,
                LineStyle::Dashed => Some("7 5"),
                LineStyle::Dotted => Some("1.5 5"),
            };
            let end = if arrow { "32" } else { "38" };
            view! {
                <svg viewBox="0 0 40 16" class="h-4 w-10 overflow-visible" aria-hidden="true">
                    <line x1="2" y1="8" x2=end y2="8" class=ink.stroke() stroke-width=w stroke-dasharray=dash stroke-linecap="round" />
                    {arrow.then(|| view! { <path d="M31 3 L39 8 L31 13 Z" class=ink.fill() /> })}
                </svg>
            }
            .into_any()
        }
        Swatch::Widths(ink) => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <line x1="2" y1="4" x2="38" y2="4" class=ink.stroke() stroke-width="1.5" />
                <line x1="2" y1="11.5" x2="38" y2="11.5" class=ink.stroke() stroke-width="4" />
            </svg>
        }
        .into_any(),
        Swatch::Area { soft, ink } => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <ellipse cx="20" cy="8" rx="17" ry="6.5" class=format!("fill-none {}", ink.stroke()) stroke-width=if soft { "1.2" } else { "2" } stroke-dasharray=soft.then_some("3 4") />
            </svg>
        }
        .into_any(),
        Swatch::Anchor(ink) => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <circle cx="20" cy="8" r="5" class=format!("fill-canvas {}", ink.stroke()) stroke-width="2.5" />
            </svg>
        }
        .into_any(),
        Swatch::Sides => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <circle cx="10" cy="8" r="6" class="fill-side-blue-deep stroke-side-blue" stroke-width="2" />
                <rect x="24" y="2" width="12" height="12" rx="1.5" class="fill-side-red-deep stroke-side-red" stroke-width="2" />
            </svg>
        }
        .into_any(),
        Swatch::Side(DraftSide::Blue) => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <circle cx="20" cy="8" r="6" class="fill-side-blue-deep stroke-side-blue" stroke-width="2" />
            </svg>
        }
        .into_any(),
        Swatch::Side(DraftSide::Red) => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10" aria-hidden="true">
                <rect x="14" y="2" width="12" height="12" rx="1.5" class="fill-side-red-deep stroke-side-red" stroke-width="2" />
            </svg>
        }
        .into_any(),
        Swatch::Contours => view! {
            <svg viewBox="0 0 40 16" class="h-4 w-10 fill-none stroke-secondary" aria-hidden="true">
                <ellipse cx="20" cy="8" rx="6" ry="4" stroke-width="1.2" />
                <ellipse cx="21" cy="8" rx="12" ry="6.5" stroke-width="1.2" />
                <ellipse cx="22" cy="8" rx="17.5" ry="7.5" stroke-width="1.2" />
            </svg>
        }
        .into_any(),
    }
}

/// The legend that explains every encoding on a canvas: shape, line
/// style, width and softness, each with words.
///
/// A11y: a labelled list (`aria-labelledby` the visible title); each entry
/// is a list item whose text says what the swatch means, so the drawing
/// (decorative, `aria-hidden`) never carries meaning alone. Swatches use
/// shape and line style, not only hue, and meet 3:1 on the canvas.
#[component]
pub fn LegendBlock(
    /// Prefix for the title id; unique on the page.
    id: &'static str,
    #[prop(default = "Legend".to_string(), into)] title: String,
    entries: Vec<LegendEntry>,
    /// Lay entries out in columns (under a wide canvas).
    #[prop(optional)]
    columns: bool,
) -> impl IntoView {
    let title_id = format!("{id}-legend");
    let list_class = if columns {
        "grid grid-cols-1 gap-x-6 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3"
    } else {
        "grid gap-1.5"
    };
    let labelled = title_id.clone();
    view! {
        <div class="flex flex-col gap-2" data-legend=id>
            <p id=title_id class="eyebrow">{title}</p>
            <ul class=list_class aria-labelledby=labelled>
                {entries
                    .into_iter()
                    .map(|e| view! {
                        <li class="grid grid-cols-[2.5rem_minmax(0,1fr)] items-center gap-3 text-label text-secondary">
                            {swatch_view(e.swatch)}
                            <span>{e.label}</span>
                        </li>
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}

/// The coordinate grid of a [`CanvasFrame`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameGrid {
    /// Numbered edges from 0 to `max` every `step` (Sand Table: 0 to 1000).
    /// The vertical axis counts upwards, like map coordinates.
    Numeric { max: u32, step: u32 },
    /// Grid references: letters across (A to H), numbers down (1 to 5),
    /// so a coach can say "Rakan, D4" (Draft Graph).
    Reference { cols: u8, rows: u8 },
}

/// Tick positions and labels along one edge, as (percent, label, major).
pub fn frame_ticks(grid: FrameGrid, horizontal: bool) -> Vec<(f32, String, bool)> {
    match grid {
        FrameGrid::Numeric { max, step } => {
            let step = step.max(1);
            (0..=max / step)
                .map(|i| {
                    let v = i * step;
                    let pct = v as f32 / max.max(1) as f32 * 100.0;
                    let pos = if horizontal { pct } else { 100.0 - pct };
                    (pos, v.to_string(), true)
                })
                .collect()
        }
        FrameGrid::Reference { cols, rows } => {
            let n = if horizontal { cols } else { rows }.max(1);
            let mut out = Vec::new();
            for i in 0..n {
                let label = if horizontal {
                    char::from(b'A' + (i % 26)).to_string()
                } else {
                    (i + 1).to_string()
                };
                out.push(((i as f32 + 0.5) / n as f32 * 100.0, label, false));
            }
            out
        }
    }
}

fn edge_lines(grid: FrameGrid, horizontal: bool) -> Vec<f32> {
    match grid {
        FrameGrid::Numeric { .. } => frame_ticks(grid, horizontal)
            .into_iter()
            .map(|(p, _, _)| p)
            .collect(),
        FrameGrid::Reference { cols, rows } => {
            let n = if horizontal { cols } else { rows }.max(1);
            (0..=n).map(|i| i as f32 / n as f32 * 100.0).collect()
        }
    }
}

/// A coordinate frame around a canvas: tick-marked, numbered edges, an
/// optional faint grid inside, a caption and a scale bar slot. Reserved
/// for the Draft Graph and the Sand Table; it has no behaviour of its own.
///
/// A11y: a `<figure>` named by its visible `<figcaption>`. The edge
/// numbers are decorative (`aria-hidden`); the canvas inside must be one
/// tab stop with a text twin (the ranked list for the graph, readouts and
/// moments for the map), which the caption can point to. The grid texture
/// is about 1.1:1 and never sits behind running text.
#[component]
pub fn CanvasFrame(
    /// What the canvas shows: "Draft Graph: support candidates for blue".
    #[prop(into)]
    caption: String,
    grid: FrameGrid,
    /// Draw the grid lines inside the frame (faint texture).
    #[prop(optional)]
    texture: bool,
    /// Scale bar (Sand Table only), bottom right.
    #[prop(optional, into)]
    scale: Option<ViewFn>,
    /// Aspect ratio class for the canvas area, for example "aspect-square".
    #[prop(default = "aspect-[16/10]")]
    aspect: &'static str,
    children: Children,
) -> impl IntoView {
    let top = frame_ticks(grid, true);
    let left = frame_ticks(grid, false);
    let vlines = edge_lines(grid, true);
    let hlines = edge_lines(grid, false);
    let is_ref = matches!(grid, FrameGrid::Reference { .. });
    let area = format!("relative overflow-hidden border border-outline bg-canvas {aspect}");
    let ticks_top = top
        .into_iter()
        .map(|(pos, label, _)| {
            // The last label sits left of its tick so it never leaves the frame.
            let shift = if pos > 99.9 { "-translate-x-full items-end" } else { "-translate-x-1/2 items-center" };
            view! {
                <span class=format!("absolute bottom-0 flex flex-col gap-0.5 {shift}") style=format!("left: {pos:.3}%;")>
                    <span class="figures text-caption leading-none text-muted">{label}</span>
                    {(!is_ref).then(|| view! { <span class="h-1.5 w-px bg-muted"></span> })}
                </span>
            }
        })
        .collect_view();
    let ticks_left = left
        .into_iter()
        .map(|(pos, label, _)| {
            view! {
                <span class="absolute right-0 flex -translate-y-1/2 items-center gap-1" style=format!("top: {pos:.3}%;")>
                    <span class="figures text-caption leading-none text-muted">{label}</span>
                    {(!is_ref).then(|| view! { <span class="h-px w-1.5 bg-muted"></span> })}
                </span>
            }
        })
        .collect_view();
    let ref_ticks_top = is_ref.then(|| {
        vlines
            .iter()
            .map(|p| view! { <span class="absolute bottom-0 h-1.5 w-px bg-muted" style=format!("left: {p:.3}%;")></span> })
            .collect_view()
    });
    let ref_ticks_left = is_ref.then(|| {
        hlines
            .iter()
            .map(|p| view! { <span class="absolute right-0 h-px w-1.5 bg-muted" style=format!("top: {p:.3}%;")></span> })
            .collect_view()
    });
    let texture_view = texture.then(|| {
        view! {
            <svg class="pointer-events-none absolute inset-0 size-full" preserveAspectRatio="none" viewBox="0 0 100 100" aria-hidden="true">
                {vlines.iter().map(|p| view! {
                    <line x1=format!("{p:.3}") y1="0" x2=format!("{p:.3}") y2="100" class="stroke-terrain-texture" stroke-width="1" vector-effect="non-scaling-stroke" />
                }).collect_view()}
                {hlines.iter().map(|p| view! {
                    <line x1="0" y1=format!("{p:.3}") x2="100" y2=format!("{p:.3}") class="stroke-terrain-texture" stroke-width="1" vector-effect="non-scaling-stroke" />
                }).collect_view()}
            </svg>
        }
    });
    // Ticks hang off the canvas box itself, so they always line up with it.
    view! {
        <figure class="flex min-w-0 flex-col pl-10 pt-6" data-canvas-frame="true">
            <div class="relative">
                <div class="absolute inset-x-0 bottom-full h-6" aria-hidden="true">{ticks_top}{ref_ticks_top}</div>
                <div class="absolute inset-y-0 right-full w-10 pr-1" aria-hidden="true">
                    <div class="relative size-full">{ticks_left}{ref_ticks_left}</div>
                </div>
                <div class=area>
                    {texture_view}
                    <div class="relative size-full">{children()}</div>
                </div>
            </div>
            <div class="flex flex-wrap items-start justify-between gap-x-6 gap-y-2 pt-2">
                <figcaption class="text-caption text-muted">{caption}</figcaption>
                {scale.map(|s| s.run())}
            </div>
        </figure>
    }
}

/// A scale bar: alternating filled and hollow segments with a distance.
/// At most once per page, and only where distance means something.
///
/// A11y: `role="img"` named "Scale: {label}"; segments are decorative.
#[component]
pub fn ScaleBar(
    /// The distance the whole bar spans: "200 units".
    #[prop(into)]
    label: String,
    #[prop(default = 4)] segments: u8,
) -> impl IntoView {
    let name = format!("Scale: {label}");
    view! {
        <span class="inline-flex items-center gap-2" role="img" aria-label=name>
            <span class="flex h-1.5 w-24 border border-secondary" aria-hidden="true">
                {(0..segments.max(1)).map(|i| {
                    let cls = if i % 2 == 0 { "h-full flex-1 bg-secondary" } else { "h-full flex-1" };
                    view! { <span class=cls></span> }
                }).collect_view()}
            </span>
            <span class="figures text-caption text-muted" aria-hidden="true">{label}</span>
        </span>
    }
}

/// Where an [`Annotation`]'s note sits relative to its anchor point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotePlacement {
    /// Up and to the right (the default).
    #[default]
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl NotePlacement {
    fn leader(self) -> &'static str {
        match self {
            NotePlacement::NorthEast => "absolute bottom-0 left-0",
            NotePlacement::NorthWest => "absolute bottom-0 right-0 -scale-x-100",
            NotePlacement::SouthEast => "absolute left-0 top-0 -scale-y-100",
            NotePlacement::SouthWest => "absolute right-0 top-0 -scale-100",
        }
    }

    fn note(self) -> &'static str {
        match self {
            NotePlacement::NorthEast => "absolute bottom-10 left-14 translate-y-1/2",
            NotePlacement::NorthWest => "absolute bottom-10 right-14 translate-y-1/2",
            NotePlacement::SouthEast => "absolute left-14 top-10 -translate-y-1/2",
            NotePlacement::SouthWest => "absolute right-14 top-10 -translate-y-1/2",
        }
    }
}

/// A surveyor's note on a canvas: the coach's words in serif italic on a
/// leader line to the point they are about. Canvases only; the serif
/// means "a person wrote this here".
///
/// A11y: `role="note"` with real text in reading order; `title` (for
/// example "17:00 Pick-off") and `at` (a grid reference such as "D4") are
/// written out so the note makes sense without the drawing. The anchor dot
/// and leader line are decorative.
#[component]
pub fn Annotation(
    /// Anchor position inside the canvas, in percent from the left.
    x: f32,
    /// Anchor position inside the canvas, in percent from the top.
    y: f32,
    #[prop(optional)] placement: NotePlacement,
    #[prop(optional, into)] title: Option<String>,
    /// Grid reference of the anchor, read to screen readers ("D4").
    #[prop(optional, into)]
    at: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="absolute size-0" style=format!("left: {x:.2}%; top: {y:.2}%;") role="note">
            <span class="absolute -left-1 -top-1 size-2 rounded-full bg-secondary" aria-hidden="true"></span>
            <svg class=format!("pointer-events-none h-10 w-14 overflow-visible {}", placement.leader()) viewBox="0 0 56 40" aria-hidden="true">
                <polyline points="0,40 40,0 56,0" class="fill-none stroke-secondary" stroke-width="1.5" />
            </svg>
            <div class=format!("w-max max-w-[min(14rem,40vw)] border border-rule bg-ink-2 px-2.5 py-1.5 {}", placement.note())>
                {title.map(|t| view! { <p class="eyebrow mb-0.5">{t}</p> })}
                {at.map(|a| view! { <span class="sr-only">{format!("At {a}: ")}</span> })}
                <p class="note text-body leading-snug text-primary">{children()}</p>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_grid_letters_across_numbers_down() {
        let g = FrameGrid::Reference { cols: 8, rows: 5 };
        let top: Vec<String> = frame_ticks(g, true).into_iter().map(|t| t.1).collect();
        assert_eq!(top, ["A", "B", "C", "D", "E", "F", "G", "H"]);
        let left: Vec<String> = frame_ticks(g, false).into_iter().map(|t| t.1).collect();
        assert_eq!(left, ["1", "2", "3", "4", "5"]);
        assert!((frame_ticks(g, true)[0].0 - 6.25).abs() < 1e-4);
    }

    #[test]
    fn numeric_grid_counts_up_the_vertical_edge() {
        let g = FrameGrid::Numeric {
            max: 1000,
            step: 250,
        };
        let left = frame_ticks(g, false);
        assert_eq!(left.len(), 5);
        assert_eq!(left[0].1, "0");
        assert!((left[0].0 - 100.0).abs() < 1e-4);
        assert!((left[4].0 - 0.0).abs() < 1e-4);
        assert_eq!(
            edge_lines(FrameGrid::Reference { cols: 4, rows: 2 }, true).len(),
            5
        );
    }
}
