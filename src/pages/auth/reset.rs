//! `/auth/reset`: set a new password with a one-time code from an admin
//! (M137 item 5, atelier card C9). Public and rate limited per IP. Every
//! failure of the code (unknown email, wrong, used or expired code) gets
//! the same message, so the page answers no question about accounts or
//! codes. On success every session of the account ends and the visitor is
//! sent to sign in with the new password.

use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, Callout, CalloutTone, Field, InputKind,
    PageHeader, PageShell, PageWidth, TextField,
};
use leptos::prelude::*;
use leptos_router::components::A;

/// The one message for every code that does not work.
pub const RESET_FAILED: &str =
    "That code did not work. It may have expired or been used. Ask the admin for a new one.";

/// Where a successful reset lands: the sign-in page with its success line.
pub const AFTER_RESET: &str = "/auth/login?reset=1";

#[server]
pub async fn reset_password_action(
    email: String,
    code: String,
    new_password: String,
    confirm_password: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::hash_password_async;
    use crate::server::rate_limit::Bucket;
    use crate::server::session_store::SurrealSessionStore;
    use crate::server::{authz, db, password};

    // Before any Argon2 work: the per-IP budget bounds both guessing and CPU.
    authz::rate_limit(Bucket::PasswordReset).await?;
    // The password rule is public (registration states it), so its message
    // reveals nothing about the code.
    password::validate_new_password(&new_password).map_err(ServerFnError::new)?;
    if new_password != confirm_password {
        return Err(ServerFnError::new(
            crate::pages::profile::NEW_PASSWORDS_DIFFER,
        ));
    }
    if email.len() > 254 || code.len() > 64 {
        return Err(ServerFnError::new(RESET_FAILED));
    }
    // Hash first, so a wrong code costs the same time as a right one.
    let new_hash = hash_password_async(new_password)
        .await
        .map_err(|e| authz::internal_error("hash password", e))?;
    let db = authz::db()?;
    let user_id = match password::redeem_reset_code(&db, &email, &code, new_hash).await {
        Ok(id) => id,
        Err(db::DbError::NotFound) => return Err(ServerFnError::new(RESET_FAILED)),
        Err(e) => return Err(authz::server_err(e)),
    };
    let store = SurrealSessionStore::new(db.arc(), "sessions".to_string());
    let ended = match store.delete_for_user(&user_id).await {
        Ok(n) => n,
        Err(e) => {
            // The new hash already ends them (the session auth hash changed).
            tracing::warn!("could not delete the sessions after a password reset: {e}");
            0
        }
    };
    tracing::info!(user = %user_id, sessions = ended, "password reset with a code");
    // A form submitted before hydration lands on the sign-in page too.
    leptos_axum::redirect(AFTER_RESET);
    Ok(AFTER_RESET.to_string())
}

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let reset = ServerAction::<ResetPasswordAction>::new();

    // Hard navigation after success (leptos rule 8).
    Effect::new(move || {
        #[allow(unused_variables)]
        if let Some(Ok(dest)) = reset.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&dest);
            }
        }
    });
    // Focus the first field once the page is interactive.
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("reset-email"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = el.focus();
            }
        }
    });

    let error = move || {
        reset
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| crate::models::utils::error_text(&e))
    };
    let min = crate::pages::auth::register::MIN_PASSWORD_LEN;

    view! {
        <PageShell width=PageWidth::Narrow class="pt-12 sm:pt-20">
            <PageHeader title="Set a new password" eyebrow="Closed beta" meta=move || view! { <span>"With the one-time reset code the Team Companion admin sent you."</span> } />

            // ActionForm has no `class` prop (leptos rule 7).
            <ActionForm action=reset>
                <div class="flex flex-col gap-5">
                    {move || error().map(|e| {
                        let tone = if e.starts_with("Too many") { CalloutTone::Warning } else { CalloutTone::Danger };
                        view! { <Callout tone=tone>{e}</Callout> }
                    })}

                    <Field id="reset-email" label="Email">
                        <TextField kind=InputKind::Email name="email" required=true autocomplete="email" />
                    </Field>
                    <Field id="reset-code" label="Reset code" hint="For example K7QM-2XWD-9HPA-R4TB.">
                        <TextField name="code" required=true autocomplete="one-time-code" maxlength=64 />
                    </Field>
                    <Field id="reset-new" label="New password" hint=format!("Use {min} or more characters.")>
                        <TextField kind=InputKind::Password name="new_password" required=true autocomplete="new-password" minlength=min as u32 />
                    </Field>
                    <Field id="reset-repeat" label="Repeat new password">
                        <TextField kind=InputKind::Password name="confirm_password" required=true autocomplete="new-password" minlength=min as u32 />
                    </Field>

                    <Button kind=ButtonType::Submit variant=ButtonVariant::Primary size=ButtonSize::Lg full_width=true loading=reset.pending()>
                        "Set new password"
                    </Button>

                    <p class="border-t border-rule pt-5 text-body text-secondary">
                        "Remembered it? "
                        <A href="/auth/login" attr:class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">
                            "Sign in"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </PageShell>
    }
}
