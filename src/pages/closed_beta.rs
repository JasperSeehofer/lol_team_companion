//! Closed-beta landing: the first page invited visitors without a link
//! see (also the signed-out home page). Signed-in users are sent to `/`. Plain type on the
//! page ground, no imagery: what the product is, who it is for, how to get
//! in.

use crate::components::ui::{Button, ButtonSize, ButtonVariant, PageShell, PageWidth};
use crate::pages::profile::get_current_user;
use leptos::prelude::*;

#[component]
pub fn ClosedBetaPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    // Signed-in users go to Match Day; the landing is for visitors.
    Effect::new(move |_| {
        if let Some(Ok(Some(_))) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });
    view! { <Landing /> }
}

/// What Team Companion is, the closed-beta note and the way in.
///
/// A11y: one `<h1>`; the loop is an ordered list; "Sign in" is the page's
/// one primary action.
#[component]
pub fn Landing() -> impl IntoView {
    let loop_steps = [
        (
            "01",
            "Scout",
            "Dossiers on every opponent, with the evidence behind each finding.",
        ),
        (
            "02",
            "Draft",
            "Plan and branch the draft; every suggestion says why it fits.",
        ),
        (
            "03",
            "Plan",
            "Write the game plan the five of you will actually play.",
        ),
        (
            "04",
            "Review",
            "Turn each game into lessons and action items for the next prep.",
        ),
    ];
    view! {
        <PageShell width=PageWidth::Standard class="pt-12 sm:pt-24">
            <p class="eyebrow">"Closed beta \u{00b7} by invitation"</p>
            <h1 class="mt-4 max-w-3xl font-display text-headline sm:text-display font-semibold tracking-[-0.025em] text-primary">
                "The coaching workspace for one team."
            </h1>
            <p class="mt-5 max-w-2xl text-reading text-secondary">
                "Team Companion keeps a team's scouting, drafts, game plans and reviews in one loop, so every scrim adds to what the team knows. It is in a closed beta: teams join with an invite link from their captain or coach."
            </p>
            <div class="mt-8 flex flex-wrap items-center gap-3">
                <Button href="/auth/login" variant=ButtonVariant::Primary size=ButtonSize::Lg>"Sign in"</Button>
                <span class="text-body text-muted">"Have an invite link? Open it to create your account."</span>
            </div>

            <ol class="mt-16 grid gap-x-8 gap-y-6 border-t border-rule pt-8 sm:grid-cols-2 lg:grid-cols-4">
                {loop_steps
                    .into_iter()
                    .map(|(n, title, text)| view! {
                        <li class="flex flex-col gap-1.5">
                            <span class="figures text-caption text-muted">{n}</span>
                            <span class="font-display text-heading font-semibold text-primary">{title}</span>
                            <span class="text-body text-secondary">{text}</span>
                        </li>
                    })
                    .collect_view()}
            </ol>
        </PageShell>
    }
}
