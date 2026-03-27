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
    analytics::AnalyticsPage,
    auth::{login::LoginPage, register::RegisterPage},
    champion_pool::ChampionPoolPage,
    draft::DraftPage,
    game_plan::GamePlanPage,
    home::HomePage,
    match_detail::MatchDetailPage,
    opponents::OpponentsPage,
    post_game::PostGamePage,
    profile::ProfilePage,
    solo_dashboard::SoloDashboardPage,
    stats::StatsPage,
    team::{dashboard::TeamDashboard, roster::RosterPage},
    team_builder::TeamBuilderPage,
    tree_drafter::TreeDrafterPage,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script>{r#"(function(){var t=localStorage.getItem('theme');if(t==='light')document.documentElement.setAttribute('data-theme','light');var a=localStorage.getItem('accent');if(a)document.documentElement.setAttribute('data-accent',a);})()"#}</script>
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
                    </Routes>
                </main>
            </ToastProvider>
        </Router>
    }
}
