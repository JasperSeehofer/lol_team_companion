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

/// Hubs an account without a team sees (M160 solo mode): Match Day (`/`,
/// the start page) and You. The team hubs would only land on "No team yet".
const SOLO_HUBS: &[&str] = &["today", "you"];

/// The phone menu without a team: the start page and the personal pages
/// (the champion pool is personal, so it moves to You).
const SOLO_MENU_GROUPS: &[(&str, &[&str])] = &[
    ("Today", &["/"]),
    (
        "You",
        &["/me", "/personal-learnings", "/champion-pool", "/profile"],
    ),
];

/// Where "Find or create a team" goes (desktop bar and phone menu).
pub const FIND_TEAM_HREF: &str = "/team/dashboard";

/// What the navigation shows for an account with or without a team. One
/// pure function drives the desktop hubs and sub-navigation, the phone menu
/// and the command bar, so no surface leads a solo player to a team page.
#[derive(Clone, Debug)]
pub struct NavModel {
    /// Hubs in bar order: (key, label, landing path).
    pub hubs: Vec<(&'static str, &'static str, &'static str)>,
    /// Phone menu groups: (heading, pages).
    pub groups: Vec<(&'static str, Vec<NavPage>)>,
    /// Every page the command bar may offer, in menu order.
    pub pages: Vec<NavPage>,
    /// Show the quiet "Find or create a team" link.
    pub find_team: bool,
}

impl NavModel {
    /// Whether the hub `key` is in the bar.
    pub fn has_hub(&self, key: &str) -> bool {
        self.hubs.iter().any(|(k, _, _)| *k == key)
    }

    /// The sub-navigation of a hub: its pages, or none when the hub is
    /// not shown.
    pub fn hub_pages(&self, key: &str) -> Vec<NavPage> {
        if !self.has_hub(key) {
            return Vec::new();
        }
        PAGES.iter().filter(|p| p.hub == key).copied().collect()
    }
}

/// The navigation for an account with (`true`) or without a team.
pub fn nav_model(has_team: bool) -> NavModel {
    let groups_src: &[(&str, &[&str])] = if has_team {
        MENU_GROUPS
    } else {
        SOLO_MENU_GROUPS
    };
    let groups: Vec<(&'static str, Vec<NavPage>)> = groups_src
        .iter()
        .map(|(heading, paths)| {
            let pages = paths
                .iter()
                .filter_map(|p| PAGES.iter().find(|x| x.path == *p).copied())
                .collect::<Vec<_>>();
            (*heading, pages)
        })
        .filter(|(_, pages)| !pages.is_empty())
        .collect();
    let pages = PAGES
        .iter()
        .filter(|p| {
            groups
                .iter()
                .any(|(_, g)| g.iter().any(|x| x.path == p.path))
        })
        .copied()
        .collect();
    let hubs = HUBS
        .iter()
        .filter(|(k, _, _)| has_team || SOLO_HUBS.contains(k))
        .copied()
        .collect();
    NavModel {
        hubs,
        groups,
        pages,
        find_team: !has_team,
    }
}

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
fn route_commands(is_admin: bool, has_team: bool) -> Vec<Command> {
    let mut out: Vec<Command> = nav_model(has_team)
        .pages
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
    if !has_team {
        out.push(
            Command::go("Find or create a team", FIND_TEAM_HREF)
                .hint("Team")
                .keywords("team join create roster invite"),
        );
    }
    out.push(
        Command::go("New learning", "/personal-learnings/new")
            .hint("You")
            .keywords("write journal lesson"),
    );
    for (label, path, keywords) in COMMUNITY_LINKS {
        out.push(
            Command::go(*label, *path)
                .hint("Community")
                .keywords(*keywords),
        );
    }
    // Guidance (M9): the checklist on demand, and what changed.
    out.push(
        Command::go("Guide me", "/?guide=1#getting-started")
            .hint("Help")
            .keywords("help getting started checklist welcome lost"),
    );
    out.push(
        Command::go("What's new", "/changelog")
            .hint("Help")
            .keywords("new features changes release notes"),
    );
    out.push(Command::go("Impressum", "/legal/impressum").hint("Legal"));
    out.push(
        Command::go("Privacy", "/legal/datenschutz")
            .hint("Legal")
            .keywords("datenschutz data stored"),
    );
    if is_admin {
        for (label, path) in ADMIN_LINKS {
            out.push(Command::go(*label, *path).hint("Admin"));
        }
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

/// Community pages (M7): footer, command bar and phone menu. They are not
/// a hub: (label, path, command-bar keywords).
pub const COMMUNITY_LINKS: &[(&str, &str, &str)] = &[
    ("Changelog", "/changelog", "release notes updates new patch"),
    (
        "Transparency",
        "/transparency",
        "money costs support ledger",
    ),
    ("What next", "/what-next", "roadmap vote ideas"),
    ("Bug board", "/bugs", "issues known problems"),
    ("Your reports", "/bugs?tab=mine", "my bug reports status"),
];

/// Admin pages added in M7 (the command bar lists each; the account menu
/// has one "Admin" entry).
pub const ADMIN_LINKS: &[(&str, &str)] = &[
    ("Admin", "/admin"),
    ("Bug triage", "/admin/bugs"),
    ("Surveys", "/admin/surveys"),
    ("What next proposals", "/admin/proposals"),
    ("Transparency ledger", "/admin/ledger"),
    ("Public texts", "/admin/texts"),
    ("Password resets", "/admin/resets"),
];

/// "Your reports" with the number of reports that changed since last seen.
pub fn reports_label(updates: usize) -> String {
    if updates == 0 {
        "Your reports".to_string()
    } else {
        format!("Your reports ({updates} updated)")
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Notifications {
    pub pending_requests: Vec<JoinRequest>,
    pub is_leader: bool,
    /// The user's bug reports whose status changed since they last looked.
    #[serde(default)]
    pub report_updates: usize,
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
    let report_updates = crate::server::community::store::count_my_report_updates(&db, &user.id)
        .await
        .unwrap_or(0);
    let base = Notifications {
        report_updates,
        ..Default::default()
    };
    let Some(team_id) = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    else {
        return Ok(base);
    };
    if !db::is_team_leader(&db, &user.id, &team_id)
        .await
        .map_err(authz::server_err)?
    {
        return Ok(base);
    }
    let pending = db::list_pending_join_requests(&db, &team_id)
        .await
        .unwrap_or_default();
    Ok(Notifications {
        pending_requests: pending,
        is_leader: true,
        report_updates,
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
    let learnings = db::count_personal_learnings(&ctx.db, &ctx.user.id)
        .await
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
    Ok(NavCounts {
        team_name: Some(counts.name.clone()).filter(|n| !n.is_empty()),
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

/// Whether the signed-in account belongs to a team (M160 solo mode: the
/// team hubs are hidden without one). Signed out is `false`; the nav shows
/// no hubs then anyway.
#[server]
pub async fn get_nav_has_team() -> Result<bool, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let Some(user) = authz::optional_user().await? else {
        return Ok(false);
    };
    let db = authz::db()?;
    Ok(db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
        .is_some())
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

/// Public share pages (`/s/{token}`), where guests see a bare header.
pub fn is_share_path(path: &str) -> bool {
    path.starts_with("/s/")
}

/// The phone menu: every page grouped by when you need it, with counts.
///
/// A11y: a modal sheet (`Dialog`, focus trapped, Escape closes, focus
/// returns to the Menu button); a `nav` named "Phone menu" with one list
/// per group under a visible heading; the current page has
/// `aria-current="page"`. Counts are plain text after the page name.

#[component]
fn PhoneMenu(
    open: RwSignal<bool>,
    is_admin: bool,
    /// Without a team the menu lists only Match Day and You, plus "Find or
    /// create a team" (M160).
    has_team: bool,
    username: String,
    counts: ReadSignal<Option<NavCounts>>,
) -> impl IntoView {
    let location = use_location();
    let model = nav_model(has_team);
    let username = StoredValue::new(username);
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
    let groups = StoredValue::new(model.groups);
    view! {
        <Dialog id="phone-menu" open=open title="Menu" size=DialogSize::Sheet>
            <nav aria-label="Phone menu" class="-mx-5 -my-4" data-phone-menu>
                <div class="flex items-center justify-between gap-3 border-b border-rule px-5 py-4">
                    <div class="min-w-0">
                        <p class="eyebrow">"Team"</p>
                        <p class="truncate font-display text-reading font-semibold text-primary">
                            {move || counts.get().and_then(|c| c.team_name).unwrap_or_else(|| "No team yet".to_string())}
                        </p>
                    </div>
                    <UserAvatar name=username.get_value() size=AvatarSize::Md />
                </div>
                {move || groups.get_value().into_iter().map(|(heading, pages)| {
                    let heading_id = format!("phone-menu-{}", heading.to_lowercase().replace(' ', "-"));
                    view! {
                        <h3 id=heading_id.clone() class="eyebrow px-5 pb-2 pt-5">{heading}</h3>
                        <ul aria-labelledby=heading_id>
                            {pages.into_iter().map(|pg| {
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
                                            <span>{pg.label}<crate::components::guidance::NewMark route=path /></span>
                                            <span class="figures text-caption text-muted">
                                                {move || counts.get().and_then(|c| count_for(path, &c))}
                                            </span>
                                        </A>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }
                }).collect_view()}
                {model.find_team.then(|| view! {
                    <ul class="mt-5 border-t border-rule">
                        <li>
                            <A
                                href=FIND_TEAM_HREF
                                attr:class="grid min-h-12 items-center border-b border-rule px-5 text-reading text-secondary transition-colors hover:bg-ink-2 hover:text-primary"
                                attr:data-testid="phone-find-team"
                                on:click=move |_| open.set(false)
                            >
                                "Find or create a team"
                            </A>
                        </li>
                    </ul>
                })}
                <h3 id="phone-menu-community" class="eyebrow px-5 pb-2 pt-5">"Community"</h3>
                <ul aria-labelledby="phone-menu-community">
                    {COMMUNITY_LINKS.iter().map(|(label, path, _)| view! {
                        <li>
                            <A
                                href=*path
                                attr:class="grid min-h-12 items-center border-b border-rule px-5 text-reading text-primary transition-colors hover:bg-ink-2"
                                on:click=move |_| open.set(false)
                            >
                                {*label}
                            </A>
                        </li>
                    }).collect_view()}
                </ul>
                {is_admin.then(|| view! {
                    <h3 id="phone-menu-admin" class="eyebrow px-5 pb-2 pt-5">"Admin"</h3>
                    <ul aria-labelledby="phone-menu-admin">
                        {[("/admin", "Admin")].into_iter().map(|(path, label)| view! {
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
    // Team membership (M160 solo mode). Until it is known the hubs are not
    // drawn at all (as before the user resolves), so a solo player never
    // sees five hubs collapse to two; a failed check keeps the full bar.
    let membership = Resource::new(move || logout_version.get(), |_| get_nav_has_team());
    // Created here, not inside the user's Suspense: a resource created after
    // an await would get a different id on the server and in the browser and
    // shift every resource of the page during hydration.
    let counts_res = Resource::new(
        move || menu_open.get(),
        |o| async move {
            if o {
                get_nav_counts().await.ok()
            } else {
                None
            }
        },
    );
    // The menu reads the counts from a plain signal, filled once they
    // arrive: reading the resource inside the menu would suspend the
    // enclosing Suspense when the menu opens, which remounts the open
    // dialog and drops its modal state.
    let (counts, set_counts) = signal(None::<NavCounts>);
    // Every answer replaces the previous one, an empty one too, so a failed
    // load or a team left meanwhile never leaves old counts behind.
    Effect::new(move |_| {
        if let Some(c) = counts_res.get() {
            set_counts.set(c);
        }
    });

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

    // Opening the command bar ends its hint (M9).
    Effect::new(move |_| {
        if command_open.get() {
            if let Some(g) = use_context::<crate::components::guidance::GuidanceCtx>() {
                g.used(crate::models::guidance::Hint::CommandBar);
            }
        }
    });

    let location = use_location();
    let active_hub = Signal::derive(move || hub_for_path(&location.pathname.get()).to_string());
    let signed_in = Signal::derive(move || matches!(user.get(), Some(Ok(Some(_)))));
    // `None` while membership is loading.
    let has_team = Signal::derive(move || membership.get().map(|r| r.unwrap_or(true)));

    view! {
        <header class="sticky top-0 z-(--z-nav) border-b border-rule bg-ink-1">
            <div class="mx-auto max-w-7xl px-4 sm:px-8">
                <div class="flex h-12 items-center gap-6">
                    <A href="/" attr:class="shrink-0 font-display text-reading font-bold tracking-[-0.01em] text-primary">
                        "Team Companion"
                    </A>

                    // The user is a resource: read it inside a Suspense.
                    <Suspense fallback=|| ()>
                    <Show when=move || signed_in.get() && has_team.get().is_some()>
                        <nav class="hidden md:flex items-center gap-5" aria-label="Primary">
                            {move || {
                                let model = nav_model(has_team.get().unwrap_or(true));
                                let find_team = model.find_team;
                                let hubs = model.hubs.into_iter().map(|(key, label, href)| {
                                    let paths: Vec<&'static str> = PAGES.iter().filter(|p| p.hub == key).map(|p| p.path).collect();
                                    view! {
                                        <A
                                            href=href
                                            attr:class=move || hub_class(active_hub.get() == key)
                                            attr:aria-current=move || (active_hub.get() == key).then_some("true")
                                        >
                                            {label}
                                            {move || crate::components::guidance::any_new(&paths).then(|| view! {
                                                <span class="ml-1.5 text-caption font-semibold text-info" data-new-hub=key><span class="sr-only">", "</span>"New"</span>
                                            })}
                                        </A>
                                    }
                                }).collect_view();
                                view! {
                                    {hubs}
                                    {find_team.then(|| view! {
                                        <A
                                            href=FIND_TEAM_HREF
                                            attr:class="flex h-12 items-center px-0.5 text-label text-secondary transition-colors hover:text-primary"
                                            attr:data-testid="find-team-link"
                                        >
                                            "Find or create a team"
                                        </A>
                                    })}
                                }
                            }}
                        </nav>
                    </Show>
                    </Suspense>

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
                                        let updates = notifications
                                            .await
                                            .map(|n| n.report_updates)
                                            .unwrap_or(0);
                                        let name = u.username.clone();
                                        let name_for_trigger = StoredValue::new(name.clone());
                                        let mut items = vec![
                                            MenuItem::link("profile", "Profile", "/profile"),
                                            MenuItem::link("my-games", "My games", "/me"),
                                            MenuItem::link("pool", "Champion pool", "/champion-pool"),
                                        ];
                                        items.push(MenuItem::link("reports", reports_label(updates), "/bugs?tab=mine"));
                                        // One admin entry for admins; the pages check on the server too.
                                        if u.is_admin {
                                            items.push(MenuItem::link("admin", "Admin", "/admin"));
                                        }
                                        items.push(MenuItem::new("sign-out", "Sign out").danger());
                                        let on_select = Callback::new(move |id: String| {
                                            if id == "sign-out" {
                                                logout_action.dispatch(Logout {});
                                            }
                                        });
                                        let has_team = membership.await.unwrap_or(true);
                                        let commands = route_commands(u.is_admin, has_team);
                                        view! {
                                            <crate::components::guidance::CommandBarHint />
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
                                            <PhoneMenu open=menu_open is_admin=u.is_admin has_team=has_team username=name.clone() counts=counts />
                                        }.into_any()
                                    }
                                    // A guest on a share page gets the brand only (M5
                                    // review): the link is for reading, not an invitation.
                                    // Tracked here (not in the match) so leaving the share
                                    // page brings the buttons back without remounting the rest.
                                    _ => view! {
                                        <Show when=move || !is_share_path(&location.pathname.get())>
                                            <div class="flex items-center gap-2">
                                                <Button href="/auth/login" variant=ButtonVariant::Quiet>"Sign in"</Button>
                                                <Button href="/closed-beta" variant=ButtonVariant::Secondary>"Request access"</Button>
                                            </div>
                                        </Show>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                // Sub-navigation for the current hub (desktop; phones use the menu).
                <Suspense fallback=|| ()>
                {move || {
                    let hub = active_hub.get();
                    if hub.is_empty() || hub == "today" || !signed_in.get() {
                        return ().into_any();
                    }
                    // A team hub's sub-navigation is not drawn without a team.
                    let Some(team) = has_team.get() else {
                        return ().into_any();
                    };
                    let pages = nav_model(team).hub_pages(&hub);
                    if pages.is_empty() {
                        return ().into_any();
                    }
                    view! {
                        <nav class="-mb-px hidden flex-wrap gap-x-5 border-t border-rule md:flex" aria-label="Sub-navigation">
                            {pages.into_iter().map(|p| {
                                let path = p.path;
                                view! {
                                    <A
                                        href=path
                                        attr:class=move || sub_class(is_current(path, &location.pathname.get()))
                                        attr:aria-current=move || is_current(path, &location.pathname.get()).then_some("page")
                                    >
                                        {p.label}
                                        <crate::components::guidance::NewMark route=path />
                                    </A>
                                }
                            }).collect_view()}
                        </nav>
                    }.into_any()
                }}
                </Suspense>
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
        let cmds = route_commands(true, true);
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
        assert!(!route_commands(false, true)
            .iter()
            .any(|c| c.href.as_deref() == Some("/admin/invites")));
    }

    #[test]
    fn community_and_admin_commands() {
        let cmds = route_commands(true, true);
        for (_, path, _) in COMMUNITY_LINKS {
            assert!(
                cmds.iter().any(|c| c.href.as_deref() == Some(*path)),
                "{path}"
            );
        }
        for (_, path) in ADMIN_LINKS {
            assert!(
                cmds.iter().any(|c| c.href.as_deref() == Some(*path)),
                "{path}"
            );
        }
        assert!(!route_commands(false, true)
            .iter()
            .any(|c| c.href.as_deref() == Some("/admin")));
        assert_eq!(reports_label(0), "Your reports");
        assert_eq!(reports_label(2), "Your reports (2 updated)");
    }

    #[test]
    fn with_a_team_the_model_is_the_full_navigation() {
        let m = nav_model(true);
        assert_eq!(m.hubs, HUBS.to_vec());
        assert_eq!(m.pages.len(), PAGES.len());
        assert_eq!(m.groups.len(), MENU_GROUPS.len());
        assert!(!m.find_team);
        for (key, _, _) in HUBS {
            assert_eq!(
                m.hub_pages(key).len(),
                PAGES.iter().filter(|p| p.hub == *key).count()
            );
        }
        assert!(!route_commands(false, true)
            .iter()
            .any(|c| c.label == "Find or create a team"));
    }

    #[test]
    fn without_a_team_only_match_day_and_you_show() {
        let m = nav_model(false);
        let keys: Vec<&str> = m.hubs.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(keys, vec!["today", "you"]);
        assert!(m.find_team);
        for team_hub in ["prep", "review", "team"] {
            assert!(!m.has_hub(team_hub));
            assert!(m.hub_pages(team_hub).is_empty(), "{team_hub}");
        }
        assert_eq!(m.hub_pages("you").len(), 3);
        // Every surface: no page that needs a team, but the personal ones.
        let team_pages = [
            "/opponents",
            "/draft",
            "/tree-drafter",
            "/game-plan",
            "/post-game",
            "/action-items",
            "/stats",
            "/team/dashboard",
            "/team/roster",
        ];
        let menu: Vec<&str> = m
            .groups
            .iter()
            .flat_map(|(_, g)| g.iter().map(|p| p.path))
            .collect();
        let headings: Vec<&str> = m.groups.iter().map(|(h, _)| *h).collect();
        assert_eq!(headings, vec!["Today", "You"]);
        let cmds = route_commands(false, false);
        for path in team_pages {
            assert!(!menu.contains(&path), "phone menu lists {path}");
            assert!(!m.pages.iter().any(|p| p.path == path), "{path}");
            assert!(
                !cmds
                    .iter()
                    .any(|c| c.href.as_deref() == Some(path) && c.label != "Find or create a team"),
                "command bar offers {path}"
            );
        }
        for path in [
            "/",
            "/me",
            "/personal-learnings",
            "/champion-pool",
            "/profile",
        ] {
            assert_eq!(menu.iter().filter(|x| **x == path).count(), 1, "{path}");
            assert!(
                cmds.iter().any(|c| c.href.as_deref() == Some(path)),
                "{path}"
            );
        }
        assert!(cmds.iter().any(
            |c| c.label == "Find or create a team" && c.href.as_deref() == Some(FIND_TEAM_HREF)
        ));
        // Every solo menu page exists in PAGES.
        for (_, paths) in SOLO_MENU_GROUPS {
            for p in *paths {
                assert!(PAGES.iter().any(|x| x.path == *p), "{p}");
            }
        }
    }

    #[test]
    fn counts_read_naturally() {
        assert_eq!(count_label(0, "draft", "drafts"), "");
        assert_eq!(count_label(1, "draft", "drafts"), "1 draft");
        assert_eq!(count_label(3, "draft", "drafts"), "3 drafts");
    }
}
