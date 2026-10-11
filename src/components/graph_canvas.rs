//! The Draft Graph canvas (M4 stage B, docs/DESIGN.md section 6.8): the
//! force-directed picture of a [`GraphResult`] inside a `CanvasFrame`
//! (grid A to H by 1 to 5) with its `LegendBlock`.
//!
//! The ranked side's locked picks are pinned in column A, the other side's
//! in column H (one row per role), the top candidates float between them
//! ([`graph_layout`](super::graph_layout) moves them), edges join a
//! candidate to the picks it relates to (synergy solid, counter dashed with
//! an arrow at the one that loses, weak or "works against" dotted; width =
//! strength; colour = the side of the pick), and fit is elevation: a spot
//! height ("▲ 84") and one contour ring per 10 fit from 50.
//!
//! It shares the panel's `focused` signal with the ranked list and the Why
//! panel: focusing a candidate anywhere lights its one-hop neighbourhood
//! here and fades the rest.
//!
//! Performance: the DOM is built once per result (and again only when a
//! resize changes how many candidates fit); each animation frame runs a few
//! simulation ticks and writes one position signal, which moves the nodes'
//! `transform` and the edges' `d` in place. The simulation stops when it
//! settles (no perpetual motion), runs synchronously under reduced motion
//! (the settled layout is drawn directly), and does nothing while the view
//! is unmounted. The measured numbers are exposed as `data-sim-*`
//! attributes on the canvas (DESIGN.md 6.8 has them).

use std::collections::HashMap;

use leptos::prelude::*;

use super::graph_layout::{self as gl, LayoutPlan, NodeKind};
use crate::components::ui::avatar::monogram;
use crate::components::ui::{
    CanvasFrame, DraftSide, FrameGrid, Keycap, LegendBlock, LegendEntry, LineStyle, LineWeight,
    Swatch, SwatchInk,
};
use crate::models::graph::{Candidate, GraphEdge, GraphResult, RelationKind, CANVAS_CANDIDATES};

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Where a counter edge's arrowhead points: at the champion that loses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    None,
    /// The candidate beats the locked pick.
    ToPick,
    /// The locked pick beats the candidate.
    ToCandidate,
}

/// Relations under this strength are drawn dotted ("weak").
pub const WEAK: f32 = 1.5;

/// How an edge is drawn: synergy solid, counter dashed with an arrow at the
/// one that loses, and dotted for a weak relation (under 1.5) or a synergy
/// that works against the candidate.
pub fn edge_style(e: &GraphEdge) -> (LineStyle, Arrow) {
    let weak = e.strength.abs() < WEAK;
    match e.kind {
        RelationKind::Synergy if e.strength < 0.0 || weak => (LineStyle::Dotted, Arrow::None),
        RelationKind::Synergy => (LineStyle::Solid, Arrow::None),
        RelationKind::Counter => (
            if weak {
                LineStyle::Dotted
            } else {
                LineStyle::Dashed
            },
            if e.strength >= 0.0 {
                Arrow::ToPick
            } else {
                Arrow::ToCandidate
            },
        ),
    }
}

/// Stroke width in px for an edge: 1.5 (weak) to 4.5 (strength 4).
pub fn edge_width(e: &GraphEdge) -> f32 {
    1.5 + 0.75 * e.strength.abs().min(4.0)
}

/// SVG dash pattern for a line style.
fn dash(style: LineStyle) -> Option<&'static str> {
    match style {
        LineStyle::Solid => None,
        LineStyle::Dashed => Some("8 6"),
        LineStyle::Dotted => Some("1.5 6"),
    }
}

/// How many candidates the canvas draws at a width (px): all of
/// [`CANVAS_CANDIDATES`] from 520 px, the top 12 on narrow screens so the
/// labels stay 12 px and readable.
pub fn candidates_for_width(width: f32) -> usize {
    if width < 520.0 {
        12
    } else {
        CANVAS_CANDIDATES
    }
}

/// Drawing sizes for a canvas width, in px (labels are always 12 px or
/// more; geometry scales with the canvas).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// Px per layout unit.
    pub scale: f32,
    pub node_r: f32,
    pub pin_r: f32,
    /// Radius of the first contour ring and the gap between rings.
    pub ring_first: f32,
    pub ring_step: f32,
    /// Pin names are written beside the pins when there is room.
    pub pin_names: bool,
}

pub fn geometry(width: f32) -> Geometry {
    let scale = (width / gl::WIDTH).max(0.1);
    let node_r = (15.0 * scale).clamp(11.0, 16.0);
    Geometry {
        scale,
        node_r,
        pin_r: (16.0 * scale).clamp(12.0, 17.0),
        ring_first: node_r + 5.0,
        ring_step: (6.0 * scale).clamp(3.5, 6.0),
        pin_names: scale >= 0.6,
    }
}

impl Geometry {
    /// Radius (px) of contour ring `k` (0-based).
    pub fn ring(&self, k: u8) -> f32 {
        self.ring_first + k as f32 * self.ring_step
    }

    /// Collision radius in layout units for a candidate of this fit: its
    /// outer ring (or the node) plus a margin.
    pub fn collision(&self, fit: u8) -> f32 {
        let rings = gl::ring_count(fit);
        let px = if rings == 0 {
            self.node_r + 4.0
        } else {
            self.ring(rings - 1) + 2.0
        };
        px / self.scale + 4.0
    }
}

/// Joins names: "Jinx", "Jinx and Azir", "Jinx, Azir and Vi".
pub fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// What the canvas says about the focused candidate (the polite status
/// under the canvas): "Rakan, fit 84, at D4 (1 of 24): synergy with Jinx
/// and Azir; beats Nautilus; loses to Syndra."
pub fn focus_sentence(
    name: &str,
    fit: u8,
    at: &str,
    rank: usize,
    total: usize,
    relations: &[(String, RelationKind, f32)],
) -> String {
    let pick = |f: &dyn Fn(RelationKind, f32) -> bool| -> Vec<String> {
        relations
            .iter()
            .filter(|(_, k, s)| f(*k, *s))
            .map(|(n, _, _)| n.clone())
            .collect()
    };
    let with = pick(&|k, s| k == RelationKind::Synergy && s > 0.0);
    let against = pick(&|k, s| k == RelationKind::Synergy && s < 0.0);
    let beats = pick(&|k, s| k == RelationKind::Counter && s > 0.0);
    let loses = pick(&|k, s| k == RelationKind::Counter && s < 0.0);
    let mut parts = Vec::new();
    if !with.is_empty() {
        parts.push(format!("synergy with {}", join_names(&with)));
    }
    if !beats.is_empty() {
        parts.push(format!("beats {}", join_names(&beats)));
    }
    if !loses.is_empty() {
        parts.push(format!("loses to {}", join_names(&loses)));
    }
    if !against.is_empty() {
        parts.push(format!("works against {}", join_names(&against)));
    }
    let head = format!("{name}, fit {fit}, at {at} ({rank} of {total})");
    if parts.is_empty() {
        format!("{head}: no relation to a locked pick yet.")
    } else {
        format!("{head}: {}.", parts.join("; "))
    }
}

