//! The app bar: the hub navigation (desktop), the phone menu, join-request
//! notifications, the command bar and the account menu.
//!
//! Information architecture (M5): pages are grouped by when a team needs
//! them. Desktop shows five hubs (Match Day, Prep, Review, Team, You) with
//! a sub-navigation under the bar; phones get one "Menu" button that opens
//! every page in the same groups, with counts on the right. There is no
//! Solo/Team switch any more: the personal pages are the "You" hub.

use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Command, CommandBar, Dialog, DialogSize, Menu,
    MenuAlign, MenuItem, UserAvatar,
};
use crate::models::user::JoinRequest;
use crate::pages::profile::{get_current_user, Logout};
use crate::pages::team::dashboard::handle_join_request;

/// Kept for stored `user.mode` compatibility (the seed tool and older
/// clients write it); nothing gates on the mode since M5.
#[server]
pub async fn set_user_mode(mode: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    if mode != "solo" && mode != "team" {
        return Err(ServerFnError::new("Unknown mode"));
    }
    let authz::Ctx { db, user } = authz::authed().await?;
    db::set_user_mode(&db, &user.id, &mode)
        .await
        .map_err(authz::server_err)?;
    Ok(())
}

/// A page in the navigation.
#[derive(Clone, Copy, Debug)]
pub struct NavPage {
    pub path: &'static str,
    pub label: &'static str,
    /// The hub it belongs to ("today", "prep", "review", "team", "you").
    pub hub: &'static str,
    /// Extra words for the command bar.
    pub keywords: &'static str,
}

const fn page(
    path: &'static str,
    label: &'static str,
    hub: &'static str,
    keywords: &'static str,
) -> NavPage {
    NavPage {
        path,
        label,
        hub,
        keywords,
    }
}

/// Every signed-in page, in menu order.
pub const PAGES: &[NavPage] = &[
    page(
        "/",
        "Match Day",
        "today",
        "home next match scrim prep brief",
    ),
    page(
        "/opponents",
        "Opponents",
        "prep",
        "scouting dossier enemy brief",
    ),
    page("/draft", "Draft", "prep", "pick ban picks bans graph"),
    page("/tree-drafter", "Draft tree", "prep", "tree branches draft"),
    page(
        "/game-plan",
        "Game plan",
        "prep",
        "plan strategy win conditions",
    ),
    page(
        "/champion-pool",
        "Champion pool",
        "prep",
        "pool champions tiers",
    ),
    page(
        "/post-game",
        "Reviews",
        "review",
        "review debrief post-game",
    ),
    page(
        "/action-items",
        "Action items",
        "review",
        "tasks todo lessons",
    ),
    page(
        "/stats",
        "Team games",
        "review",
        "matches games history stats",
    ),
    page(
        "/team/dashboard",
        "Team",
        "team",
        "notes roster bench substitutes",
    ),
    page(
        "/team/roster",
        "Roster",
        "team",
        "members players join invite",
    ),
    page("/me", "My games", "you", "solo ranked lp trends history"),
    page(
        "/personal-learnings",
        "Learnings",
        "you",
        "journal notes lessons",
    ),
    page("/profile", "Profile", "you", "account riot link region"),
];

/// Hubs in bar order: (key, label, landing path).
pub const HUBS: &[(&str, &str, &str)] = &[
    ("today", "Match Day", "/"),
    ("prep", "Prep", "/draft"),
    ("review", "Review", "/post-game"),
    ("team", "Team", "/team/dashboard"),
    ("you", "You", "/me"),
];

/// The hub a path belongs to, `""` for pages outside the hubs (sign-in,
/// legal, closed beta, admin, shared links). Match detail pages sit under
/// Review (they open from team and personal games alike).
pub fn hub_for_path(path: &str) -> &'static str {
    if path == "/" {
        return "today";
    }
    if path.starts_with("/match/") {
        return "review";
    }
    if path.starts_with("/team/player") {
        return "team";
    }
    PAGES
        .iter()
        .filter(|p| p.path != "/")
        .find(|p| path == p.path || path.starts_with(&format!("{}/", p.path)))
        .map(|p| p.hub)
        .unwrap_or("")
}

