//! The roster page. Members see their team's roster and how players join;
//! people without a team create one or ask to join one. Leader tools
//! (placing, benching, removing players, answering requests) live on the
//! team page.

use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, EmptySize, EmptyState, Field, HeadingLevel,
    InputKind, PageHeader, PageShell, PageWidth, Role, RoleLabel, SectionHeader, Select,
    SelectOption, Skeleton, SkeletonGroup, SkeletonShape, Table, Td, TextField, Th, Toast,
    ToastContext, Tr, UserAvatar,
};
use crate::models::team::Team;
use crate::models::user::TeamMember;
use crate::pages::team::dashboard::{
    get_pending_requests, get_team_dashboard, group_roster, TEAM_REGIONS,
};
use leptos::prelude::*;
use leptos_router::components::A;

#[server]
pub async fn create_team(name: String, region: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;
    use leptos_axum::redirect;

    let authz::Ctx { db, user } = authz::authed().await?;
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ServerFnError::new("A team name has 1 to 60 characters."));
    }
    if !TEAM_REGIONS.contains(&region.as_str()) {
        return Err(ServerFnError::new("Unknown region"));
    }
    db::create_team(&db, &user.id, name, region)
        .await
        .map_err(authz::server_err)?;
    redirect("/team/dashboard");
    Ok(())
}

#[server]
pub async fn list_teams() -> Result<Vec<Team>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    // Signed-in users only: the team list is not public. Members cannot
    // join another team, so they get an empty list: the page creates this
    // resource for everyone (before its first Suspense) and members must
    // not pay for reading every team.
    let ctx = authz::authed().await?;
    if ctx.team().await?.is_some() {
        return Ok(Vec::new());
    }
    db::list_all_teams(&ctx.db).await.map_err(authz::server_err)
}

#[server]
pub async fn request_to_join(team_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::create_join_request(&db, &user.id, &team_id)
        .await
        .map_err(authz::server_err)
}

