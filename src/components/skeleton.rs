//! Page-level loading and empty states for the draft and solo pages.
//! Batch 2's `Skeleton` and `EmptyState` replace these in M3/M5.

use leptos::prelude::*;

/// Loading placeholder for a page-level Suspense fallback.
/// `variant` picks the rough layout: "draft" | "solo" | "team".
///
/// The blocks are decorative; a visually hidden status says "Loading".
#[component]
pub fn PageLoading(#[prop(into)] variant: String) -> impl IntoView {
    let block = "animate-pulse motion-reduce:animate-none bg-ink-1 border border-rule";
    let layout = match variant.as_str() {
        "draft" => view! {
            <div class="grid grid-cols-3 gap-4" aria-hidden="true">
                <div class=format!("{block} h-72")></div>
                <div class=format!("{block} h-72")></div>
                <div class=format!("{block} h-72")></div>
            </div>
        }
        .into_any(),
        "solo" => view! {
            <div class="flex gap-6" aria-hidden="true">
                <div class=format!("{block} h-48 w-48")></div>
                <div class="grid flex-1 grid-cols-2 gap-4">
                    {(0..4).map(|_| view! { <div class=format!("{block} h-20")></div> }).collect_view()}
                </div>
            </div>
        }
        .into_any(),
        _ => view! {
            <div class="flex flex-col gap-3" aria-hidden="true">
                {(0..4).map(|_| view! { <div class=format!("{block} h-14")></div> }).collect_view()}
            </div>
        }
        .into_any(),
    };
    view! {
        <div class="w-full py-6" role="status">
            <span class="sr-only">"Loading"</span>
            {layout}
        </div>
    }
}

/// Empty state for a loaded page section with no records.
/// `kind`: "draft" | "matches" | "team" | "pool" | "scout".
#[component]
pub fn PageEmpty(#[prop(into)] kind: String) -> impl IntoView {
    let copy = match kind.as_str() {
        "draft" => "No drafts yet. Drafts you save on the draft page appear here.",
        "matches" => "No matches yet. Link your Riot account on your profile and sync to see your recent games here.",
        "team" => "You are not in a team yet. Create one or ask to join one on the roster page.",
        "pool" => "Your champion pool is empty. Add champions on the champion pool page.",
        "scout" => "No scouting yet. Add an opponent on the opponents page.",
        _ => "Nothing here yet.",
    };
    view! {
        <p class="border-t border-rule py-8 text-body text-secondary">{copy}</p>
    }
}
