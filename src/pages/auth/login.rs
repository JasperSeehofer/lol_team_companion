//! Phase 17 plan 17-06 — Login page.
//!
//! D-13 (utility tier) per Open-Design `lol-companion` design system.
//! UI-SPEC §"Auth Flows / Login page" lines 354–378.
//!
//! Preserves the existing `login_action` server fn + redirect Effect
//! (leptos rule 8: hard-nav after login refreshes auth state). Only
//! the visual layer is restyled.

use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, ErrorBanner, Field, InputKind, PageShell,
    PageWidth, TextField,
};
use leptos::prelude::*;
use leptos_router::components::A;

#[server]
pub async fn login_action(email: String, password: String) -> Result<String, ServerFnError> {
    use crate::server::auth::{AuthSession, Credentials};
    use crate::server::rate_limit::{slow_down_message, Bucket, Limiter};
    use crate::server::{authz, db};

    const INVALID: &str = "Invalid email or password";

    // Per-IP limit on attempts, plus a per-account limit on failures.
    authz::rate_limit(Bucket::Login).await?;
    let email = db::normalize_email(&email);
    // No account has a longer email (registration caps it), and the email is
    // a rate-limiter key, so oversized values are refused before they are
    // stored.
    if email.len() > 254 || password.len() > 1024 {
        return Err(ServerFnError::new(INVALID));
    }
    if let Err(wait) = Limiter::global().check(Bucket::LoginFailuresPerEmail, &email) {
        return Err(ServerFnError::new(slow_down_message(wait)));
    }

    let mut auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| authz::internal_error("extract auth session", e))?;
    let creds = Credentials {
        email: email.clone(),
        password,
    };
    let user = match auth.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let _ = Limiter::global().hit(Bucket::LoginFailuresPerEmail, &email);
            return Err(ServerFnError::new(INVALID));
        }
        Err(e) => return Err(authz::internal_error("authenticate", e)),
    };

    let db = authz::db()?;
    authz::sync_admin_flag(&db, &user).await;
    auth.login(&user).await.map_err(authz::server_err)?;

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
        login.value().get().and_then(|r| r.err()).map(|e| {
            let msg = e.to_string();
            msg.strip_prefix("error running server function: ")
                .unwrap_or(&msg)
                .to_string()
        })
    };

    view! {
        <PageShell width=PageWidth::Narrow class="pt-12 sm:pt-20">
            <p class="eyebrow">"Closed beta"</p>
            <h1 class="mt-3 font-display text-headline font-semibold tracking-[-0.02em] text-primary">
                "Sign in"
            </h1>
            <p class="mt-2 text-body text-secondary">
                "Your team's drafts, plans and reviews are where you left them."
            </p>

            // ActionForm has no `class` prop (leptos rule 7).
            <ActionForm action=login>
                <div class="mt-8 flex flex-col gap-5">
                    {move || error().map(|e| view! { <ErrorBanner message=e /> })}

                    <Field id="login-email" label="Email">
                        <TextField kind=InputKind::Email name="email" required=true autocomplete="email" />
                    </Field>
                    <Field id="login-password" label="Password">
                        <TextField kind=InputKind::Password name="password" required=true autocomplete="current-password" />
                    </Field>

                    <Button kind=ButtonType::Submit variant=ButtonVariant::Primary size=ButtonSize::Lg full_width=true>
                        "Sign in"
                    </Button>

                    <p class="border-t border-rule pt-5 text-body text-secondary">
                        "No account yet? Registration needs an invite link from your team. "
                        <A href="/closed-beta" attr:class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">
                            "About the closed beta"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </PageShell>
    }
}