/// Every page the command bar can jump to.
fn route_commands(is_admin: bool) -> Vec<Command> {
    let mut out: Vec<Command> = PAGES
        .iter()
        .map(|p| {
            let hint = HUBS
                .iter()
                .find(|(k, _, _)| *k == p.hub)
                .map(|(_, l, _)| *l)
                .unwrap_or("");
            Command::go(p.label, p.path).hint(hint).keywords(p.keywords)
        })
        .collect();
    out.push(
        Command::go("New learning", "/personal-learnings/new")
            .hint("You")
            .keywords("write journal lesson"),
    );
    out.push(Command::go("Impressum", "/legal/impressum").hint("Legal"));
    out.push(
        Command::go("Privacy", "/legal/datenschutz")
            .hint("Legal")
            .keywords("datenschutz data stored"),
    );
    if is_admin {
        out.push(Command::go("Invites", "/admin/invites").hint("Admin"));
        out.push(
            Command::go("Champion traits", "/admin/traits")
                .hint("Admin")
                .keywords("draft graph kit trait table"),
        );
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
    let Some(team_id) = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    else {
        return Ok(Notifications::default());
    };
    if !db::is_team_leader(&db, &user.id, &team_id)
        .await
        .map_err(authz::server_err)?
    {
        return Ok(Notifications::default());
    }
    let pending = db::list_pending_join_requests(&db, &team_id)
        .await
        .unwrap_or_default();
    Ok(Notifications {
        pending_requests: pending,
        is_leader: true,
    })
}

/// Counts shown on the right of the phone menu.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub struct NavCounts {
    pub team_name: Option<String>,
    pub next_match_at: Option<String>,
    pub opponents: usize,
    pub drafts: usize,
    pub plans: usize,
    pub reviews: usize,
    pub open_items: usize,
    pub members: usize,
    pub learnings: usize,
}

