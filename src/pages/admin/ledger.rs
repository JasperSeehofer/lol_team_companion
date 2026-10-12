//! `/admin/ledger`: the transparency ledger (M7). Admins enter each month's
//! support and costs as a draft, check the computed split and publish it.
//! A published month that is edited later is marked as revised. No figure
//! is ever seeded or estimated.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    Badge, BadgeTone, Button, ButtonSize, ButtonVariant, ConfirmDialog, ConfirmTone, Dialog,
    DialogSize, EmptySize, EmptyState, Field, PageHeader, PageShell, PageWidth, Skeleton,
    SkeletonGroup, Table, Td, TextField, Th, ToastContext, ToastKind, Tr,
};
use crate::error_template::NotFound;
use crate::models::community::{
    cents_to_input, format_euros, month_label, parse_euros, shortfall_cents, to_maintainer_cents,
    LedgerInput, LedgerMonth,
};
use crate::pages::admin::bugs::err_text;

#[server]
pub async fn admin_list_ledger() -> Result<(String, Vec<LedgerMonth>), ServerFnError> {
    use crate::server::{authz, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    let months = store::admin_list_ledger(&db)
        .await
        .map_err(authz::server_err)?;
    Ok((chrono::Utc::now().to_rfc3339(), months))
}

/// Returns whether the save was a revision of a published month.
#[server]
pub async fn admin_save_ledger_month(month_json: String) -> Result<bool, ServerFnError> {
    use crate::models::community::validate_ledger;
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let input: LedgerInput = serde_json::from_str(&month_json).map_err(authz::server_err)?;
    let current = chrono::Utc::now().format("%Y-%m").to_string();
    let m = validate_ledger(&input, &current).map_err(ServerFnError::new)?;
    let revision = store::admin_save_ledger_month(&db, &admin.id, &m)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, month = %m.month, revision, "ledger month saved");
    Ok(revision)
}

#[server]
pub async fn admin_publish_ledger_month(month: String, publish: bool) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_publish_ledger_month(&db, &admin.id, &month, publish)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, %month, publish, "ledger month publish state changed");
    Ok(())
}

#[server]
pub async fn admin_delete_ledger_month(month: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_delete_ledger_month(&db, &month)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, %month, "ledger draft deleted");
    Ok(())
}

#[derive(Clone, Copy)]
struct Form {
    open: RwSignal<bool>,
    editing: RwSignal<Option<String>>,
    month: RwSignal<String>,
    support: RwSignal<String>,
    hosting: RwSignal<String>,
    domain: RwSignal<String>,
    ai: RwSignal<String>,
    other: RwSignal<String>,
    other_label: RwSignal<String>,
    note: RwSignal<String>,
    error: RwSignal<Option<String>>,
}

impl Form {
    fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            editing: RwSignal::new(None),
            month: RwSignal::new(String::new()),
            support: RwSignal::new(String::new()),
            hosting: RwSignal::new(String::new()),
            domain: RwSignal::new(String::new()),
            ai: RwSignal::new(String::new()),
            other: RwSignal::new(String::new()),
            other_label: RwSignal::new(String::new()),
            note: RwSignal::new(String::new()),
            error: RwSignal::new(None),
        }
    }
    fn fill(&self, m: Option<&LedgerMonth>) {
        self.editing.set(m.map(|m| m.month.clone()));
        self.month
            .set(m.map(|m| m.month.clone()).unwrap_or_default());
        self.support.set(
            m.map(|m| cents_to_input(m.support_cents))
                .unwrap_or_default(),
        );
        self.hosting.set(
            m.map(|m| cents_to_input(m.hosting_cents))
                .unwrap_or_default(),
        );
        self.domain.set(
            m.map(|m| cents_to_input(m.domain_cents))
                .unwrap_or_default(),
        );
        self.ai.set(
            m.map(|m| cents_to_input(m.ai_tokens_cents))
                .unwrap_or_default(),
        );
        self.other
            .set(m.map(|m| cents_to_input(m.other_cents)).unwrap_or_default());
        self.other_label
            .set(m.and_then(|m| m.other_label.clone()).unwrap_or_default());
        self.note
            .set(m.and_then(|m| m.note.clone()).unwrap_or_default());
        self.error.set(None);
        self.open.set(true);
    }
    fn input(&self) -> LedgerInput {
        LedgerInput {
            month: self.month.get_untracked(),
            support: self.support.get_untracked(),
            hosting: self.hosting.get_untracked(),
            domain: self.domain.get_untracked(),
            ai_tokens: self.ai.get_untracked(),
            other: self.other.get_untracked(),
            other_label: self.other_label.get_untracked(),
            note: self.note.get_untracked(),
        }
    }
}

