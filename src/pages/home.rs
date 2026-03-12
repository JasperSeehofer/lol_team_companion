use leptos::prelude::*;
use leptos_router::components::A;

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

    view! {
        <Suspense fallback=|| view! {
            <div class="max-w-5xl mx-auto py-16 px-6">
                <div class="animate-pulse flex flex-col gap-6">
                    <div class="h-10 bg-elevated rounded w-64"></div>
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
                    <div class="max-w-4xl mx-auto py-16 px-6">
                        <ErrorBanner message=format!("Failed to load dashboard: {e}") />
                    </div>
                }.into_any(),
                Ok(data) if !data.logged_in => view! {
                    <LandingPage />
                }.into_any(),
                Ok(data) => view! {
                    <Dashboard data=data />
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn LandingPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto py-20 px-6 text-center">
            <h1 class="text-5xl font-bold text-primary mb-4 tracking-tight">
                "LoL Team Companion"
            </h1>
            <p class="text-muted text-lg mb-10 max-w-2xl mx-auto">
                "Draft planning, team stats, and strategic tools for competitive League of Legends teams."
            </p>
            <div class="flex gap-4 justify-center mb-16">
                <A href="/auth/register">
                    <div class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-8 py-3 transition-colors">
                        "Get Started"
                    </div>
                </A>
                <A href="/auth/login">
                    <div class="bg-elevated hover:bg-overlay text-primary font-medium rounded-lg px-8 py-3 border border-divider transition-colors">
                        "Sign In"
                    </div>
                </A>
            </div>

            <div class="grid grid-cols-3 gap-6 text-left">
                <div class="bg-elevated/50 border border-divider/50 rounded-xl p-6">
                    <div class="text-accent font-bold text-lg mb-2">"Draft Trees"</div>
                    <p class="text-muted text-sm">"Plan branching draft strategies. Navigate decisions in real-time during live games."</p>
                </div>
                <div class="bg-elevated/50 border border-divider/50 rounded-xl p-6">
                    <div class="text-accent font-bold text-lg mb-2">"Team Stats"</div>
                    <p class="text-muted text-sm">"Sync match history from the Riot API. Filter by roster, date, and more."</p>
                </div>
                <div class="bg-elevated/50 border border-divider/50 rounded-xl p-6">
                    <div class="text-accent font-bold text-lg mb-2">"Game Plans"</div>
                    <p class="text-muted text-sm">"Create matchup strategies with macro and role-specific sections."</p>
                </div>
            </div>
        </div>
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
        <div class="max-w-5xl mx-auto py-10 px-6 flex flex-col gap-8">
            // Welcome
            <div>
                <h1 class="text-3xl font-bold text-primary">
                    "Welcome back, " <span class="text-accent">{data.username.clone()}</span>
                </h1>
                {data.team_name.clone().map(|name| view! {
                    <p class="text-muted text-sm mt-1">"Team: " <span class="text-primary font-medium">{name}</span></p>
                })}
            </div>

            // Alerts
            {(data.pending_requests > 0).then(|| view! {
                <A href="/team/dashboard">
                    <div class="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 flex items-center gap-3 hover:bg-amber-500/15 transition-colors cursor-pointer">
                        <span class="bg-amber-500 text-primary text-xs font-bold rounded-full w-6 h-6 flex items-center justify-center">
                            {data.pending_requests}
                        </span>
                        <span class="text-amber-400 text-sm font-medium">
                            {format!("Pending join request{}", if data.pending_requests == 1 { "" } else { "s" })}
                        </span>
                    </div>
                </A>
            })}

            {(!data.has_team).then(|| view! {
                <A href="/team/roster">
                    <div class="bg-blue-500/10 border border-blue-500/30 rounded-xl p-4 flex items-center gap-3 hover:bg-blue-500/15 transition-colors cursor-pointer">
                        <span class="text-blue-400 text-sm font-medium">"You're not in a team yet. Create or join one to get started."</span>
                    </div>
                </A>
            })}

            {(!data.has_riot_key).then(|| view! {
                <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                    <p class="text-muted text-sm">"RIOT_API_KEY not set — stats syncing is disabled. Add it to .env."</p>
                </div>
            })}

            // Stats overview
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                <StatBox label="Roster" value=format!("{} members", data.roster_count) href="/team/dashboard" />
                <StatBox label="Drafts" value=format!("{}", data.draft_count + data.tree_count) href="/draft" />
                <StatBox label="Game Plans" value=format!("{}", data.plan_count) href="/game-plan" />
                <StatBox label="Recent Games" value=recent_games_display href="/stats" />
            </div>

            // Quick nav
            <div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
                <NavCard
                    href="/team/dashboard"
                    title="Team"
                    desc="Manage roster and team settings"
                    accent="border-l-blue-500"
                />
                <NavCard
                    href="/draft"
                    title="Draft Planner"
                    desc="Plan pick/ban phases"
                    accent="border-l-purple-500"
                />
                <NavCard
                    href="/tree-drafter"
                    title="Tree Drafter"
                    desc="Branching draft strategies"
                    accent="border-l-emerald-500"
                />
                <NavCard
                    href="/stats"
                    title="Stats"
                    desc="Match history and performance"
                    accent="border-l-cyan-500"
                />
                <NavCard
                    href="/game-plan"
                    title="Game Plans"
                    desc="Matchup strategy and tactics"
                    accent="border-l-yellow-500"
                />
                <NavCard
                    href="/post-game"
                    title="Post-Game"
                    desc="Reviews and pattern analysis"
                    accent="border-l-red-500"
                />
            </div>
        </div>
    }
}

#[component]
fn StatBox(label: &'static str, value: String, href: &'static str) -> impl IntoView {
    view! {
        <A href=href>
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 hover:bg-elevated transition-colors cursor-pointer">
                <div class="text-muted text-xs uppercase tracking-wider mb-1">{label}</div>
                <div class="text-primary text-xl font-bold">{value}</div>
            </div>
        </A>
    }
}

#[component]
fn NavCard(
    href: &'static str,
    title: &'static str,
    desc: &'static str,
    accent: &'static str,
) -> impl IntoView {
    view! {
        <A href=href>
            <div class=format!("bg-elevated/50 border border-divider/50 border-l-4 {accent} rounded-xl p-5 hover:bg-elevated transition-colors cursor-pointer h-full")>
                <div class="text-primary text-lg font-semibold mb-1">{title}</div>
                <div class="text-muted text-sm">{desc}</div>
            </div>
        </A>
    }
}
