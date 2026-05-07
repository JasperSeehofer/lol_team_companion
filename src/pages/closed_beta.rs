//! Phase 17 plan 17-01 placeholder. Real content lands in plan 06
//! (closed-beta surfaces). This stub exists so router/e2e wiring can
//! be tested in Wave 0.

use leptos::prelude::*;

#[component]
pub fn ClosedBetaPage() -> impl IntoView {
    view! {
        <div class="px-6 py-12 text-secondary">
            <h1 class="font-imperial text-2xl uppercase tracking-[0.18em] text-primary">
                "Closed beta · by invitation"
            </h1>
            <p class="mt-4 text-muted">
                "closed-beta placeholder (plan 06)"
            </p>
        </div>
    }
}