/// How a node is lit relative to the focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    /// Nothing is focused on the canvas.
    Normal,
    Focused,
    /// One hop from the focused node.
    Lit,
    Faded,
}

/// Lighting for every node and edge: the focused node, its one-hop
/// neighbours and the edges between them light up; everything else fades.
/// `edges` are (free node, pinned node) pairs.
pub fn lighting(
    nodes: usize,
    edges: &[(usize, usize)],
    focus: Option<usize>,
) -> (Vec<Light>, Vec<bool>) {
    let Some(f) = focus.filter(|f| *f < nodes) else {
        return (vec![Light::Normal; nodes], vec![false; edges.len()]);
    };
    let mut light = vec![Light::Faded; nodes];
    let mut lit = vec![false; edges.len()];
    light[f] = Light::Focused;
    for (i, (a, b)) in edges.iter().enumerate() {
        if *a == f || *b == f {
            lit[i] = true;
            let other = if *a == f { *b } else { *a };
            if other != f {
                light[other] = Light::Lit;
            }
        }
    }
    (light, lit)
}

/// Shorten a segment by `ra` at the start and `rb` at the end (px).
fn trimmed(a: (f32, f32), b: (f32, f32), ra: f32, rb: f32) -> (f32, f32, f32, f32) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let d = (dx * dx + dy * dy).sqrt();
    if d <= ra + rb + 1.0 {
        return (a.0, a.1, b.0, b.1);
    }
    let (ux, uy) = (dx / d, dy / d);
    (a.0 + ux * ra, a.1 + uy * ra, b.0 - ux * rb, b.1 - uy * rb)
}

// ---------------------------------------------------------------------------
// Simulation statistics (shown as data attributes)
// ---------------------------------------------------------------------------

/// What the last run measured.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SimStats {
    pub settled: bool,
    /// The settled layout was drawn directly (reduced motion).
    pub instant: bool,
    pub ticks: u32,
    pub nodes: usize,
    /// Wall-clock time from the first tick to the frozen layout.
    pub settle_ms: f64,
    pub tick_max_ms: f64,
    pub tick_total_ms: f64,
    /// Longest gap between two animation frames.
    pub frame_max_ms: f64,
}

/// Ticks per animation frame: 150 ticks settle in 25 frames (about 0.4 s
/// at 60 Hz).
pub const TICKS_PER_FRAME: u32 = 6;

// ---------------------------------------------------------------------------
// The component
// ---------------------------------------------------------------------------

/// A node as the view draws it.
#[derive(Clone, Debug)]
struct DrawNode {
    idx: usize,
    /// Champion id.
    id: String,
    name: String,
    mono: String,
    pinned: bool,
    left: bool,
    side: DraftSide,
    fit: u8,
    rings: u8,
}

