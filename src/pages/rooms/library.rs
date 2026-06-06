//! The Library (Hall of Records) — team knowledge hub, second multi-surface
//! consumer of the room system.
//!
//! Composition: an [`AtmosphereRoot`] (parallax planes + growth overlays) with
//! all five Library surfaces ([`LibrarySurfaces`]) floating over it at `z-10`.
//! Growth overlays fade in by tier as the team accumulates match records,
//! written learnings, and analytical depth — "what you've recorded, you can see."
//!
//! Region is read ONCE at [`LibraryRoom`] top level and threaded as a plain
//! `region: String` prop (CLAUDE.md rule #58 / leptos-patterns #58).

use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane, PanelFn, RoomSurface, SurfaceTab};
use crate::pages::analytics::AnalyticsSurface;
use crate::pages::match_detail::MatchDetailSurface;
use crate::pages::personal_learnings::PersonalLearningsSurface;
use crate::pages::post_game::PostGameSurface;
use crate::pages::stats::StatsSurface;

// ─── Server function ─────────────────────────────────────────────────────────

/// Fetch the team's Library growth state for atmosphere overlay tiers.
///
/// Returns [`LibraryGrowthState::default()`] (all tiers 0) when the caller has no
/// team — an empty library is a valid initial state, not an error (rule #44).
/// Rejects unauthenticated callers with `ServerFnError` (T-28-05-01).
#[server]
pub async fn get_library_growth_state(
) -> Result<crate::models::growth::LibraryGrowthState, ServerFnError> {
    use crate::models::growth::LibraryGrowthState;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    // No team → all-zero default (rule #44: never Err on absent optional data).
    let team_id = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(id) => id,
        None => return Ok(LibraryGrowthState::default()),
    };

    let counts = db::get_library_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(LibraryGrowthState::from_counts(counts))
}

// ─── Page component ───────────────────────────────────────────────────────────

/// The Library page — parallax atmosphere over the five Library surfaces.
///
/// Region is read here and NOWHERE ELSE in this module (rule #58).
#[component]
pub fn LibraryRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives) — drives both
    // the surfaces and which region's art the planes/overlays load.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    // Team growth state drives the atmosphere overlays. No team → all Seed (tier 0).
    let growth = Resource::new(|| (), |_| get_library_growth_state());
    let records_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.records_tier)
            .unwrap_or(0)
    });
    let tomes_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.tomes_tier)
            .unwrap_or(0)
    });
    let analysis_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.analysis_tier)
            .unwrap_or(0)
    });

    // Active region's asset directory (T-28-05-02: region is server-set, never
    // a URL/query param, so no path traversal possible).
    let dir = format!("/img/rooms/library/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            records_tier=records_tier
            tomes_tier=tomes_tier
            analysis_tier=analysis_tier
        >
            // ── Parallax planes (back → front) ──────────────────────────────
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />

            // ── Growth overlays (fade in by tier via CSS data-*-tier) ───────
            <GrowthOverlay axis="records" src=format!("{dir}/overlay-records.webp") z=4 />
            <GrowthOverlay axis="tomes" src=format!("{dir}/overlay-tomes.webp") z=5 />
            <GrowthOverlay axis="analysis" src=format!("{dir}/overlay-analysis.webp") z=6 />

            // Contrast scrim so surfaces read against any art (mirrors solo_journey.rs).
            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            // ── Surface layer — the real, accessible, fast UI ───────────────
            <div class="relative z-10 px-8 py-6" data-feedback-label="Library">
                <LibrarySurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}

// ─── Surface composer ─────────────────────────────────────────────────────────

/// Composes all five Library surfaces as a [`RoomSurface`] tab strip.
///
/// `region` is a plain `String` prop — never read from context here (rule #58).
/// Provides the shared `selected_match_id` signal BEFORE the view so both
/// `StatsSurface` and `MatchDetailSurface` can consume it (D-07, Pitfall 4).
#[component]
fn LibrarySurfaces(region: String) -> impl IntoView {
    // Shared match-selection signal: Stats rows set this to switch to the Match
    // Detail tab with that match expanded. MUST be provided before view! so
    // both keep-mounted panels find it in context (Pitfall 4 prevention).
    let selected_match_id: RwSignal<Option<String>> = RwSignal::new(None);
    provide_context(selected_match_id);

    // region accepted for prop parity with WarSurfaces; none of the five
    // Library surfaces need it for content, so prefix _ to suppress compiler warning.
    let _region = region;

    let panels: Vec<(SurfaceTab, PanelFn)> = vec![
        (
            SurfaceTab { id: "stats", label: "Stats" },
            Box::new(move || view! { <StatsSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "match", label: "Match Detail" },
            Box::new(move || view! { <MatchDetailSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "analytics", label: "Analytics" },
            Box::new(move || view! { <AnalyticsSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "post-game", label: "Post-Game" },
            Box::new(move || view! { <PostGameSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "personal", label: "Personal Learnings" },
            Box::new(move || view! { <PersonalLearningsSurface /> }.into_any()) as PanelFn,
        ),
    ];

    view! { <RoomSurface panels=panels /> }
}
