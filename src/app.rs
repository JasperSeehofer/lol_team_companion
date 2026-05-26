use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::bug_report_widget::BugReportWidget;
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

    // Phase 18.2 Plan 04 — Hydration-side InitialTheme provider.
    //
    // The SSR render path receives `InitialTheme` via
    // `leptos_routes_with_context` in `main.rs` (the closure calls
    // `provide_context(theme)` where `theme` is the request-scoped
    // `REQUEST_THEME` task-local resolved by `theme_injection_middleware`).
    //
    // On the WASM hydrate path, `hydrate_body(App)` mounts this `App`
    // component directly — `leptos_routes_with_context` does NOT run in
    // the browser. Without an equivalent hydrate-side provider, every
    // descendant `use_context::<InitialTheme>()` returns `None` and
    // falls back to the `"demacia"` default. Pages like `Nav`,
    // `DraftPage`, `SoloDashboardPage`, etc. read the context, derive a
    // `region: String`, and pass it down to region-branching primitives
    // (`Btn`, `ModeToggle`, `Card`, `Glitch`, `SectionHead`, ...). When
    // SSR computed `region = "pandemonium"` and WASM hydrate computed
    // `region = "demacia"`, every divergent primitive arm causes a
    // structural mismatch that propagates up to `tachys-0.2.14/src/
    // html/mod.rs:217 InertElement::hydrate Option::unwrap() on None`.
    //
    // Fix: on hydrate, read `<html data-theme="...">` (which the SSR
    // shell rendered with the same resolved theme value) and provide
    // it as context BEFORE the `view!` macro instantiates Routes /
    // child components. The DOM attribute is the canonical SSR→WASM
    // serialization channel — it survives across page navigations and
    // is identical to what the SSR `theme_injection_middleware`
    // resolved for the same request (D-02 precedence).
    //
    // On SSR, no provide is needed here because the outer
    // `leptos_routes_with_context` closure already provided it. We
    // gate the hydrate-side provide behind `cfg(feature = "hydrate")`
    // because `web_sys::window()` is WASM-only and would not link in
    // the SSR build; the SSR owner already has `InitialTheme` from the
    // parent scope so descendants resolve correctly.
    #[cfg(feature = "hydrate")]
    {
        let theme = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .and_then(|el| el.get_attribute("data-theme"))
            .filter(|v| v == "demacia" || v == "pandemonium")
            .unwrap_or_else(|| "demacia".to_string());
        provide_context(InitialTheme(theme));
    }

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
                // Floating bug-report widget (Phase 17 visual stub;
                // Phase 18 wires submit-to-DB). Self-gates on auth +
                // pathname so it never shows on /, /auth/*,
                // /closed-beta, /legal/* per UI-SPEC line 590.
                <BugReportWidget />
            </ToastProvider>
        </Router>
    }
}
