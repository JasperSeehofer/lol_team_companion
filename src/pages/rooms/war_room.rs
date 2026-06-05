//! The War Room — team-strategy hub, first multi-surface consumer of the room system.
//!
//! Composition: an [`AtmosphereRoot`] (parallax planes + growth overlays) with
//! all five War Room surfaces ([`WarSurfaces`]) floating over it at `z-10`.
//! Growth overlays fade in by tier as the team accumulates draft work, opponent
//! intel, and post-game learnings — "what you build together, you can see."
//!
//! Region is read ONCE at [`WarRoomRoom`] top level and threaded as a plain
//! `region: String` prop (CLAUDE.md rule #58 / leptos-patterns #58).

use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane, PanelFn, RoomSurface, SurfaceTab};
use crate::pages::draft::{BanPrioritySurface, DraftSurface};
use crate::pages::game_plan::GamePlanSurface;
use crate::pages::opponents::OpponentsSurface;
use crate::pages::tree_drafter::TreeDrafterSurface;

// ─── Server function ─────────────────────────────────────────────────────────

/// Fetch the team's War Room growth state for atmosphere overlay tiers.
///
/// Returns [`TeamGrowthState::default()`] (all tiers 0) when the caller has no
/// team — an empty room is a valid initial state, not an error (rule #44).
/// Rejects unauthenticated callers with `ServerFnError` (T-27-07-01).
#[server]
pub async fn get_team_growth_state(
) -> Result<crate::models::growth::TeamGrowthState, ServerFnError> {
    use crate::models::growth::TeamGrowthState;
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
        None => return Ok(TeamGrowthState::default()),
    };

    let counts = db::get_team_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(TeamGrowthState::from_counts(counts))
}

// ─── Page component ───────────────────────────────────────────────────────────

/// The War Room page — parallax atmosphere over the five War Room surfaces.
///
/// Region is read here and NOWHERE ELSE in this module (rule #58).
#[component]
pub fn WarRoomRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives) — drives both
    // the surfaces and which region's art the planes/overlays load.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    // Team growth state drives the atmosphere overlays. No team → all Seed (tier 0).
    let growth = Resource::new(|| (), |_| get_team_growth_state());
    let tactics_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.tactics_tier)
            .unwrap_or(0)
    });
    let intel_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.intel_tier)
            .unwrap_or(0)
    });
    let command_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.command_tier)
            .unwrap_or(0)
    });

    // Active region's asset directory (T-27-07-02: region is server-set, never
    // a URL/query param, so no path traversal possible).
    let dir = format!("/img/rooms/war-room/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            tactics_tier=tactics_tier
            intel_tier=intel_tier
            command_tier=command_tier
        >
            // ── Parallax planes (back → front) ──────────────────────────────
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />

            // ── Growth overlays (fade in by tier via CSS data-*-tier) ───────
            <GrowthOverlay axis="tactics" src=format!("{dir}/overlay-tactics.webp") z=4 />
            <GrowthOverlay axis="intel" src=format!("{dir}/overlay-intel.webp") z=5 />
            <GrowthOverlay axis="command" src=format!("{dir}/overlay-command.webp") z=6 />

            // Contrast scrim so surfaces read against any art (mirrors solo_journey.rs).
            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            // ── Surface layer — the real, accessible, fast UI ───────────────
            <div class="relative z-10 px-8 py-6" data-feedback-label="War Room">
                <WarSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}

// ─── Surface composer ─────────────────────────────────────────────────────────

/// Composes all five War Room surfaces as a [`RoomSurface`] tab strip.
///
/// `region` is a plain `String` prop — never read from context here (rule #58).
/// Only the two surfaces that require it (Draft, Tree-Drafter) receive clones.
#[component]
fn WarSurfaces(region: String) -> impl IntoView {
    // Clone region only for the two surfaces that need it.
    let region_for_draft = region.clone();
    let region_for_tree = region.clone();

    let panels: Vec<(SurfaceTab, PanelFn)> = vec![
        (
            SurfaceTab { id: "draft", label: "Draft" },
            Box::new(move || {
                let r = region_for_draft.clone();
                view! { <DraftSurface region=r /> }.into_any()
            }) as PanelFn,
        ),
        (
            SurfaceTab { id: "tree-drafter", label: "Tree-Drafter" },
            Box::new(move || {
                let r = region_for_tree.clone();
                view! { <TreeDrafterSurface region=r /> }.into_any()
            }) as PanelFn,
        ),
        (
            SurfaceTab { id: "game-plan", label: "Game Plan" },
            Box::new(move || view! { <GamePlanSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "opponents", label: "Opponents" },
            Box::new(move || view! { <OpponentsSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "ban-priority", label: "Ban Priority" },
            Box::new(move || view! { <BanPrioritySurface /> }.into_any()) as PanelFn,
        ),
    ];

    view! { <RoomSurface panels=panels /> }
}
