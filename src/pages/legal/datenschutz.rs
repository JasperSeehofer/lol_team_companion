//! Phase 17 plan 17-06 — Datenschutzerklärung route stub.
//!
//! D-13 utility tier. Full 4-section DSE (logfiles, registration,
//! hosting, betroffenenrechte) lands in Phase 21 per guardrails.
//! Phase 17 only ensures the route returns 200 with canvas-grain
//! chrome so it exists pre-deploy.

use leptos::prelude::*;

use crate::components::region::HeraldicDivider;

#[component]
pub fn DatenschutzPage() -> impl IntoView {
    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-12">
            <div class="max-w-2xl mx-auto flex flex-col gap-6">
                <header>
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "Legal"
                    </div>
                    <h1 class="font-display italic text-[44px] leading-tight text-primary mt-2">
                        "Datenschutzerkl\u{00e4}rung"
                    </h1>
                    <div class="mt-3"><HeraldicDivider width=240 /></div>
                </header>

                <section class="bg-elevated border border-divider rounded-xl p-6">
                    <p class="text-secondary text-sm leading-relaxed">
                        "Content to be added in Phase 21 \u{2014} will follow the 4-section \
                         DSE structure (logfiles, registration, hosting, Betroffenenrechte) \
                         per guardrails. Each section will name the data category, the legal \
                         basis (Art. 6 DSGVO), the retention window, and the recipient set."
                    </p>
                </section>

                <section class="bg-surface border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "Sections planned"
                    </div>
                    <ul class="text-secondary text-sm mt-3 space-y-2 list-disc list-inside">
                        <li>"Logfiles \u{2014} server access logs, retention 14 days"</li>
                        <li>"Registration \u{2014} username, email, hashed password, theme preference"</li>
                        <li>"Hosting \u{2014} provider details and DPA"</li>
                        <li>"Betroffenenrechte \u{2014} Auskunft, Berichtigung, L\u{00f6}schung, Widerspruch"</li>
                    </ul>
                </section>
            </div>
        </div>
    }
}
