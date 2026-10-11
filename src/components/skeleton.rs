//! Page-level loading and empty states for the draft and solo pages.
//! Batch 2's `Skeleton` and `EmptyState` replace these in M3/M5.

use leptos::prelude::*;

use crate::components::ui::{Skeleton, SkeletonShape};

/// Loading placeholder for a page-level Suspense fallback.
/// `variant` picks the rough layout: "draft" | "solo" | "team".
///
/// The blocks are decorative; a visually hidden status says "Loading".
#[component]
pub fn PageLoading(#[prop(into)] variant: String) -> impl IntoView {
    let layout = match variant.as_str() {
        "draft" => view! {
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-3" aria-hidden="true">
                <Skeleton shape=SkeletonShape::Block size="h-72" />
                <Skeleton shape=SkeletonShape::Block size="h-72" />
                <Skeleton shape=SkeletonShape::Block size="h-72" />
            </div>
        }
        .into_any(),
        "solo" => view! {
            <div class="flex flex-col gap-6 sm:flex-row" aria-hidden="true">
                <div class="sm:w-48"><Skeleton shape=SkeletonShape::Block size="h-48" /></div>
                <div class="grid flex-1 grid-cols-2 gap-4">
                    {(0..4).map(|_| view! { <Skeleton shape=SkeletonShape::Block size="h-20" /> }).collect_view()}
                </div>
            </div>
        }
        .into_any(),
        _ => view! {
            <div class="flex flex-col gap-3" aria-hidden="true">
                {(0..4).map(|_| view! { <Skeleton shape=SkeletonShape::Block size="h-14" /> }).collect_view()}
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