#[component]
pub fn RosterPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    let team = Resource::new(|| (), |_| get_team_dashboard());
    // Created before any Suspense, so server and browser agree on them.
    let requests = Resource::new(|| (), |_| get_pending_requests());
    let teams = Resource::new(|| (), |_| list_teams());
    view! {
        <PageShell width=PageWidth::Standard attr:data-feedback-label="Roster">
            <Suspense fallback=|| view! {
                <SkeletonGroup label="Loading the roster">
                    <Skeleton width="w-56 max-w-full" size="h-10" />
                    <Skeleton shape=SkeletonShape::Block size="h-48" />
                </SkeletonGroup>
            }>
                {move || Suspend::new(async move {
                    match team.await {
                        Ok(Some((t, members, me))) => view! { <MemberRoster team=t members=members me=me requests=requests /> }.into_any(),
                        Ok(None) => view! { <NoTeamRoster teams=teams /> }.into_any(),
                        Err(_) => view! {
                            <PageHeader title="Roster" />
                            <EmptyState title="The roster did not load" message="The server did not answer. Reload the page in a moment." />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </PageShell>
    }
}

/// A member's view: the roster by place and how new players join.
#[component]
fn MemberRoster(
    team: Team,
    members: Vec<TeamMember>,
    me: String,
    requests: Resource<Result<Vec<crate::models::user::JoinRequest>, ServerFnError>>,
) -> impl IntoView {
    let is_leader = team.created_by == me;
    let groups = group_roster(&members);
    let place_word = |m: &TeamMember| -> &'static str {
        match m.roster_type.as_str() {
            "starter" => "Starting five",
            "substitute" => "Substitute",
            "coach" => "Coach",
            _ => "Not placed yet",
        }
    };
    let mut ordered: Vec<TeamMember> = groups.starters.iter().flatten().cloned().collect();
    ordered.extend(groups.substitutes.iter().cloned());
    ordered.extend(groups.coaches.iter().cloned());
    ordered.extend(groups.unplaced.iter().cloned());
    let count = ordered.len();
    let team_name = team.name.clone();
    view! {
        <PageHeader
            title="Roster"
            eyebrow="Team"
            meta=move || view! {
                <span>{team_name.clone()}</span>
                <span aria-hidden="true" class="text-muted">"/"</span>
                <span>{if count == 1 { "1 member".to_string() } else { format!("{count} members") }}</span>
            }
            actions=move || view! { <Button href="/team/dashboard">"Team page"</Button> }
        />
        <section aria-labelledby="members-h" data-roster-members>
            <SectionHeader title="Members" id="members-h" meta=if is_leader { "Place, bench or remove players on the team page." } else { "The leader places players on the team page." } />
            <Table caption="Team members" caption_hidden=true>
                <thead>
                    <tr>
                        <Th>"Player"</Th>
                        <Th>"Position"</Th>
                        <Th>"Place"</Th>
                        <Th>"Riot account"</Th>
                    </tr>
                </thead>
                <tbody>
                    {ordered.into_iter().map(|m| {
                        let role = Role::parse(&m.role);
                        let href = format!("/team/player/{}", m.user_id);
                        let place = place_word(&m);
                        let is_me = m.user_id == me;
                        view! {
                            <Tr>
                                <Td>
                                    <span class="flex items-center gap-2">
                                        <UserAvatar name=m.username.clone() size=AvatarSize::Xs />
                                        <A href=href attr:class="font-medium text-primary underline decoration-transparent underline-offset-4 hover:decoration-outline">{m.username.clone()}</A>
                                        {is_me.then(|| view! { <span class="text-caption text-muted">"you"</span> })}
                                    </span>
                                </Td>
                                <Td>{match role { Some(r) => view! { <RoleLabel role=r /> }.into_any(), None => view! { <span class="text-muted">"None"</span> }.into_any() }}</Td>
                                <Td>{place}</Td>
                                <Td muted=true>{m.riot_summoner_name.clone().unwrap_or_else(|| "Not linked".to_string())}</Td>
                            </Tr>
                        }
                    }).collect_view()}
                </tbody>
            </Table>
        </section>
        <section class="mt-12 max-w-prose" aria-labelledby="joining-h" data-roster-joining>
            <SectionHeader title="How players join" id="joining-h" />
            <p class="text-body text-secondary">
                {format!("New players sign up with an invite link, open this page, find {} under \u{201c}Join a team\u{201d} and ask to join. The team leader answers on the team page; accepted players start as \u{201c}not placed yet\u{201d} until the leader puts them in the starting five, on the bench or with the coaches.", team.name)}
            </p>
            {is_leader.then(|| view! {
                <Suspense fallback=|| ()>
                    {move || requests.get().map(|r| {
                        let n = r.map(|l| l.len()).unwrap_or(0);
                        view! {
                            <p class="mt-3 text-body text-primary" data-pending-requests=n>
                                {match n {
                                    0 => "No requests waiting right now.".to_string(),
                                    1 => "1 request is waiting.".to_string(),
                                    n => format!("{n} requests are waiting."),
                                }}
                                " "
                                <A href="/team/dashboard" attr:class="underline decoration-outline underline-offset-4 hover:decoration-accent">"Answer on the team page"</A>
                            </p>
                        }
                    })}
                </Suspense>
            })}
        </section>
    }
}

/// No team yet: create one, or ask to join one.
#[component]
fn NoTeamRoster(teams: Resource<Result<Vec<Team>, ServerFnError>>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let name = RwSignal::new(String::new());
    let region = RwSignal::new("EUW".to_string());
    let create_error = RwSignal::new(None::<String>);
    let creating = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let requested = RwSignal::new(Vec::<String>::new());

    let create = Callback::new(move |_| {
        if name.get_untracked().trim().is_empty() {
            create_error.set(Some("Give the team a name.".into()));
            return;
        }
        creating.set(true);
        create_error.set(None);
        leptos::task::spawn_local(async move {
            match create_team(name.get_untracked(), region.get_untracked()).await {
                Ok(()) =>
                {
                    #[cfg(feature = "hydrate")]
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_href("/team/dashboard");
                    }
                }
                Err(e) => {
                    create_error.set(Some(e.to_string()));
                    creating.set(false);
                }
            }
        });
    });
    let ask = Callback::new(move |(id, team_name): (String, String)| {
        leptos::task::spawn_local(async move {
            match request_to_join(id.clone()).await {
                Ok(()) => {
                    requested.update(|r| r.push(id));
                    toast
                        .push
                        .run(Toast::success(format!("Request sent to {team_name}")));
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let region_options: Vec<SelectOption> = TEAM_REGIONS
        .iter()
        .map(|r| SelectOption::new(*r, *r))
        .collect();

    view! {
        <PageHeader title="Roster" eyebrow="Team" meta=move || view! { <span>"You are not on a team yet. Create one, or ask to join one."</span> } />
        <div class="grid gap-x-12 gap-y-12 lg:grid-cols-2">
            <section aria-labelledby="create-h" data-create-team>
                <SectionHeader title="Create a team" id="create-h" />
                <p class="mb-4 text-body text-secondary">"You lead the team you create: you place players and answer join requests."</p>
                <div class="flex max-w-md flex-col gap-4">
                    <Field id="create-team-name" label="Team name" required=true error=Signal::derive(move || create_error.get())>
                        <TextField value=name name="name" maxlength=60 required=true />
                    </Field>
                    <Field id="create-team-region" label="Region">
                        <Select value=region name="region" options=region_options />
                    </Field>
                    <div>
                        <Button variant=ButtonVariant::Primary on_click=create loading=creating attr:data-testid="create-team">"Create team"</Button>
                    </div>
                </div>
            </section>
            <section aria-labelledby="join-h" data-join-team>
                <SectionHeader title="Join a team" id="join-h" />
                <Field id="team-search" label="Find a team" hint="Type part of the team's name.">
                    <TextField value=query kind=InputKind::Search placeholder="Team name" />
                </Field>
                <Suspense fallback=|| view! { <SkeletonGroup label="Loading teams"><Skeleton lines=3 /></SkeletonGroup> }>
                    {move || teams.get().map(|r| match r {
                        Err(_) => view! { <p class="mt-4 text-body text-danger" role="alert">"The team list did not load."</p> }.into_any(),
                        Ok(list) => {
                            let q = query.get().trim().to_lowercase();
                            if q.is_empty() {
                                return view! { <p class="mt-4 text-body text-muted">{format!("{} teams on Companion. Search to find yours.", list.len())}</p> }.into_any();
                            }
                            let found: Vec<Team> = list.into_iter().filter(|t| t.name.to_lowercase().contains(&q)).collect();
                            if found.is_empty() {
                                return view! { <div class="mt-4"><EmptyState size=EmptySize::Compact level=HeadingLevel::H3 message="No team matches. Check the spelling, or ask your captain for the exact name." /></div> }.into_any();
                            }
                            view! {
                                <ul class="mt-4 border-b border-rule">
                                    {found.into_iter().map(|t| {
                                        let id = t.id.clone().unwrap_or_default();
                                        let id_for_check = id.clone();
                                        let name = t.name.clone();
                                        view! {
                                            <li class="flex flex-wrap items-center justify-between gap-3 border-t border-rule py-2.5">
                                                <span>
                                                    <span class="block font-medium text-primary">{t.name.clone()}</span>
                                                    <span class="figures block text-caption text-muted">{format!("{} \u{00b7} {} members", t.region, t.member_count.unwrap_or(0))}</span>
                                                </span>
                                                {move || if requested.get().contains(&id_for_check) {
                                                    view! { <span class="text-label text-success">"Request sent"</span> }.into_any()
                                                } else {
                                                    let (id, name) = (id.clone(), name.clone());
                                                    view! { <Button size=ButtonSize::Sm on_click=Callback::new(move |_| ask.run((id.clone(), name.clone())))>"Ask to join"</Button> }.into_any()
                                                }}
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </section>
        </div>
    }
}
