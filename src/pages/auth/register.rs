//! Phase 17 plan 17-06 — Register page (invited).
//!
//! D-13 (utility tier) per UI-SPEC §"Register page — invited"
//! lines 380–402 + D-16 invite-token URL handling. The invite code
//! travels through the form as a hidden field; Phase 19.1 will wire
//! actual server-side validation against an `invite_code` table.
//!
//! Preserves the existing `register_action` server fn + auto-login
//! redirect Effect (leptos rules 8, 51). Visual layer only.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::app::InitialTheme;
use crate::components::region::CompanionSigil;
use crate::components::ui::ErrorBanner;

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
    let gate = config::get().closed_beta;
    let code = db::normalize_invite_code(&invite_code);
    if gate
        && !db::consume_invite(&db, &code)
            .await
            .map_err(authz::server_err)?
    {
        return Err(ServerFnError::new(db::GENERIC_REGISTRATION_ERROR));
    }

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

    // New users default to solo mode (D-03)
    Ok("/solo".to_string())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    // Phase 18.2-03 — read region ONCE at page entry; pass as a prop
    // to CompanionSigil so SSR and WASM hydrate see the same value.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let register = ServerAction::<RegisterAction>::new();
    let query = use_query_map();

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
        register.value().get().and_then(|r| r.err()).map(|e| {
            let msg = e.to_string();
            msg.strip_prefix("error running server function: ")
                .unwrap_or(&msg)
                .to_string()
        })
    };

    view! {
        <div class="canvas-grain bg-base min-h-screen flex items-center justify-center px-6 relative overflow-hidden">
            <img
                src="/img/auth-bg-demacia.jpg"
                alt=""
                aria-hidden="true"
                loading="lazy"
                class="auth-bg-demacia fixed inset-0 -z-10 w-full h-full object-cover"
            />

            <div class="relative z-10 bg-surface border border-divider rounded-xl p-8 max-w-sm w-full shadow-xl">
                <div class="flex justify-center mb-6">
                    <CompanionSigil region=region.clone() />
                </div>

                <h1 class="font-display italic text-[24px] text-primary text-center">
                    "Join the beta"
                </h1>
                <p class="text-sm text-muted mt-1 text-center">
                    "You have been invited."
                </p>

                <ActionForm action=register>
                    <div class="flex flex-col gap-4 mt-6">
                        {move || error().map(|e| view! {
                            <ErrorBanner message=e />
                        })}

                        // Hidden invite code travels with the form
                        // (D-16). Phase 19.1 wires real server-side
                        // validation. Threat T-17-24 mitigation:
                        // Leptos view! macro escapes attribute values.
                        <input
                            type="hidden"
                            name="invite_code"
                            prop:value=move || invite.get()
                        />

                        <div>
                            <label class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                "Username"
                            </label>
                            <input
                                type="text"
                                name="username"
                                required
                                autocomplete="username"
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            />
                        </div>

                        <div>
                            <label class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                "Email"
                            </label>
                            <input
                                type="email"
                                name="email"
                                required
                                autocomplete="email"
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            />
                        </div>

                        <div>
                            <label class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                "Password"
                            </label>
                            <input
                                type="password"
                                name="password"
                                required
                                minlength="10"
                                autocomplete="new-password"
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            />
                        </div>

                        <button
                            type="submit"
                            class="bg-accent text-accent-contrast font-semibold w-full py-3 rounded-lg hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors mt-2"
                        >
                            "Create account"
                        </button>

                        <p class="text-sm text-muted text-center mt-2">
                            "Already have an account? "
                            <A href="/auth/login" attr:class="text-accent hover:text-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded">
                                "Sign in"
                            </A>
                        </p>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
