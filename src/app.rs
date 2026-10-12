use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    path, SsrMode,
};

use crate::components::bug_report_widget::BugReportWidget;
use crate::components::footer::SiteFooter;
use crate::components::nav::Nav;
use crate::components::ui::ToastProvider;
use crate::error_template::NotFound;
use crate::pages::{
    action_items::ActionItemsPage,
    admin::invites::AdminInvitesPage,
    admin::traits::AdminTraitsPage,
    auth::{login::LoginPage, register::RegisterPage},
    champion_pool::ChampionPoolPage,
    closed_beta::ClosedBetaPage,
    design::DesignGalleryPage,
    draft::DraftPage,
    game_plan::GamePlanPage,
    home::HomePage,
    legal::{datenschutz::DatenschutzPage, impressum::ImpressumPage},
    match_detail::MatchDetailPage,
    me::MyGamesPage,
    opponents::OpponentsPage,
    personal_learnings::{NewLearningPage, PersonalLearningsPage},
    post_game::PostGamePage,
    profile::ProfilePage,
    stats::StatsPage,
    team::{dashboard::TeamDashboard, roster::RosterPage},
    tree_drafter::TreeDrafterPage,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    // One theme (dark, "Survey"); tokens live on :root in input.css.
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="color-scheme" content="dark" />
                // The two faces every page needs first (UI text, titles).
                <link rel="preload" href="/fonts/public-sans/public-sans-latin-wght-normal.woff2" r#as="font" r#type="font/woff2" crossorigin="anonymous" />
                <link rel="preload" href="/fonts/familjen-grotesk/familjen-grotesk-latin-wght-normal.woff2" r#as="font" r#type="font/woff2" crossorigin="anonymous" />
                // Live Data Dragon version for image URLs built in the browser.
                <meta name=crate::models::ddragon::META_NAME content=crate::models::ddragon::version() />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-page min-h-screen text-primary">
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
        <Title text="Companion" />

        <Router>
            <ToastProvider>
                <a
                    href="#main"
                    class="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-3 focus:z-(--z-toast) focus:rounded-ctl focus:bg-accent focus:px-3 focus:py-2 focus:text-label focus:font-semibold focus:text-accent-ink"
                >
                    "Skip to content"
                </a>
                <Nav />
                <main id="main" tabindex="-1">
                    <Routes fallback=|| view! { <NotFound /> }>
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
                        // Parked pages and the old solo dashboard: plain redirects.
                        <Route path=path!("/team-builder") view=|| view! { <Redirect path="/draft" /> } />
                        <Route path=path!("/game-plan") view=GamePlanPage />
                        <Route path=path!("/post-game") view=PostGamePage />
                        <Route path=path!("/opponents") view=OpponentsPage />
                        <Route path=path!("/action-items") view=ActionItemsPage />
                        <Route path=path!("/analytics") view=|| view! { <Redirect path="/stats" /> } />
                        <Route path=path!("/solo") view=|| view! { <Redirect path="/me" /> } />
                        <Route path=path!("/me") view=MyGamesPage />
                        <Route path=path!("/match/:id") view=MatchDetailPage />
                        <Route path=path!("/personal-learnings") view=PersonalLearningsPage />
                        <Route path=path!("/personal-learnings/new") view=NewLearningPage />
                        <Route path=path!("/closed-beta") view=ClosedBetaPage />
                        <Route path=path!("/admin/invites") view=AdminInvitesPage ssr=SsrMode::Async />
                        <Route path=path!("/admin/traits") view=AdminTraitsPage ssr=SsrMode::Async />
                        <Route path=path!("/design") view=DesignGalleryPage ssr=SsrMode::Async />
                        <Route path=path!("/legal/impressum") view=ImpressumPage />
                        <Route path=path!("/legal/datenschutz") view=DatenschutzPage />
                    </Routes>
                </main>
                // The bug-report button docks above the footer (it self-gates
                // on auth and on public paths).
                <BugReportWidget />
                <SiteFooter />
            </ToastProvider>
        </Router>
    }
}
