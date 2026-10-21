//! M160 solo mode: a team-only editor page (draft, game plan, review) shows
//! [`NoTeamState`] instead of its editor when the account has no team.
//!
//! The editor stays built exactly as before (so its resources are created
//! in the same order on the server and in the browser, leptos rule 58) and
//! is only hidden; the no-team state is drawn after it, so its page title
//! is the one the tab shows. The membership resource is read inside a
//! `Transition`, so the server waits for it: neither the editor nor the
//! no-team state flashes before the other.

use leptos::prelude::*;

use crate::components::ui::{
    Density, NoTeamState, PageHeader, PageShell, Skeleton, SkeletonGroup, SkeletonShape,
};

/// Whether the signed-in account belongs to a team: the signal `/opponents`
/// and `/stats` use (`get_team_dashboard` returns `None` without a team).
pub fn has_team_resource() -> Resource<bool> {
    Resource::new(
        || (),
        |_| async {
            crate::pages::team::dashboard::get_team_dashboard()
                .await
                .ok()
                .flatten()
                .is_some()
        },
    )
}

/// The page's editor for team members, the page's no-team state otherwise.
#[component]
pub fn TeamGate(
    /// From [`has_team_resource`], created in the page before any Suspense.
    has_team: Resource<bool>,
    /// The page title and eyebrow ("Draft").
    page: &'static str,
    /// The feedback label of the page ("Draft"), as on its `PageShell`.
    feedback_label: &'static str,
    /// "Drafts belong to a team".
    no_team_title: &'static str,
    /// What the page needs and what appears once there is a team.
    no_team_message: &'static str,
    children: Children,
) -> impl IntoView {
    let no_team = move || matches!(has_team.get(), Some(false));
    view! {
        <Transition fallback=move || view! {
            <PageShell density=Density::Compact>
                <PageHeader title=page eyebrow=page />
                <SkeletonGroup label=format!("Loading {}", page.to_lowercase())>
                    <Skeleton shape=SkeletonShape::Block size="h-96 w-full" />
                </SkeletonGroup>
            </PageShell>
        }>
            <div hidden=no_team>{children()}</div>
            {move || no_team().then(|| view! {
                <PageShell density=Density::Compact attr:data-feedback-label=feedback_label attr:data-testid="no-team-page">
                    <PageHeader title=page eyebrow=page />
                    <NoTeamState
                        title=no_team_title
                        message=no_team_message
                        cta_label="Find or create a team"
                        cta_href="/team/dashboard"
                    />
                </PageShell>
            })}
        </Transition>
    }
}