#[server]
pub async fn get_nav_counts() -> Result<NavCounts, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let learnings = db::list_personal_learnings(&ctx.db, &ctx.user.id)
        .await
        .map(|l| l.len())
        .unwrap_or(0);
    let Some(team_id) = ctx.team().await? else {
        return Ok(NavCounts {
            learnings,
            ..Default::default()
        });
    };
    let counts = db::team_counts(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let next = db::next_scheduled_match(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let team_name = db::get_user_team_with_members(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)?
        .map(|(t, _)| t.name);
    Ok(NavCounts {
        team_name,
        next_match_at: next.map(|m| m.starts_at),
        opponents: counts.opponents,
        drafts: counts.drafts,
        plans: counts.plans,
        reviews: counts.reviews,
        open_items: counts.open_items,
        members: counts.members,
        learnings,
    })
}

/// "3 drafts", "1 dossier", "" for none.
fn count_label(n: usize, one: &str, many: &str) -> String {
    match n {
        0 => String::new(),
        1 => format!("1 {one}"),
        n => format!("{n} {many}"),
    }
}

/// The phone menu groups: (heading, pages by path).
const MENU_GROUPS: &[(&str, &[&str])] = &[
    (
        "Today",
        &["/", "/opponents", "/draft", "/tree-drafter", "/game-plan"],
    ),
    ("After the game", &["/post-game", "/action-items", "/stats"]),
    (
        "Team",
        &["/team/dashboard", "/team/roster", "/champion-pool"],
    ),
    ("You", &["/me", "/personal-learnings", "/profile"]),
];

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

/// Whether `path` is the current page (exact, or a sub-page of it).
fn is_current(path: &str, current: &str) -> bool {
    if path == "/" {
        current == "/"
    } else {
        current == path || current.starts_with(&format!("{path}/"))
    }
}

/// The phone menu: every page grouped by when you need it, with counts.
///
/// A11y: a modal sheet (`Dialog`, focus trapped, Escape closes, focus
/// returns to the Menu button); a `nav` named "Phone menu" with one list
/// per group under a visible heading; the current page has
/// `aria-current="page"`. Counts are plain text after the page name.
#[component]
fn PhoneMenu(open: RwSignal<bool>, is_admin: bool, username: String) -> impl IntoView {
    let location = use_location();
    let username = StoredValue::new(username);
    let counts = Resource::new(
        move || open.get(),
        |o| async move {
            if o {
                get_nav_counts().await.ok()
            } else {
                None
            }
        },
    );
    let count_for = move |path: &str, c: &NavCounts| -> Option<AnyView> {
        let text = match path {
            "/" => {
                return c
                    .next_match_at
                    .clone()
                    .map(|iso| view! { <LocalTime iso=iso style=TimeStyle::Short /> }.into_any())
            }
            "/opponents" => count_label(c.opponents, "dossier", "dossiers"),
            "/draft" => count_label(c.drafts, "draft", "drafts"),
            "/game-plan" => count_label(c.plans, "plan", "plans"),
            "/post-game" => count_label(c.reviews, "review", "reviews"),
            "/action-items" => match c.open_items {
                0 => String::new(),
                n => format!("{n} open"),
            },
            "/team/roster" => count_label(c.members, "member", "members"),
            "/personal-learnings" => count_label(c.learnings, "note", "notes"),
            _ => String::new(),
        };
        (!text.is_empty()).then(|| view! { <span>{text}</span> }.into_any())
    };
    view! {
        <Dialog id="phone-menu" open=open title="Menu" size=DialogSize::Sheet>
            <nav aria-label="Phone menu" class="-mx-5 -my-4" data-phone-menu>
                <div class="flex items-center justify-between gap-3 border-b border-rule px-5 py-4">
                    <div class="min-w-0">
                        <p class="eyebrow">"Team"</p>
                        <p class="truncate font-display text-reading font-semibold text-primary">
                            {move || counts.get().flatten().and_then(|c| c.team_name).unwrap_or_else(|| "No team yet".to_string())}
                        </p>
                    </div>
                    <UserAvatar name=username.get_value() size=AvatarSize::Md />
                </div>
                {MENU_GROUPS.iter().map(|(heading, paths)| {
                    let heading_id = format!("phone-menu-{}", heading.to_lowercase().replace(' ', "-"));
                    view! {
                        <h3 id=heading_id.clone() class="eyebrow px-5 pb-2 pt-5">{*heading}</h3>
                        <ul aria-labelledby=heading_id>
                            {paths.iter().filter_map(|p| PAGES.iter().find(|x| x.path == *p)).map(|pg| {
                                let path = pg.path;
                                view! {
                                    <li>
                                        <A
                                            href=path
                                            attr:class=move || if is_current(path, &location.pathname.get()) {
                                                "grid min-h-12 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border-b border-rule bg-ink-2 px-5 text-reading font-semibold text-primary"
                                            } else {
                                                "grid min-h-12 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border-b border-rule px-5 text-reading text-primary transition-colors hover:bg-ink-2"
                                            }
                                            attr:aria-current=move || is_current(path, &location.pathname.get()).then_some("page")
                                            on:click=move |_| open.set(false)
                                        >
                                            <span>{pg.label}</span>
                                            <span class="figures text-caption text-muted">
                                                {move || counts.get().flatten().and_then(|c| count_for(path, &c))}
                                            </span>
                                        </A>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }
                }).collect_view()}
                {is_admin.then(|| view! {
                    <h3 id="phone-menu-admin" class="eyebrow px-5 pb-2 pt-5">"Admin"</h3>
                    <ul aria-labelledby="phone-menu-admin">
                        {[("/admin/invites", "Invites"), ("/admin/traits", "Champion traits"), ("/design", "Design system")].into_iter().map(|(path, label)| view! {
                            <li>
                                <A
                                    href=path
                                    attr:class="grid min-h-12 items-center border-b border-rule px-5 text-reading text-primary transition-colors hover:bg-ink-2"
                                    on:click=move |_| open.set(false)
                                >
                                    {label}
                                </A>
                            </li>
                        }).collect_view()}
                    </ul>
                })}
            </nav>
        </Dialog>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let notif_open = RwSignal::new(false);
    let command_open = RwSignal::new(false);
    let menu_open = RwSignal::new(false);

    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());
    let notifications = Resource::new(|| (), |_| get_notifications());

    Effect::new(move || {
        if let Some(Ok(())) = logout_action.value().get() {
            notif_open.set(false);
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });

    // Escape closes the join-request popover.
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        notif_open.set(false);
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

    let location = use_location();
    let active_hub = Signal::derive(move || hub_for_path(&location.pathname.get()).to_string());
    let signed_in = Signal::derive(move || matches!(user.get(), Some(Ok(Some(_)))));

    view! {
        <header class="sticky top-0 z-(--z-nav) border-b border-rule bg-ink-1">
            <div class="mx-auto max-w-7xl px-4 sm:px-8">
                <div class="flex h-12 items-center gap-6">
                    <A href="/" attr:class="shrink-0 font-display text-reading font-bold tracking-[-0.01em] text-primary">
                        "Companion"
                    </A>

                    <Show when=move || signed_in.get()>
                        <nav class="hidden md:flex items-center gap-5" aria-label="Primary">
                            {HUBS.iter().map(|(key, label, href)| {
                                let key = *key;
                                view! {
                                    <A
                                        href=*href
                                        attr:class=move || hub_class(active_hub.get() == key)
                                        attr:aria-current=move || (active_hub.get() == key).then_some("true")
                                    >
                                        {*label}
                                    </A>
                                }
                            }).collect_view()}
                        </nav>
                    </Show>

                    <div class="flex-1" />

                    <div class="flex items-center gap-3">
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
                                            on:click=move |_| notif_open.update(|v| *v = !*v)
                                            class="relative inline-flex h-8 items-center gap-2 rounded-ctl px-2 text-label text-secondary transition-colors hover:bg-ink-3 hover:text-primary cursor-pointer"
                                            aria-expanded=move || notif_open.get().to_string()
                                            aria-label=format!("Join requests: {count} waiting")
                                        >
                                            <svg class="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                            </svg>
                                            <span class="figures inline-grid h-5 min-w-5 place-items-center rounded-ctl border border-outline px-1 text-caption font-semibold text-primary">
                                                {count}
                                            </span>
                                        </button>

                                        <div
                                            class="absolute right-0 mt-2 w-80 max-w-[calc(100vw-2rem)] overflow-hidden rounded-ctl border border-outline bg-ink-2 shadow-overlay z-(--z-popover)"
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
                                                "Open the team page"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            }}
                        </Suspense>

                        // Command bar trigger, phone menu, account menu, or sign-in links.
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
                                            MenuItem::link("my-games", "My games", "/me"),
                                            MenuItem::link("pool", "Champion pool", "/champion-pool"),
                                        ];
                                        // Admin links only for admins; the pages check on the server too.
                                        if u.is_admin {
                                            items.push(MenuItem::link("invites", "Invites", "/admin/invites"));
                                            items.push(MenuItem::link("traits", "Champion traits", "/admin/traits"));
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
                                                <kbd class="hidden rounded-ctl border border-outline px-1 text-caption leading-4 text-muted sm:inline-block" aria-hidden="true">"Ctrl K"</kbd>
                                            </button>
                                            <CommandBar id="command-bar" open=command_open commands=commands />
                                            <div class="hidden md:block">
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
                                                            <span class="hidden text-label font-medium lg:inline">{n}</span>
                                                            <svg class="size-3.5 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                                                <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
                                                            </svg>
                                                        }
                                                    }
                                                />
                                            </div>
                                            <button
                                                type="button"
                                                class="inline-flex h-8 cursor-pointer items-center rounded-ctl border border-outline px-3 text-label font-medium text-primary transition-colors hover:bg-ink-3 md:hidden"
                                                aria-haspopup="dialog"
                                                aria-expanded=move || menu_open.get().to_string()
                                                data-testid="phone-menu-trigger"
                                                on:click=move |_| menu_open.set(true)
                                            >
                                                "Menu"
                                            </button>
                                            <PhoneMenu open=menu_open is_admin=u.is_admin username=name.clone() />
                                        }.into_any()
                                    }
                                    _ => view! {
                                        <div class="flex items-center gap-2">
                                            <Button href="/auth/login" variant=ButtonVariant::Quiet>"Sign in"</Button>
                                            <Button href="/closed-beta" variant=ButtonVariant::Secondary>"Request access"</Button>
                                        </div>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                // Sub-navigation for the current hub (desktop; phones use the menu).
                {move || {
                    let hub = active_hub.get();
                    if hub.is_empty() || hub == "today" || !signed_in.get() {
                        return ().into_any();
                    }
                    view! {
                        <nav class="-mb-px hidden flex-wrap gap-x-5 border-t border-rule md:flex" aria-label="Sub-navigation">
                            {PAGES.iter().filter(|p| p.hub == hub).map(|p| {
                                let path = p.path;
                                view! {
                                    <A
                                        href=path
                                        attr:class=move || sub_class(is_current(path, &location.pathname.get()))
                                        attr:aria-current=move || is_current(path, &location.pathname.get()).then_some("page")
                                    >
                                        {p.label}
                                    </A>
                                }
                            }).collect_view()}
                        </nav>
                    }.into_any()
                }}
            </div>
        </header>

        // Click-outside backdrop behind the join-request popover.
        {move || notif_open.get().then(|| view! {
            <div class="fixed inset-0 z-20" on:click=move |_| notif_open.set(false) />
        })}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_has_a_hub_and_lands_in_it() {
        for p in PAGES {
            assert_eq!(hub_for_path(p.path), p.hub, "{}", p.path);
            assert!(HUBS.iter().any(|(k, _, _)| *k == p.hub));
        }
        assert_eq!(hub_for_path("/match/EUW1_1"), "review");
        assert_eq!(hub_for_path("/personal-learnings/new"), "you");
        assert_eq!(hub_for_path("/team/player/user:x"), "team");
        assert_eq!(hub_for_path("/auth/login"), "");
        assert_eq!(hub_for_path("/s/abc"), "");
        assert_eq!(hub_for_path("/draft-x"), "");
    }

    #[test]
    fn the_phone_menu_lists_every_page_once() {
        let listed: Vec<&str> = MENU_GROUPS
            .iter()
            .flat_map(|(_, p)| p.iter().copied())
            .collect();
        for p in PAGES {
            assert_eq!(
                listed.iter().filter(|x| **x == p.path).count(),
                1,
                "{}",
                p.path
            );
        }
        assert_eq!(listed.len(), PAGES.len());
    }

    #[test]
    fn the_command_bar_covers_every_page() {
        let cmds = route_commands(true);
        for p in PAGES {
            assert!(
                cmds.iter().any(|c| c.href.as_deref() == Some(p.path)),
                "{}",
                p.path
            );
        }
        assert!(cmds
            .iter()
            .any(|c| c.href.as_deref() == Some("/legal/datenschutz")));
        assert!(!route_commands(false)
            .iter()
            .any(|c| c.href.as_deref() == Some("/admin/invites")));
    }

    #[test]
    fn counts_read_naturally() {
        assert_eq!(count_label(0, "draft", "drafts"), "");
        assert_eq!(count_label(1, "draft", "drafts"), "1 draft");
        assert_eq!(count_label(3, "draft", "drafts"), "3 drafts");
    }
}
