use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::Nav;
use crate::components::ui::ToastProvider;
use crate::error_template::ErrorTemplate;
use crate::pages::{
    action_items::ActionItemsPage,
    admin::invites::AdminInvitesPage,
    analytics::AnalyticsPage,
    auth::{login::LoginPage, register::RegisterPage},
    champion_pool::ChampionPoolPage,
    closed_beta::ClosedBetaPage,
    draft::DraftPage,
    game_plan::GamePlanPage,
    home::HomePage,
    legal::{datenschutz::DatenschutzPage, impressum::ImpressumPage},
    match_detail::MatchDetailPage,
    opponents::OpponentsPage,
    personal_learnings::{NewLearningPage, PersonalLearningsPage},
    post_game::PostGamePage,
    profile::ProfilePage,
    solo_dashboard::SoloDashboardPage,
    stats::StatsPage,
    team::{dashboard::TeamDashboard, roster::RosterPage},
    team_builder::TeamBuilderPage,
    tree_drafter::TreeDrafterPage,
};

/// Per-request initial theme provided via context from the axum
/// request handler. Defaults to "demacia" when no user session theme
/// is available (unauthenticated visitors, hydration mismatch fallback).
///
/// Per plan 17-01 task 6 fallback: SSR sets a default; the
/// `ThemeToggle` component performs a post-hydration sync from the
/// user's DB record. Brief flicker on the very first authenticated page
/// load is acceptable; subsequent navigations are flicker-free because
/// the `data-theme` attribute is preserved across SSR turnarounds.
#[derive(Clone, Debug)]
pub struct InitialTheme(pub String);

impl Default for InitialTheme {
    fn default() -> Self {
        InitialTheme("demacia".to_string())
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    // Read the initial theme from request context (provided per-request in
    // main.rs). Fall back to "demacia" when context is absent (e.g. static
    // file errors or before context is wired).
    let theme = use_context::<InitialTheme>()
        .map(|t| t.0)
        .unwrap_or_else(|| "demacia".to_string());

    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme=theme>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-base min-h-screen text-primary">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/lol_team_companion.css" />
        <Title text="LoL Team Companion" />

        <Router>
            <ToastProvider>
                <Nav />
                <main>
                    <Routes fallback=|| view! { <ErrorTemplate outside_errors=Errors::default() /> }>
                        <Route path=path!("/") view=HomePage />
                        <Route path=path!("/profile") view=ProfilePage />
                        <Route path=path!("/auth/login") view=LoginPage />
                        <Route path=path!("/auth/register") view=RegisterPage />
                        <Route path=path!("/team/dashboard") view=TeamDashboard />
                        <Route path=path!("/team/roster") view=RosterPage />
                        <Route path=path!("/draft") view=DraftPage />
                        <Route path=path!("/tree-drafter") view=TreeDrafterPage />
                        <Route path=path!("/stats") view=StatsPage />
                        <Route path=path!("/champion-pool") view=ChampionPoolPage />
                        <Route path=path!("/team-builder") view=TeamBuilderPage />
                        <Route path=path!("/game-plan") view=GamePlanPage />
                        <Route path=path!("/post-game") view=PostGamePage />
                        <Route path=path!("/opponents") view=OpponentsPage />
                        <Route path=path!("/action-items") view=ActionItemsPage />
                        <Route path=path!("/analytics") view=AnalyticsPage />
                        <Route path=path!("/solo") view=SoloDashboardPage />
                        <Route path=path!("/match/:id") view=MatchDetailPage />
                        <Route path=path!("/personal-learnings") view=PersonalLearningsPage />
                        <Route path=path!("/personal-learnings/new") view=NewLearningPage />
                        // Phase 17 plan 17-01 stubs (filled by plan 06)
                        <Route path=path!("/closed-beta") view=ClosedBetaPage />
                        <Route path=path!("/admin/invites") view=AdminInvitesPage />
                        <Route path=path!("/legal/impressum") view=ImpressumPage />
                        <Route path=path!("/legal/datenschutz") view=DatenschutzPage />
                    </Routes>
                </main>
            </ToastProvider>
        </Router>
    }
}
