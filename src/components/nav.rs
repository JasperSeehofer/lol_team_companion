use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::theme_toggle::ThemeToggle;
use crate::models::user::JoinRequest;
use crate::pages::profile::{get_current_user, Logout};
use crate::pages::team::dashboard::handle_join_request;

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

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let menu_open = RwSignal::new(false);
    let notif_open = RwSignal::new(false);
    let mobile_open = RwSignal::new(false);

    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());
    let notifications = Resource::new(|| (), |_| get_notifications());

    let close_all = move || {
        menu_open.set(false);
        notif_open.set(false);
        mobile_open.set(false);
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

    let is_authed = move || user.get().and_then(|r| r.ok()).flatten().is_some();

    let close_link = Callback::new(move |_: ()| close_all());

    let nav_links = move |extra_class: &'static str| {
        let link_cls = format!("{extra_class} text-secondary hover:text-primary transition-colors");
        let link_cls2 = link_cls.clone();
        let link_cls3 = link_cls.clone();
        let link_cls4 = link_cls.clone();
        let link_cls5 = link_cls.clone();
        let link_cls6 = link_cls.clone();
        let link_cls7 = link_cls.clone();
        let link_cls8 = link_cls.clone();
        view! {
            <A href="/" attr:class=link_cls
                on:click=move |_| close_link.run(())>
                "Home"
            </A>
            {move || if is_authed() {
                view! {
                    <A href="/team/dashboard" attr:class=link_cls2.clone()
                        on:click=move |_| close_link.run(())>
                        "Team"
                    </A>
                    <A href="/draft" attr:class=link_cls3.clone()
                        on:click=move |_| close_link.run(())>
                        "Draft"
                    </A>
                    <A href="/tree-drafter" attr:class=link_cls4.clone()
                        on:click=move |_| close_link.run(())>
                        "Tree Drafter"
                    </A>
                    <A href="/stats" attr:class=link_cls5.clone()
                        on:click=move |_| close_link.run(())>
                        "Stats"
                    </A>
                    <A href="/game-plan" attr:class=link_cls6.clone()
                        on:click=move |_| close_link.run(())>
                        "Game Plan"
                    </A>
                    <A href="/post-game" attr:class=link_cls7.clone()
                        on:click=move |_| close_link.run(())>
                        "Post Game"
                    </A>
                    <A href="/opponents" attr:class=link_cls8.clone()
                        on:click=move |_| close_link.run(())>
                        "Opponents"
                    </A>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        }
    };

    view! {
        <nav class="bg-surface/80 backdrop-blur-md border-b border-divider sticky top-0 z-50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6">
                <div class="flex items-center justify-between h-14">
                    // Logo
                    <A href="/" attr:class="flex items-center gap-2 shrink-0">
                        <span class="text-accent font-bold text-lg tracking-wide">"LoL Team Companion"</span>
                    </A>

                    // Desktop nav links
                    <div class="hidden md:flex items-center gap-5 text-sm">
                        {nav_links("")}
                    </div>

                    // Right side: theme + notifications + user menu
                    <div class="flex items-center gap-2">
                        <ThemeToggle />

                        // Notifications bell
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
                                            class="relative text-muted hover:text-primary transition-colors p-1.5 rounded-lg hover:bg-elevated cursor-pointer"
                                            aria-label="Notifications"
                                        >
                                            // Bell SVG
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                            </svg>
                                            <span class="absolute -top-0.5 -right-0.5 bg-red-500 text-white text-[10px] font-bold rounded-full min-w-[16px] h-4 flex items-center justify-center px-1 leading-none">
                                                {count}
                                            </span>
                                        </button>

                                        // Notifications dropdown
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
                                                                    class="bg-green-700 hover:bg-green-600 text-white text-xs font-medium rounded px-2 py-1 transition-colors cursor-pointer"
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
                                                                    class="bg-overlay-strong hover:bg-red-700 text-secondary hover:text-primary text-xs font-medium rounded px-2 py-1 transition-colors cursor-pointer"
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

                        // User menu
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
                                                class="flex items-center gap-2 text-secondary hover:text-primary transition-colors cursor-pointer px-2 py-1 rounded-lg hover:bg-elevated"
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
                                                        class="block w-full text-left px-4 py-2.5 text-red-400 hover:bg-elevated hover:text-red-300 transition-colors cursor-pointer text-sm"
                                                    >
                                                        "Sign Out"
                                                    </button>
                                                </ActionForm>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <div class="flex items-center gap-2 text-sm">
                                                <A href="/auth/login" attr:class="text-secondary hover:text-primary transition-colors px-3 py-1.5 rounded-lg hover:bg-elevated">
                                                    "Sign In"
                                                </A>
                                                <A href="/auth/register" attr:class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-3 py-1.5 rounded-lg transition-colors">
                                                    "Register"
                                                </A>
                                            </div>
                                        }.into_any(),
                                    }
                                })}
                            </Suspense>
                        </div>

                        // Mobile menu button
                        <button
                            on:click=move |_| mobile_open.update(|v| *v = !*v)
                            class="md:hidden text-muted hover:text-primary p-1.5 rounded-lg hover:bg-elevated transition-colors cursor-pointer"
                            aria-label="Menu"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Mobile nav
            <div
                class="md:hidden border-t border-divider"
                style:display=move || if mobile_open.get() { "block" } else { "none" }
            >
                <div class="px-4 py-3 flex flex-col gap-2 text-sm">
                    {nav_links("block py-2 px-3 rounded-lg hover:bg-elevated")}
                </div>
            </div>
        </nav>

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