/// The force-directed Draft Graph for the current pick.
///
/// A11y: the canvas is one tab stop (`role="group"`, described by its
/// status line and key legend) with a text twin, the ranked list next to it
/// (the `CanvasFrame` caption says so). Arrow keys move the focus between
/// candidates in rank order (Home/End to the ends), Enter picks the focused
/// candidate, Escape returns to the board; the status line names the
/// focused candidate, its fit, grid reference and relations, and is a
/// polite live region while the canvas has focus. Pointer: click focuses,
/// double-click picks, drag moves a candidate and the layout settles again;
/// clicking a locked pick lights the candidates related to it. Nothing is
/// shown only on hover; reduced motion draws the settled layout directly.
#[component]
pub fn DraftGraphCanvas(
    result: GraphResult,
    /// The champion id the list, the Why panel and the canvas share.
    focused: RwSignal<Option<String>>,
    on_pick: Callback<Candidate>,
    on_escape: Callback<()>,
    /// Last positions by champion id (kept by the panel across results).
    memory: StoredValue<HashMap<String, (f32, f32)>>,
    /// The slot in words ("Blue pick 3").
    #[prop(into)]
    slot_label: String,
) -> impl IntoView {
    let result = StoredValue::new(result);
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let size = RwSignal::new((gl::WIDTH, gl::HEIGHT));
    let plan: StoredValue<Option<LayoutPlan>> = StoredValue::new(None);
    let plan_ver = RwSignal::new(0u32);
    let pos: RwSignal<Vec<(f32, f32)>> = RwSignal::new(Vec::new());
    let stats = RwSignal::new(SimStats::default());
    let pin_focus: RwSignal<Option<usize>> = RwSignal::new(None);
    let has_focus = RwSignal::new(false);
    #[allow(unused_variables)]
    let running = StoredValue::new(false);
    #[allow(unused_variables)]
    let drag: StoredValue<Option<(usize, f64, f64, bool)>> = StoredValue::new(None);
    #[allow(unused_variables)]
    let last_click: StoredValue<Option<(usize, f64)>> = StoredValue::new(None);

    let geo = Memo::new(move |_| geometry(size.get().0));
    let shown = Memo::new(move |_| {
        let _ = plan_ver.get();
        plan.with_value(|p| p.as_ref().map(|p| p.sim.nodes.len() - p.pins).unwrap_or(0))
    });
    let pins = Memo::new(move |_| {
        let _ = plan_ver.get();
        plan.with_value(|p| p.as_ref().map(|p| p.pins).unwrap_or(0))
    });
    let edge_pairs = Memo::new(move |_| {
        let _ = plan_ver.get();
        plan.with_value(|p| {
            p.as_ref()
                .map(|p| {
                    p.edges
                        .iter()
                        .map(|e| (e.free, e.pinned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    });
    // The focused candidate's node index (when it is drawn).
    let focus_node = Memo::new(move |_| {
        let f = focused.get()?;
        let pins = pins.get();
        let n = shown.get();
        result.with_value(|g| {
            g.candidates
                .iter()
                .take(n)
                .position(|c| c.champion == f)
                .map(|r| pins + r)
        })
    });
    let light = Memo::new(move |_| {
        let total = pins.get() + shown.get();
        let focus = pin_focus.get().or(focus_node.get());
        lighting(total, &edge_pairs.get(), focus)
    });
    let node_light =
        move |i: usize| light.with(|(l, _)| l.get(i).copied().unwrap_or(Light::Normal));
    let edge_lit = move |i: usize| light.with(|(_, e)| e.get(i).copied().unwrap_or(false));
    let px = move |i: usize| -> (f32, f32) {
        let (w, h) = size.get();
        pos.with(|p| p.get(i).copied())
            .map(|(x, y)| (x * w / gl::WIDTH, y * h / gl::HEIGHT))
            .unwrap_or((-100.0, -100.0))
    };

    // Build (or rebuild) the layout for the measured width and start it.
    let build = move |width: f32| {
        let g = geometry(width);
        let max = candidates_for_width(width);
        let previous = memory.get_value();
        let p = result.with_value(|r| {
            gl::plan(
                r,
                max,
                |fit| g.collision(fit),
                g.pin_r / g.scale + 6.0,
                &previous,
            )
        });
        pos.set(p.sim.positions());
        plan.set_value(Some(p));
        plan_ver.update(|v| *v += 1);
    };

    #[cfg(feature = "hydrate")]
    {
        let start = move || start_run(plan, pos, stats, running, memory);
        Effect::new(move |_| {
            let Some(el) = node_ref.get() else { return };
            let rect = el.get_bounding_client_rect();
            let w = rect.width() as f32;
            let h = rect.height() as f32;
            if w > 0.0 && h > 0.0 {
                size.set((w, h));
            }
            build(if w > 0.0 { w } else { gl::WIDTH });
            start();
        });
        let handle = window_event_listener(leptos::ev::resize, move |_| {
            let Some(el) = node_ref.get_untracked() else {
                return;
            };
            let rect = el.get_bounding_client_rect();
            let (w, h) = (rect.width() as f32, rect.height() as f32);
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let before = size.get_untracked().0;
            size.set((w, h));
            if candidates_for_width(before) != candidates_for_width(w) {
                build(w);
                start();
            }
        });
        on_cleanup(move || handle.remove());
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (build, node_ref, memory);

    // Focus helpers.
    let candidate_at = move |rank: usize| -> Option<Candidate> {
        result.with_value(|g| g.candidates.get(rank).cloned())
    };
    let rank_of_focus = move || -> Option<usize> {
        let f = focused.get_untracked()?;
        let n = shown.get_untracked();
        result.with_value(|g| g.candidates.iter().take(n).position(|c| c.champion == f))
    };

    let on_key = move |ev: leptos::ev::KeyboardEvent| {
        let n = shown.get_untracked();
        if n == 0 {
            return;
        }
        let cur = rank_of_focus();
        let key = ev.key();
        let next = match key.as_str() {
            "ArrowDown" | "ArrowRight" => Some(cur.map_or(0, |c| (c + 1) % n)),
            "ArrowUp" | "ArrowLeft" => Some(cur.map_or(n - 1, |c| (c + n - 1) % n)),
            "Home" => Some(0),
            "End" => Some(n - 1),
            _ => None,
        };
        if let Some(r) = next {
            ev.prevent_default();
            pin_focus.set(None);
            if let Some(c) = candidate_at(r) {
                focused.set(Some(c.champion));
            }
            return;
        }
        match key.as_str() {
            "Enter" => {
                if let Some(c) = cur.and_then(candidate_at) {
                    ev.prevent_default();
                    on_pick.run(c);
                }
            }
            "Escape" => {
                ev.prevent_default();
                pin_focus.set(None);
                on_escape.run(());
            }
            _ => {}
        }
    };

    // Pointer: click focuses, a second click on the same candidate within
    // 450 ms picks, a drag moves it and the layout settles again.
    #[allow(unused_variables)]
    let click_node = move |i: usize, now: f64| {
        let pins_n = pins.get_untracked();
        if i < pins_n {
            pin_focus.update(|p| *p = if *p == Some(i) { None } else { Some(i) });
            return;
        }
        pin_focus.set(None);
        let Some(c) = candidate_at(i - pins_n) else {
            return;
        };
        let double = last_click
            .get_value()
            .is_some_and(|(j, t)| j == i && now - t < 450.0);
        if double {
            last_click.set_value(None);
            on_pick.run(c);
        } else {
            last_click.set_value(Some((i, now)));
            focused.set(Some(c.champion));
        }
    };
    #[allow(unused_variables)]
    let on_node_down = move |i: usize, ev: leptos::ev::PointerEvent| {
        if ev.button() != 0 {
            return;
        }
        drag.set_value(Some((i, ev.client_x() as f64, ev.client_y() as f64, false)));
    };
    #[allow(unused_variables)]
    let on_move = move |ev: leptos::ev::PointerEvent| {
        #[cfg(feature = "hydrate")]
        {
            let Some((i, sx, sy, moved)) = drag.get_value() else {
                return;
            };
            let (cx, cy) = (ev.client_x() as f64, ev.client_y() as f64);
            let now_moved = moved || (cx - sx).abs() + (cy - sy).abs() > 5.0;
            if !now_moved || i < pins.get_untracked() {
                return;
            }
            drag.set_value(Some((i, sx, sy, true)));
            let Some(el) = node_ref.get_untracked() else {
                return;
            };
            let rect = el.get_bounding_client_rect();
            if rect.width() <= 0.0 || rect.height() <= 0.0 {
                return;
            }
            let ux = ((cx - rect.left()) / rect.width()) as f32 * gl::WIDTH;
            let uy = ((cy - rect.top()) / rect.height()) as f32 * gl::HEIGHT;
            plan.update_value(|p| {
                if let Some(p) = p {
                    p.sim.fix(i, ux, uy);
                    p.sim.reheat(0.3);
                }
            });
            if reduced_motion() {
                pos.set(
                    plan.with_value(|p| p.as_ref().map(|p| p.sim.positions()).unwrap_or_default()),
                );
            } else {
                start_run(plan, pos, stats, running, memory);
            }
        }
    };
    #[allow(unused_variables)]
    let end_drag = move |click: bool| {
        #[cfg(feature = "hydrate")]
        {
            let Some((i, _, _, moved)) = drag.get_value() else {
                return;
            };
            drag.set_value(None);
            if moved {
                plan.update_value(|p| {
                    if let Some(p) = p {
                        p.sim.release(i);
                        p.sim.reheat(0.3);
                    }
                });
                start_run(plan, pos, stats, running, memory);
            } else if click {
                click_node(i, now_ms());
            }
        }
    };

    let label = format!(
        "Draft Graph canvas for {slot_label}. Arrow keys move between candidates in rank order, Enter picks, Escape returns to the board."
    );
    let caption = format!(
        "Draft Graph for {slot_label}: the top candidates by fit float between the ranked side's locked picks (column A) and the other side's (column H). The ranked list below is the same ranking in words."
    );

    let status = move || {
        if let Some(p) = pin_focus.get() {
            let (name, own, ours) = result.with_value(|g| {
                plan.with_value(|pl| {
                    let id = pl
                        .as_ref()
                        .and_then(|pl| pl.sim.nodes.get(p))
                        .map(|n| n.id.clone())
                        .unwrap_or_default();
                    let node = g.locked.iter().find(|l| l.champion == id);
                    (
                        node.map(|n| n.name.clone()).unwrap_or(id),
                        node.is_some_and(|n| n.own),
                        g.target.as_ref().is_none_or(|t| t.ours),
                    )
                })
            });
            let related = edge_pairs.with(|e| e.iter().filter(|(_, b)| *b == p).count());
            let at = pos
                .with_untracked(|v| v.get(p).map(|(x, y)| gl::grid_ref(*x, *y)))
                .unwrap_or_default();
            return format!(
                "{name} ({} pick, at {at}): {related} of the drawn candidates relate to it. Click it again or press an arrow key to return to the candidates.",
                if own == ours { "your" } else { "their" }
            );
        }
        let Some(i) = focus_node.get() else {
            // Focused in the list but below the candidates drawn here.
            let below = focused.get().and_then(|f| {
                result.with_value(|g| {
                    g.candidates
                        .iter()
                        .position(|c| c.champion == f)
                        .map(|r| format!("{} (rank {}) is below the {} candidates drawn here; the list below has it.", g.candidates[r].name, r + 1, shown.get()))
                })
            });
            return below.unwrap_or_else(|| "Focus a candidate in the list or here to see how it relates to the locked picks.".to_string());
        };
        let pins_n = pins.get();
        let rank = i - pins_n;
        let total = shown.get();
        // The grid reference once the layout has settled (not every frame).
        let read = |v: &Vec<(f32, f32)>| v.get(i).map(|(x, y)| gl::grid_ref(*x, *y));
        let at = if stats.get().settled {
            pos.with(read)
        } else {
            pos.with_untracked(read)
        }
        .unwrap_or_default();
        let (name, fit, rels) = result.with_value(|g| {
            let c = g.candidates.get(rank);
            let name = c.map(|c| c.name.clone()).unwrap_or_default();
            let fit = c.map(|c| c.fit).unwrap_or_default();
            let id = c.map(|c| c.champion.clone()).unwrap_or_default();
            let rels: Vec<(String, RelationKind, f32)> = g
                .edges
                .iter()
                .filter(|e| e.from == id)
                .map(|e| {
                    let n = g
                        .locked
                        .iter()
                        .find(|l| l.champion == e.to)
                        .map(|l| l.name.clone())
                        .unwrap_or_else(|| e.to.clone());
                    (n, e.kind, e.strength)
                })
                .collect();
            (name, fit, rels)
        });
        focus_sentence(&name, fit, &at, rank + 1, total, &rels)
    };

    let legend = vec![
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Solid,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: false,
            },
            "Synergy: solid",
        ),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Dashed,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: true,
            },
            "Counter: dashed, the arrow points at the one that loses",
        ),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Dotted,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: false,
            },
            "Weak (under 1.5) or works against: dotted",
        ),
        LegendEntry::new(Swatch::Widths(SwatchInk::Muted), "Line width: strength"),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Solid,
                weight: LineWeight::Medium,
                ink: SwatchInk::SideBlue,
                arrow: false,
            },
            "Line colour: the side of the locked pick",
        ),
        LegendEntry::new(
            Swatch::Contours,
            "Fit as elevation: ▲ spot height, a ring at 50, 60, 70 \u{2026} up to the fit",
        ),
        LegendEntry::new(
            Swatch::Sides,
            "Locked picks: blue circle, red square (ranked side in column A)",
        ),
        LegendEntry::new(
            Swatch::Anchor(SwatchInk::Accent),
            "Focused candidate; its one-hop neighbours stay lit",
        ),
    ];

    view! {
        <div class="flex min-w-0 flex-col gap-3" data-graph-canvas-wrap="">
            <CanvasFrame caption=caption grid=FrameGrid::Reference { cols: gl::COLS, rows: gl::ROWS } texture=true aspect="aspect-square sm:aspect-[16/10]">
                <div
                    node_ref=node_ref
                    class="absolute inset-0 select-none focus-visible:-outline-offset-4"
                    tabindex="0"
                    role="group"
                    aria-roledescription="graph canvas"
                    aria-label=label
                    aria-describedby="graph-canvas-status graph-canvas-keys"
                    data-graph-canvas=""
                    data-sim-state=move || if stats.get().settled { "settled" } else { "running" }
                    data-sim-mode=move || if stats.get().instant { "instant" } else { "animated" }
                    data-sim-ticks=move || stats.get().ticks
                    data-sim-nodes=move || stats.get().nodes
                    data-sim-settle-ms=move || format!("{:.1}", stats.get().settle_ms)
                    data-sim-tick-max-ms=move || format!("{:.3}", stats.get().tick_max_ms)
                    data-sim-tick-mean-ms=move || { let s = stats.get(); format!("{:.3}", s.tick_total_ms / s.ticks.max(1) as f64) }
                    data-sim-frame-max-ms=move || format!("{:.1}", stats.get().frame_max_ms)
                    data-candidates=move || shown.get()
                    on:keydown=on_key
                    on:focus=move |_| has_focus.set(true)
                    on:blur=move |_| has_focus.set(false)
                >
                    <svg
                        class="absolute inset-0 size-full"
                        viewBox=move || { let (w, h) = size.get(); format!("0 0 {w:.0} {h:.0}") }
                        aria-hidden="true"
                        on:pointermove=on_move
                        on:pointerup=move |_| end_drag(true)
                        on:pointerleave=move |_| end_drag(false)
                        on:pointercancel=move |_| end_drag(false)
                    >
                        <defs>
                            <marker id="dg-arrow-blue" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                                <path d="M0 0 L10 5 L0 10 Z" class="fill-side-blue" />
                            </marker>
                            <marker id="dg-arrow-red" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                                <path d="M0 0 L10 5 L0 10 Z" class="fill-side-red" />
                            </marker>
                        </defs>
                        <rect x="0" y="0" width="100%" height="100%" fill="transparent" on:click=move |_| pin_focus.set(None) />
                        {move || {
                            let _ = plan_ver.get();
                            let Some((nodes, edges, open)) = plan.with_value(|p| p.as_ref().map(|p| (draw_nodes(p, &result.get_value()), p.edges.clone(), p.open_roles.clone()))) else {
                                return ().into_any();
                            };
                            let contours = nodes.iter().filter(|n| !n.pinned && n.rings > 0).map(|n| {
                                let (i, rings) = (n.idx, n.rings);
                                view! {
                                    <g transform=move || { let (x, y) = px(i); format!("translate({x:.1} {y:.1})") }>
                                        {(0..rings).map(|k| {
                                            view! {
                                                <ellipse
                                                    cx=format!("{:.1}", k as f32 * 1.5)
                                                    cy=format!("{:.1}", k as f32 * 0.8)
                                                    rx=move || format!("{:.1}", geo.get().ring(k) * 1.12)
                                                    ry=move || format!("{:.1}", geo.get().ring(k) * 0.88)
                                                    class=move || match node_light(i) {
                                                        Light::Focused => "fill-none stroke-secondary",
                                                        Light::Faded => "fill-none stroke-rule",
                                                        _ => "fill-none stroke-muted",
                                                    }
                                                    stroke-width=move || if node_light(i) == Light::Focused { "1.25" } else { "1" }
                                                />
                                            }
                                        }).collect_view()}
                                    </g>
                                }
                            }).collect_view();
                            // Outside the focus only the strong relations stay, faint.
                            let base_edges = edges.iter().filter(|e| e.edge.strength.abs() >= WEAK).map(|e| {
                                let (style, _) = edge_style(&e.edge);
                                let (a, b) = (e.free, e.pinned);
                                view! {
                                    <path
                                        d=move || { let g = geo.get(); let (x1, y1, x2, y2) = trimmed(px(a), px(b), g.node_r, g.pin_r); format!("M{x1:.1} {y1:.1}L{x2:.1} {y2:.1}") }
                                        class="fill-none stroke-rule"
                                        stroke-width="1"
                                        stroke-dasharray=dash(style)
                                        stroke-linecap="round"
                                    />
                                }
                            }).collect_view();
                            let lit_edges = edges.iter().cloned().enumerate().map(|(k, e)| {
                                let (style, arrow) = edge_style(&e.edge);
                                let width = format!("{:.2}", edge_width(&e.edge));
                                let (a, b) = (e.free, e.pinned);
                                let blue = nodes.iter().find(|n| n.idx == b).is_some_and(|n| n.side == DraftSide::Blue);
                                let (stroke, marker) = if blue {
                                    ("fill-none stroke-side-blue", "url(#dg-arrow-blue)")
                                } else {
                                    ("fill-none stroke-side-red", "url(#dg-arrow-red)")
                                };
                                view! {
                                    <path
                                        d=move || { let g = geo.get(); let (x1, y1, x2, y2) = trimmed(px(a), px(b), g.node_r + 2.0, g.pin_r + 3.0); format!("M{x1:.1} {y1:.1}L{x2:.1} {y2:.1}") }
                                        class=stroke
                                        stroke-width=width
                                        stroke-dasharray=dash(style)
                                        stroke-linecap="round"
                                        marker-end=(arrow == Arrow::ToPick).then_some(marker)
                                        marker-start=(arrow == Arrow::ToCandidate).then_some(marker)
                                        visibility=move || if edge_lit(k) { "visible" } else { "hidden" }
                                        data-edge=format!("{}-{}", e.edge.from, e.edge.to)
                                    />
                                }
                            }).collect_view();
                            let ghosts = open.into_iter().map(|o| {
                                let (ux, uy) = (o.x, o.y);
                                let word = format!("{} open", o.role.label());
                                view! {
                                    <g transform=move || { let (w, h) = size.get(); format!("translate({:.1} {:.1})", ux * w / gl::WIDTH, uy * h / gl::HEIGHT) }>
                                        <circle r=move || format!("{:.1}", geo.get().pin_r) class="fill-none stroke-muted" stroke-width="1.25" stroke-dasharray="3 4" />
                                        <text class="figures text-caption fill-muted" text-anchor="middle" dominant-baseline="central">"?"</text>
                                        {move || geo.get().pin_names.then(|| view! {
                                            <text
                                                x=move || format!("{:.1}", geo.get().pin_r + 6.0)
                                                class="text-caption fill-muted stroke-canvas"
                                                stroke-width="4" stroke-linejoin="round" paint-order="stroke"
                                                dominant-baseline="central"
                                            >{word.clone()}</text>
                                        })}
                                    </g>
                                }
                            }).collect_view();
                            let node_views = nodes.iter().cloned().map(|n| draw_node(n, px, geo, node_light, on_node_down)).collect_view();
                            let labels = nodes.into_iter().map(|n| draw_label(n, px, geo, node_light)).collect_view();
                            view! {
                                <g data-layer="contours" class="pointer-events-none">{contours}</g>
                                <g data-layer="edges" class="pointer-events-none">{base_edges}</g>
                                <g data-layer="open-roles" class="pointer-events-none">{ghosts}</g>
                                <g data-layer="lit-edges" class="pointer-events-none">{lit_edges}</g>
                                <g data-layer="nodes">{node_views}</g>
                                <g data-layer="labels">{labels}</g>
                                <g data-layer="focus">
                                    {move || {
                                        let i = focus_node.get()?;
                                        if pin_focus.get().is_some() {
                                            return None;
                                        }
                                        let rank = i.checked_sub(pins.get_untracked())?;
                                        let (name, fit) = result.with_value(|g| g.candidates.get(rank).map(|c| (c.name.clone(), c.fit)))?;
                                        Some(draw_focus_label(i, name, fit, px, geo, move || size.get().1))
                                    }}
                                </g>
                            }.into_any()
                        }}
                    </svg>
                </div>
            </CanvasFrame>
            <p
                id="graph-canvas-status"
                class="text-label text-secondary"
                aria-live=move || if has_focus.get() { "polite" } else { "off" }
                data-graph-canvas-status=""
            >{status}</p>
            <p id="graph-canvas-keys" class="flex flex-wrap items-center gap-x-3 gap-y-1 text-caption text-muted">
                <span class="inline-flex items-center gap-1"><Keycap keys="\u{2190}" /><Keycap keys="\u{2192}" />" or "<Keycap keys="\u{2191}" /><Keycap keys="\u{2193}" />" next by rank"</span>
                <span class="inline-flex items-center gap-1"><Keycap keys="Enter" />" or double-click: pick"</span>
                <span class="inline-flex items-center gap-1"><Keycap keys="Esc" />" back to the board"</span>
                <span>"Drag a candidate: the layout settles again"</span>
            </p>
            <LegendBlock id="graph-canvas" title="Graph legend" entries=legend columns=true />
        </div>
    }
}

