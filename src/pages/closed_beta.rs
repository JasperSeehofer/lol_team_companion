//! Closed-beta landing: the first page invited visitors without a link
//! see (also the signed-out home page). Signed-in users are sent to `/`. The
//! drawn Rift on the page ground beside what the product is, who it is for
//! and how to get in.

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, PageShell, PageWidth, RiftAnchor, RiftLabel, RiftSketch,
};
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
    view! {
        <leptos_meta::Title text=crate::components::ui::page_title("Closed beta") />
        <Landing />
    }
}

/// What Team Companion is, the closed-beta note and the way in (M152
/// hero, recipe card D): the drawn Rift with "YOUR TEAM" surveyed onto the
/// circle base, and the lead-figure grammar (one figure: the headline; one
/// sentence; one action: "Sign in"). Chalk twice in the first viewport:
/// the surveyed label and the primary button (`data-chalk`). 1280: text
/// 5/12, sketch 7/12; phones: title, sentence, button, then the sketch.
///
/// A11y: one `<h1>`; the drawing is `aria-hidden` (the sentence says what
/// the product does); the loop is an ordered list; "Sign in" is the page's
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
        <PageShell width=PageWidth::Standard class="pt-10 sm:pt-16">
            <div class="grid items-start gap-10 lg:grid-cols-12" data-hero>
                <div class="min-w-0 lg:col-span-5 lg:pt-5">
                    <p class="eyebrow">"Closed beta \u{00b7} join with an invite link"</p>
                    <h1 class="mt-4 font-display text-headline font-semibold tracking-[-0.025em] text-primary lg:text-display">
                        "The coaching workspace for one team."
                    </h1>
                    <p class="mt-5 max-w-[60ch] text-reading text-secondary">
                        "Scouting, drafts, game plans and reviews in one loop."
                    </p>
                    <div class="mt-8 grid sm:block">
                        <Button href="/auth/login" variant=ButtonVariant::Primary size=ButtonSize::Lg attr:data-chalk="action">"Sign in"</Button>
                    </div>
                </div>
                <div class="min-w-0 lg:col-span-7 lg:pl-6 lg:pt-5">
                    <RiftSketch
                        label=RiftLabel::new("Your team", RiftAnchor::BlueBase)
                        class="mx-auto w-full max-w-[560px] lg:mr-0"
                    />
                </div>
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
