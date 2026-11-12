//! The no-team state of the team page (create one, or ask to join one) and
//! the server functions behind it. The roster itself lives on
//! `/team/dashboard`; `/team/roster` redirects there in axum (`main.rs`).

use crate::components::ui::enter_submits;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, EmptySize, EmptyState, Field, HeadingLevel, InputKind,
    PageHeader, SectionHeader, Select, SelectOption, Skeleton, SkeletonGroup, TextField, Toast,
    ToastContext,
};
use crate::models::team::Team;
use crate::pages::team::dashboard::TEAM_REGIONS;
use leptos::prelude::*;

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

/// No team yet: create one, or ask to join one. Rendered by the team page
/// (`/team/dashboard`) when the account has no team.
#[component]
pub fn NoTeamRoster(teams: Resource<Result<Vec<Team>, ServerFnError>>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let name = RwSignal::new(String::new());
    let region = RwSignal::new("EUW".to_string());
    let create_error = RwSignal::new(None::<String>);
    let creating = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let requested = RwSignal::new(Vec::<String>::new());

    let create = Callback::new(move |_: ()| {
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
        <PageHeader title="Team" meta=move || view! { <span>"You are not on a team yet. Create one, or ask to join one."</span> } />
        <div class="grid gap-x-12 gap-y-12 lg:grid-cols-2">
            <section aria-labelledby="create-h" data-create-team>
                <SectionHeader title="Create a team" id="create-h" />
                <p class="mb-4 text-body text-secondary">"You lead the team you create: you place players and answer join requests."</p>
                <div class="flex max-w-md flex-col gap-4">
                    <Field id="create-team-name" label="Team name" required=true error=Signal::derive(move || create_error.get())>
                        <TextField value=name name="name" maxlength=60 required=true on_keydown=enter_submits(create) />
                    </Field>
                    <Field id="create-team-region" label="Region">
                        <Select value=region name="region" options=region_options />
                    </Field>
                    <div>
                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| create.run(())) loading=creating attr:data-testid="create-team">"Create team"</Button>
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
                                return view! { <p class="mt-4 text-body text-muted">{format!("{} teams on Team Companion. Search to find yours.", list.len())}</p> }.into_any();
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
