//! Profile: the account (username, the email it signs in with, region), the
//! Riot account link and signing out. Part of the personal layer with My
//! games, Learnings and the champion pool.

use crate::components::ui::{
    Button, ButtonType, ButtonVariant, Callout, CalloutTone, Field, PageHeader, PageShell,
    PageWidth, SectionHeader, Select, SelectOption, Skeleton, SkeletonGroup, TextField, Toast,
    ToastContext,
};
use crate::models::user::PublicUser;
use leptos::prelude::*;

#[server]
pub async fn get_current_user() -> Result<Option<PublicUser>, ServerFnError> {
    use crate::server::authz;

    Ok(authz::optional_user().await?.map(|u| PublicUser {
        id: u.id,
        username: u.username,
        email: u.email,
        riot_summoner_name: u.riot_summoner_name,
        mode: u.mode,
        riot_region: u.riot_region,
        is_admin: u.is_admin,
    }))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::authz;
    use leptos_axum::redirect;

    let mut auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| authz::internal_error("extract auth session", e))?;
    auth.logout().await.map_err(authz::server_err)?;
    redirect("/");
    Ok(())
}

#[server]
pub async fn update_profile(username: String) -> Result<(), ServerFnError> {
    use crate::server::authz;

    let authz::Ctx { db, user } = authz::authed().await?;
    crate::server::db::update_username(&db, &user.id, username)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_region(region: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let region = region.trim().to_string();
    if !crate::server::riot::is_known_region(&region) {
        return Err(ServerFnError::new("Choose a region from the list."));
    }
    db::set_user_region(&db, &user.id, &region)
        .await
        .map_err(authz::server_err)?;
    Ok(())
}

/// Link a Riot account by Riot ID (`GameName#TAG`): one account lookup at
/// Riot, rate limited per IP.
#[server]
pub async fn link_riot_account(riot_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;
    use crate::server::rate_limit::Bucket;
    use crate::server::riot;

    let authz::Ctx { db, user } = authz::authed().await?;
    let riot_id = riot_id.trim().to_string();
    let Some((game_name, tag_line)) = riot_id
        .split_once('#')
        .filter(|(a, b)| !a.trim().is_empty() && !b.trim().is_empty())
    else {
        return Err(ServerFnError::new("Write the Riot ID as GameName#TAG."));
    };
    authz::rate_limit(Bucket::Riot).await?;
    let platform = riot::platform_for(&[user.riot_region.as_deref()]);
    let puuid = authz::riot()?
        .get_puuid(game_name.trim(), tag_line.trim(), platform)
        .await
        .map_err(authz::server_err)?;
    db::update_user_riot(&db, user.id, puuid, riot_id)
        .await
        .map_err(authz::server_err)?;
    Ok(())
}

const REGIONS: &[(&str, &str)] = &[
    ("EUW", "EUW (Europe West)"),
    ("EUNE", "EUNE (Europe Nordic & East)"),
    ("NA", "NA (North America)"),
    ("KR", "KR (Korea)"),
    ("BR", "BR (Brazil)"),
    ("LAN", "LAN (Latin America North)"),
    ("LAS", "LAS (Latin America South)"),
    ("OCE", "OCE (Oceania)"),
    ("TR", "TR (Turkey)"),
    ("RU", "RU (Russia)"),
    ("JP", "JP (Japan)"),
    ("SG", "SG (Singapore)"),
    ("TW", "TW (Taiwan)"),
    ("VN", "VN (Vietnam)"),
    ("ME", "ME (Middle East)"),
];

fn clean_error(e: ServerFnError) -> String {
    e.to_string()
        .trim_start_matches("error running server function: ")
        .to_string()
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let surveys_off = Resource::new(
        || (),
        |_| crate::components::survey_prompt::get_survey_opt_out(),
    );
    Effect::new(move || {
        if let Some(Ok(None)) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    view! {
        <PageShell width=PageWidth::Reading attr:data-feedback-label="Profile">
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading your profile"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(Some(u)) => view! { <ProfileBody user=u on_change=Callback::new(move |_| user.refetch()) /> }.into_any(),
                        _ => view! { <PageHeader title="Profile" /> }.into_any(),
                    }
                })}
            </Suspense>
            <Suspense fallback=|| ()>
                {move || surveys_off.get().and_then(|r| r.ok()).map(|off| view! { <SurveySettings off=off /> })}
            </Suspense>
        </PageShell>
    }
}

/// The survey switch (M7): on by default; off also expires waiting questions.
#[component]
fn SurveySettings(off: bool) -> impl IntoView {
    use crate::components::ui::{Switch, ToastKind};
    let toast = use_context::<ToastContext>();
    let ask = RwSignal::new(!off);
    let on_change = Callback::new(move |on: bool| {
        leptos::task::spawn_local(async move {
            match crate::components::survey_prompt::set_survey_opt_out(!on).await {
                Ok(()) => {
                    if let Some(t) = toast {
                        t.show.run((
                            ToastKind::Success,
                            if on {
                                "Survey questions are on."
                            } else {
                                "Survey questions are off."
                            }
                            .into(),
                        ));
                    }
                }
                Err(e) => {
                    ask.set(!on);
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Error, clean_error(e)));
                    }
                }
            }
        });
    });
    view! {
        <section class="mt-12 scroll-mt-20" id="surveys" aria-labelledby="surveys-h">
            <SectionHeader title="Survey questions" id="surveys-h" />
            <Switch label="Ask me short survey questions" checked=ask on_change=on_change />
            <p class="mt-2 text-caption text-muted">"At most one question a week, always right after something you just did. You can dismiss any question."</p>
        </section>
    }
}