fn draw_nodes(p: &LayoutPlan, g: &GraphResult) -> Vec<DrawNode> {
    p.sim
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, n)| {
            let (pinned, left) = match n.kind {
                NodeKind::Pinned { left } => (true, left),
                NodeKind::Free => (false, false),
            };
            let (name, side, fit) = if pinned {
                let l = g.locked.iter().find(|l| l.champion == n.id);
                (
                    l.map(|l| l.name.clone()).unwrap_or_else(|| n.id.clone()),
                    l.and_then(|l| DraftSide::parse(&l.side))
                        .unwrap_or(DraftSide::Blue),
                    0,
                )
            } else {
                let c = g.candidates.iter().find(|c| c.champion == n.id);
                (
                    c.map(|c| c.name.clone()).unwrap_or_else(|| n.id.clone()),
                    DraftSide::Blue,
                    c.map(|c| c.fit).unwrap_or(0),
                )
            };
            DrawNode {
                idx,
                id: n.id.clone(),
                mono: monogram(&name),
                name,
                pinned,
                left,
                side,
                fit,
                rings: if pinned { 0 } else { gl::ring_count(fit) },
            }
        })
        .collect()
}

fn draw_node(
    n: DrawNode,
    px: impl Fn(usize) -> (f32, f32) + Copy + Send + Sync + 'static,
    geo: Memo<Geometry>,
    node_light: impl Fn(usize) -> Light + Copy + Send + Sync + 'static,
    on_down: impl Fn(usize, leptos::ev::PointerEvent) + Copy + Send + Sync + 'static,
) -> AnyView {
    let i = n.idx;
    let title = if n.pinned {
        format!("{} (locked)", n.name)
    } else {
        format!("{}, fit {}", n.name, n.fit)
    };
    let text_class = move || {
        if node_light(i) == Light::Faded {
            "figures text-caption font-semibold fill-muted pointer-events-none"
        } else {
            "figures text-caption font-semibold fill-primary pointer-events-none"
        }
    };
    let mono = n.mono.clone();
    let shape = if n.pinned {
        match n.side {
            DraftSide::Blue => view! {
                <circle
                    r=move || format!("{:.1}", geo.get().pin_r)
                    class=move || if node_light(i) == Light::Faded { "fill-canvas stroke-side-blue" } else { "fill-side-blue-deep stroke-side-blue" }
                    stroke-width=move || if node_light(i) == Light::Faded { "1.25" } else { "2" }
                />
            }
            .into_any(),
            DraftSide::Red => view! {
                <rect
                    x=move || format!("{:.1}", -geo.get().pin_r * 0.9)
                    y=move || format!("{:.1}", -geo.get().pin_r * 0.9)
                    width=move || format!("{:.1}", geo.get().pin_r * 1.8)
                    height=move || format!("{:.1}", geo.get().pin_r * 1.8)
                    rx="3"
                    class=move || if node_light(i) == Light::Faded { "fill-canvas stroke-side-red" } else { "fill-side-red-deep stroke-side-red" }
                    stroke-width=move || if node_light(i) == Light::Faded { "1.25" } else { "2" }
                />
            }
            .into_any(),
        }
    } else {
        view! {
            <circle
                r=move || format!("{:.1}", geo.get().node_r)
                class=move || match node_light(i) {
                    Light::Focused => "fill-ink-3 stroke-accent",
                    Light::Lit => "fill-ink-2 stroke-secondary",
                    Light::Normal => "fill-ink-2 stroke-outline",
                    Light::Faded => "fill-ink-1 stroke-rule",
                }
                stroke-width=move || if node_light(i) == Light::Focused { "2.5" } else { "1.5" }
            />
        }
        .into_any()
    };
    let kind = if n.pinned { "pin" } else { "candidate" };
    let pinned = n.pinned;
    let name = n.name.clone();
    let champion = n.id.clone();
    view! {
        <g
            transform=move || { let (x, y) = px(i); format!("translate({x:.1} {y:.1})") }
            class="cursor-pointer"
            data-node=kind
            data-name=name
            data-champion=champion
            data-fit=(!pinned).then_some(n.fit)
            data-light=move || match node_light(i) {
                Light::Focused => "focused",
                Light::Lit => "lit",
                Light::Normal => "normal",
                Light::Faded => "faded",
            }
            on:pointerdown=move |ev| on_down(i, ev)
        >
            <title>{title}</title>
            {move || (node_light(i) == Light::Focused).then(|| {
                let r = if pinned { geo.get().pin_r } else { geo.get().node_r };
                view! {
                    <circle r=format!("{:.1}", r + 5.0) class="fill-none stroke-accent" stroke-width="1.5" stroke-dasharray="3 4" />
                }
            })}
            {shape}
            <text class=text_class text-anchor="middle" dominant-baseline="central">{mono}</text>
        </g>
    }
    .into_any()
}

