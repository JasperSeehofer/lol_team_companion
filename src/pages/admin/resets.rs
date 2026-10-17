//! `/admin/resets`: password reset codes (M137 item 5, atelier card C9).
//! There is no mailer: an admin mints a one-time code for an account and
//! sends it to the player privately; the player sets a new password on
//! `/auth/reset`. The code is shown once here and stored only as a hash.
//!
//! Admin-only like `/admin/invites`: every server function checks
//! `require_admin()`, everyone else gets the 404 page (`SsrMode::Async`).

use leptos::prelude::*;

use crate::components::clipboard;
use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, Callout, CalloutTone, EmptySize, EmptyState,
    Field, InputKind, PageHeader, PageShell, PageWidth, SectionHeader, Skeleton, SkeletonGroup,
    Table, Td, TextField, Th, Tr,
};
use crate::error_template::NotFound;
use crate::models::password_reset::{MintedCode, ResetSummary};

/// The newest reset codes (never the codes themselves) and the server's
/// time (RFC 3339) to tell open codes from expired ones.
#[server]
pub async fn list_password_resets() -> Result<(String, Vec<ResetSummary>), ServerFnError> {
    use crate::server::{authz, password};

    authz::require_admin().await?;
    let db = authz::db()?;
    let rows = password::list_recent_resets(&db, 20)
        .await
        .map_err(authz::server_err)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    Ok((now, rows))
}

/// Mint a one-time reset code for the account that signs in with `email`.
/// The admin and the account are logged (never the code).
#[server]
pub async fn mint_password_reset(email: String) -> Result<MintedCode, ServerFnError> {
    use crate::server::{authz, db, password};

    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let email = db::normalize_email(&email);
    if email.is_empty() || email.len() > 254 {
        return Err(ServerFnError::new(
            "Enter the email the account signs in with.",
        ));
    }
    let minted = match password::mint_reset_code(&db, &email, Some(&admin.id)).await {
        Ok(m) => m,
        Err(db::DbError::NotFound) => {
            return Err(ServerFnError::new("No account with that email."))
        }
        Err(e) => return Err(authz::server_err(e)),
    };
    tracing::info!(admin = %admin.id, user = %minted.user_id, "password reset code minted");
    Ok(MintedCode {
        email,
        username: minted.username,
        code: minted.code,
        expires_at: minted.expires_at,
    })
}

/// Where the "Copy code" button stands (success only after the clipboard
/// promise resolved).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyState {
    Idle,
    Copied,
    Failed,
}

fn status_class(status: &str) -> &'static str {
    match status {
        "open" => "text-success",
        "used" => "text-secondary",
        _ => "text-muted",
    }
}

