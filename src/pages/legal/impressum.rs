//! Phase 17 plan 17-01 placeholder. Real Impressum (§5 DDG, Steuernummer)
//! content lands in plan 06 (closed-beta surfaces / legal pages).

use leptos::prelude::*;

#[component]
pub fn ImpressumPage() -> impl IntoView {
    view! {
        <div class="px-6 py-12 text-secondary">
            <h1 class="font-imperial text-2xl uppercase tracking-[0.18em] text-primary">
                "Impressum"
            </h1>
            <p class="mt-4 text-muted">
                "legal/impressum placeholder (plan 06)"
            </p>
        </div>
    }
}
