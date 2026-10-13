//! Impressum (provider information, § 5 DDG). The operator is a private
//! person; the facts below are the owner's own (launch checklist, 12 October
//! 2026). There is no second contact, no separate person responsible for
//! content and no registration or VAT ID.

use crate::components::ui::{PageHeader, PageShell, PageWidth, SectionHeader};
use leptos::prelude::*;

/// A placeholder the owner replaces: shown in mono, in the warning tone,
/// with the words "[OWNER: ...]". The legal pages carry none at launch; the
/// component stays for any fact the operator has not given yet.
#[component]
pub fn OwnerPlaceholder(#[prop(into)] what: String) -> impl IntoView {
    view! { <span class="font-mono text-label text-warning" data-owner-placeholder>{format!("[OWNER: {what}]")}</span> }
}

/// The operator's name, as in the Impressum (also the privacy controller).
pub const OPERATOR_NAME: &str = "Jasper Seehofer";
/// The operator's postal address, one line per entry.
pub const OPERATOR_ADDRESS: &[&str] = &["Augustenstrasse 4", "70178 Stuttgart", "Germany"];
/// The operator's contact email (also the privacy contact).
pub const OPERATOR_EMAIL: &str = "jasper@jseehofer.eu";

#[component]
pub fn ImpressumPage() -> impl IntoView {
    view! {
        <PageShell width=PageWidth::Reading>
            <PageHeader title="Impressum" eyebrow="Legal" meta=move || view! { <span>"Who runs Team Companion and how to reach them."</span> } />
            <section aria-labelledby="provider-h">
                <SectionHeader title="Provider" id="provider-h" />
                <dl class="grid gap-x-6 gap-y-3 border-t border-rule pt-3 text-body sm:grid-cols-[12rem_minmax(0,1fr)]">
                    <dt class="text-muted">"Operator"</dt>
                    <dd class="text-primary">{OPERATOR_NAME}</dd>
                    <dt class="text-muted">"Address"</dt>
                    <dd class="text-primary">
                        {OPERATOR_ADDRESS.iter().map(|line| view! { <span class="block">{*line}</span> }).collect_view()}
                    </dd>
                    <dt class="text-muted">"Email"</dt>
                    <dd>
                        <a href=format!("mailto:{OPERATOR_EMAIL}") class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">{OPERATOR_EMAIL}</a>
                    </dd>
                </dl>
                <p class="mt-4 max-w-prose text-body text-secondary">
                    "Team Companion is run by a private person."
                </p>
            </section>
            <section class="mt-10" aria-labelledby="beta-h">
                <SectionHeader title="During the closed beta" id="beta-h" />
                <p class="max-w-prose text-body text-secondary">
                    "Questions about the beta can also go to the captain or coach who sent your invite, or through the bug-report button on any page after signing in."
                </p>
            </section>
        </PageShell>
    }
}
