//! Phase 17 plan 17-01 placeholder. Real admin invite UI lands in plan 06
//! (closed-beta surfaces). For Wave 0 this only needs to return 200.

use leptos::prelude::*;

#[component]
pub fn AdminInvitesPage() -> impl IntoView {
    view! {
        <div class="px-6 py-12 text-secondary">
            <h1 class="font-imperial text-2xl uppercase tracking-[0.18em] text-primary">
                "Admin · Invites"
            </h1>
            <p class="mt-4 text-muted">
                "admin/invites placeholder (plan 06)"
            </p>
        </div>
    }
}