fn draw_label(
    n: DrawNode,
    px: impl Fn(usize) -> (f32, f32) + Copy + Send + Sync + 'static,
    geo: Memo<Geometry>,
    node_light: impl Fn(usize) -> Light + Copy + Send + Sync + 'static,
) -> AnyView {
    let i = n.idx;
    const HALO: &str = "4";
    if n.pinned {
        let left = n.left;
        let name = n.name.clone();
        return view! {
            <g transform=move || { let (x, y) = px(i); format!("translate({x:.1} {y:.1})") } class="pointer-events-none">
                {move || geo.get().pin_names.then(|| {
                    let r = geo.get().pin_r;
                    view! {
                        <text
                            x=if left { format!("{:.1}", r + 6.0) } else { format!("{:.1}", -r - 6.0) }
                            text-anchor=if left { "start" } else { "end" }
                            dominant-baseline="central"
                            class=move || if node_light(i) == Light::Faded { "text-caption fill-muted stroke-canvas" } else { "text-caption fill-secondary stroke-canvas" }
                            stroke-width=HALO stroke-linejoin="round" paint-order="stroke"
                        >{name.clone()}</text>
                    }
                })}
            </g>
        }
        .into_any();
    }
    let fit = n.fit;
    view! {
        <g transform=move || { let (x, y) = px(i); format!("translate({x:.1} {y:.1})") } class="pointer-events-none">
            // The focused candidate's label is drawn on top (the focus layer).
            {move || (node_light(i) != Light::Focused).then(|| view! {
                <text
                    x=move || format!("{:.1}", geo.get().node_r * 0.6)
                    // Above the node, or below it at the top edge of the canvas.
                    y=move || {
                        let r = geo.get().node_r;
                        if px(i).1 < r + 18.0 { format!("{:.1}", r + 13.0) } else { format!("{:.1}", -r - 4.0) }
                    }
                    class=move || if node_light(i) == Light::Faded { "figures text-caption fill-muted stroke-canvas" } else { "figures text-caption fill-secondary stroke-canvas" }
                    stroke-width=HALO stroke-linejoin="round" paint-order="stroke"
                >{format!("\u{25b2}{fit}")}</text>
            })}
        </g>
    }
    .into_any()
}