#[component]
pub fn AdminResetsPage() -> impl IntoView {
    let list = Resource::new(|| (), |_| list_password_resets());
    let mint = ServerAction::<MintPasswordReset>::new();
    let email = RwSignal::new(String::new());

    Effect::new(move |_| {
        if let Some(Ok(_)) = mint.value().get() {
            email.set(String::new());
            list.refetch();
        }
    });
    let mint_error = Signal::derive(move || {
        mint.value()
            .get()
            .and_then(|r| r.err())
            .map(|e| crate::models::utils::error_text(&e))
    });
    let minted = move || mint.value().get().and_then(|r| r.ok());

    // "Copy code": "Copied" only after the clipboard promise resolved; a
    // refusal says so and selects the code for copying by hand.
    let copy_state = RwSignal::new(CopyState::Idle);
    let code_ref = NodeRef::<leptos::html::Code>::new();
    Effect::new(move |_| {
        if let Some(Ok(_)) = mint.value().get() {
            copy_state.set(CopyState::Idle);
        }
    });
    let copy = Callback::new(move |_| {
        let Some(code) = mint
            .value()
            .get_untracked()
            .and_then(|r| r.ok())
            .map(|m| m.code)
        else {
            return;
        };
        leptos::task::spawn_local(async move {
            match clipboard::copy_text(&code).await {
                Ok(()) => {
                    copy_state.set(CopyState::Copied);
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::prelude::*;
                        let cb = Closure::once(move || {
                            if copy_state.get_untracked() == CopyState::Copied {
                                copy_state.set(CopyState::Idle);
                            }
                        });
                        if let Some(w) = web_sys::window() {
                            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                2000,
                            );
                        }
                        cb.forget();
                    }
                }
                Err(_e) => {
                    copy_state.set(CopyState::Failed);
                    #[cfg(feature = "hydrate")]
                    {
                        leptos::logging::warn!("copy reset code: {_e}");
                        if let Some(el) = code_ref.get_untracked() {
                            clipboard::select_contents(&el);
                        }
                    }
                }
            }
        });
    });

    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading password resets"><Skeleton lines=4 /></SkeletonGroup></PageShell> }>
            {move || list.get().map(|result| match result {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((now, rows)) => view! {
                    <PageShell width=PageWidth::Standard attr:data-feedback-label="Password resets">
                        <PageHeader
                            title="Password resets"
                            eyebrow="Admin"
                            meta=move || view! { <span>"A one-time code for a player who forgot their password. They set a new one on /auth/reset."</span> }
                        />
                        <section aria-labelledby="reset-mint-h">
                            <SectionHeader title="Create a reset code" id="reset-mint-h" />
                            <ActionForm action=mint>
                                <div class="flex flex-wrap items-end gap-3">
                                    <div class="min-w-64 flex-1">
                                        <Field id="reset-email" label="Email of the account" error=mint_error>
                                            <TextField kind=InputKind::Email name="email" value=email required=true autocomplete="off" />
                                        </Field>
                                    </div>
                                    <Button kind=ButtonType::Submit variant=ButtonVariant::Primary size=ButtonSize::Touch loading=mint.pending()>"Create reset code"</Button>
                                </div>
                            </ActionForm>
                            <div class="mt-4" role="status" data-testid="reset-result">
                                {move || minted().map(|m| view! {
                                    <Callout tone=CalloutTone::Info title=format!("Reset code for {}", m.email)>
                                        <code node_ref=code_ref class="block select-all font-mono text-title figures text-primary" data-testid="reset-code">{m.code.clone()}</code>
                                        <p class="mt-2">
                                            "Valid for 15 minutes, until "
                                            <LocalTime iso=m.expires_at.clone() style=TimeStyle::When />
                                            ", and works once. Send it to " {m.username.clone()} " in a private message, with the address /auth/reset. It is not shown again."
                                        </p>
                                        <div class="mt-3">
                                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Touch reserve=&["Copy code", "Copied"] on_click=copy attr:data-testid="copy-reset-code">
                                                {move || if copy_state.get() == CopyState::Copied { "Copied" } else { "Copy code" }}
                                            </Button>
                                        </div>
                                        // Inside the page's one live region (the wrapper's
                                        // role="status"): announced only once the copy worked.
                                        {move || (copy_state.get() == CopyState::Copied).then(|| view! {
                                            <span class="sr-only" data-testid="reset-copy-status">"Code copied"</span>
                                        })}
                                        {move || (copy_state.get() == CopyState::Failed).then(|| view! {
                                            <p role="alert" class="mt-2 text-label text-danger" data-testid="reset-copy-error">
                                                "Could not copy. Select the code and copy it by hand."
                                            </p>
                                        })}
                                    </Callout>
                                })}
                            </div>
                        </section>
                        <section class="mt-12" aria-labelledby="reset-list-h">
                            <SectionHeader title="Recent codes" id="reset-list-h" />
                            {if rows.is_empty() {
                                view! { <EmptyState size=EmptySize::Compact message="No reset codes yet. Codes you create appear here with who made them and whether they were used." /> }.into_any()
                            } else {
                                view! {
                                    <Table caption="Recent password reset codes" caption_hidden=true>
                                        <thead>
                                            <tr>
                                                <Th>"Account"</Th>
                                                <Th>"Status"</Th>
                                                <Th>"Created"</Th>
                                                <Th>"By"</Th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows.into_iter().map(|r| {
                                                let status = r.status(&now);
                                                view! {
                                                    <Tr>
                                                        <Td header=true>{r.username.clone()}</Td>
                                                        <Td><span class=status_class(status)>{status}</span></Td>
                                                        <Td muted=true><LocalTime iso=r.created_at.clone() style=TimeStyle::When /></Td>
                                                        <Td muted=true>{r.created_by.clone().unwrap_or_else(|| "reset-password command".to_string())}</Td>
                                                    </Tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </Table>
                                }.into_any()
                            }}
                        </section>
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}
