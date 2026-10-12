//! Phase 17 plan 17-06 — Register page (invited).
//!
//! D-13 (utility tier) per UI-SPEC §"Register page — invited"
//! lines 380–402 + D-16 invite-token URL handling. The invite code
//! travels through the form as a hidden field; Phase 19.1 will wire
//! actual server-side validation against an `invite_code` table.
//!
//! Preserves the existing `register_action` server fn + auto-login
//! redirect Effect (leptos rules 8, 51). Visual layer only.

use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, Callout, CalloutTone, Field, InputKind,
    PageHeader, PageShell, PageWidth, TextField,
};
use crate::models::invite::InvitePreview;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

/// Minimum password length enforced by `register_action`.
pub const MIN_PASSWORD_LEN: usize = 10;

#[server]
pub async fn register_action(
    username: String,
    email: String,
    password: String,
    invite_code: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{hash_password_async, AuthSession};
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, config, db};
    use axum_login::AuthnBackend;

    authz::rate_limit(Bucket::Register).await?;

    let username = username.trim().to_string();
    let email = db::normalize_email(&email);
    let username_len = username.chars().count();
    if !(2..=32).contains(&username_len) {
        return Err(ServerFnError::new(
            "Username must be between 2 and 32 characters.",
        ));
    }
    if email.len() > 254 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(ServerFnError::new("Enter a valid email address."));
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(ServerFnError::new(format!(
            "Password must be at least {MIN_PASSWORD_LEN} characters."
        )));
    }
    if password.len() > 1024 {
        return Err(ServerFnError::new("Password is too long."));
    }

    let db = authz::db()?;

    // Closed beta: one use of a valid invite code per account. The check and
    // the use are one atomic update; the use is given back if the account
    // cannot be created.
    // A team-bound code also puts the new account in its team.
    let gate = config::get().closed_beta;
    let code = db::normalize_invite_code(&invite_code);
    let grant = if gate {
        match db::consume_invite_grant(&db, &code)
            .await
            .map_err(authz::server_err)?
        {
            Some(g) => g,
            None => return Err(ServerFnError::new(db::GENERIC_REGISTRATION_ERROR)),
        }
    } else {
        db::InviteGrant::default()
    };

    let created = match hash_password_async(password).await {
        Ok(password_hash) => db::create_user(&db, username, email, password_hash)
            .await
            .map_err(authz::server_err),
        Err(e) => Err(authz::internal_error("hash password", e)),
    };
    let user_id = match created {
        Ok(id) => id,
        Err(e) => {
            if gate {
                if let Err(re) = db::release_invite(&db, &code).await {
                    tracing::warn!("could not release invite use: {re}");
                }
            }
            return Err(e);
        }
    };

    // A team-bound invite: the account joins that team, a member not placed
    // yet. The account exists either way; if joining fails (the team was
    // deleted a moment ago), the welcome offers to create or join a team.
    if grant.team_id.is_some() {
        if let Err(e) = db::join_team_from_invite(&db, &user_id, &grant).await {
            tracing::warn!(user = %user_id, error = %e, "could not join the invite's team");
        }
    }

    // Auto-login after registration (leptos rule 51), without a second
    // Argon2 verification.
    let mut auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| authz::internal_error("extract auth session", e))?;
    match auth.backend.get_user(&user_id).await {
        Ok(Some(user)) => {
            authz::sync_admin_flag(&db, &user).await;
            if let Err(e) = auth.login(&user).await {
                tracing::warn!("auto-login after registration failed: {e}");
            }
        }
        Ok(None) => tracing::warn!("new user {user_id} not found for auto-login"),
        Err(e) => tracing::warn!("auto-login lookup failed: {e}"),
    }

    // New users land on Home (no team yet: create or join one). Redirect on
    // the server too, so a form submitted before hydration still lands there.
    leptos_axum::redirect("/");
    Ok("/".to_string())
}

/// What the register page says about an invite link: the team a usable
/// team-bound code leads to, and who is signed in (a signed-in visitor can
/// use a team code with the account they have). Rate limited per IP like
/// other public reads; unknown, used-up and plain codes all answer "no team".
#[server]
pub async fn preview_invite(invite_code: String) -> Result<InvitePreview, ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, db};

    authz::rate_limit(Bucket::PublicRead).await?;
    let db = authz::db()?;
    let team_name = db::team_invite_name(&db, &invite_code)
        .await
        .map_err(authz::server_err)?;
    let user = authz::optional_user().await?;
    let signed_in_has_team = match &user {
        Some(u) => authz::current_team_for(&db, u).await?.is_some(),
        None => false,
    };
    Ok(InvitePreview {
        team_name,
        signed_in_as: user.map(|u| u.username),
        signed_in_has_team,
    })
}