/// The focused candidate's name and spot height, above everything else.
fn draw_focus_label(
    i: usize,
    name: String,
    fit: u8,
    px: impl Fn(usize) -> (f32, f32) + Copy + Send + Sync + 'static,
    geo: Memo<Geometry>,
    height: impl Fn() -> f32 + Copy + Send + Sync + 'static,
) -> AnyView {
    // Below the node, or above it near the bottom edge.
    let below = move || px(i).1 + geo.get().node_r + 40.0 < height();
    view! {
        <g transform=move || { let (x, y) = px(i); format!("translate({x:.1} {y:.1})") } class="pointer-events-none" data-focus-label="">
            <text
                y=move || { let r = geo.get().node_r; format!("{:.1}", if below() { r + 16.0 } else { -r - 26.0 }) }
                text-anchor="middle"
                class="text-label font-semibold fill-primary stroke-canvas"
                stroke-width="4" stroke-linejoin="round" paint-order="stroke"
            >{name}</text>
            <text
                y=move || { let r = geo.get().node_r; format!("{:.1}", if below() { r + 32.0 } else { -r - 10.0 }) }
                text-anchor="middle"
                class="figures text-caption font-semibold fill-accent stroke-canvas"
                stroke-width="4" stroke-linejoin="round" paint-order="stroke"
            >{format!("\u{25b2} {fit}")}</text>
        </g>
    }
    .into_any()
}

