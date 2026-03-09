use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::Nav;
use crate::error_template::ErrorTemplate;
use crate::pages::{
    auth::{login::LoginPage, register::RegisterPage},
    draft::DraftPage,
    game_plan::GamePlanPage,
    home::HomePage,
    post_game::PostGamePage,
    profile::ProfilePage,
    stats::StatsPage,
    team::{dashboard::TeamDashboard, roster::RosterPage},
    team_builder::TeamBuilderPage,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-gray-950 min-h-screen">
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
                    <Route path=path!("/stats") view=StatsPage />
                    <Route path=path!("/team-builder") view=TeamBuilderPage />
                    <Route path=path!("/game-plan") view=GamePlanPage />
                    <Route path=path!("/post-game") view=PostGamePage />
                </Routes>
            </main>
        </Router>
    }
}
