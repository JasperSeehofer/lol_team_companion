//! The Throne — the player's personal hub: identity, team, and growth in one place.
//!
//! Composition: an [`AtmosphereRoot`] (parallax planes + growth overlays) with
//! all five Throne surfaces ([`ThroneSurfaces`]) floating over it at `z-10`.
//! Growth overlays fade in by tier as the user recruits teammates (roster),
//! climbs in ranked play (progression), and earns team wins (crown).
//!
//! Region is read ONCE at [`ThroneRoom`] top level and threaded as a plain
//! `region: String` prop (CLAUDE.md rule #58 / leptos-patterns #58).

use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane, PanelFn, RoomSurface, SurfaceTab};
use crate::pages::profile::IdentitySurface;
use crate::pages::team::dashboard::{DashboardSurface, DevelopmentSurface, RosterSurface};
use crate::pages::solo_dashboard::SoloSurfaces;

// ─── IA-Healing Redirect Server Functions ────────────────────────────────────

/// Redirect `/profile` → `/throne?tab=identity` (D-12/Plan 05).
#[server]
pub async fn redirect_to_throne_identity() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/throne?tab=identity");
    Ok(())
}

/// Redirect `/team/dashboard` → `/throne?tab=dashboard` (D-12/Plan 05).
#[server]
pub async fn redirect_to_throne_dashboard() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/throne?tab=dashboard");
    Ok(())
}

/// Redirect `/action-items` → `/throne?tab=development` (D-12/Plan 05).
#[server]
pub async fn redirect_to_throne_development() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/throne?tab=development");
    Ok(())
}

// ─── IA-Healing Redirect Shell Components ────────────────────────────────────

/// Thin shell that 302-redirects `/profile` → `/throne?tab=identity`.
///
/// Uses the dual pattern: `leptos_axum::redirect()` for SSR 302 headers +
/// `window.location().set_href()` for the client-side navigation path
/// (analog `library.rs` `StatsRedirect` — wasm-patterns rule #37).
#[component]
pub fn ProfileRedirect() -> impl IntoView {
    let action = ServerAction::<RedirectToThroneIdentity>::new();
    action.dispatch(RedirectToThroneIdentity {});

    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/throne?tab=identity");
    }

    view! { <></> }
}

/// Thin shell that 302-redirects `/team/dashboard` → `/throne?tab=dashboard`.
#[component]
pub fn TeamDashboardRedirect() -> impl IntoView {
    let action = ServerAction::<RedirectToThroneDashboard>::new();
    action.dispatch(RedirectToThroneDashboard {});

    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/throne?tab=dashboard");
    }

    view! { <></> }
}

/// Thin shell that 302-redirects `/action-items` → `/throne?tab=development`.
#[component]
pub fn ActionItemsRedirect() -> impl IntoView {
    let action = ServerAction::<RedirectToThroneDevelopment>::new();
    action.dispatch(RedirectToThroneDevelopment {});

    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/throne?tab=development");
    }

    view! { <></> }
}

// ─── Server function ─────────────────────────────────────────────────────────

/// Fetch the blended Throne growth state for atmosphere overlay tiers.
///
/// This fn diverges from sibling rooms (war_room.rs, library.rs) by computing
/// `progression` BEFORE the team check — a teamless ranked grinder's room still
/// lights the progression overlay (D-05/D-07). The team axes (roster, crown)
/// fall to 0 without a team; that is valid initial state, not an error (rule #44).
///
/// Rejects unauthenticated callers with `ServerFnError` (T-29-05-05).
#[server]
pub async fn get_throne_growth_state(
) -> Result<crate::models::growth::ThroneGrowthState, ServerFnError> {
    use crate::models::growth::{ThroneGrowthCounts, ThroneGrowthState};
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

    // 1. PERSONAL axis FIRST — never fails on absent data (rule #44, D-07).
    //    progression lights even with no team — the inverse of both sibling rooms.
    let ranked = db::get_latest_ranked_snapshot(&surreal, &user.id, "RANKED_SOLO_5x5")
        .await
        .unwrap_or(None);

    // 2. TEAM axes — fall to 0 without a team (do NOT early-return the whole state,
    //    D-07: we computed progression above and must include it).
    let team_counts = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(team_id) => db::get_throne_growth_counts(&surreal, &team_id)
            .await
            .unwrap_or_default(),
        None => ThroneGrowthCounts::default(),
    };

    Ok(ThroneGrowthState::from_parts(team_counts, ranked))
}

// ─── E2E Seed Helpers (debug builds only) ────────────────────────────────────

