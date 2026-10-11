use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Command, CommandBar, Menu, MenuAlign, MenuItem,
    Segment, SegmentSize, SegmentedControl, UserAvatar,
};
use crate::models::user::JoinRequest;
use crate::pages::profile::{get_current_user, Logout};
use crate::pages::team::dashboard::handle_join_request;

#[server]
pub async fn set_user_mode(mode: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::set_user_mode(&db, &user.id, &mode)
        .await
        .map_err(authz::server_err)?;
    Ok(())
}

/// The Solo / Team workspace switch in the nav.
#[component]
pub fn ModeToggle(mode: String) -> impl IntoView {
    let value = RwSignal::new(mode);
    let on_change = Callback::new(move |m: String| {
        leptos::task::spawn_local(async move {
            let _ = set_user_mode(m).await;
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    });
    // Shown at every width: on phones it is the only way to switch.
    view! {
        <SegmentedControl
            id="workspace"
            label="Workspace"
            options=vec![Segment::new("solo", "Solo"), Segment::new("team", "Team")]
            value=value
            size=SegmentSize::Sm
            on_change=on_change
        />
    }
}

/// Every page the command bar can jump to: the hub routes plus home.
fn route_commands(is_admin: bool) -> Vec<Command> {
    let mut out = vec![Command::go("Home", "/").hint("Companion")];
    for (hub, routes) in HUB_ROUTES {
        let hint = match *hub {
            "strategy" => "Strategy",
            "history" => "History",
            _ => "Profile",
        };
        for (path, label) in routes.iter() {
            let words = match *path {
                "/draft" => "pick ban picks bans",
                "/tree-drafter" => "tree branches draft",
                "/champion-pool" => "pool champions tiers",
                "/post-game" => "review debrief",
                "/opponents" => "scouting dossier enemy",
                "/stats" => "matches games history",
                "/team/roster" => "members players join",
                _ => "",
            };
            out.push(Command::go(*label, *path).hint(hint).keywords(words));
        }
    }
    if is_admin {
        out.push(Command::go("Invites", "/admin/invites").hint("Admin"));
        out.push(
            Command::go("Design system", "/design")
                .hint("Admin")
                .keywords("gallery primitives tokens"),
        );
    }
    out
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Notifications {
    pub pending_requests: Vec<JoinRequest>,
    pub is_leader: bool,
}

#[server]
pub async fn get_notifications() -> Result<Notifications, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let user = match authz::optional_user().await? {
        Some(u) => u,
        None => return Ok(Notifications::default()),
    };
    let db = authz::db()?;

    let result = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(authz::server_err)?;

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
// Hub Information Architecture
// ---------------------------------------------------------------------------

/// Hub-grouped sub-routes. Each hub maps to its sub-nav children.
/// Match detail pages (`/match/:id`) are reached from Stats, so they have no
/// sub-nav entry of their own but still highlight the History hub.
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
    } else if path.starts_with("/profile") || path.starts_with("/team") || path.starts_with("/solo")
    {
        "profile"
    } else {
        ""
    }
}

/// Classes for a primary hub link.
fn hub_class(active: bool) -> &'static str {
    if active {
        "flex h-12 items-center border-b-2 border-secondary px-0.5 text-label font-semibold text-primary"
    } else {
        "flex h-12 items-center border-b-2 border-transparent px-0.5 text-label text-muted transition-colors hover:text-primary"
    }
}

