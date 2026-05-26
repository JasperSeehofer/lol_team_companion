//! Phase 17 plan 17-06 — Closed-beta hero landing.
//!
//! D-15 hero tier: Claude Design quality + FLUX background. First
//! surface external (incognito) visitors see when they reach
//! `/closed-beta`. Authenticated users are redirected to
//! `/team/dashboard` because there is no point showing the public
//! landing to a logged-in user (UI-SPEC §"Closed-Beta Surfaces").
//!
//! Theme-conditional background: both `<img>` tags are rendered;
//! CSS in `input.css` shows only the variant matching the active
//! `[data-theme]` on `<html>`. This keeps the markup static (no
//! reactive `if`) and the swap is a pure-CSS toggle on theme change.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::InitialTheme;
use crate::components::region::{CompanionSigil, FleurDeLis};
use crate::pages::profile::get_current_user;

#[component]
pub fn ClosedBetaPage() -> impl IntoView {
    // Phase 18.2-03 — read region ONCE at page entry; pass as a prop
    // to CompanionSigil so its branch decision matches SSR on hydrate.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let user = Resource::new(|| (), |_| get_current_user());

    // D-15: redirect authenticated users to /team/dashboard. The
    // public landing is for unauthenticated incognito visitors only.
    Effect::new(move |_| {
        if let Some(Ok(Some(_))) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/team/dashboard");
            }
        }
    });

    view! {
        <div class="canvas-grain bg-base min-h-screen relative overflow-hidden">
            // FLUX-style background images. Both rendered; CSS in
            // input.css shows only the variant matching [data-theme].
            // `aria-hidden` + `loading="lazy"` per UI-SPEC perf budget.
            <img
                src="/img/beta-landing-demacia.jpg"
                alt=""
                aria-hidden="true"
                loading="lazy"
                class="closed-beta-bg-demacia fixed inset-0 -z-10 w-full h-full object-cover"
            />
            <img
                src="/img/beta-landing-pandemonium.jpg"
                alt=""
                aria-hidden="true"
                loading="lazy"
                class="closed-beta-bg-pandemonium fixed inset-0 -z-10 w-full h-full object-cover"
            />

            // Soft vignette so headline reads against any FLUX variant.
            <div
                class="fixed inset-0 -z-10 pointer-events-none"
                style="background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            <div class="relative z-10 max-w-3xl mx-auto py-24 px-6 text-center flex flex-col items-center">
                <div class="mb-2">
                    <CompanionSigil region=region.clone() />
                </div>

                <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mt-8">
                    "Closed beta · by invitation"
                </div>

                <h1 class="font-display italic text-[64px] leading-[1.1] text-primary mt-6">
                    "The Strategy Room"
                </h1>

                <p class="font-display italic text-[20px] text-secondary mt-4 max-w-xl">
                    "Reserved for the named few. Ask your captain for an invite."
                </p>

                <div class="mt-12">
                    <A
                        href="/auth/login"
                        attr:class="inline-block bg-accent text-accent-contrast font-semibold rounded-lg px-8 py-3 hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                    >
                        "Sign in"
                    </A>
                </div>

                <p class="mt-6 text-sm text-muted">
                    "Already have an invite? Ask your captain for the link."
                </p>

                // Wax-seal ornament at the bottom — UI-SPEC line 544.
                <div class="mt-16 flex justify-center" aria-hidden="true">
                    <FleurDeLis size=24 />
                </div>
            </div>
        </div>
    }
}