/// E2E-only: insert a `RANKED_SOLO_5x5` ranked snapshot for the current user.
///
/// Compiled only in debug builds (`debug_assertions`). This lets the SC1b
/// Playwright test seed a ranked record without going through the Riot API.
/// Auth-gated — rejects unauthenticated callers.
///
/// Inserts: Gold II, 75 LP, 25 wins, 10 losses (total 35 games ≥ 20 → tier 2).
#[cfg(debug_assertions)]
#[server]
pub async fn seed_ranked_snapshot_for_test() -> Result<(), ServerFnError> {
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

    db::store_ranked_snapshot(
        &surreal,
        &user.id,
        "RANKED_SOLO_5x5",
        "Gold",
        "II",
        75,
        25,
        10,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

// ─── Page component ───────────────────────────────────────────────────────────

/// The Throne page — parallax atmosphere over the five Throne surfaces.
///
/// Region is read here and NOWHERE ELSE in this module (rule #58).
#[component]
pub fn ThroneRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives) — drives both
    // the surfaces and which region's art the planes/overlays load.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    // Blended growth state drives the atmosphere overlays.
    // No team → roster/crown stay at Seed (tier 0); progression advances independently.
    //
    // Phase 29.1 hydration fix: reading `growth.get()` directly inside a
    // `Signal::derive` happens OUTSIDE any <Suspense> (Leptos warns about this at
    // throne.rs render). For a TEAM user the resource resolves to non-zero tiers
    // during SSR but is still pending on hydrate (→ 0 via the fallback), so the
    // `data-*-tier` attributes on AtmosphereRoot's root differ between SSR and
    // hydrate. That desyncs the tachys hydration cursor and panics at the first
    // complex descendant (ActionItemsPanel) — the team-user-only Phase 29 regression
    // that the no-team SC4 canary (all tiers 0 both ways) never surfaced.
    //
    // Fix: tiers start at 0 (matching BOTH the SSR pending-state render AND the
    // hydrate initial render — no structural/attribute divergence), then an Effect
    // updates them once the resource resolves on the client. The overlays fade in
    // reactively post-hydrate; initial paint is identical SSR↔hydrate.
    let growth = Resource::new(|| (), |_| get_throne_growth_state());
    let roster_tier = RwSignal::new(0u8);
    let progression_tier = RwSignal::new(0u8);
    let crown_tier = RwSignal::new(0u8);
    Effect::new(move |_| {
        if let Some(Ok(g)) = growth.get() {
            roster_tier.set(g.roster_tier);
            progression_tier.set(g.progression_tier);
            crown_tier.set(g.crown_tier);
        }
    });

    // Active region's asset directory (T-29-05-03: region is server-set, never
    // a URL/query param, so no path traversal possible).
    let dir = format!("/img/rooms/throne/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            roster_tier=roster_tier
            progression_tier=progression_tier
            crown_tier=crown_tier
        >
            // ── Parallax planes (back → front) ──────────────────────────────
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />

            // ── Growth overlays (discrete per-tier art via CSS data-*-tier) ──
            // Three stacked images per axis — exactly one is visible at a given
            // tier (CSS data-attr toggle, no reactive branching — ROOM-03).
            <GrowthOverlay axis="roster--tier-1"      src=format!("{dir}/overlay-roster-1.webp")      z=4 />
            <GrowthOverlay axis="roster--tier-2"      src=format!("{dir}/overlay-roster-2.webp")      z=4 />
            <GrowthOverlay axis="roster--tier-3"      src=format!("{dir}/overlay-roster-3.webp")      z=4 />
            <GrowthOverlay axis="progression--tier-1" src=format!("{dir}/overlay-progression-1.webp") z=5 />
            <GrowthOverlay axis="progression--tier-2" src=format!("{dir}/overlay-progression-2.webp") z=5 />
            <GrowthOverlay axis="progression--tier-3" src=format!("{dir}/overlay-progression-3.webp") z=5 />
            <GrowthOverlay axis="crown--tier-1"       src=format!("{dir}/overlay-crown-1.webp")       z=6 />
            <GrowthOverlay axis="crown--tier-2"       src=format!("{dir}/overlay-crown-2.webp")       z=6 />
            <GrowthOverlay axis="crown--tier-3"       src=format!("{dir}/overlay-crown-3.webp")       z=6 />

            // Contrast scrim so surfaces read against any art (D-15, mirrors solo_journey.rs:69).
            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            // ── Surface layer — the real, accessible, fast UI ───────────────
            <div class="relative z-10 px-8 py-6" data-feedback-label="Throne">
                <ThroneSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}

// ─── Surface composer ─────────────────────────────────────────────────────────

/// Composes all five Throne surfaces as a [`RoomSurface`] tab strip.
///
/// `region` is a plain `String` prop — never read from context here (rule #58).
/// Only the `SoloSurfaces` panel consumes `region`; all other surfaces are
/// region-agnostic.
#[component]
fn ThroneSurfaces(region: String) -> impl IntoView {
    let panels: Vec<(SurfaceTab, PanelFn)> = vec![
        (
            SurfaceTab { id: "identity", label: "Identity" },
            Box::new(move || view! { <IdentitySurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "dashboard", label: "Dashboard" },
            Box::new(move || view! { <DashboardSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "roster", label: "Roster" },
            Box::new(move || view! { <RosterSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "development", label: "Development" },
            Box::new(move || view! { <DevelopmentSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "solo", label: "Solo" },
            Box::new(move || {
                let r = region.clone();
                view! { <SoloSurfaces region=r /> }.into_any()
            }) as PanelFn,
        ),
    ];

    view! { <RoomSurface panels=panels /> }
}