/// Classes for a sub-navigation link.
fn sub_class(active: bool) -> &'static str {
    if active {
        "flex h-10 items-center whitespace-nowrap border-b-2 border-accent text-label font-medium text-primary"
    } else {
        "flex h-10 items-center whitespace-nowrap border-b-2 border-transparent text-label text-muted transition-colors hover:text-primary"
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let notif_open = RwSignal::new(false);
    let command_open = RwSignal::new(false);

    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());
    let notifications = Resource::new(|| (), |_| get_notifications());

    let close_all = move || {
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

    // Escape closes any open dropdown.
    #[cfg(feature = "hydrate")]
    {
        let close_all_esc = close_all;
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

    let any_dropdown_open = move || notif_open.get();

    let location = use_location();
    let active_hub = Signal::derive(move || hub_for_path(&location.pathname.get()).to_string());

    view! {
        <header class="sticky top-0 z-(--z-nav) border-b border-rule bg-ink-1">
            <div class="mx-auto max-w-7xl px-4 sm:px-8">
                <div class="flex h-12 items-center gap-6">
                    <A href="/" attr:class="shrink-0 font-display text-reading font-bold tracking-[-0.01em] text-primary">
                        "Companion"
                    </A>

                    <nav class="hidden md:flex items-center gap-5" aria-label="Primary">
                        <A
                            href="/draft"
                            attr:class=move || hub_class(active_hub.get() == "strategy")
                            attr:aria-current=move || (active_hub.get() == "strategy").then_some("true")
                        >
                            "Strategy"
                        </A>
                        <A
                            href="/stats"
                            attr:class=move || hub_class(active_hub.get() == "history")
                            attr:aria-current=move || (active_hub.get() == "history").then_some("true")
                        >
                            "History"
                        </A>
                        <A
                            href="/profile"
                            attr:class=move || hub_class(active_hub.get() == "profile")
                            attr:aria-current=move || (active_hub.get() == "profile").then_some("true")
                        >
                            "Profile"
                        </A>
                    </nav>

                    <div class="flex-1" />

                    <div class="flex items-center gap-3">
                        <Suspense fallback=|| ()>
                            {move || Suspend::new(async move {
                                match user.await {
                                    Ok(Some(u)) => view! { <ModeToggle mode=u.mode /> }.into_any(),
                                    _ => ().into_any(),
                                }
                            })}
                        </Suspense>

                        // Join requests waiting for the team leader.
                        <Suspense fallback=|| ()>
                            {move || {
                                let notifs = notifications.get().and_then(|r| r.ok()).unwrap_or_default();
                                let count = notifs.pending_requests.len();
                                if count == 0 {
                                    return ().into_any();
                                }
                                let reqs = notifs.pending_requests;
                                view! {
                                    <div class="relative">
                                        <button
                                            type="button"
                                            on:click=move |_| {
                                                notif_open.update(|v| *v = !*v);
                                            }
                                            class="relative inline-flex h-8 items-center gap-2 rounded-ctl px-2 text-label text-secondary transition-colors hover:bg-ink-3 hover:text-primary cursor-pointer"
                                            aria-expanded=move || notif_open.get().to_string()
                                            aria-label=format!("Notifications: {count} join request(s)")
                                        >
                                            <svg class="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                            </svg>
                                            <span class="figures inline-grid h-5 min-w-5 place-items-center rounded-ctl bg-accent px-1 text-caption font-semibold text-accent-ink">
                                                {count}
                                            </span>
                                        </button>

                                        <div
                                            class="absolute right-0 mt-2 w-80 overflow-hidden rounded-ctl border border-rule bg-ink-2 shadow-overlay z-(--z-popover)"
                                            style:display=move || if notif_open.get() { "block" } else { "none" }
                                        >
                                            <p class="border-b border-rule px-4 py-3 eyebrow">"Join requests"</p>
                                            <div class="max-h-64 overflow-y-auto">
                                                {reqs.into_iter().map(|req| {
                                                    let req_id_accept = req.id.clone();
                                                    let req_id_decline = req.id.clone();
                                                    let accept = Callback::new(move |_| {
                                                        let id = req_id_accept.clone();
                                                        leptos::task::spawn_local(async move {
                                                            let _ = handle_join_request(id, true).await;
                                                            notifications.refetch();
                                                        });
                                                        notif_open.set(false);
                                                    });
                                                    let decline = Callback::new(move |_| {
                                                        let id = req_id_decline.clone();
                                                        leptos::task::spawn_local(async move {
                                                            let _ = handle_join_request(id, false).await;
                                                            notifications.refetch();
                                                        });
                                                        notif_open.set(false);
                                                    });
                                                    view! {
                                                        <div class="flex items-center justify-between gap-3 border-b border-rule px-4 py-3">
                                                            <div class="min-w-0">
                                                                <p class="truncate text-body font-medium text-primary">{req.username}</p>
                                                                {req.riot_summoner_name.map(|n| view! {
                                                                    <p class="truncate text-caption text-muted">{n}</p>
                                                                })}
                                                                <p class="text-caption text-muted">"Wants to join"</p>
                                                            </div>
                                                            <div class="flex shrink-0 gap-1.5">
                                                                <Button size=ButtonSize::Sm on_click=accept>"Accept"</Button>
                                                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=decline>"Decline"</Button>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                            <A
                                                href="/team/dashboard"
                                                attr:class="block px-4 py-2.5 text-center text-label text-muted transition-colors hover:text-primary"
                                                on:click=move |_| notif_open.set(false)
                                            >
                                                "View all on the team dashboard"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            }}
                        </Suspense>

                        // Command bar trigger, user menu, or sign-in links.
                        <Suspense fallback=move || view! {
                            <span class="text-label text-muted">"\u{2026}"</span>
                        }>
                            {move || Suspend::new(async move {
                                match user.await {
                                    Ok(Some(u)) => {
                                        let name = u.username.clone();
                                        let name_for_trigger = StoredValue::new(name.clone());
                                        let mut items = vec![
                                            MenuItem::link("profile", "Profile", "/profile"),
                                            MenuItem::link("pool", "Champion pool", "/champion-pool"),
                                        ];
                                        // Admin links only for admins; the pages check on the server too.
                                        if u.is_admin {
                                            items.push(MenuItem::link("invites", "Invites", "/admin/invites"));
                                            items.push(MenuItem::link("design", "Design system", "/design"));
                                        }
                                        items.push(MenuItem::new("sign-out", "Sign out").danger());
                                        let on_select = Callback::new(move |id: String| {
                                            if id == "sign-out" {
                                                logout_action.dispatch(Logout {});
                                            }
                                        });
                                        let commands = route_commands(u.is_admin);
                                        view! {
                                            <button
                                                type="button"
                                                class="inline-flex h-8 cursor-pointer items-center gap-2 rounded-ctl border border-rule px-2 text-label text-muted transition-colors hover:border-outline hover:text-primary"
                                                aria-keyshortcuts="Control+K Meta+K"
                                                aria-haspopup="dialog"
                                                data-testid="command-trigger"
                                                on:click=move |_| command_open.set(true)
                                            >
                                                <svg class="size-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                                                    <circle cx="7" cy="7" r="4.5" />
                                                    <path d="M10.5 10.5L14 14" stroke-linecap="round" />
                                                </svg>
                                                <span class="sr-only lg:not-sr-only">"Go to"</span>
                                                <kbd class="hidden rounded-ctl border border-outline/70 px-1 text-caption leading-4 text-muted sm:inline-block" aria-hidden="true">"Ctrl K"</kbd>
                                            </button>
                                            <CommandBar id="command-bar" open=command_open commands=commands />
                                            <Menu
                                                id="user-menu"
                                                label=format!("Account menu, {name}")
                                                items=items
                                                on_select=on_select
                                                align=MenuAlign::End
                                                variant=ButtonVariant::Quiet
                                                size=ButtonSize::Md
                                                trigger=move || {
                                                    let n = name_for_trigger.get_value();
                                                    view! {
                                                        <UserAvatar name=n.clone() size=AvatarSize::Sm />
                                                        <span class="hidden text-label font-medium sm:inline">{n}</span>
                                                        <svg class="size-3.5 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
                                                        </svg>
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    }
                                    _ => view! {
                                        <div class="flex items-center gap-2">
                                            <Button href="/auth/login" variant=ButtonVariant::Quiet>"Sign in"</Button>
                                            <Button href="/closed-beta" variant=ButtonVariant::Primary>"Request access"</Button>
                                        </div>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                // Sub-navigation for the current hub.
                {move || {
                    let hub = active_hub.get();
                    if hub.is_empty() {
                        ().into_any()
                    } else {
                        let routes: &[(&str, &str)] = HUB_ROUTES
                            .iter()
                            .find(|(h, _)| *h == hub)
                            .map(|(_, r)| *r)
                            .unwrap_or(&[]);
                        view! {
                            <nav class="-mb-px flex gap-5 overflow-x-auto border-t border-rule [scrollbar-width:none]" aria-label="Sub-navigation">
                                {routes.iter().map(|(path, label)| {
                                    let path_owned = path.to_string();
                                    let path_for_class = path_owned.clone();
                                    let path_for_aria = path_owned.clone();
                                    let location = use_location();
                                    let is_active = move |p: &str| location.pathname.get().starts_with(p);
                                    view! {
                                        <A
                                            href=path_owned
                                            attr:class=move || sub_class(is_active(&path_for_class))
                                            attr:aria-current=move || is_active(&path_for_aria).then_some("page")
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

        // Click-outside backdrop behind open dropdowns.
        {move || {
            if any_dropdown_open() {
                view! {
                    <div class="fixed inset-0 z-20" on:click=move |_| close_all() />
                }.into_any()
            } else {
                ().into_any()
            }
        }}
    }
}