#[component]
fn ProfileBody(user: PublicUser, on_change: Callback<()>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let logout_action = ServerAction::<Logout>::new();
    Effect::new(move || {
        if let Some(Ok(())) = logout_action.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });
    let username = RwSignal::new(user.username.clone());
    let region = RwSignal::new(user.riot_region.clone().unwrap_or_else(|| "EUW".into()));
    let riot_id = RwSignal::new(String::new());
    let name_error = RwSignal::new(None::<String>);
    let riot_error = RwSignal::new(None::<String>);
    let linking = RwSignal::new(false);

    let save_name = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match update_profile(username.get_untracked()).await {
                Ok(()) => {
                    name_error.set(None);
                    toast.push.run(Toast::success("Username saved"));
                    on_change.run(());
                }
                Err(e) => name_error.set(Some(clean_error(e))),
            }
        });
    });
    let save_reg = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match save_region(region.get_untracked()).await {
                Ok(()) => toast.push.run(Toast::success("Region saved")),
                Err(e) => toast.push.run(Toast::error(clean_error(e))),
            }
        });
    });
    let link = Callback::new(move |_| {
        linking.set(true);
        leptos::task::spawn_local(async move {
            match link_riot_account(riot_id.get_untracked()).await {
                Ok(()) => {
                    riot_error.set(None);
                    riot_id.set(String::new());
                    toast.push.run(Toast::success("Riot account linked"));
                    on_change.run(());
                }
                Err(e) => riot_error.set(Some(clean_error(e))),
            }
            linking.set(false);
        });
    });
    let region_options: Vec<SelectOption> = REGIONS
        .iter()
        .map(|(v, l)| SelectOption::new(*v, *l))
        .collect();
    let linked = user.riot_summoner_name.clone();

    view! {
        <PageHeader title=user.username.clone() eyebrow="Profile" meta=move || view! { <span>"Your account and your Riot link. Your games are on My games."</span> } />

        <section aria-labelledby="account-h" class="flex flex-col gap-5" data-account>
            <SectionHeader title="Account" id="account-h" />
            <div class="flex flex-wrap items-end gap-3">
                <div class="min-w-0 flex-1">
                    <Field id="profile-username" label="Username" hint="Shown to your team; unique, 3 to 24 characters." error=Signal::derive(move || name_error.get())>
                        <TextField value=username name="username" maxlength=24 required=true />
                    </Field>
                </div>
                <Button on_click=save_name>"Save username"</Button>
            </div>
            <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-y-1 text-body">
                <dt class="text-muted">"Email"</dt>
                <dd class="text-primary" data-testid="profile-email">{user.email.clone()}</dd>
            </dl>
            <div class="flex flex-wrap items-end gap-3">
                <div class="min-w-0 flex-1">
                    <Field id="profile-region" label="Region" hint="Where your games are looked up at Riot.">
                        <Select value=region options=region_options />
                    </Field>
                </div>
                <Button on_click=save_reg>"Save region"</Button>
            </div>
        </section>

        <section aria-labelledby="riot-h" class="mt-12 flex flex-col gap-4" data-riot>
            <SectionHeader title="Riot account" id="riot-h" />
            {match linked {
                Some(n) => view! { <p class="text-body text-primary">"Linked to " <span class="font-medium">{n}</span>". Link again to switch accounts."</p> }.into_any(),
                None => view! { <p class="text-body text-secondary">"Not linked. Link it to sync your games, ranked standing and LP on My games, and to show them on your player card."</p> }.into_any(),
            }}
            <div class="flex flex-wrap items-end gap-3">
                <div class="min-w-0 flex-1">
                    <Field id="profile-riot" label="Riot ID" hint="GameName#TAG, as shown in the Riot client." error=Signal::derive(move || riot_error.get())>
                        <TextField value=riot_id name="riot_id" placeholder="GameName#TAG" />
                    </Field>
                </div>
                <Button on_click=link loading=linking>"Link account"</Button>
            </div>
            <Callout tone=CalloutTone::Info>
                "Linking makes one request to Riot. Syncing your games (on My games) makes about 21 and is possible once a minute; when the server's shared Riot budget is used up, it asks you to try again in a few seconds."
            </Callout>
        </section>

        <section aria-labelledby="signout-h" class="mt-12" data-sign-out>
            <SectionHeader title="Sign out" id="signout-h" />
            <p class="mb-3 text-body text-secondary">"Ends this session on this device."</p>
            <ActionForm action=logout_action>
                <Button kind=ButtonType::Submit variant=ButtonVariant::Secondary>"Sign out"</Button>
            </ActionForm>
        </section>
    }
}