#[component]
pub fn AdminLedgerPage() -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let data = Resource::new(|| (), |_| admin_list_ledger());
    let form = Form::new();
    let saving = RwSignal::new(false);
    // The month a confirmation is about.
    let target = RwSignal::new(String::new());
    let confirm_publish = RwSignal::new(false);
    let confirm_delete = RwSignal::new(false);
    let editing_published = RwSignal::new(false);

    let notify = move |kind: ToastKind, msg: String| {
        if let Some(t) = toast {
            t.show.run((kind, msg));
        }
    };
    let save = move |_| {
        let json = serde_json::to_string(&form.input()).unwrap_or_default();
        form.error.set(None);
        saving.set(true);
        spawn_local(async move {
            let r = admin_save_ledger_month(json).await;
            saving.set(false);
            match r {
                Ok(revision) => {
                    form.open.set(false);
                    notify(
                        ToastKind::Success,
                        if revision {
                            "Revision saved. It is marked as revised on the public page.".into()
                        } else {
                            "Draft saved.".into()
                        },
                    );
                    data.refetch();
                }
                Err(e) => form.error.set(Some(err_text(&e))),
            }
        });
    };
    let publish = Callback::new(move |publish: bool| {
        let m = target.get_untracked();
        spawn_local(async move {
            match admin_publish_ledger_month(m, publish).await {
                Ok(()) => {
                    notify(
                        ToastKind::Success,
                        if publish {
                            "Published.".into()
                        } else {
                            "Unpublished.".into()
                        },
                    );
                    data.refetch();
                }
                Err(e) => notify(ToastKind::Error, err_text(&e)),
            }
        });
    });
    let delete = Callback::new(move |_| {
        let m = target.get_untracked();
        spawn_local(async move {
            match admin_delete_ledger_month(m).await {
                Ok(()) => {
                    notify(ToastKind::Success, "Draft deleted.".into());
                    data.refetch();
                }
                Err(e) => notify(ToastKind::Error, err_text(&e)),
            }
        });
    });

    let computed = move || {
        let c = |s: RwSignal<String>| parse_euros(&s.get()).ok();
        let (Some(sup), Some(h), Some(d), Some(a), Some(o)) = (
            c(form.support),
            c(form.hosting),
            c(form.domain),
            c(form.ai),
            c(form.other),
        ) else {
            return "Amounts must be zero or more, in euros with up to two decimals.".to_string();
        };
        let costs = h + d + a + o;
        let rest = if shortfall_cents(sup, costs) > 0 {
            format!(
                "Not covered by support: {}",
                format_euros(shortfall_cents(sup, costs))
            )
        } else {
            format!(
                "To the maintainer: {}",
                format_euros(to_maintainer_cents(sup, costs))
            )
        };
        format!("Running costs: {} · {rest}", format_euros(costs))
    };
    let amount =
        |id: &'static str, label: &'static str, value: RwSignal<String>, hint: &'static str| {
            view! {
                <Field id=id label=label hint=hint>
                    <TextField value=value placeholder="0.00" attr:inputmode="decimal" />
                </Field>
            }
        };

    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading the ledger"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((now, months)) => view! {
                    <PageShell width=PageWidth::Wide>
                        <PageHeader
                            title="Transparency ledger"
                            eyebrow="Admin"
                            meta=|| view! { <span>"Enter each month's support and costs. Nothing is public until you publish it, and the public page also waits for the approved texts on Public texts."</span> }
                            actions=move || view! {
                                <Button href="/admin" variant=ButtonVariant::Quiet>"All admin areas"</Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| { editing_published.set(false); form.fill(None) })>"Add month"</Button>
                            }
                        />
                        <MonthsTable months=months now=now form=form target=target confirm_publish=confirm_publish confirm_delete=confirm_delete editing_published=editing_published publish=publish />
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
        <Dialog
            id="ledger-form"
            open=form.open
            title="Month"
            size=DialogSize::Md
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| form.open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary loading=saving on_click=Callback::new(save)>
                    {move || if editing_published.get() { "Save revision" } else { "Save as draft" }}
                </Button>
            }
        >
            <div class="flex flex-col gap-4">
                <Field id="ledger-month" label="Month" hint="Year and month, for example 2026-10.">
                    <TextField value=form.month placeholder="2026-10" disabled=Signal::derive(move || form.editing.get().is_some()) />
                </Field>
                {amount("ledger-support", "Support received", form.support, "In euros, for example 12.50. What actually arrived after the payment platform's fees.")}
                <div class="grid gap-4 sm:grid-cols-3">
                    {amount("ledger-hosting", "Hosting", form.hosting, "In euros")}
                    {amount("ledger-domain", "Domain", form.domain, "In euros")}
                    {amount("ledger-ai", "AI tokens", form.ai, "In euros")}
                </div>
                <div class="grid gap-4 sm:grid-cols-[10rem_minmax(0,1fr)]">
                    {amount("ledger-other", "Other costs", form.other, "In euros")}
                    <Field id="ledger-other-label" label="What the other costs were">
                        <TextField value=form.other_label maxlength=120 />
                    </Field>
                </div>
                <Field id="ledger-note" label="Public note" hint="Shown under the month on the transparency page.">
                    <TextField value=form.note rows=2 maxlength=600 />
                </Field>
                <p class="figures text-label text-secondary" data-testid="ledger-computed">{computed}</p>
                {move || form.error.get().map(|e| view! { <p class="text-label text-danger" role="alert">{e}</p> })}
            </div>
        </Dialog>
        <ConfirmDialog
            id="ledger-publish"
            open=confirm_publish
            title="Publish this month?"
            message="Everyone can see it on the transparency page. Later edits are marked as revisions."
            confirm_label="Publish"
            on_confirm=Callback::new(move |_| publish.run(true))
        />
        <ConfirmDialog
            id="ledger-delete"
            open=confirm_delete
            title="Delete this draft?"
            message="This cannot be undone."
            confirm_label="Delete draft"
            tone=ConfirmTone::Destructive
            on_confirm=delete
        />
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn MonthsTable(
    months: Vec<LedgerMonth>,
    now: String,
    form: Form,
    target: RwSignal<String>,
    confirm_publish: RwSignal<bool>,
    confirm_delete: RwSignal<bool>,
    editing_published: RwSignal<bool>,
    publish: Callback<bool>,
) -> impl IntoView {
    if months.is_empty() {
        return view! { <EmptyState size=EmptySize::Compact message="No months yet. Add the first month; it stays a draft until you publish it." /> }.into_any();
    }
    let now = chrono::DateTime::parse_from_rfc3339(&now).ok();
    view! {
        <Table caption="Months">
            <thead>
                <tr>
                    <Th>"Month"</Th>
                    <Th numeric=true>"Support"</Th>
                    <Th numeric=true>"Running costs"</Th>
                    <Th numeric=true>"To the maintainer"</Th>
                    <Th>"State"</Th>
                    <Th><span class="sr-only">"Actions"</span></Th>
                </tr>
            </thead>
            <tbody>
                {months.into_iter().map(|m| {
                    let label = month_label(&m.month);
                    let state = if !m.published {
                        if m.published_at.is_some() { ("Unpublished", BadgeTone::Warning) } else { ("Draft", BadgeTone::Neutral) }
                    } else if m.revised_at.is_some() {
                        ("Revised", BadgeTone::Info)
                    } else {
                        ("Published", BadgeTone::Success)
                    };
                    let within_day = m.published && m.published_at.as_deref()
                        .and_then(|p| chrono::DateTime::parse_from_rfc3339(p).ok())
                        .zip(now)
                        .is_some_and(|(p, n)| (n - p) < chrono::Duration::hours(24));
                    let was_public = m.published || m.published_at.is_some();
                    let row = StoredValue::new(m.clone());
                    let (support, costs, to_m, short) = (m.support_cents, m.costs_cents(), m.to_maintainer_cents(), m.not_covered_cents());
                    let published = m.published;
                    let key = m.month.clone();
                    let (k1, k2, k3) = (key.clone(), key.clone(), key);
                    view! {
                        <Tr>
                            <Td header=true>{label}</Td>
                            <Td numeric=true>{format_euros(support)}</Td>
                            <Td numeric=true>{format_euros(costs)}</Td>
                            <Td numeric=true>
                                {format_euros(to_m)}
                                {(short > 0).then(|| view! { <span class="block text-caption text-secondary">{format!("Not covered: {}", format_euros(short))}</span> })}
                            </Td>
                            <Td><Badge tone=state.1>{state.0}</Badge></Td>
                            <Td>
                                <span class="flex flex-wrap justify-end gap-2">
                                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| {
                                        editing_published.set(was_public);
                                        row.with_value(|r| form.fill(Some(r)));
                                    })>"Edit"</Button>
                                    {(!published).then(|| view! {
                                        <Button size=ButtonSize::Sm variant=ButtonVariant::Secondary on_click=Callback::new(move |_| { target.set(k1.clone()); confirm_publish.set(true); })>"Publish"</Button>
                                    })}
                                    {within_day.then(|| view! {
                                        <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| { target.set(k2.clone()); publish.run(false); })>"Unpublish"</Button>
                                    })}
                                    {(!was_public).then(|| view! {
                                        <Button size=ButtonSize::Sm variant=ButtonVariant::Danger on_click=Callback::new(move |_| { target.set(k3.clone()); confirm_delete.set(true); })>"Delete draft"</Button>
                                    })}
                                </span>
                            </Td>
                        </Tr>
                    }
                }).collect_view()}
            </tbody>
        </Table>
    }
    .into_any()
}
