//! Phase 17 plan 17-06 — Login page.
//!
//! D-13 (utility tier) per Open-Design `lol-companion` design system.
//! UI-SPEC §"Auth Flows / Login page" lines 354–378.
//!
//! Preserves the existing `login_action` server fn + redirect Effect
//! (leptos rule 8: hard-nav after login refreshes auth state). Only
//! the visual layer is restyled.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::region::CompanionSigil;
use crate::components::ui::ErrorBanner;

#[server]
pub async fn login_action(email: String, password: String) -> Result<String, ServerFnError> {
    use crate::server::auth::{AuthSession, Credentials};
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let mut auth: AuthSession = leptos_axum::extract().await?;
    let creds = Credentials { email, password };
    match auth.authenticate(creds).await {
        Ok(Some(user)) => {
            auth.login(&user)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            let db = use_context::<Arc<Surreal<Db>>>()
                .ok_or_else(|| ServerFnError::new("No DB context"))?;
            let mode = db::get_user_mode(&db, &user.id)
                .await
                .unwrap_or_else(|_| "solo".to_string());
            let dest = if mode == "team" {
                "/team/dashboard".to_string()
            } else {
                "/solo".to_string()
            };
            Ok(dest)
        }
        Ok(None) => Err(ServerFnError::new("Invalid email or password")),
        Err(e) => Err(ServerFnError::new(e.to_string())),
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = ServerAction::<LoginAction>::new();

    // Hard navigate after successful login so the nav refetches auth state
    // (leptos rule 8). Use if-let-Some chain — never .unwrap() in WASM.
    Effect::new(move || {
        #[allow(unused_variables)]
        if let Some(Ok(dest)) = login.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&dest);
            }
        }
    });

    let error = move || {
        login
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| e.to_string())
    };

    view! {
        <div class="canvas-grain bg-base min-h-screen flex items-center justify-center px-6 relative overflow-hidden">
            // Optional FLUX-style background — Demacia auth bg only.
            // CSS in input.css hides this for Pandemonium (which falls
            // back to solid bg-base + canvas-grain).
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
                    "Welcome back"
                </h1>
                <p class="text-sm text-muted mt-1 text-center">
                    "Sign in to continue"
                </p>

                // ActionForm has no `class` prop (leptos rule 7) — wrap
                // form contents in a styled <div> for spacing.
                <ActionForm action=login>
                    <div class="flex flex-col gap-4 mt-6">
                        {move || error().map(|e| view! {
                            <ErrorBanner message=e />
                        })}

                        <div>
                            <label class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                "Email"
                            </label>
                            <input
                                type="email"
                                name="email"
                                required
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
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            />
                        </div>

                        <button
                            type="submit"
                            class="bg-accent text-accent-contrast font-semibold w-full py-3 rounded-lg hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors mt-2"
                        >
                            "Sign in"
                        </button>

                        <p class="text-sm text-muted text-center mt-2">
                            "No account? "
                            // Register requires an invite link from
                            // captain — link points to closed-beta
                            // landing (D-14). Direct register without
                            // an invite redirects to /closed-beta.
                            <A href="/closed-beta" attr:class="text-accent hover:text-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded">
                                "Register with an invite link"
                            </A>
                        </p>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
