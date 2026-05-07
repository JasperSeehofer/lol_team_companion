//! Phase 17 plan 17-06 — Impressum route stub.
//!
//! D-13 utility tier. Full content (§5 DDG operator block,
//! Steuernummer, Verantwortlicher, Berufsbezeichnung) lands in
//! Phase 21 per guardrail G-03. Phase 17 only ensures the route
//! returns 200 with canvas-grain chrome so it exists in the router
//! before deploy.

use leptos::prelude::*;

use crate::components::ornaments::HeraldicDivider;

#[component]
pub fn ImpressumPage() -> impl IntoView {
    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-12">
            <div class="max-w-2xl mx-auto flex flex-col gap-6">
                <header>
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "Legal"
                    </div>
                    <h1 class="font-display italic text-[44px] leading-tight text-primary mt-2">
                        "Impressum"
                    </h1>
                    <div class="mt-3"><HeraldicDivider width=240 /></div>
                </header>

                <section class="bg-elevated border border-divider rounded-xl p-6">
                    <p class="text-secondary text-sm leading-relaxed">
                        "Content to be added in Phase 21 \u{2014} will cite \u{00a7}5 DDG \
                         (Digitale-Dienste-Gesetz) per guardrail G-03. The operator block, \
                         Steuernummer, and Verantwortlicher entries are populated when the \
                         hosting and registration details are finalised before the closed-beta \
                         launch."
                    </p>
                </section>

                <section class="bg-surface border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "Contact"
                    </div>
                    <p class="text-secondary text-sm mt-2">
                        "For inquiries about this beta, contact the captain who issued your invite."
                    </p>
                </section>
            </div>
        </div>
    }
}