// ---------------------------------------------------------------------------
// The frame loop (browser only)
// ---------------------------------------------------------------------------

#[cfg(feature = "hydrate")]
fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

#[cfg(feature = "hydrate")]
fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

/// Run the simulation to its frozen layout: all at once under reduced
/// motion, otherwise a few ticks per animation frame. A run already in
/// progress just continues (it picks up a reheat).
#[cfg(feature = "hydrate")]
fn start_run(
    plan: StoredValue<Option<LayoutPlan>>,
    pos: RwSignal<Vec<(f32, f32)>>,
    stats: RwSignal<SimStats>,
    running: StoredValue<bool>,
    memory: StoredValue<HashMap<String, (f32, f32)>>,
) {
    let nodes = plan.with_value(|p| p.as_ref().map(|p| p.sim.nodes.len()).unwrap_or(0));
    if reduced_motion() {
        let started = now_ms();
        let mut s = SimStats {
            instant: true,
            nodes,
            ..SimStats::default()
        };
        plan.update_value(|p| {
            if let Some(p) = p {
                while !p.sim.is_settled() {
                    let t = now_ms();
                    p.sim.tick();
                    let dt = now_ms() - t;
                    s.ticks += 1;
                    s.tick_total_ms += dt;
                    s.tick_max_ms = s.tick_max_ms.max(dt);
                }
            }
        });
        s.settled = true;
        s.settle_ms = now_ms() - started;
        finish(plan, pos, memory);
        stats.set(s);
        return;
    }
    if running.get_value() {
        return;
    }
    running.set_value(true);
    stats.set(SimStats {
        nodes,
        ..SimStats::default()
    });
    let started = now_ms();
    frame(plan, pos, stats, running, memory, started, started);
}

#[cfg(feature = "hydrate")]
fn frame(
    plan: StoredValue<Option<LayoutPlan>>,
    pos: RwSignal<Vec<(f32, f32)>>,
    stats: RwSignal<SimStats>,
    running: StoredValue<bool>,
    memory: StoredValue<HashMap<String, (f32, f32)>>,
    started: f64,
    last: f64,
) {
    request_animation_frame(move || {
        // The view is gone (another result, or the graph was switched off).
        let Some(Some(done)) = plan.try_update_value(|p| {
            let p = p.as_mut()?;
            let mut s = stats.try_get_untracked().unwrap_or_default();
            let now = now_ms();
            if s.ticks > 0 {
                s.frame_max_ms = s.frame_max_ms.max(now - last);
            }
            for _ in 0..TICKS_PER_FRAME {
                if p.sim.is_settled() {
                    break;
                }
                let t = now_ms();
                p.sim.tick();
                let dt = now_ms() - t;
                s.ticks += 1;
                s.tick_total_ms += dt;
                s.tick_max_ms = s.tick_max_ms.max(dt);
            }
            let done = p.sim.is_settled();
            if done {
                s.settled = true;
                s.settle_ms = now_ms() - started;
            }
            let _ = stats.try_set(s);
            Some(done)
        }) else {
            let _ = running.try_set_value(false);
            return;
        };
        let positions =
            plan.with_value(|p| p.as_ref().map(|p| p.sim.positions()).unwrap_or_default());
        let _ = pos.try_set(positions);
        if done {
            finish(plan, pos, memory);
            let _ = running.try_set_value(false);
        } else {
            frame(plan, pos, stats, running, memory, started, now_ms());
        }
    });
}

