use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::use_location,
};

use crate::components::region::CompanionSigil;
use crate::components::theme_toggle::ThemeToggle;
use crate::models::user::JoinRequest;
use crate::pages::profile::{get_current_user, Logout};
use crate::pages::team::dashboard::handle_join_request;

#[server]
pub async fn set_user_mode(mode: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_mode(&db, &user.id, &mode)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[component]
pub fn ModeToggle(mode: String) -> impl IntoView {
    let current_mode = RwSignal::new(mode);

    let on_click_solo = move |_| {
        let m = current_mode.get_untracked();
        if m == "solo" {
            return;
        }
        current_mode.set("solo".to_string());
        leptos::task::spawn_local(async move {
            let _ = set_user_mode("solo".to_string()).await;
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    };

    let on_click_team = move |_| {
        let m = current_mode.get_untracked();
        if m == "team" {
            return;
        }
        current_mode.set("team".to_string());
        leptos::task::spawn_local(async move {
            let _ = set_user_mode("team".to_string()).await;
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    };

    view! {
        <div class="bg-elevated rounded-lg p-0.5 flex">
            <button
                class=move || {
                    if current_mode.get() == "solo" {
                        "bg-accent text-accent-contrast font-semibold px-3 py-1 text-sm rounded-md cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    } else {
                        "text-muted hover:text-secondary px-3 py-1 text-sm rounded-md cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    }
                }
                on:click=on_click_solo
            >
                "Solo"
            </button>
            <button
                class=move || {
                    if current_mode.get() == "team" {
                        "bg-accent text-accent-contrast font-semibold px-3 py-1 text-sm rounded-md cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    } else {
                        "text-muted hover:text-secondary px-3 py-1 text-sm rounded-md cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    }
                }
                on:click=on_click_team
            >
                "Team"
            </button>
        </div>
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Notifications {
    pub pending_requests: Vec<JoinRequest>,
    pub is_leader: bool,
}

#[server]
pub async fn get_notifications() -> Result<Notifications, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = match auth.user {
        Some(u) => u,
        None => return Ok(Notifications::default()),
    };
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let result = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (team, _) = match result {
        Some(t) => t,
        None => return Ok(Notifications::default()),
    };

    let is_leader = team.created_by == user.id;
    if !is_leader {
        return Ok(Notifications {
            pending_requests: Vec::new(),
            is_leader: false,
        });
    }

    let team_id = match team.id {
        Some(id) => id,
        None => {
            return Ok(Notifications {
                pending_requests: Vec::new(),
                is_leader: true,
            })
        }
    };

    let pending = db::list_pending_join_requests(&db, &team_id)
        .await
        .unwrap_or_default();

    Ok(Notifications {
        pending_requests: pending,
        is_leader: true,
    })
}

// ---------------------------------------------------------------------------
// 4-Hub Information Architecture (D-09)
// ---------------------------------------------------------------------------

/// Hub-grouped sub-routes. Each hub maps to its sub-nav children.
/// `live` hub omitted from sub-nav (single-page hub, deferred feature).
const HUB_ROUTES: &[(&str, &[(&str, &str)])] = &[
    (
        "strategy",
        &[
            ("/draft", "Draft"),
            ("/tree-drafter", "Tree"),
            ("/champion-pool", "Pool"),
            ("/game-plan", "Game plan"),
            ("/post-game", "Post-game"),
            ("/opponents", "Opponents"),
            ("/action-items", "Action items"),
        ],
    ),
    (
        "history",
        &[
            ("/stats", "Stats"),
            ("/match", "Match"),
            ("/personal-learnings", "Learnings"),
            ("/analytics", "Analytics"),
        ],
    ),
    (
        "profile",
        &[
            ("/profile", "Profile"),
            ("/team/dashboard", "Team"),
            ("/team/roster", "Roster"),
            ("/team-builder", "Team builder"),
            ("/solo", "Solo"),
        ],
    ),
];

/// Map a URL path to its primary hub. Returns `""` for paths outside any hub
/// (e.g. `/`, `/auth/*`, `/closed-beta`, `/legal/*`) so the sub-nav strip
/// suppresses itself on those routes.
fn hub_for_path(path: &str) -> &'static str {
    if path.starts_with("/draft")
        || path.starts_with("/tree-drafter")
        || path.starts_with("/champion-pool")
        || path.starts_with("/game-plan")
        || path.starts_with("/post-game")
        || path.starts_with("/opponents")
        || path.starts_with("/action-items")
    {
        "strategy"
    } else if path.starts_with("/stats")
        || path.starts_with("/match")
        || path.starts_with("/personal-learnings")
        || path.starts_with("/analytics")
    {
        "history"
    } else if path.starts_with("/profile")
        || path.starts_with("/team")
        || path.starts_with("/solo")
    {
        "profile"
    } else if path.starts_with("/live") {
        "live"
    } else {
        ""
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let menu_open = RwSignal::new(false);
    let notif_open = RwSignal::new(false);

    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());
    let notifications = Resource::new(|| (), |_| get_notifications());

    let close_all = move || {
        menu_open.set(false);
        notif_open.set(false);
    };

    Effect::new(move || {
        if let Some(Ok(())) = logout_action.value().get() {
            close_all();
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });

    // Escape key listener to close all dropdowns
    #[cfg(feature = "hydrate")]
    {
        let close_all_esc = close_all.clone();
        Effect::new(move |_| {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        close_all_esc();
                    }
                },
            );
            if let Some(window) = web_sys::window() {
                let _ =
                    window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            }
            cb.forget();
        });
    }

    let any_dropdown_open = move || menu_open.get() || notif_open.get();

    // Reactive hub derivation from current URL path.
    let location = use_location();
    let active_hub =
        Signal::derive(move || hub_for_path(&location.pathname.get()).to_string());

    // 4-hub primary buttons. `Live` is visually disabled (deferred feature).
    let hub_btn_class = move |hub_id: &'static str, disabled: bool| -> Memo<String> {
        let active_hub = active_hub.clone();
        Memo::new(move |_| {
            let active = active_hub.get() == hub_id;
            let base = "px-4 py-2 rounded-md font-imperial text-[10px] uppercase tracking-[0.18em] transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none";
            if disabled {
                format!("{base} text-dimmed pointer-events-none opacity-50")
            } else if active {
                format!("{base} bg-accent text-accent-contrast font-semibold cursor-pointer")
            } else {
                format!("{base} text-muted hover:text-secondary cursor-pointer")
            }
        })
    };

    let strategy_btn_cls = hub_btn_class("strategy", false);
    let live_btn_cls = hub_btn_class("live", true);
    let history_btn_cls = hub_btn_class("history", false);
    let profile_btn_cls = hub_btn_class("profile", false);

    view! {
        <header class="sticky top-0 z-50 bg-surface/80 backdrop-blur-md border-b border-divider">
            <div class="max-w-7xl mx-auto px-4 sm:px-8">
                // ── Top row: Sigil · Hubs · Notifications · Theme · User ──
                <div class="flex items-center gap-7 h-14">
                    // Sigil → home
                    <A href="/" attr:class="flex items-center shrink-0 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded">
                        <CompanionSigil />
                    </A>

                    // 4 primary hubs
                    <nav class="hidden md:flex items-center gap-1" aria-label="Primary">
                        <A href="/draft" attr:class=move || strategy_btn_cls.get()>
                            "Strategy"
                        </A>
                        // Live hub disabled (deferred feature)
                        <span class=move || live_btn_cls.get() aria-disabled="true" title="Live coming soon">
                            "Live"
                        </span>
                        <A href="/stats" attr:class=move || history_btn_cls.get()>
                            "History"
                        </A>
                        <A href="/profile" attr:class=move || profile_btn_cls.get()>
                            "Profile"
                        </A>
                    </nav>

                    <div class="flex-1" />

                    // Right cluster: ModeToggle (auth-only) · ThemeToggle · Notifications · User menu
                    <div class="flex items-center gap-3">
                        <Suspense fallback=|| ()>
                            {move || Suspend::new(async move {
                                match user.await {
                                    Ok(Some(u)) => {
                                        let theme = u.theme.clone();
                                        view! {
                                            <ModeToggle mode=u.mode />
                                            <ThemeToggle initial_theme=theme />
                                        }.into_any()
                                    },
                                    _ => view! { <ThemeToggle /> }.into_any(),
                                }
                            })}
                        </Suspense>

                        // Notifications bell (unchanged from previous nav, focus-visible added)
                        <Suspense fallback=|| ()>
                            {move || {
                                let notifs = notifications.get().and_then(|r| r.ok()).unwrap_or_default();
                                let count = notifs.pending_requests.len();
                                if count == 0 {
                                    return view! { <span></span> }.into_any();
                                }
                                let reqs = notifs.pending_requests;
                                view! {
                                    <div class="relative">
                                        <button
                                            on:click=move |_| {
                                                notif_open.update(|v| *v = !*v);
                                                menu_open.set(false);
                                            }
                                            class="relative text-muted hover:text-primary transition-colors p-1.5 rounded-lg hover:bg-elevated cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                            aria-label="Notifications"
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                            </svg>
                                            <span class="absolute -top-0.5 -right-0.5 bg-red-500 text-white text-[10px] font-bold rounded-full min-w-[16px] h-4 flex items-center justify-center px-1 leading-none">
                                                {count}
                                            </span>
                                        </button>

                                        <div
                                            class="absolute right-0 mt-2 w-80 bg-elevated border border-divider rounded-xl shadow-xl overflow-hidden z-[60]"
                                            style:display=move || if notif_open.get() { "block" } else { "none" }
                                        >
                                            <div class="px-4 py-3 border-b border-divider">
                                                <span class="text-primary text-sm font-semibold">"Notifications"</span>
                                            </div>
                                            <div class="max-h-64 overflow-y-auto">
                                                {reqs.into_iter().map(|req| {
                                                    let req_id_accept = req.id.clone();
                                                    let req_id_decline = req.id.clone();
                                                    view! {
                                                        <div class="px-4 py-3 border-b border-divider/50 flex items-center justify-between gap-2">
                                                            <div class="min-w-0">
                                                                <p class="text-primary text-sm font-medium truncate">{req.username}</p>
                                                                {req.riot_summoner_name.map(|n| view! {
                                                                    <p class="text-muted text-xs truncate">{n}</p>
                                                                })}
                                                                <p class="text-dimmed text-xs">"Wants to join"</p>
                                                            </div>
                                                            <div class="flex gap-1.5 flex-shrink-0">
                                                                <button
                                                                    class="bg-green-700 hover:bg-green-600 text-white text-xs font-medium rounded px-2 py-1 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    title="Accept"
                                                                    on:click=move |_| {
                                                                        let id = req_id_accept.clone();
                                                                        leptos::task::spawn_local(async move {
                                                                            let _ = handle_join_request(id, true).await;
                                                                            notifications.refetch();
                                                                        });
                                                                        notif_open.set(false);
                                                                    }
                                                                >
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                                                    </svg>
                                                                </button>
                                                                <button
                                                                    class="bg-overlay-strong hover:bg-red-700 text-secondary hover:text-primary text-xs font-medium rounded px-2 py-1 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    title="Decline"
                                                                    on:click=move |_| {
                                                                        let id = req_id_decline.clone();
                                                                        leptos::task::spawn_local(async move {
                                                                            let _ = handle_join_request(id, false).await;
                                                                            notifications.refetch();
                                                                        });
                                                                        notif_open.set(false);
                                                                    }
                                                                >
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                                                    </svg>
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                            <A
                                                href="/team/dashboard"
                                                attr:class="block px-4 py-2.5 text-center text-muted hover:text-primary text-xs transition-colors"
                                                on:click=move |_| notif_open.set(false)
                                            >
                                                "View all on Team Dashboard"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            }}
                        </Suspense>

                        // User menu / sign-in
                        <div class="relative text-sm">
                            <Suspense fallback=move || view! {
                                <span class="text-dimmed text-sm">"..."</span>
                            }>
                                {move || Suspend::new(async move {
                                    match user.await {
                                        Ok(Some(u)) => view! {
                                            <button
                                                on:click=move |_| {
                                                    menu_open.update(|v| *v = !*v);
                                                    notif_open.set(false);
                                                }
                                                class="flex items-center gap-2 text-secondary hover:text-primary transition-colors cursor-pointer px-2 py-1 rounded-lg hover:bg-elevated focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                            >
                                                <span class="bg-accent/20 text-accent rounded-full w-7 h-7 flex items-center justify-center text-xs font-bold uppercase">
                                                    {u.username.chars().next().unwrap_or('?').to_string()}
                                                </span>
                                                <span class="hidden sm:inline text-sm font-medium">{u.username}</span>
                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 text-dimmed" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
                                                </svg>
                                            </button>
                                            <div
                                                class="absolute right-0 mt-2 w-44 bg-elevated border border-divider rounded-xl shadow-xl overflow-hidden z-[60]"
                                                style:display=move || if menu_open.get() { "block" } else { "none" }
                                            >
                                                <A
                                                    href="/profile"
                                                    attr:class="block px-4 py-2.5 text-secondary hover:bg-elevated hover:text-primary transition-colors text-sm"
                                                    on:click=move |_| menu_open.set(false)
                                                >
                                                    "Profile"
                                                </A>
                                                <A
                                                    href="/champion-pool"
                                                    attr:class="block px-4 py-2.5 text-secondary hover:bg-elevated hover:text-primary transition-colors text-sm"
                                                    on:click=move |_| menu_open.set(false)
                                                >
                                                    "Champion Pool"
                                                </A>
                                                <div class="border-t border-divider"></div>
                                                <ActionForm action=logout_action>
                                                    <button
                                                        type="submit"
                                                        class="block w-full text-left px-4 py-2.5 text-red-400 hover:bg-elevated hover:text-red-300 transition-colors cursor-pointer text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                    >
                                                        "Sign Out"
                                                    </button>
                                                </ActionForm>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <div class="flex items-center gap-2 text-sm">
                                                <A href="/auth/login" attr:class="text-secondary hover:text-primary transition-colors px-3 py-1.5 rounded-lg hover:bg-elevated focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none">
                                                    "Sign In"
                                                </A>
                                                <A href="/auth/register" attr:class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-3 py-1.5 rounded-lg transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none">
                                                    "Register"
                                                </A>
                                            </div>
                                        }.into_any(),
                                    }
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>

                // ── Sub-nav strip — driven by use_location() ──
                {move || {
                    let hub = active_hub.get();
                    if hub.is_empty() || hub == "live" {
                        view! { <div class="hidden" /> }.into_any()
                    } else {
                        let routes: &[(&str, &str)] = HUB_ROUTES
                            .iter()
                            .find(|(h, _)| *h == hub)
                            .map(|(_, r)| *r)
                            .unwrap_or(&[]);
                        view! {
                            <nav class="flex gap-1 py-2 border-t border-divider/30 overflow-x-auto" aria-label="Sub-navigation">
                                {routes.iter().map(|(path, label)| {
                                    let path_owned = path.to_string();
                                    let path_for_class = path_owned.clone();
                                    let location = use_location();
                                    let is_active = move || location.pathname.get().starts_with(&path_for_class);
                                    view! {
                                        <A
                                            href=path_owned
                                            attr:class=move || {
                                                if is_active() {
                                                    "px-3 py-1.5 rounded-md bg-accent-soft text-accent text-sm font-semibold focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none whitespace-nowrap"
                                                } else {
                                                    "px-3 py-1.5 rounded-md text-dimmed hover:text-muted text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none whitespace-nowrap"
                                                }
                                            }
                                        >
                                            {*label}
                                        </A>
                                    }
                                }).collect_view()}
                            </nav>
                        }.into_any()
                    }
                }}
            </div>
        </header>

        // Click-outside backdrop: covers full screen behind dropdowns
        {move || {
            if any_dropdown_open() {
                view! {
                    <div
                        class="fixed inset-0 z-40"
                        on:click=move |_| close_all()
                    />
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }
        }}
    }
}
