use crate::models::game_plan::AnalyticsPayload;
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_analytics_data() -> Result<AnalyticsPayload, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // Return empty payload if user has no team (per CLAUDE.md rule 44)
    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => {
            return Ok(AnalyticsPayload {
                tag_summaries: Vec::new(),
                plan_effectiveness: Vec::new(),
            })
        }
    };

    let (tag_summaries, plan_effectiveness) = db::get_analytics(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(AnalyticsPayload {
        tag_summaries,
        plan_effectiveness,
    })
}

// ---------------------------------------------------------------------------
// Helper types for sort state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum SortColumn {
    WinLoss,
    Rating,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Map strategy tags to semantic-token color pairs (bg+border classes, text class).
///
/// All raw color literals (red/blue/violet/orange/emerald/cyan/amber) were
/// retired in plan 17-04 in favour of theme-aware semantic tokens. The mapping
/// preserves the editorial intent (warm/cool/neutral) while allowing the
/// pandemonium theme to recolor without code changes.
fn tag_colors(tag: &str) -> (&'static str, &'static str) {
    match tag {
        "teamfight" => ("bg-danger/10 border border-danger/30", "text-danger"),
        "split-push" => ("bg-info/10 border border-info/30", "text-info"),
        "poke" => ("bg-accent/10 border border-accent/30", "text-accent"),
        "engage" => ("bg-warning/10 border border-warning/30", "text-warning"),
        "protect-the-adc" => ("bg-success/10 border border-success/30", "text-success"),
        "scaling" => ("bg-info/10 border border-info/30", "text-info"),
        "skirmish" => ("bg-warning/10 border border-warning/30", "text-warning"),
        _ => ("bg-elevated border border-divider", "text-muted"),
    }
}

fn stars_display(rating: Option<u8>) -> String {
    match rating {
        None => "\u{2014}".to_string(),
        Some(r) => {
            let r = r.min(5) as usize;
            let filled = "\u{2605}".repeat(r);
            let empty = "\u{2606}".repeat(5usize.saturating_sub(r));
            format!("{filled}{empty}")
        }
    }
}

