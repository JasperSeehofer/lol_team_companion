//! Shims for the solo dashboard: `RankBadge` and `LPProgress`.
//! The solo dashboard dissolves into the personal layer in M5.

use leptos::prelude::*;

/// Rank tier as a typographic mark: tier initials in a square, the full
/// tier and division beside it when `large`.
#[component]
pub fn RankBadge(
    tier: String,
    #[prop(optional, into)] division: Option<String>,
    #[prop(optional, default = false)] large: bool,
) -> impl IntoView {
    let div_label = division.unwrap_or_default();
    let initials: String = tier.to_uppercase().chars().take(2).collect();
    let box_class = if large {
        "figures inline-grid size-14 place-items-center rounded-ctl border border-outline bg-ink-2 text-title font-semibold text-primary"
    } else {
        "figures inline-grid size-8 place-items-center rounded-ctl border border-outline bg-ink-2 text-caption font-semibold text-primary"
    };
    let full = format!("{tier} {div_label}").trim().to_string();
    view! {
        <div class="flex items-center gap-3">
            <span class=box_class aria-hidden="true">{initials}</span>
            {if large {
                view! { <span class="font-display text-heading font-semibold capitalize text-primary">{full}</span> }.into_any()
            } else {
                view! { <span class="sr-only">{full}</span> }.into_any()
            }}
        </div>
    }
}

/// League points towards the next division, as a flat meter.
#[component]
pub fn LPProgress(lp: u32, #[prop(optional, default = 100)] max: u32) -> impl IntoView {
    let max = max.max(1);
    let pct = ((lp as f64 / max as f64) * 100.0).clamp(0.0, 100.0);
    let width_style = format!("width: {pct:.1}%;");
    view! {
        <div class="flex flex-col gap-1">
            <div
                class="h-2 border-b border-outline"
                role="meter"
                aria-valuemin="0"
                aria-valuemax=max
                aria-valuenow=lp
                aria-label="League points"
            >
                <div class="h-full bg-secondary" style=width_style></div>
            </div>
            <span class="figures text-caption text-muted">{format!("{lp} / {max} LP")}</span>
        </div>
    }
}
