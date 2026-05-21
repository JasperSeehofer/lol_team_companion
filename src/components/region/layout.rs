//! Region-branching layout primitives: Card, SectionHead, Themed.
//! Card and SectionHead have structurally different Demacia vs Pandemonium compositions.
//! Themed is a region-neutral wrapper that sets [data-theme] attribute.
//!
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

use crate::components::region::ornaments::{GiltCorner, HeraldicDivider, RiotTape};
use crate::components::region::typography::{Eyebrow, Glitch, H};

/// Region-aware card container.
/// `variant`: "default" | "gilt" | "zine"
/// - Demacia default: rounded-xl, surface bg, subtle border
/// - Demacia gilt: inset accent shadow + GiltCorner decorations in each corner
/// - Pandemonium default: rounded-none, tighter border
/// - Pandemonium zine: bracket corners (4 absolutely positioned accent divs)
///
/// Uses `ChildrenFn` because both region arms call `children()`.
#[component]
pub fn Card(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let var = variant.unwrap_or_else(|| "default".to_string());
    let var_is_gilt = var == "gilt";
    let var_is_zine = var == "zine";

    move || if is_pandemonium {
        view! {
            <div class="bg-surface border border-outline/30 rounded-none p-6 relative">
                {if var_is_zine {
                    view! {
                        // 4 absolutely positioned bracket corners (Pandemonium zine aesthetic)
                        <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
                        <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
                        <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
                        <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}
                {children()}
            </div>
        }.into_any()
    } else {
        // Demacia
        let box_shadow = if var_is_gilt {
            "box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--color-accent) 15%, transparent);"
        } else {
            ""
        };
        view! {
            <div
                class="bg-surface border border-outline/50 rounded-xl p-6 relative"
                style=box_shadow
            >
                {if var_is_gilt {
                    view! {
                        <div class="absolute top-1 left-1"><GiltCorner corner="tl" size=20 /></div>
                        <div class="absolute top-1 right-1"><GiltCorner corner="tr" size=20 /></div>
                        <div class="absolute bottom-1 left-1"><GiltCorner corner="bl" size=20 /></div>
                        <div class="absolute bottom-1 right-1"><GiltCorner corner="br" size=20 /></div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}
                {children()}
            </div>
        }.into_any()
    }
}

/// Region-aware section header with optional eyebrow, title, and action slot.
/// - Demacia: Eyebrow + H level 2 + optional action, separated by HeraldicDivider below
/// - Pandemonium: Glitch `// eyebrow` + H level 2 (glitch treatment) + optional action, separated by RiotTape
///
/// Static branch: `region` is an SSR prop that doesn't change reactively in-page.
/// Builds optional views outside the view! macro to avoid FnOnce-in-view conflicts.
#[component]
pub fn SectionHead(
    region: String,
    title: String,
    #[prop(optional, into)] eyebrow: Option<String>,
    #[prop(optional)] action: Option<Children>,
) -> impl IntoView {
    let is_pandemonium = region.clone() == "pandemonium";

    if is_pandemonium {
        let pan_eyebrow = eyebrow.map(|e| format!("// {}", e));
        let pan_title = title.to_uppercase();
        // Pre-build optional views outside view! to avoid FnOnce closure issues
        let eyebrow_view = pan_eyebrow.map(|e| {
            let e_stored = StoredValue::new(e);
            view! { <Glitch region=region.clone()>{move || e_stored.get_value()}</Glitch> }
        });
        let action_view = action.map(|a| view! { <div>{a()}</div> });
        view! {
            <div class="flex flex-col gap-2">
                <div class="flex items-start justify-between gap-4">
                    <div class="flex flex-col gap-1">
                        {eyebrow_view}
                        <H level=2 class="font-glitch uppercase tracking-[0.12em]">{pan_title}</H>
                    </div>
                    {action_view}
                </div>
                <RiotTape width=240 label="SECTION" />
            </div>
        }.into_any()
    } else {
        // Demacia — pre-build optional views outside view!
        let eyebrow_view = eyebrow.map(|e| view! { <Eyebrow>{e}</Eyebrow> });
        let action_view = action.map(|a| view! { <div>{a()}</div> });
        view! {
            <div class="flex flex-col gap-2">
                <div class="flex items-start justify-between gap-4">
                    <div class="flex flex-col gap-1">
                        {eyebrow_view}
                        <H level=2>{title}</H>
                    </div>
                    {action_view}
                </div>
                <HeraldicDivider width=240 />
            </div>
        }.into_any()
    }
}

/// Sets `[data-theme]` attribute on a wrapper div for scoped theme application.
/// Region-neutral — pure attribute passthrough.
/// Useful for component-level snapshot tests and isolated regional compositions.
#[component]
pub fn Themed(
    region: String,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-theme=region>
            {children()}
        </div>
    }
}