/// A signed-in account without a team uses a team invite: it joins that
/// team (a member not placed yet). Refused without using the code when the
/// account is already in a team. Returns where to go next.
#[server]
pub async fn join_with_team_invite(invite_code: String) -> Result<String, ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::TeamInvite, &ctx.user)?;
    let team = db::redeem_team_invite(&ctx.db, &ctx.user.id, &invite_code)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(user = %ctx.user.id, team = %team, "joined a team with an invite");
    Ok("/".to_string())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<RegisterAction>::new();
    let query = use_query_map();
    let invite_for_preview = move || query.read().get("invite").unwrap_or_default();
    // Created before any Suspense (leptos rule 58).
    let preview = Resource::new(invite_for_preview, preview_invite);
    let joining = RwSignal::new(false);
    let join_error = RwSignal::new(None::<String>);

    // Read the ?invite=... query param. The hidden input echoes the
    // value; Phase 19.1 server fn will validate against the
    // `invite_code` table. Per threat T-17-24, Leptos' view! macro
    // escapes attribute values by default — no `inner_html` use.
    let invite = Signal::derive(move || query.read().get("invite").unwrap_or_default());

    // Visual gate (D-16): if no invite param present, redirect to
    // /closed-beta. Phase 19.1 will additionally validate the format
    // server-side.
    Effect::new(move || {
        #[allow(unused_variables)]
        let code = invite.get();
        #[cfg(feature = "hydrate")]
        if code.trim().is_empty() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/closed-beta");
            }
        }
    });

    // Hard-nav after auto-login (leptos rule 8).
    Effect::new(move || {
        #[allow(unused_variables)]
        if let Some(Ok(dest)) = register.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&dest);
            }
        }
    });

    // The server sends user-safe messages only (generic for invite and
    // duplicate-account failures, specific for password rules).
    let error = move || {
        register
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| crate::models::utils::error_text(&e))
    };

    let join = Callback::new(move |_| {
        joining.set(true);
        join_error.set(None);
        let code = invite.get_untracked();
        leptos::task::spawn_local(async move {
            match join_with_team_invite(code).await {
                Ok(_dest) =>
                {
                    #[cfg(feature = "hydrate")]
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&_dest);
                    }
                }
                Err(e) => {
                    join_error.set(Some(crate::models::utils::error_text(&e)));
                    joining.set(false);
                }
            }
        });
    });

    view! {
        <PageShell width=PageWidth::Narrow class="pt-12 sm:pt-20">
            <PageHeader title="Create your account" eyebrow="Closed beta \u{00b7} invitation" meta=move || view! { <span>"You were invited to the closed beta. Pick a username your team will recognise."</span> } />
            <Suspense fallback=|| ()>
                {move || preview.get().and_then(|r| r.ok()).map(|p| {
                    let team = p.team_name.clone();
                    let signed_in = p.signed_in_as.clone();
                    match (team, signed_in) {
                        (Some(team), Some(name)) if !p.signed_in_has_team => view! {
                            <div class="mb-6" data-testid="invite-join-existing">
                                <Callout tone=CalloutTone::Info title=format!("You are signed in as {name}")>
                                    <p>{format!("This link is an invite to {team}. Join it with the account you have instead of creating another one.")}</p>
                                    <div class="mt-3">
                                        <Button variant=ButtonVariant::Secondary loading=joining on_click=join attr:data-testid="invite-join-button">{format!("Join {team}")}</Button>
                                    </div>
                                    {move || join_error.get().map(|e| view! { <p class="mt-2 text-label text-danger" role="alert">{e}</p> })}
                                </Callout>
                            </div>
                        }.into_any(),
                        (Some(team), Some(name)) => view! {
                            <div class="mb-6" data-testid="invite-in-team">
                                <Callout tone=CalloutTone::Warning title=format!("You are signed in as {name}")>
                                    {format!("This link is an invite to {team}, but your account already belongs to a team. Leave that team first to use it, or sign out to create a new account.")}
                                </Callout>
                            </div>
                        }.into_any(),
                        (Some(team), None) => view! {
                            <p class="mb-6 text-body text-primary" data-testid="invite-team">{format!("This invite signs you up for {team}. You join the team as soon as your account exists.")}</p>
                        }.into_any(),
                        (None, _) => ().into_any(),
                    }
                })}
            </Suspense>

            <ActionForm action=register>
                <div class="flex flex-col gap-5">
                    {move || error().map(|e| view! { <Callout tone=CalloutTone::Danger>{e}</Callout> })}

                    // The invite code travels with the form; the server checks it.
                    // `value`, not `prop:value`: the attribute is rendered on the
                    // server, so a form sent before hydration carries the code.
                    <input type="hidden" name="invite_code" value=move || invite.get() />

                    <Field id="register-username" label="Username">
                        <TextField name="username" required=true autocomplete="username" />
                    </Field>
                    <Field id="register-email" label="Email">
                        <TextField kind=InputKind::Email name="email" required=true autocomplete="email" />
                    </Field>
                    <Field id="register-password" label="Password" hint="Use 10 or more characters.">
                        <TextField kind=InputKind::Password name="password" required=true minlength=10 autocomplete="new-password" />
                    </Field>

                    <Button kind=ButtonType::Submit variant=ButtonVariant::Primary size=ButtonSize::Lg full_width=true>
                        "Create account"
                    </Button>

                    <p class="border-t border-rule pt-5 text-body text-secondary">
                        "Already have an account? "
                        <A href="/auth/login" attr:class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">
                            "Sign in"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </PageShell>
    }
}
