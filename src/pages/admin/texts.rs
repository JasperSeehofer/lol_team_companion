//! `/admin/texts`: public wording that needs the owner's approval before it
//! is shown (M7): the money policy, the ledger labels, the split caption,
//! the not-covered caption and the support sentence. Any edit withdraws the
//! approval; the transparency page shows only approved texts.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    Badge, BadgeTone, Button, ButtonVariant, Callout, CalloutTone, Field, PageHeader, PageShell,
    PageWidth, SectionHeader, Skeleton, SkeletonGroup, TextField, ToastContext, ToastKind,
};
use crate::error_template::NotFound;
use crate::models::community::{short_date, SiteText};
use crate::pages::admin::bugs::err_text;

#[server]
pub async fn admin_list_site_texts() -> Result<(Vec<SiteText>, bool), ServerFnError> {
    use crate::server::{authz, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    let texts = store::list_site_texts(&db)
        .await
        .map_err(authz::server_err)?;
    Ok((texts, crate::server::config::get().support_url.is_some()))
}

#[server]
pub async fn admin_save_site_text(slug: String, body: String) -> Result<(), ServerFnError> {
    use crate::models::community::validate_site_text;
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let body = validate_site_text(&slug, &body).map_err(ServerFnError::new)?;
    store::admin_save_site_text(&db, &admin.id, &slug, &body)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, %slug, "site text saved");
    Ok(())
}

/// `approve = true` approves exactly `body` (what the admin saw);
/// `false` withdraws the approval.
#[server]
pub async fn admin_approve_site_text(
    slug: String,
    body: String,
    approve: bool,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    if approve {
        store::admin_approve_site_text(&db, &admin.id, &slug, &body)
            .await
            .map_err(authz::server_err)?;
    } else {
        store::admin_withdraw_site_text(&db, &admin.id, &slug)
            .await
            .map_err(authz::server_err)?;
    }
    tracing::info!(admin = %admin.id, %slug, approve, "site text approval changed");
    Ok(())
}

#[component]
pub fn AdminTextsPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| admin_list_site_texts());
    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading texts"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((texts, has_url)) => view! {
                    <PageShell width=PageWidth::Standard>
                        <PageHeader
                            title="Public texts"
                            eyebrow="Admin"
                            meta=|| view! { <span>"Wording about money that the owner approves before anyone sees it. Editing a text withdraws its approval."</span> }
                            actions=|| view! { <Button href="/admin" variant=ButtonVariant::Quiet>"All admin areas"</Button> }
                        />
                        <Callout tone=CalloutTone::Info>
                            {if has_url {
                                "SUPPORT_URL is set: the support button appears on the transparency page once the policy and the support sentence are approved."
                            } else {
                                "SUPPORT_URL is not set: no support button appears anywhere, whatever is approved here."
                            }}
                        </Callout>
                        <div class="mt-8 flex flex-col gap-12">
                            {texts.into_iter().map(|t| view! { <TextEditor text=t on_saved=Callback::new(move |_| data.refetch()) /> }).collect_view()}
                        </div>
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

fn title_for(slug: &str) -> &'static str {
    match slug {
        "money_policy" => "Money policy",
        "ledger_labels" => "Ledger labels",
        "ledger_split_legend" => "Split caption",
        "ledger_not_covered_note" => "Not covered caption",
        "support_sentence" => "Support sentence",
        _ => "Text",
    }
}

#[component]
fn TextEditor(text: SiteText, on_saved: Callback<()>) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let slug = StoredValue::new(text.slug.clone());
    let saved_body = StoredValue::new(text.body.clone());
    let body = RwSignal::new(text.body.clone());
    let busy = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let dirty = Signal::derive(move || body.get().trim() != saved_body.get_value().trim());
    let rows = if text.slug == "money_policy" || text.slug == "ledger_labels" {
        9
    } else {
        3
    };
    let call = move |save: bool, approve: bool| {
        busy.set(true);
        error.set(None);
        let (s, b) = (
            slug.get_value(),
            if save {
                body.get_untracked()
            } else {
                saved_body.get_value()
            },
        );
        spawn_local(async move {
            let r = if save {
                admin_save_site_text(s, b).await
            } else {
                admin_approve_site_text(s, b, approve).await
            };
            busy.set(false);
            match r {
                Ok(()) => {
                    if let Some(t) = toast {
                        t.show.run((
                            ToastKind::Success,
                            if save {
                                "Saved.".into()
                            } else if approve {
                                "Approved.".into()
                            } else {
                                "Approval withdrawn.".into()
                            },
                        ));
                    }
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    };
    let id = format!("text-{}", text.slug);
    let approved = text.approved;
    view! {
        <section aria-labelledby=format!("{id}-h") data-site-text=text.slug.clone()>
            <SectionHeader title=title_for(&text.slug) id=format!("{id}-h") />
            <div class="mb-3 flex flex-wrap items-center gap-3">
                {if approved {
                    view! { <Badge tone=BadgeTone::Success>"Approved"</Badge> }.into_any()
                } else {
                    view! { <Badge tone=BadgeTone::Warning>"Needs approval"</Badge> }.into_any()
                }}
                <span class="text-caption text-muted">
                    {match (&text.approved_by, &text.approved_at) {
                        (Some(by), Some(at)) if approved => format!("Approved by {by} on {}.", short_date(at)),
                        _ => match (&text.updated_by, &text.updated_at) {
                            (Some(by), Some(at)) => format!("Last edited by {by} on {}.", short_date(at)),
                            _ => "Proposed wording, not yet reviewed.".to_string(),
                        },
                    }}
                </span>
            </div>
            <Field id=id.clone() label=title_for(&text.slug) hint=text.purpose.clone()>
                <TextField value=body rows=rows maxlength=3000 />
            </Field>
            {move || error.get().map(|e| view! { <p class="mt-2 text-label text-danger" role="alert">{e}</p> })}
            <div class="mt-3 flex flex-wrap items-center gap-3">
                <Button variant=ButtonVariant::Secondary disabled=Signal::derive(move || busy.get() || !dirty.get()) on_click=Callback::new(move |_| call(true, false))>"Save"</Button>
                {if approved {
                    view! { <Button variant=ButtonVariant::Quiet disabled=busy on_click=Callback::new(move |_| call(false, false))>"Withdraw approval"</Button> }.into_any()
                } else {
                    view! { <Button variant=ButtonVariant::Secondary disabled=Signal::derive(move || busy.get() || dirty.get()) on_click=Callback::new(move |_| call(false, true))>"Approve this wording"</Button> }.into_any()
                }}
                {move || dirty.get().then(|| view! { <span class="text-caption text-muted">"Save your changes first."</span> })}
            </div>
        </section>
    }
}
