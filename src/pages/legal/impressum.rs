//! Impressum (provider information). The operator's details are the
//! owner's to fill in before launch (M8); until then every item is a
//! clearly marked placeholder. Nothing here is invented.

use crate::components::ui::{
    Callout, CalloutTone, PageHeader, PageShell, PageWidth, SectionHeader,
};
use leptos::prelude::*;

/// A placeholder the owner replaces: shown in mono, in the warning tone,
/// with the words "[OWNER: ...]".
#[component]
pub fn OwnerPlaceholder(#[prop(into)] what: String) -> impl IntoView {
    view! { <span class="font-mono text-label text-warning" data-owner-placeholder>{format!("[OWNER: {what}]")}</span> }
}

#[component]
pub fn ImpressumPage() -> impl IntoView {
    let rows: [(&str, &str); 6] = [
        (
            "Operator",
            "full name of the person or organisation running Companion",
        ),
        ("Address", "street and number, postcode, city, country"),
        ("Email", "contact email address"),
        (
            "Second contact",
            "phone number or another fast way to reach the operator, if required",
        ),
        (
            "Responsible for content",
            "name and address, if different from the operator",
        ),
        ("Registration or VAT ID", "only if applicable"),
    ];
    view! {
        <PageShell width=PageWidth::Reading>
            <PageHeader title="Impressum" eyebrow="Legal" meta=move || view! { <span>"Who runs Companion and how to reach them."</span> } />
            <div class="mb-8">
                <Callout tone=CalloutTone::Warning title="Not filled in yet">
                    "The operator's details below are placeholders. They are completed before Companion opens beyond the closed beta."
                </Callout>
            </div>
            <section aria-labelledby="provider-h">
                <SectionHeader title="Provider" id="provider-h" />
                <dl class="grid gap-x-6 gap-y-3 border-t border-rule pt-3 text-body sm:grid-cols-[12rem_minmax(0,1fr)]">
                    {rows.into_iter().map(|(label, what)| view! {
                        <dt class="text-muted">{label}</dt>
                        <dd><OwnerPlaceholder what=what /></dd>
                    }).collect_view()}
                </dl>
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
