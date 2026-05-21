//! Data visualization primitives: Stat, Sparkline, MoodMeter.
//! All region-neutral — font tokens carry the regional voice.
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

/// Stat number cluster. Eyebrow label + mono value + optional unit + optional delta.
/// Region-neutral: font tokens determine the regional voice.
#[component]
pub fn Stat(
    label: String,
    value: String,
    #[prop(optional, into)] unit: Option<String>,
    #[prop(optional)] delta: Option<f32>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1">
            <div class="font-mono text-[10px] uppercase tracking-[0.16em] text-muted">
                {label}
            </div>
            <div class="flex items-baseline gap-1.5 flex-wrap">
                <span class="font-mono text-[22px] font-semibold tabular-nums text-primary">
                    {value}
                </span>
                {unit.map(|u| view! {
                    <span class="font-mono text-[12px] text-muted ml-1">{u}</span>
                })}
                {delta.map(|d| {
                    let (sign, color_class) = if d >= 0.0 {
                        ("+", "text-accent")
                    } else {
                        ("", "text-danger")
                    };
                    let formatted = format!("{}{:.1}", sign, d);
                    view! {
                        <span class=format!("font-mono text-[12px] tabular-nums {}", color_class)>
                            {formatted}
                        </span>
                    }
                })}
            </div>
        </div>
    }
}

/// SVG sparkline with gradient fill area.
/// `data`: Vec of values; `width`: SVG width (default 120); `height`: SVG height (default 32).
/// Stroke uses `var(--color-accent)`. Fill gradient at 30% opacity.
#[component]
pub fn Sparkline(
    data: Vec<f64>,
    #[prop(optional, default = 120)] width: u32,
    #[prop(optional, default = 32)] height: u32,
) -> impl IntoView {
    if data.is_empty() {
        return view! { <svg width=width height=height></svg> }.into_any();
    }

    let pad = 2.0_f64;
    let w = width as f64;
    let h = height as f64;
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max_val - min_val).max(1.0);
    let n = data.len();

    let points: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, &v)| {
        let x = pad + (i as f64 / (n - 1).max(1) as f64) * (w - pad * 2.0);
        let y = h - pad - ((v - min_val) / range) * (h - pad * 2.0);
        (x, y)
    }).collect();

    let polyline_pts = points.iter()
        .map(|(x, y)| format!("{:.1},{:.1}", x, y))
        .collect::<Vec<_>>()
        .join(" ");

    // Closed polygon for fill: down to bottom-right, across bottom, up to first point
    let last = points.last().unwrap();
    let first = points.first().unwrap();
    let polygon_pts = format!(
        "{} {:.1},{:.1} {:.1},{:.1}",
        polyline_pts,
        last.0, h - pad,
        first.0, h - pad
    );

    view! {
        <svg
            width=width
            height=height
            viewBox=format!("0 0 {} {}", width, height)
            style="overflow: visible;"
        >
            <defs>
                <linearGradient id="sparkline-grad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" style="stop-color: var(--color-accent); stop-opacity: 0.3;" />
                    <stop offset="100%" style="stop-color: var(--color-accent); stop-opacity: 0.0;" />
                </linearGradient>
            </defs>
            <polygon
                points=polygon_pts
                fill="url(#sparkline-grad)"
            />
            <polyline
                points=polyline_pts
                fill="none"
                stroke="var(--color-accent)"
                stroke-width="1.2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }.into_any()
}

/// 5-segment mood bar. `value` clamped to [0.0, 1.0].
/// Filled segments use `bg-accent`; unfilled use `bg-elevated`.
/// Each segment ~24px wide × 8px tall with 2px gap.
#[component]
pub fn MoodMeter(
    value: f64,
) -> impl IntoView {
    let clamped = value.clamp(0.0, 1.0);
    let filled = (clamped * 5.0).round() as usize;

    view! {
        <div class="flex gap-0.5" role="meter" aria-valuenow=format!("{:.0}", clamped * 100.0) aria-valuemin="0" aria-valuemax="100">
            {(0..5usize).map(|i| {
                let is_filled = i < filled;
                let bg = if is_filled { "bg-accent" } else { "bg-elevated" };
                view! {
                    <div class=format!("rounded-sm {}", bg) style="width: 24px; height: 8px; flex-shrink: 0;"></div>
                }
            }).collect_view()}
        </div>
    }
}
