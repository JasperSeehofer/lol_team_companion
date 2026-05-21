//! Region-aware typography primitives.
//! Display, Imperial, H, Eyebrow, Mono, Glitch — type primitives.
//! Named `typography` because `type` is a Rust reserved keyword.
//!
//! All font families are CSS custom properties (`--font-display`, `--font-imperial`, etc.)
//! so region switching automatically swaps fonts via `[data-theme]` CSS token redefinitions.
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

/// Large display heading. `size` controls pixel size: "xl"=64, "lg"=44, "md"=30, "sm"=22.
/// Font family from `--font-display` (Cormorant Garamond / Cinzel per region CSS var swap).
/// Region-neutral: font swap is controlled by `[data-theme]` CSS variable.
#[component]
pub fn Display(
    #[prop(optional, into)] size: Option<String>,
    children: Children,
) -> impl IntoView {
    let size_class = match size.as_deref().unwrap_or("lg") {
        "xl" => "text-[64px] leading-[1.05] font-medium",
        "md" => "text-[30px] leading-[1.05] font-medium",
        "sm" => "text-[22px] leading-[1.05] font-medium",
        _ => "text-[44px] leading-[1.05] font-medium", // lg default
    };
    view! {
        <div class=format!("font-display text-primary {}", size_class)>
            {children()}
        </div>
    }
}

/// Imperial label — Cinzel/Trajan uppercase with wide tracking.
/// Region-neutral: `--font-imperial` CSS var handles the font-family.
#[component]
pub fn Imperial(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    view! {
        <span class=format!(
            "font-imperial uppercase tracking-[0.14em] {}",
            class.unwrap_or_default()
        )>
            {children()}
        </span>
    }
}

/// Heading at semantic level 1–3. Maps to font sizes [30, 22, 17]px.
/// Uses `--font-display`. Region-neutral (font-family from CSS var).
#[component]
pub fn H(
    level: u8,
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let size_class = match level {
        1 => "text-[30px] leading-[1.2] font-semibold",
        3 => "text-[17px] leading-[1.2] font-semibold",
        _ => "text-[22px] leading-[1.2] font-semibold", // h2 default
    };
    view! {
        <div class=format!("font-display text-primary {} {}", size_class, class.unwrap_or_default())>
            {children()}
        </div>
    }
}

/// Small eyebrow label. JetBrains Mono, uppercase, wide tracking.
/// Region-neutral (same appearance both regions).
#[component]
pub fn Eyebrow(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    view! {
        <div class=format!(
            "font-mono text-[10px] uppercase tracking-[0.16em] text-muted {}",
            class.unwrap_or_default()
        )>
            {children()}
        </div>
    }
}

/// Monospace number/data label. Tabular nums for aligned data display.
/// Region-neutral.
#[component]
pub fn Mono(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    view! {
        <span class=format!("font-mono tabular-nums {}", class.unwrap_or_default())>
            {children()}
        </span>
    }
}

/// Region-branching glitch label.
/// - Demacia: plain Eyebrow treatment (font-mono uppercase tracking)
/// - Pandemonium: VT323 glitch font with text-shadow neon offset
///
/// Uses `ChildrenFn` (not `Children`) because both region arms call children() —
/// per 18-RESEARCH.md Pitfall 1 and leptos-patterns rule 19.
#[component]
pub fn Glitch(
    region: String,
    children: ChildrenFn,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    move || if is_pandemonium {
        view! {
            <span
                class="font-glitch tracking-[0.18em] text-[10px] uppercase"
                style="color: var(--color-accent); text-shadow: -2px -2px 0 var(--accent-2), 2px 2px 0 var(--t-accent);"
            >
                {children()}
            </span>
        }.into_any()
    } else {
        view! {
            <span class="font-mono uppercase text-[10px] tracking-[0.16em] text-muted">
                {children()}
            </span>
        }.into_any()
    }
}
