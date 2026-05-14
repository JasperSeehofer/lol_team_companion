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

use crate::components::region::CompanionSigil;
use crate::components::ui::ErrorBanner;

#[server]
pub async fn register_action(
    username: String,
    email: String,
    password: String,
    #[allow(unused_variables)] invite_code: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{hash_password, AuthSession, Credentials};
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    // Phase 19.1 will validate `invite_code` against the
    // `invite_code` table and 404/error for invalid/consumed codes.
    // Phase 17: the value travels through (visual stub only).
    let _ = invite_code;

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let password_hash = hash_password(&password).map_err(ServerFnError::new)?;

    db::create_user(&db, username, email.clone(), password_hash)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Auto-login after registration (leptos rule 51).
    let mut auth: AuthSession = leptos_axum::extract().await?;
    let creds = Credentials { email, password };
    if let Ok(Some(user)) = auth.authenticate(creds).await {
        let _ = auth.login(&user).await;
    }

    // New users default to solo mode (D-03)
    Ok("/solo".to_string())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
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

    let error = move || {
        register
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|_| {
                // UI-SPEC line 689 copywriting: invite-specific error
                // message regardless of the underlying ServerFnError.
                "This invite link is invalid or has already been used. \
                 Contact your captain for a new one."
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
                    <CompanionSigil />
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
                                minlength="8"
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
