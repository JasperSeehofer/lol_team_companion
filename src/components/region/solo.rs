//! Solo-specific region-branching primitives: RankBadge, LPProgress.
//! Region branching via AnyView pattern per ornaments.rs CompanionSigil reference.
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

/// SVG shield rank badge. `tier`: "iron" | "bronze" | "silver" | "gold" | "platinum" |
/// "emerald" | "diamond" | "master" | "grandmaster" | "challenger".
/// `division`: "I" | "II" | "III" | "IV" (not used for master+).
/// `large`: renders 64×80 instead of 32×40.
/// Region-neutral — tier color via `[data-theme]` token swap on accent.
#[component]
pub fn RankBadge(
    tier: String,
    #[prop(optional, into)] division: Option<String>,
    #[prop(optional, default = false)] large: bool,
) -> impl IntoView {
    let (w, h) = if large { (64u32, 80u32) } else { (32u32, 40u32) };
    let label_size = if large { "text-[11px]" } else { "text-[8px]" };
    let div_label = division.unwrap_or_default();

    view! {
        <div class="flex flex-col items-center gap-0.5">
            <svg
                width=w
                height=h
                viewBox="0 0 32 40"
                style="stroke: var(--color-accent); fill: none;"
            >
                <path
                    d="M3 6 L16 2 L29 6 V20 C29 30 22 36 16 39 C10 36 3 30 3 20 Z"
                    stroke="currentColor"
                    stroke-width="1.2"
                    fill="color-mix(in oklab, var(--color-accent) 10%, transparent)"
                />
                <text
                    x="16" y="24"
                    text-anchor="middle"
                    dominant-baseline="middle"
                    font-size="8"
                    fill="var(--color-accent)"
                    font-family="var(--font-imperial, Cinzel, serif)"
                    letter-spacing="0.06em"
                >
                    {tier.to_uppercase().chars().take(2).collect::<String>()}
                </text>
            </svg>
            {if large {
                view! {
                    <div class=format!(
                        "font-imperial uppercase tracking-[0.14em] text-accent text-center {}",
                        label_size
                    )>
                        {format!("{} {}", tier, div_label).trim().to_string()}
                    </div>
                }.into_any()
            } else {
                view! { <span class=format!("font-mono text-muted text-center {}", label_size)>{div_label}</span> }.into_any()
            }}
        </div>
    }
}

/// LP progress bar. Region-branching:
/// - Demacia: gold gradient fill, rounded-full bar, gilt border, Cormorant Garamond label
/// - Pandemonium: flat accent fill, square bar, bracket-style border, mono label
///
/// Uses AnyView branch pattern (both arms: `.into_any()`).
#[component]
pub fn LPProgress(
    region: String,
    lp: u32,
    #[prop(optional, default = 100)] max: u32,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let pct = ((lp as f64 / max as f64) * 100.0).clamp(0.0, 100.0);
    let width_style = format!("width: {:.1}%;", pct);

    move || if is_pandemonium {
        view! {
            <div class="flex flex-col gap-1">
                <div
                    class="border-l-2 border-r-2 border-accent"
                    style="height: 8px; position: relative; background: color-mix(in oklab, var(--color-accent) 10%, transparent);"
                >
                    <div
                        class="h-full bg-accent rounded-none"
                        style=width_style.clone()
                    ></div>
                </div>
                <span class="font-mono text-[10px] text-muted tabular-nums">
                    {format!("{} / {} LP", lp, max)}
                </span>
            </div>
        }.into_any()
    } else {
        view! {
            <div class="flex flex-col gap-1">
                <div
                    class="border border-outline/50 rounded-full p-0.5"
                    style="height: 12px; position: relative;"
                >
                    <div
                        class="h-full rounded-full"
                        style=format!(
                            "{}background: linear-gradient(90deg, var(--gold-1, var(--color-accent)) 0%, var(--gold-2, var(--color-accent)) 50%, var(--gold-3, var(--color-accent)) 100%);",
                            width_style.clone()
                        )
                    ></div>
                </div>
                <span class="font-display text-[10px] text-muted tabular-nums italic">
                    {format!("{} / {} LP", lp, max)}
                </span>
            </div>
        }.into_any()
    }
}
