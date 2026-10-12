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

/// Whether the user keeps their player card out of share links (M8).
#[server]
pub async fn get_share_card_hidden() -> Result<bool, ServerFnError> {
    use crate::server::{account, authz};
    let ctx = authz::authed().await?;
    account::share_card_hidden(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

/// Keep the player card out of share links (or allow it again). Hiding also
/// revokes the live links to the card.
#[server]
pub async fn set_share_card_hidden(hidden: bool) -> Result<(), ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{account, authz};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    account::set_share_card_hidden(&ctx.db, &ctx.user.id, hidden)
        .await
        .map_err(authz::server_err)
}

/// Delete the signed-in account (`server::account::delete_account`). Needs
/// the username typed again and the password; rate limited per user. Ends
/// every session of the account, this one included.
#[server]
pub async fn delete_my_account(
    confirm_username: String,
    password: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::{verify_password_async, AuthSession};
    use crate::server::rate_limit::Bucket;
    use crate::server::session_store::SurrealSessionStore;
    use crate::server::{account, authz, db};

    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Account, &ctx.user)?;
    if confirm_username.trim() != ctx.user.username {
        return Err(ServerFnError::new(
            "Type your username exactly as shown to confirm.",
        ));
    }
    let hash = db::password_hash_of(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)?;
    if !verify_password_async(password, hash).await {
        return Err(ServerFnError::new("That password is not right."));
    }
    let deleted = account::delete_account(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(
        user = %ctx.user.id,
        team_deleted = deleted.team_deleted,
        "account deleted"
    );
    let store = SurrealSessionStore::new(ctx.db.clone(), "sessions".to_string());
    match store.delete_for_user(&ctx.user.id).await {
        Ok(n) => tracing::info!(user = %ctx.user.id, sessions = n, "sessions ended"),
        Err(e) => tracing::warn!("could not end the deleted account's sessions: {e}"),
    }
    let mut auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| authz::internal_error("extract auth session", e))?;
    if let Err(e) = auth.logout().await {
        tracing::warn!("logout after account deletion failed: {e}");
    }
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
    crate::models::utils::error_text(&e)
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let surveys_off = Resource::new(
        || (),
        |_| crate::components::survey_prompt::get_survey_opt_out(),
    );
    let card_hidden = Resource::new(|| (), |_| get_share_card_hidden());
    let guidance_on = Resource::new(
        || (),
        |_| crate::components::guidance::get_guidance_enabled(),
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
                {move || guidance_on.get().and_then(|r| r.ok()).map(|on| view! { <crate::components::guidance::GuidanceSettings enabled=on /> })}
            </Suspense>
            <Suspense fallback=|| ()>
                {move || surveys_off.get().and_then(|r| r.ok()).map(|off| view! { <SurveySettings off=off /> })}
            </Suspense>
            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let hidden = card_hidden.await.unwrap_or(false);
                    user.await.ok().flatten().map(|u| view! { <DataSettings username=u.username card_hidden=hidden /> })
                })}
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

/// Your data (M8): the player-card switch, the export and account deletion.
#[component]
fn DataSettings(username: String, card_hidden: bool) -> impl IntoView {
    use crate::components::ui::{Dialog, DialogSize, InputKind, Switch, ToastKind};
    let toast = use_context::<ToastContext>();
    let hide = RwSignal::new(card_hidden);
    let on_hide = Callback::new(move |on: bool| {
        leptos::task::spawn_local(async move {
            match set_share_card_hidden(on).await {
                Ok(()) => {
                    if let Some(t) = toast {
                        t.show.run((
                            ToastKind::Success,
                            if on {
                                "Your card is out of share links; live links to it were revoked."
                            } else {
                                "Teammates can share your card again."
                            }
                            .into(),
                        ));
                    }
                }
                Err(e) => {
                    hide.set(!on);
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Error, clean_error(e)));
                    }
                }
            }
        });
    });

    let delete_open = RwSignal::new(false);
    let typed = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let deleting = RwSignal::new(false);
    let name = StoredValue::new(username.clone());
    let matches = Signal::derive(move || typed.get().trim() == name.get_value());
    let open_delete = Callback::new(move |_| {
        typed.set(String::new());
        password.set(String::new());
        error.set(None);
        delete_open.set(true);
    });
    let confirm = Callback::new(move |_| {
        if !matches.get_untracked() || deleting.get_untracked() {
            return;
        }
        deleting.set(true);
        leptos::task::spawn_local(async move {
            match delete_my_account(typed.get_untracked(), password.get_untracked()).await {
                Ok(()) =>
                {
                    #[cfg(feature = "hydrate")]
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/?account=deleted");
                    }
                }
                Err(e) => {
                    error.set(Some(clean_error(e)));
                    deleting.set(false);
                }
            }
        });
    });
    let hint = format!("Type {username} to confirm.");

    view! {
        <section class="mt-12 scroll-mt-20 flex flex-col gap-4" id="your-data" aria-labelledby="data-h" data-your-data>
            <SectionHeader title="Your data" id="data-h" />
            <div>
                <Switch label="Keep my player card out of share links" checked=hide on_change=on_hide />
                <p class="mt-2 text-caption text-muted">"Teammates still see your card inside the team. When it is off, a teammate can make a read-only link to it that works without signing in."</p>
            </div>
            <div class="flex flex-col gap-2">
                <h3 class="font-display text-heading font-semibold text-primary">"Download my data"</h3>
                <p class="max-w-prose text-body text-secondary">"A JSON file with everything Team Companion stores about you: your account and Riot link, team membership, champion pool and notes, learnings, goals, synced games, notes you wrote, bug reports, votes and survey answers. Team records you created are listed by name; their content belongs to the team."</p>
                <div>
                    <Button href="/account/export" variant=ButtonVariant::Secondary attr:download="" attr:data-testid="export-data">"Download my data"</Button>
                </div>
            </div>
            <div class="flex flex-col gap-2">
                <h3 class="font-display text-heading font-semibold text-primary">"Delete my account"</h3>
                <p class="max-w-prose text-body text-secondary">"Deletes your account, your personal pages, your synced games and your votes and survey answers, and signs you out everywhere. Drafts, plans, reviews and notes you wrote for your team stay with the team, shown as written by a former member. If you lead a team with other members, hand leadership to one of them on the team page first; if you are its only member, the team is deleted too. Backups keep a copy for up to 30 days."</p>
                <div>
                    <Button variant=ButtonVariant::Danger on_click=open_delete attr:data-testid="delete-account">"Delete my account"</Button>
                </div>
            </div>
        </section>
        <Dialog
            id="delete-account"
            open=delete_open
            title="Delete your account?"
            description="This cannot be undone. Your team keeps what you wrote for it, without your name."
            size=DialogSize::Sm
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| delete_open.set(false)) attr:autofocus=true attr:data-confirm="cancel">"Cancel"</Button>
                <Button
                    variant=ButtonVariant::Danger
                    on_click=confirm
                    loading=deleting
                    disabled=Signal::derive(move || !matches.get())
                    attr:data-confirm="confirm"
                >"Delete my account"</Button>
            }
        >
            <div class="flex flex-col gap-4">
                <Field id="delete-confirm-name" label="Username" hint=hint.clone()>
                    <TextField value=typed name="confirm_username" />
                </Field>
                <Field id="delete-password" label="Password" error=Signal::derive(move || error.get())>
                    <TextField value=password name="password" kind=InputKind::Password />
                </Field>
            </div>
        </Dialog>
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
