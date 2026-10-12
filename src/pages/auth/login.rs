//! Phase 17 plan 17-06 — Login page.
//!
//! D-13 (utility tier) per Open-Design `lol-companion` design system.
//! UI-SPEC §"Auth Flows / Login page" lines 354–378.
//!
//! Preserves the existing `login_action` server fn + redirect Effect
//! (leptos rule 8: hard-nav after login refreshes auth state). Only
//! the visual layer is restyled.

use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, Callout, CalloutTone, Field, InputKind,
    PageHeader, PageShell, PageWidth, TextField,
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

    // Everyone lands on Home: Match Day for team members, the start page
    // (create or join a team, personal pages) for everyone else.
    let dest = "/".to_string();
    // A form submitted before hydration posts here directly; without the
    // redirect it would land back on the sign-in page (signed in, but
    // stuck). A hydrated submit still navigates in the Effect below.
    leptos_axum::redirect(&dest);
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
        login
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| crate::models::utils::error_text(&e))
    };

    view! {
        <PageShell width=PageWidth::Narrow class="pt-12 sm:pt-20">
            <PageHeader title="Sign in" eyebrow="Closed beta" meta=move || view! { <span>"Your team's drafts, plans and reviews are where you left them."</span> } />

            // ActionForm has no `class` prop (leptos rule 7).
            <ActionForm action=login>
                <div class="flex flex-col gap-5">
                    {move || error().map(|e| view! { <Callout tone=CalloutTone::Danger>{e}</Callout> })}

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
