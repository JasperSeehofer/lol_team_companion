use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::region::{GiltCorner, HeraldicDivider};
use crate::components::ui::ErrorBanner;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub logged_in: bool,
    pub username: String,
    pub has_team: bool,
    pub team_name: Option<String>,
    pub roster_count: usize,
    pub pending_requests: usize,
    pub is_leader: bool,
    pub draft_count: usize,
    pub tree_count: usize,
    pub plan_count: usize,
    pub review_count: usize,
    pub recent_games: usize,
    pub recent_win_rate: Option<String>,
    pub has_riot_key: bool,
}

#[server]
pub async fn get_dashboard() -> Result<DashboardData, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = match auth.user {
        Some(u) => u,
        None => {
            return Ok(DashboardData {
                logged_in: false,
                username: String::new(),
                has_team: false,
                team_name: None,
                roster_count: 0,
                pending_requests: 0,
                is_leader: false,
                draft_count: 0,
                tree_count: 0,
                plan_count: 0,
                review_count: 0,
                recent_games: 0,
                recent_win_rate: None,
                has_riot_key: riot::has_api_key(),
            })
        }
    };

    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_data = db::get_user_team_with_members(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (team_name, roster_count, is_leader, team_id_opt) = match &team_data {
        Some((team, members)) => (
            Some(team.name.clone()),
            members.len(),
            team.created_by == user.id,
            team.id.clone(),
        ),
        None => (None, 0, false, None),
    };

    let pending_requests = if is_leader {
        if let Some(ref tid) = team_id_opt {
            db::count_pending_join_requests(&surreal, tid)
                .await
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    let (draft_count, tree_count, plan_count, review_count, recent_games, recent_win_rate) =
        match &team_id_opt {
            Some(tid) => {
                let drafts = db::list_drafts(&surreal, tid).await.unwrap_or_default();
                let trees = db::list_draft_trees(&surreal, tid)
                    .await
                    .unwrap_or_default();
                let plans = db::list_game_plans(&surreal, tid).await.unwrap_or_default();
                let reviews = db::list_post_game_learnings(&surreal, tid)
                    .await
                    .unwrap_or_default();
                let match_rows = db::get_team_match_stats(&surreal, tid)
                    .await
                    .unwrap_or_default();

                // Compute recent win rate from unique matches
                let mut seen = std::collections::HashSet::new();
                let mut wins = 0usize;
                let mut total = 0usize;
                for r in &match_rows {
                    if seen.insert(r.riot_match_id.clone()) {
                        total += 1;
                        if r.win {
                            wins += 1;
                        }
                    }
                }
                let wr = if total > 0 {
                    Some(format!("{:.0}%", wins as f64 / total as f64 * 100.0))
                } else {
                    None
                };

                (
                    drafts.len(),
                    trees.len(),
                    plans.len(),
                    reviews.len(),
                    total,
                    wr,
                )
            }
            None => (0, 0, 0, 0, 0, None),
        };

    Ok(DashboardData {
        logged_in: true,
        username: user.username,
        has_team: team_data.is_some(),
        team_name,
        roster_count,
        pending_requests,
        is_leader,
        draft_count,
        tree_count,
        plan_count,
        review_count,
        recent_games,
        recent_win_rate,
        has_riot_key: riot::has_api_key(),
    })
}

#[component]
pub fn HomePage() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_dashboard());

    // D-14: Unauthenticated visitors are redirected to /closed-beta.
    // Authenticated users with no team are redirected to /team/roster.
    // Authenticated team members see the Strategy Room dashboard.
    Effect::new(move || {
        if let Some(Ok(data)) = dashboard.get() {
            if !data.logged_in {
                #[cfg(feature = "hydrate")]
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/closed-beta");
                }
            } else if !data.has_team {
                #[cfg(feature = "hydrate")]
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/team/roster");
                }
            }
        }
    });

    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6">
            <Suspense fallback=|| view! {
                <div class="max-w-5xl mx-auto py-12">
                    <div class="animate-pulse flex flex-col gap-6">
                        <div class="h-12 bg-elevated rounded w-72"></div>
                        <div class="h-6 bg-elevated rounded w-96"></div>
                        <div class="grid grid-cols-3 gap-4">
                            <div class="h-32 bg-elevated rounded-xl"></div>
                            <div class="h-32 bg-elevated rounded-xl"></div>
                            <div class="h-32 bg-elevated rounded-xl"></div>
                        </div>
                    </div>
                </div>
            }>
                {move || dashboard.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="max-w-4xl mx-auto py-16">
                            <ErrorBanner message=format!("Failed to load dashboard: {e}") />
                        </div>
                    }.into_any(),
                    Ok(data) if !data.logged_in => view! {
                        // Brief blank state while the redirect Effect fires.
                        // Plan 06 will style /closed-beta as the public landing.
                        <div class="max-w-4xl mx-auto py-16 text-center">
                            <div class="font-imperial uppercase tracking-[0.18em] text-[11px] text-muted">"Redirecting"</div>
                            <p class="text-muted text-sm mt-2">"Taking you to the closed beta page..."</p>
                        </div>
                    }.into_any(),
                    Ok(data) if !data.has_team => view! {
                        // Brief blank state while the redirect Effect fires.
                        <div class="max-w-4xl mx-auto py-16 text-center">
                            <div class="font-imperial uppercase tracking-[0.18em] text-[11px] text-muted">"Redirecting"</div>
                            <p class="text-muted text-sm mt-2">"Taking you to team setup..."</p>
                        </div>
                    }.into_any(),
                    Ok(data) => view! {
                        <Dashboard data=data />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// Card.gilt panel — Demacia-tier folio panel with 4 corner ornaments.
#[component]
fn GiltCard(children: Children) -> impl IntoView {
    view! {
        <section class="bg-elevated border border-outline rounded-xl p-6 relative">
            <div class="absolute top-2 left-2 pointer-events-none"><GiltCorner corner="tl" size=18 /></div>
            <div class="absolute top-2 right-2 pointer-events-none"><GiltCorner corner="tr" size=18 /></div>
            <div class="absolute bottom-2 left-2 pointer-events-none"><GiltCorner corner="bl" size=18 /></div>
            <div class="absolute bottom-2 right-2 pointer-events-none"><GiltCorner corner="br" size=18 /></div>
            <div class="relative">
                {children()}
            </div>
        </section>
    }
}

#[component]
fn Dashboard(data: DashboardData) -> impl IntoView {
    let recent_games_display = if data.recent_games > 0 {
        format!(
            "{} ({})",
            data.recent_games,
            data.recent_win_rate.clone().unwrap_or("–".into())
        )
    } else {
        "–".into()
    };

    view! {
        <div class="max-w-5xl mx-auto py-6 flex flex-col gap-8">
            // Strategy Room hero header
            <header>
                <div class="font-imperial uppercase tracking-[0.18em] text-[11px] text-accent">
                    "The Strategy Room"
                </div>
                <h1 class="font-display italic text-[44px] leading-tight text-primary mt-1">
                    "Welcome back, " <span class="text-accent">{data.username.clone()}</span>
                </h1>
                {data.team_name.clone().map(|name| view! {
                    <p class="text-muted text-sm mt-2 font-mono">
                        "Team \u{2014} " <span class="text-secondary font-medium">{name}</span>
                    </p>
                })}
                <div class="mt-3"><HeraldicDivider width=320 /></div>
            </header>

            // Alerts (gilt cards)
            {(data.pending_requests > 0).then(|| view! {
                <A href="/team/dashboard" attr:class="block focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-xl">
                    <div class="bg-warning/10 border border-warning/30 rounded-xl p-4 flex items-center gap-3 hover:bg-warning/15 transition-colors cursor-pointer">
                        <span class="bg-warning text-base text-xs font-bold rounded-full w-6 h-6 flex items-center justify-center tabular-nums" aria-hidden="true">
                            {data.pending_requests}
                        </span>
                        <span class="text-warning text-sm font-medium">
                            {format!("Pending join request{}", if data.pending_requests == 1 { "" } else { "s" })}
                        </span>
                    </div>
                </A>
            })}

            {(!data.has_riot_key).then(|| view! {
                <div class="bg-elevated border border-outline/50 rounded-xl p-4">
                    <p class="text-muted text-sm">
                        <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-warning mr-2">"Notice"</span>
                        "RIOT_API_KEY not set \u{2014} stats syncing is disabled. Add it to .env."
                    </p>
                </div>
            })}

            // Stats overview
            <section>
                <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-3">"At a glance"</div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                    <StatBox label="Roster" value=format!("{} members", data.roster_count) href="/team/dashboard" />
                    <StatBox label="Drafts" value=format!("{}", data.draft_count + data.tree_count) href="/draft" />
                    <StatBox label="Game Plans" value=format!("{}", data.plan_count) href="/game-plan" />
                    <StatBox label="Recent Games" value=recent_games_display href="/stats" />
                </div>
            </section>

            // Quick nav (gilt card per major hub)
            <section>
                <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-3">"The folio"</div>
                <div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
                    <NavCard
                        href="/team/dashboard"
                        title="Team"
                        desc="Manage roster and team settings"
                    />
                    <NavCard
                        href="/draft"
                        title="Draft Planner"
                        desc="Plan pick/ban phases"
                    />
                    <NavCard
                        href="/tree-drafter"
                        title="Tree Drafter"
                        desc="Branching draft strategies"
                    />
                    <NavCard
                        href="/stats"
                        title="Stats"
                        desc="Match history and performance"
                    />
                    <NavCard
                        href="/game-plan"
                        title="Game Plans"
                        desc="Matchup strategy and tactics"
                    />
                    <NavCard
                        href="/post-game"
                        title="Post-Game"
                        desc="Reviews and pattern analysis"
                    />
                </div>
            </section>

            // Recent activity gilt panel (placeholder — derived from existing counters)
            <GiltCard>
                <div class="flex items-baseline justify-between mb-4">
                    <h2 class="font-display italic text-[22px] text-primary">"Recent activity"</h2>
                    <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Folio"</span>
                </div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    <ActivityTile label="Drafts saved" value=format!("{}", data.draft_count) />
                    <ActivityTile label="Trees saved" value=format!("{}", data.tree_count) />
                    <ActivityTile label="Plans" value=format!("{}", data.plan_count) />
                    <ActivityTile label="Reviews" value=format!("{}", data.review_count) />
                </div>
            </GiltCard>
        </div>
    }
}

#[component]
fn StatBox(label: &'static str, value: String, href: &'static str) -> impl IntoView {
    view! {
        <A href=href attr:class="block focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-xl">
            <div class="bg-elevated border border-outline/50 rounded-xl p-4 hover:bg-surface transition-colors cursor-pointer">
                <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">{label}</div>
                <div class="font-display italic text-primary text-2xl tabular-nums">{value}</div>
            </div>
        </A>
    }
}

#[component]
fn NavCard(
    href: &'static str,
    title: &'static str,
    desc: &'static str,
) -> impl IntoView {
    view! {
        <A href=href attr:class="block focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-xl">
            <div class="bg-elevated border border-outline/50 border-l-4 border-l-accent rounded-xl p-5 hover:bg-surface transition-colors cursor-pointer h-full">
                <div class="font-display italic text-primary text-xl mb-1">{title}</div>
                <div class="text-muted text-sm">{desc}</div>
            </div>
        </A>
    }
}

#[component]
fn ActivityTile(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline/50 rounded-lg px-3 py-2 flex flex-col gap-1">
            <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">{label}</span>
            <span class="font-display italic text-primary text-2xl tabular-nums">{value}</span>
        </div>
    }
}