fn rating_to_stars(avg: Option<f32>) -> String {
    match avg {
        None => "\u{2014}".to_string(),
        Some(v) => {
            let r = v.round() as usize;
            let r = r.min(5);
            let filled = "\u{2605}".repeat(r);
            let empty = "\u{2606}".repeat(5usize.saturating_sub(r));
            format!("{filled}{empty}")
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    // Auth guard
    #[allow(unused_variables)]
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(Ok(None)) = user.get() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let analytics = Resource::new(|| (), |_| get_analytics_data());

    // Sort state: (column, direction) — default most wins first
    let sort_state: RwSignal<(SortColumn, SortDir)> =
        RwSignal::new((SortColumn::WinLoss, SortDir::Desc));

    // Accordion: open plan id
    let open_plan: RwSignal<Option<String>> = RwSignal::new(None);

    view! {
        <div class="canvas-grain bg-base min-h-screen">
            <main class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
                // Imperial header
                <div class="flex flex-col gap-2 mb-6">
                    <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "The ledger"
                    </span>
                    <h1 class="font-display italic text-primary text-3xl">"Analytics"</h1>
                    <p class="text-muted text-sm">"Track strategy effectiveness and plan outcomes."</p>
                </div>

                <Suspense fallback=|| view! { <p class="text-muted py-8">"Loading analytics..."</p> }>
                    {move || {
                        Suspend::new(async move {
                            match analytics.await {
                                Err(_) => view! {
                                    <div class="bg-elevated border border-divider rounded-xl py-12 px-6 text-center">
                                        <p class="text-muted text-sm">
                                            "Failed to load analytics \u{2014} try refreshing the page"
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(payload) => {
                                    let no_team = payload.tag_summaries.is_empty()
                                        && payload.plan_effectiveness.is_empty();

                                    if no_team {
                                        view! {
                                            <div class="bg-elevated border border-divider rounded-xl py-16 px-6 text-center">
                                                <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"No analytics yet"</span>
                                                <h2 class="font-display italic text-primary text-2xl mt-2">"No data to chart."</h2>
                                                <p class="text-muted text-sm mt-3 max-w-md mx-auto">
                                                    "Create or join a team to get started."
                                                </p>
                                                <a
                                                    href="/team/roster"
                                                    class="inline-flex items-center mt-5 bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-5 py-2 rounded-lg text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                >
                                                    "Go to team roster"
                                                </a>
                                            </div>
                                        }.into_any()
                                    } else if payload.tag_summaries.is_empty() && !payload.plan_effectiveness.is_empty() {
                                        // Has plans but no tagged reviews
                                        view! {
                                            <div class="bg-elevated border border-divider rounded-xl py-16 px-6 text-center">
                                                <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Awaiting reviews"</span>
                                                <h2 class="font-display italic text-primary text-2xl mt-2">"No plan effectiveness data yet."</h2>
                                                <p class="text-muted text-sm mt-3 max-w-md mx-auto">
                                                    "Link post-game reviews to game plans to start tracking strategy outcomes."
                                                </p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let tag_summaries = payload.tag_summaries.clone();
                                        let plan_effectiveness = payload.plan_effectiveness.clone();

                                        view! {
                                            // Strategy Tag Cards
                                            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
                                                {tag_summaries.into_iter().filter(|s| s.games_played > 0).map(|s| {
                                                    let (bg_border, text_color) = tag_colors(&s.tag);
                                                    let win_pct = if s.games_played > 0 {
                                                        s.wins * 100 / s.games_played
                                                    } else {
                                                        0
                                                    };
                                                    let stars = rating_to_stars(s.avg_rating);
                                                    let tag_label = s.tag.clone();
                                                    let wl = format!("{}-{}", s.wins, s.losses);
                                                    let games = s.games_played;
                                                    view! {
                                                        <div class=format!("rounded-xl p-4 {bg_border}")>
                                                            <div class=format!("font-imperial uppercase tracking-wider text-[10px] mb-2 {text_color}")>
                                                                {tag_label}
                                                            </div>
                                                            <div class="font-display italic text-primary text-3xl tabular-nums">
                                                                {format!("{win_pct}%")}
                                                            </div>
                                                            <div class="flex items-center gap-3 mt-1">
                                                                <span class="font-mono text-secondary text-sm tabular-nums">{wl}</span>
                                                                <span class="text-accent text-sm">{stars}</span>
                                                            </div>
                                                            <div class="font-mono text-dimmed text-xs mt-1 tabular-nums">
                                                                {format!("{games} games")}
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>

                                            // Game Plan Effectiveness Table
                                            <div class="mt-10 mb-4 flex flex-col gap-1">
                                                <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"By plan"</span>
                                                <h2 class="font-display italic text-primary text-2xl">"Game plan effectiveness"</h2>
                                            </div>
                                            <div class="bg-elevated border border-divider rounded-xl overflow-hidden">
                                                <table class="w-full">
                                                    <thead class="bg-surface/50">
                                                        <tr>
                                                            <th class="px-4 py-4 text-left font-imperial uppercase tracking-wider text-[10px] text-muted">
                                                                "Plan"
                                                            </th>
                                                            <th class="px-4 py-4 text-left font-imperial uppercase tracking-wider text-[10px] text-muted">
                                                                "Tag"
                                                            </th>
                                                            <th
                                                                class=move || {
                                                                    let (col, _) = sort_state.get();
                                                                    if col == SortColumn::WinLoss {
                                                                        "px-4 py-4 text-center font-imperial uppercase tracking-wider text-[10px] text-primary cursor-pointer hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    } else {
                                                                        "px-4 py-4 text-center font-imperial uppercase tracking-wider text-[10px] text-muted cursor-pointer hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    }
                                                                }
                                                                on:click=move |_| {
                                                                    sort_state.update(|(col, dir)| {
                                                                        if *col == SortColumn::WinLoss {
                                                                            *dir = if *dir == SortDir::Asc { SortDir::Desc } else { SortDir::Asc };
                                                                        } else {
                                                                            *col = SortColumn::WinLoss;
                                                                            *dir = SortDir::Desc;
                                                                        }
                                                                    });
                                                                }
                                                            >
                                                                "W-L"
                                                                {move || {
                                                                    let (col, dir) = sort_state.get();
                                                                    if col == SortColumn::WinLoss {
                                                                        if dir == SortDir::Asc {
                                                                            view! { <span class="text-accent">" \u{2191}"</span> }.into_any()
                                                                        } else {
                                                                            view! { <span class="text-accent">" \u{2193}"</span> }.into_any()
                                                                        }
                                                                    } else {
                                                                        view! { <span></span> }.into_any()
                                                                    }
                                                                }}
                                                            </th>
                                                            <th
                                                                class=move || {
                                                                    let (col, _) = sort_state.get();
                                                                    if col == SortColumn::Rating {
                                                                        "px-4 py-4 text-center font-imperial uppercase tracking-wider text-[10px] text-primary cursor-pointer hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    } else {
                                                                        "px-4 py-4 text-center font-imperial uppercase tracking-wider text-[10px] text-muted cursor-pointer hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    }
                                                                }
                                                                on:click=move |_| {
                                                                    sort_state.update(|(col, dir)| {
                                                                        if *col == SortColumn::Rating {
                                                                            *dir = if *dir == SortDir::Asc { SortDir::Desc } else { SortDir::Asc };
                                                                        } else {
                                                                            *col = SortColumn::Rating;
                                                                            *dir = SortDir::Desc;
                                                                        }
                                                                    });
                                                                }
                                                            >
                                                                "Avg rating"
                                                                {move || {
                                                                    let (col, dir) = sort_state.get();
                                                                    if col == SortColumn::Rating {
                                                                        if dir == SortDir::Asc {
                                                                            view! { <span class="text-accent">" \u{2191}"</span> }.into_any()
                                                                        } else {
                                                                            view! { <span class="text-accent">" \u{2193}"</span> }.into_any()
                                                                        }
                                                                    } else {
                                                                        view! { <span></span> }.into_any()
                                                                    }
                                                                }}
                                                            </th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {move || {
                                                            let (col, dir) = sort_state.get();
                                                            let mut sorted = plan_effectiveness.clone();
                                                            sorted.sort_by(|a, b| {
                                                                let cmp = match col {
                                                                    SortColumn::WinLoss => a.wins.cmp(&b.wins),
                                                                    SortColumn::Rating => {
                                                                        let ra = a.avg_rating.unwrap_or(0.0);
                                                                        let rb = b.avg_rating.unwrap_or(0.0);
                                                                        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
                                                                    }
                                                                };
                                                                if dir == SortDir::Desc { cmp.reverse() } else { cmp }
                                                            });

                                                            sorted.into_iter().map(|plan| {
                                                                let plan_id = plan.plan_id.clone();
                                                                let plan_id_toggle = plan_id.clone();
                                                                let plan_id_chevron = plan_id.clone();
                                                                let plan_id_accordion = plan_id.clone();
                                                                let plan_name = plan.plan_name.clone();
                                                                let tag = plan.tag.clone();
                                                                let wins = plan.wins;
                                                                let losses = plan.losses;
                                                                let avg_rating = plan.avg_rating;
                                                                let reviews = plan.reviews.clone();

                                                                let (tag_bg, tag_text) = match &tag {
                                                                    Some(t) => tag_colors(t),
                                                                    None => ("bg-elevated border border-divider", "text-muted"),
                                                                };

                                                                view! {
                                                                    // Data row
                                                                    <tr
                                                                        class="border-t border-divider hover:bg-overlay/30 cursor-pointer transition-colors"
                                                                        on:click=move |_| {
                                                                            let id = plan_id_toggle.clone();
                                                                            open_plan.update(|current| {
                                                                                if current.as_deref() == Some(&id) {
                                                                                    *current = None;
                                                                                } else {
                                                                                    *current = Some(id);
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        <td class="px-4 py-4 text-sm text-primary">
                                                                            {plan_name}
                                                                            " "
                                                                            <span class="text-muted text-xs" aria-hidden="true">
                                                                                {move || {
                                                                                    if open_plan.get().as_deref() == Some(&plan_id_chevron) {
                                                                                        "\u{25BC}"
                                                                                    } else {
                                                                                        "\u{25B6}"
                                                                                    }
                                                                                }}
                                                                            </span>
                                                                        </td>
                                                                        <td class="px-4 py-4">
                                                                            <span class=format!("font-imperial uppercase tracking-wider text-[10px] px-2 py-1 rounded {tag_bg} {tag_text}")>
                                                                                {match &tag {
                                                                                    Some(t) => t.clone(),
                                                                                    None => "\u{2014}".to_string(),
                                                                                }}
                                                                            </span>
                                                                        </td>
                                                                        <td class="px-4 py-4 text-center font-mono text-sm tabular-nums">
                                                                            <span class="text-success">{wins}</span>
                                                                            <span class="text-muted mx-1">"-"</span>
                                                                            <span class="text-danger">{losses}</span>
                                                                        </td>
                                                                        <td class="px-4 py-4 text-center text-sm text-accent">
                                                                            {rating_to_stars(avg_rating)}
                                                                        </td>
                                                                    </tr>

                                                                    // Accordion expansion
                                                                    {move || {
                                                                        let plan_id_check = plan_id_accordion.clone();
                                                                        let reviews_inner = reviews.clone();
                                                                        if open_plan.get().as_deref() == Some(&plan_id_check) {
                                                                            view! {
                                                                                <tr class="border-t border-divider bg-surface/30">
                                                                                    <td colspan="4" class="px-4 py-4">
                                                                                        {if reviews_inner.is_empty() {
                                                                                            view! {
                                                                                                <p class="text-dimmed text-sm">"No reviews linked to this plan."</p>
                                                                                            }.into_any()
                                                                                        } else {
                                                                                            view! {
                                                                                                <div>
                                                                                                    {reviews_inner.into_iter().map(|review| {
                                                                                                        let outcome_class = match review.win_loss.as_deref() {
                                                                                                            Some("win") => "text-success",
                                                                                                            Some("loss") => "text-danger",
                                                                                                            _ => "text-muted",
                                                                                                        };
                                                                                                        let outcome_label = match review.win_loss.as_deref() {
                                                                                                            Some("win") => "WIN",
                                                                                                            Some("loss") => "LOSS",
                                                                                                            _ => "\u{2014}",
                                                                                                        };
                                                                                                        let stars = stars_display(review.rating);
                                                                                                        let note = review.improvements.first()
                                                                                                            .cloned()
                                                                                                            .unwrap_or_else(|| "No notes".to_string());
                                                                                                        view! {
                                                                                                            <div class="border-l-2 border-accent pl-3 mb-2 last:mb-0">
                                                                                                                <div class="flex items-center gap-3">
                                                                                                                    <span class=format!("font-imperial uppercase tracking-wider text-[10px] {outcome_class}")>
                                                                                                                        {outcome_label}
                                                                                                                    </span>
                                                                                                                    <span class="text-accent text-xs">{stars}</span>
                                                                                                                </div>
                                                                                                                <p class="text-secondary text-sm mt-1 line-clamp-2">{note}</p>
                                                                                                            </div>
                                                                                                        }
                                                                                                    }).collect_view()}
                                                                                                </div>
                                                                                            }.into_any()
                                                                                        }}
                                                                                    </td>
                                                                                </tr>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! { <tr></tr> }.into_any()
                                                                        }
                                                                    }}
                                                                }
                                                            }).collect_view()
                                                        }}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    }
                                }
                            }
                        })
                    }}
                </Suspense>
            </main>
        </div>
    }
}
