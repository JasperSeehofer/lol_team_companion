//! Per-region loading skeleton + empty-state components per phase 18 SPEC REQ-2.
//! Source: .local-design-source/.../skeletons-{demacia,pandemonium}.jsx
//!
//! Both components branch on `region: String` using the canonical AnyView pattern.
//! Animations come from input.css @theme @keyframes: dem-shimmer, pan-flicker, pan-scan, pan-cursor.

use leptos::prelude::*;

/// Loading skeleton for a page-level Suspense fallback.
/// `variant` selects the layout shape: "draft" | "solo" | "team".
/// `region` controls the visual grammar: "demacia" (parchment shimmer + serif italic caption)
/// or "pandemonium" (xerox flicker + monospace caption with blinking cursor).
#[component]
pub fn PageLoading(
    region: String,
    #[prop(into)] variant: String,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let v = variant.clone();
    let caption_dem = match v.as_str() {
        "draft" => "Awaiting word from the field\u{2026}",
        "solo"  => "Reading the stars\u{2026}",
        "team"  => "Mustering the company\u{2026}",
        _       => "Loading\u{2026}",
    };
    let caption_pan = match v.as_str() {
        "draft" => "// LOADING_DRAFT",
        "solo"  => "// LOADING_SOLO_PROFILE",
        "team"  => "// LOADING_TEAM_BRIEF",
        _       => "// LOADING_",
    };
    let variant_for_layout = variant.clone();
    move || if is_pandemonium {
        // Pandemonium: dark scanline background, flicker animation, mono caption with blinking cursor
        view! {
            <div class="relative w-full min-h-[60vh] bg-base bg-scanline animate-pan-flicker p-6">
                {render_pan_skeleton_layout(&variant_for_layout)}
                <div class="absolute bottom-6 left-6 font-mono text-[12px] tracking-[0.14em] text-accent">
                    {caption_pan}
                    <span class="inline-block ml-1 animate-pan-cursor">"_"</span>
                </div>
            </div>
        }.into_any()
    } else {
        // Demacia: parchment-tan shimmer, serif italic caption
        view! {
            <div class="relative w-full min-h-[60vh] bg-base bg-parchment-shimmer animate-dem-shimmer p-6">
                {render_dem_skeleton_layout(&variant_for_layout)}
                <div class="absolute bottom-6 left-6 font-display italic text-[14px] text-muted">
                    {caption_dem}
                </div>
            </div>
        }.into_any()
    }
}

fn render_dem_skeleton_layout(variant: &str) -> AnyView {
    // Per skeletons-demacia.jsx: parchment-toned rectangular blocks with gilt accent borders.
    // 3 variants:
    // - draft: 3 column layout (left blue side, mid mini-map, right red side); 10 horizontal tile rows
    // - solo: large rank-badge placeholder on left, 4 stat tiles on right
    // - team: 5-row roster placeholder + 3-column data surface (mood, captain note, bans)
    match variant {
        "draft" => view! {
            <div class="grid grid-cols-3 gap-4 h-full">
                <div class="bg-elevated border border-accent/30 rounded">
                    {(0..5).map(|_| view! { <div class="h-12 bg-surface border-b border-outline/30"></div> }).collect_view()}
                </div>
                <div class="bg-elevated border border-accent/30 rounded"></div>
                <div class="bg-elevated border border-accent/30 rounded">
                    {(0..5).map(|_| view! { <div class="h-12 bg-surface border-b border-outline/30"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        "solo" => view! {
            <div class="flex gap-6">
                <div class="w-48 h-64 bg-elevated border border-accent/30 rounded"></div>
                <div class="grid grid-cols-2 gap-4 flex-1">
                    {(0..4).map(|_| view! { <div class="h-24 bg-elevated border border-accent/30 rounded"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        "team" => view! {
            <div class="space-y-4">
                {(0..5).map(|_| view! { <div class="h-16 bg-elevated border border-accent/30 rounded"></div> }).collect_view()}
                <div class="grid grid-cols-3 gap-4">
                    {(0..3).map(|_| view! { <div class="h-32 bg-elevated border border-accent/30 rounded"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        _ => view! { <div class="h-full bg-elevated"></div> }.into_any(),
    }
}

fn render_pan_skeleton_layout(variant: &str) -> AnyView {
    // Per skeletons-pandemonium.jsx: rounded-none zine-style blocks with bracket corners,
    // scanline overlay, accent-magenta tinted edges.
    match variant {
        "draft" => view! {
            <div class="grid grid-cols-3 gap-2 h-full">
                <div class="bg-elevated rounded-none border-l-2 border-accent">
                    {(0..5).map(|_| view! { <div class="h-12 bg-surface rounded-none mb-1"></div> }).collect_view()}
                </div>
                <div class="bg-elevated rounded-none"></div>
                <div class="bg-elevated rounded-none border-r-2 border-accent">
                    {(0..5).map(|_| view! { <div class="h-12 bg-surface rounded-none mb-1"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        "solo" => view! {
            <div class="flex gap-2">
                <div class="w-48 h-64 bg-elevated rounded-none border-l-2 border-accent"></div>
                <div class="grid grid-cols-2 gap-2 flex-1">
                    {(0..4).map(|_| view! { <div class="h-24 bg-elevated rounded-none"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        "team" => view! {
            <div class="space-y-2">
                {(0..5).map(|_| view! { <div class="h-16 bg-elevated rounded-none border-l-2 border-accent"></div> }).collect_view()}
                <div class="grid grid-cols-3 gap-2">
                    {(0..3).map(|_| view! { <div class="h-32 bg-elevated rounded-none"></div> }).collect_view()}
                </div>
            </div>
        }.into_any(),
        _ => view! { <div class="h-full bg-elevated"></div> }.into_any(),
    }
}

/// Empty-state component for when data is loaded but no records exist.
/// `kind` selects the copy + icon: "draft" | "matches" | "team" | "pool" | "scout".
#[component]
pub fn PageEmpty(
    region: String,
    #[prop(into)] kind: String,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let k = kind.clone();
    let copy_dem = match k.as_str() {
        "draft"   => "No drafts recorded. Begin your first campaign.",
        "matches" => "No matches in the ledger. Sync your history.",
        "team"    => "Your company has not assembled. Join or form a team.",
        "pool"    => "Your champion pool awaits its first entries.",
        "scout"   => "No opponent intelligence on file.",
        _         => "Nothing here yet.",
    };
    let copy_pan = match k.as_str() {
        "draft"   => "// NO_DRAFTS_FOUND \u{2014} start a session",
        "matches" => "// MATCH_HISTORY_EMPTY \u{2014} run sync",
        "team"    => "// NO_TEAM \u{2014} join_or_create()",
        "pool"    => "// POOL_EMPTY \u{2014} add champions",
        "scout"   => "// NO_INTEL \u{2014} begin scouting",
        _         => "// EMPTY",
    };
    move || if is_pandemonium {
        view! {
            <div class="w-full p-12 bg-surface border-l-2 border-accent rounded-none text-center">
                <div class="font-mono text-[14px] tracking-[0.14em] text-accent">{copy_pan}</div>
            </div>
        }.into_any()
    } else {
        view! {
            <div class="w-full p-12 bg-surface border border-outline/50 rounded-xl text-center">
                <div class="font-display italic text-[16px] text-muted">{copy_dem}</div>
            </div>
        }.into_any()
    }
}