/// Draw the frozen layout and remember it for the next result.
#[cfg(feature = "hydrate")]
fn finish(
    plan: StoredValue<Option<LayoutPlan>>,
    pos: RwSignal<Vec<(f32, f32)>>,
    memory: StoredValue<HashMap<String, (f32, f32)>>,
) {
    let Some(Some((positions, ids))) = plan.try_with_value(|p| {
        p.as_ref().map(|p| {
            (
                p.sim.positions(),
                p.sim
                    .nodes
                    .iter()
                    .filter(|n| n.kind == NodeKind::Free && !n.fixed)
                    .map(|n| (n.id.clone(), (n.x, n.y)))
                    .collect::<Vec<_>>(),
            )
        })
    }) else {
        return;
    };
    let _ = pos.try_set(positions);
    let _ = memory.try_update_value(|m| m.extend(ids));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::ReasonSource;

    fn e(kind: RelationKind, s: f32) -> GraphEdge {
        GraphEdge {
            from: "a".into(),
            to: "b".into(),
            kind,
            strength: s,
            sources: vec![ReasonSource::Rule],
        }
    }

    #[test]
    fn edges_follow_the_line_language() {
        assert_eq!(
            edge_style(&e(RelationKind::Synergy, 2.0)),
            (LineStyle::Solid, Arrow::None)
        );
        assert_eq!(
            edge_style(&e(RelationKind::Synergy, 1.0)),
            (LineStyle::Dotted, Arrow::None)
        );
        assert_eq!(
            edge_style(&e(RelationKind::Synergy, -2.0)),
            (LineStyle::Dotted, Arrow::None)
        );
        assert_eq!(
            edge_style(&e(RelationKind::Counter, 2.0)),
            (LineStyle::Dashed, Arrow::ToPick)
        );
        assert_eq!(
            edge_style(&e(RelationKind::Counter, -3.0)),
            (LineStyle::Dashed, Arrow::ToCandidate)
        );
        assert_eq!(
            edge_style(&e(RelationKind::Counter, -1.0)),
            (LineStyle::Dotted, Arrow::ToCandidate)
        );
        assert!(
            edge_width(&e(RelationKind::Synergy, 3.0)) > edge_width(&e(RelationKind::Synergy, 1.0))
        );
        assert!((edge_width(&e(RelationKind::Counter, -9.0)) - 4.5).abs() < 1e-4);
    }

    #[test]
    fn narrow_canvases_draw_fewer_candidates_and_keep_sizes_readable() {
        assert_eq!(candidates_for_width(640.0), CANVAS_CANDIDATES);
        assert_eq!(candidates_for_width(330.0), 12);
        let narrow = geometry(330.0);
        assert!(narrow.node_r >= 11.0, "a 12 px monogram fits");
        assert!(!narrow.pin_names);
        let wide = geometry(700.0);
        assert!(wide.pin_names);
        assert!(wide.collision(84) > wide.collision(55));
        assert!(wide.ring(3) > wide.ring(0));
    }

    #[test]
    fn the_status_line_says_where_and_why() {
        let rels = vec![
            ("Jinx".to_string(), RelationKind::Synergy, 3.0),
            ("Azir".to_string(), RelationKind::Synergy, 2.0),
            ("Nautilus".to_string(), RelationKind::Counter, 2.0),
            ("Syndra".to_string(), RelationKind::Counter, -2.0),
        ];
        assert_eq!(
            focus_sentence("Rakan", 84, "D4", 1, 24, &rels),
            "Rakan, fit 84, at D4 (1 of 24): synergy with Jinx and Azir; beats Nautilus; loses to Syndra."
        );
        assert_eq!(
            focus_sentence("Braum", 61, "F2", 5, 24, &[]),
            "Braum, fit 61, at F2 (5 of 24): no relation to a locked pick yet."
        );
        assert_eq!(
            join_names(&["A".into(), "B".into(), "C".into()]),
            "A, B and C"
        );
    }

    #[test]
    fn focus_lights_one_hop_and_fades_the_rest() {
        // Nodes 0, 1 pinned; 2, 3, 4 candidates. Edges: 2-0, 3-0, 3-1.
        let edges = [(2, 0), (3, 0), (3, 1)];
        let (l, lit) = lighting(5, &edges, Some(3));
        assert_eq!(
            l,
            vec![
                Light::Lit,
                Light::Lit,
                Light::Faded,
                Light::Focused,
                Light::Faded
            ]
        );
        assert_eq!(lit, vec![false, true, true]);
        // A pin's neighbourhood: the candidates related to it.
        let (l, lit) = lighting(5, &edges, Some(0));
        assert_eq!(
            l,
            vec![
                Light::Focused,
                Light::Faded,
                Light::Lit,
                Light::Lit,
                Light::Faded
            ]
        );
        assert_eq!(lit, vec![true, true, false]);
        let (l, _) = lighting(5, &edges, None);
        assert!(l.iter().all(|x| *x == Light::Normal));
    }

    #[test]
    fn edges_are_trimmed_to_the_node_edges() {
        let (x1, y1, x2, y2) = trimmed((0.0, 0.0), (100.0, 0.0), 10.0, 20.0);
        assert_eq!((x1, y1, x2, y2), (10.0, 0.0, 80.0, 0.0));
        // Too short to trim: drawn as is.
        assert_eq!(
            trimmed((0.0, 0.0), (5.0, 0.0), 10.0, 20.0),
            (0.0, 0.0, 5.0, 0.0)
        );
    }
}
