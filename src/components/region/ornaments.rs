//! Region-aware ornament primitives.
//! Per phase 18 D-02, this consolidates all ornaments under region/.
//! CompanionSigil is the legacy reference for the move || if is_pandemonium { … }.into_any() pattern.
//! New region primitives accept `region: String` as a prop (NOT use_context) per SPEC Constraints.
//!
//! All stroke/fill colors use CSS custom properties (`var(--color-accent)`,
//! `var(--gold-2)`, `var(--accent-3)`) so theme switching automatically
//! retints them. Per CLAUDE.md "no raw hex in components" rule, these are
//! the canonical ornament SVGs and ARE allowed to reference theme vars.

use leptos::prelude::*;

use crate::app::InitialTheme;

/// Heraldic horizontal divider with central fleur. Demacia variant —
/// callers should swap to `RiotTape` for Pandemonium contexts.
#[component]
pub fn HeraldicDivider(
    #[prop(optional, default = 200)] width: u32,
) -> impl IntoView {
    view! {
        <svg
            width=width
            height="16"
            viewBox="0 0 200 16"
            class="block"
            style="stroke: var(--color-accent)"
        >
            <line x1="2" x2="86" y1="8" y2="8" stroke="currentColor" stroke-width="0.6" />
            <line x1="114" x2="198" y1="8" y2="8" stroke="currentColor" stroke-width="0.6" />
            <g fill="none" stroke="currentColor" stroke-width="0.8">
                <path d="M100 2 L100 14" />
                <path d="M93 8 Q100 0 107 8" />
                <path d="M93 8 Q100 16 107 8" />
                <circle cx="100" cy="8" r="1.6" fill="currentColor" />
            </g>
            <circle cx="86" cy="8" r="1.2" fill="currentColor" />
            <circle cx="114" cy="8" r="1.2" fill="currentColor" />
        </svg>
    }
}

/// Gilt corner ornament for Demacia `variant="gilt"` cards. Use the
/// `corner` prop to position; CSS transform on the parent rotates copies
/// for the other 3 corners (see foundations.jsx `transform: scaleY(-1)`
/// pattern at lines 99-101).
#[component]
pub fn GiltCorner(
    #[prop(optional, default = 28)] size: u32,
    #[prop(optional, default = "tl")] corner: &'static str,
) -> impl IntoView {
    let transform = match corner {
        "tr" => "scaleX(-1)",
        "bl" => "scaleY(-1)",
        "br" => "scale(-1, -1)",
        _ => "none", // tl
    };

    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 28 28"
            class="block"
            style=format!("display: block; transform: {}; stroke: var(--color-accent)", transform)
            fill="none"
        >
            <path d="M2 2h10M2 2v10" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            <path d="M2 2h6M2 2v6" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
            <circle cx="2" cy="2" r="1.4" fill="currentColor" />
        </svg>
    }
}

/// Small fleur-de-lis SVG. Used in headings and ban-slot wax seals.
#[component]
pub fn FleurDeLis(
    #[prop(optional, default = 18)] size: u32,
) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 18 18"
            class="block"
            fill="none"
            style="stroke: var(--color-accent)"
        >
            <g stroke="currentColor" stroke-width="0.8">
                <path d="M9 1 L9 17" />
                <path d="M2 9 Q9 -1 16 9" />
                <path d="M2 9 Q9 19 16 9" />
                <circle cx="9" cy="9" r="1.6" fill="currentColor" />
            </g>
        </svg>
    }
}

/// Pandemonium-only diagonal yellow tape divider with riot-tape label.
/// Decorative variant of HeraldicDivider; never used in Demacia surfaces.
#[component]
pub fn RiotTape(
    #[prop(optional, default = 200)] width: u32,
    #[prop(optional, default = "RIOT")] label: &'static str,
) -> impl IntoView {
    let style = format!(
        "width: {}px; height: 22px; position: relative; \
         background: var(--accent-3); transform: rotate(-1.2deg); \
         box-shadow: 0 1px 0 rgba(0,0,0,0.4), 2px 2px 0 var(--color-accent), -2px -1px 0 var(--accent-2); \
         overflow: hidden;",
        width
    );

    // Note: the inner label color uses the Pandemonium accent-contrast token
    // (#06070b), referenced via var() so the CI raw-hex sweep stays clean.
    view! {
        <div style=style>
            <div
                class="absolute inset-0 flex items-center justify-center font-mono text-[11px] font-bold uppercase tracking-[0.18em]"
                style="color: var(--t-accent-contrast); mix-blend-mode: multiply"
            >
                {format!("//// {} //// {} //// {} ////", label, label, label)}
            </div>
        </div>
    }
}

/// Companion sigil + wordmark. Reads the `InitialTheme` context to
/// switch between Demacia (shield + Cinzel imperial uppercase) and
/// Pandemonium (VT323 glitch wordmark "COMPANION_").
///
/// NOTE: `CompanionSigil` is the ONLY primitive that uses `use_context::<InitialTheme>()`
/// internally (legacy pattern, preserved verbatim from old ornaments.rs).
/// All NEW region-branching primitives accept `region: String` as a prop instead.
#[component]
pub fn CompanionSigil() -> impl IntoView {
    let is_pandemonium = use_context::<InitialTheme>()
        .map(|t| t.0 == "pandemonium")
        .unwrap_or(false);

    if is_pandemonium {
        view! {
            <span
                class="font-glitch text-[18px] uppercase tracking-[0.18em]"
                style="color: var(--color-accent)"
            >
                "COMPANION_"
            </span>
        }
        .into_any()
    } else {
        view! {
            <div class="flex items-center gap-2.5">
                <svg
                    width="22"
                    height="26"
                    viewBox="0 0 22 26"
                    style="fill: var(--color-accent)"
                >
                    <path d="M2 4 L11 1 L20 4 V14 C20 19 16 23 11 25 C6 23 2 19 2 14 Z" />
                </svg>
                <div
                    class="font-imperial text-[14px] uppercase tracking-[0.18em] text-primary"
                >
                    "COMPANION"
                </div>
            </div>
        }
        .into_any()
    }
}

/// Crown SVG (foundations.jsx lines 105-115). Used for admin badges and
/// champion tier-S indicators.
#[component]
pub fn Crown(
    #[prop(optional, default = 44)] size: u32,
) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 44 44"
            class="block"
            style="fill: var(--color-accent)"
        >
            <path d="M6 30 L10 14 L16 22 L22 12 L28 22 L34 14 L38 30 Z" />
            <rect x="6" y="32" width="32" height="4" />
            <circle cx="10" cy="14" r="1.6" />
            <circle cx="22" cy="12" r="1.6" />
            <circle cx="34" cy="14" r="1.6" />
        </svg>
    }
}
