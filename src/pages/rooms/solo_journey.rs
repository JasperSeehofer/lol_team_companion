//! The Solo Journey room — first consumer of the immersive room system.
//!
//! Composition: an [`AtmosphereRoot`] (parallax planes + growth overlays) with
//! the existing Solo surfaces ([`SoloSurfaces`]) floating over it at `z-10`.
//! The growth overlays fade in by tier as the player accumulates learnings,
//! champion-pool entries, and ranked snapshots — "your time shows."

use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane};
use crate::pages::solo_dashboard::{get_growth_state, SoloSurfaces};

#[component]
pub fn SoloJourneyRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives) — drives both
    // the surfaces and which region's art the planes/overlays load.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    // Growth state drives the atmosphere overlays. Empty → all Seed (tier 0).
    let growth = Resource::new(|| (), |_| get_growth_state());
    let lib_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.library_tier)
            .unwrap_or(0)
    });
    let armory_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.armory_tier)
            .unwrap_or(0)
    });
    let banner_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.banner_tier)
            .unwrap_or(0)
    });

    // Active region's asset directory — only one region's art is ever rendered
    // (per the 400 KB budget; CSS-hiding the other region still triggers fetch).
    let dir = format!("/img/rooms/solo-journey/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            lib_tier=lib_tier
            armory_tier=armory_tier
            banner_tier=banner_tier
        >
            // ── Parallax planes (back → front) ──────────────────────────────
            // Positive z within the room's isolated stacking context: they
            // paint above the base background but below the surface (z-10).
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />

            // ── Growth overlays (fade in by tier via CSS data-*-tier) ───────
            <GrowthOverlay axis="banner" src=format!("{dir}/overlay-banner.webp") z=4 />
            <GrowthOverlay axis="lib" src=format!("{dir}/overlay-lib.webp") z=5 />
            <GrowthOverlay axis="armory" src=format!("{dir}/overlay-armory.webp") z=6 />

            // Contrast scrim so surfaces read against any art (mirrors closed_beta).
            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            // ── Surface layer — the real, accessible, fast UI ───────────────
            <div class="relative z-10 px-8 py-6" data-feedback-label="Solo Journey room">
                <SoloSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
