//! `/admin/surveys`: micro-surveys (M7). An admin writes one question, the
//! owner approves the exact wording (any wording change withdraws the
//! approval), and only then can it go live. Results group answers by the
//! wording people saw; text answers carry no names.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::components::survey_prompt::SurveyBand;
use crate::components::ui::{
    Badge, BadgeTone, BarInk, BarSegment, Button, ButtonVariant, Callout, CalloutTone,
    ConfirmDialog, ConfirmTone, EmptySize, EmptyState, Field, PageHeader, PageShell, PageWidth,
    ProportionBar, ReadoutSize, SectionHeader, Select, SelectOption, Skeleton, SkeletonGroup,
    StatReadout, TabItem, TabPanel, Table, Tabs, Td, TextField, Th, ToastContext, ToastKind, Tr,
};
use crate::error_template::NotFound;
use crate::models::community::{
    percent, short_date, Survey, SurveyInput, SurveyKind, SurveyPromptView, SurveyResults,
    SurveyStatus, SurveyTrigger,
};
use crate::pages::admin::bugs::err_text;
use crate::pages::admin::ListRow;

#[server]
pub async fn admin_list_surveys() -> Result<Vec<Survey>, ServerFnError> {
    use crate::server::{authz, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_list_surveys(&db)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn admin_save_survey(survey_json: String) -> Result<String, ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let input: SurveyInput = serde_json::from_str(&survey_json).map_err(authz::server_err)?;
    let id = store::admin_save_survey(&db, &admin.id, &input)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, survey = %id, "survey saved");
    Ok(id)
}

#[server]
pub async fn admin_approve_survey(id: String, wording_hash: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_approve_survey(&db, &admin.id, &id, &wording_hash)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, survey = %id, hash = %wording_hash, "survey wording approved");
    Ok(())
}

#[server]
pub async fn admin_set_survey_status(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let to = SurveyStatus::parse(&status).ok_or_else(|| ServerFnError::new("Unknown status"))?;
    store::admin_set_survey_status(&db, &admin.id, &id, to)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, survey = %id, status = %status, "survey status changed");
    Ok(())
}

#[server]
pub async fn admin_delete_survey(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_delete_survey(&db, &id)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, survey = %id, "survey deleted");
    Ok(())
}

#[server]
pub async fn admin_survey_results(id: String) -> Result<SurveyResults, ServerFnError> {
    use crate::server::{authz, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_survey_results(&db, &id)
        .await
        .map_err(authz::server_err)
}

fn status_badge(s: &Survey) -> AnyView {
    let status = SurveyStatus::parse(&s.status).unwrap_or(SurveyStatus::Draft);
    if !s.approval_valid() && status != SurveyStatus::Closed {
        return view! { <Badge tone=BadgeTone::Warning>"Needs approval"</Badge> }.into_any();
    }
    let tone = match status {
        SurveyStatus::Approved => BadgeTone::Info,
        SurveyStatus::Live => BadgeTone::Success,
        SurveyStatus::Paused => BadgeTone::Warning,
        _ => BadgeTone::Neutral,
    };
    view! { <Badge tone=tone>{status.label()}</Badge> }.into_any()
}

#[component]
pub fn AdminSurveysPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| admin_list_surveys());
    let query = use_query_map();
    let selected = Signal::derive(move || query.get().get("id"));
    // Results load for the selected survey (created here, before any
    // Suspense, so resource ids match between server and browser).
    let results = Resource::new(
        move || selected.get(),
        |id| async move {
            match id {
                Some(id) if id != "new" => admin_survey_results(id).await.ok(),
                _ => None,
            }
        },
    );
    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading surveys"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok(list) => view! {
                    <PageShell width=PageWidth::Wide>
                        <PageHeader
                            title="Surveys"
                            eyebrow="Admin"
                            meta=|| view! { <span>"One question at a time, at most one a week per person, only after something they just did."</span> }
                            actions=|| view! {
                                <Button href="/admin" variant=ButtonVariant::Quiet>"All admin areas"</Button>
                                <Button href="/admin/surveys?id=new" variant=ButtonVariant::Secondary>"New survey"</Button>
                            }
                        />
                        <Surveys list=list selected=selected results=results on_saved=Callback::new(move |_| { data.refetch(); results.refetch(); }) />
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn Surveys(
    list: Vec<Survey>,
    selected: Signal<Option<String>>,
    results: Resource<Option<SurveyResults>>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let list = StoredValue::new(list);
    view! {
        <div class="grid gap-8 lg:grid-cols-[minmax(0,24rem)_minmax(0,1fr)]">
            <div class="min-w-0">
                {if list.get_value().is_empty() {
                    view! { <EmptyState size=EmptySize::Compact message="No surveys yet. Write one with New survey; it stays a draft until the wording is approved." /> }.into_any()
                } else {
                    view! {
                        <ul class="flex flex-col border-t border-rule" aria-label="Surveys">
                            {list.get_value().into_iter().map(|s| {
                                let id = s.id.clone();
                                let is_sel = Signal::derive({
                                    let id = id.clone();
                                    move || selected.get().as_deref() == Some(id.as_str())
                                });
                                let q: String = s.wording.question.chars().take(80).collect();
                                let trigger = SurveyTrigger::parse(&s.trigger).map(|t| t.label()).unwrap_or("");
                                view! {
                                    <li class="border-b border-rule">
                                        <ListRow href=format!("/admin/surveys?id={id}") selected=is_sel>
                                            <span class="flex w-full flex-col items-start gap-1 py-1 text-left">
                                                <span class="text-body text-primary">{q}</span>
                                                <span class="flex flex-wrap items-center gap-2 text-caption text-muted">
                                                    {status_badge(&s)}
                                                    <span>{trigger}</span>
                                                </span>
                                            </span>
                                        </ListRow>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </div>
            <div class="min-w-0">
                {move || match selected.get().as_deref() {
                    Some("new") => view! { <Editor survey=None results=results on_saved=on_saved /> }.into_any(),
                    Some(id) => match list.get_value().into_iter().find(|s| s.id == id) {
                        Some(s) => view! { <Editor survey=Some(s) results=results on_saved=on_saved /> }.into_any(),
                        None => view! { <EmptyState size=EmptySize::Compact message="That survey no longer exists." /> }.into_any(),
                    },
                    None => view! { <EmptyState size=EmptySize::Compact message="Choose a survey from the list, or write a new one." /> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn Editor(
    survey: Option<Survey>,
    results: Resource<Option<SurveyResults>>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let navigate = StoredValue::new(use_navigate());
    let saved = StoredValue::new(survey.clone());
    let id = StoredValue::new(survey.as_ref().map(|s| s.id.clone()));
    let w = survey
        .as_ref()
        .map(|s| s.wording.clone())
        .unwrap_or_default();
    let question = RwSignal::new(w.question.clone());
    let kind = RwSignal::new(if w.kind.is_empty() {
        "single_choice".to_string()
    } else {
        w.kind.clone()
    });
    let choices = RwSignal::new(w.choices.join("\n"));
    let low = RwSignal::new(w.scale_low_label.clone().unwrap_or_default());
    let high = RwSignal::new(w.scale_high_label.clone().unwrap_or_default());
    let trigger = RwSignal::new(
        survey
            .as_ref()
            .map(|s| s.trigger.clone())
            .unwrap_or_else(|| "review_saved".into()),
    );
    let note = RwSignal::new(
        survey
            .as_ref()
            .and_then(|s| s.note.clone())
            .unwrap_or_default(),
    );
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let busy = RwSignal::new(false);
    let tab = RwSignal::new("wording".to_string());
    let confirm_close = RwSignal::new(false);
    let confirm_wording = RwSignal::new(false);

    let input = move || SurveyInput {
        id: id.get_value(),
        question: question.get(),
        kind: kind.get(),
        choices_text: choices.get(),
        scale_low_label: low.get(),
        scale_high_label: high.get(),
        trigger: trigger.get(),
        note: note.get(),
    };
    // Unsaved wording changes (compared with what was loaded).
    let dirty = Signal::derive(move || {
        let i = input();
        match saved.get_value() {
            None => true,
            Some(s) => {
                let lines: Vec<String> = i
                    .choices_text
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                i.question.trim() != s.wording.question
                    || i.kind != s.wording.kind
                    || (i.kind == "single_choice" && lines != s.wording.choices)
                    || (i.kind == "scale"
                        && (Some(i.scale_low_label.trim().to_string())
                            != s.wording.scale_low_label
                            || Some(i.scale_high_label.trim().to_string())
                                != s.wording.scale_high_label))
                    || i.trigger != s.trigger
                    || i.note.trim() != s.note.clone().unwrap_or_default()
            }
        }
    });
    let status = survey
        .as_ref()
        .and_then(|s| SurveyStatus::parse(&s.status))
        .unwrap_or(SurveyStatus::Draft);
    let approved = survey.as_ref().is_some_and(|s| s.approval_valid());
    let withdrawn = survey.as_ref().is_some_and(|s| s.approval_withdrawn());

    let run = move |fut: std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, ServerFnError>>>,
    >,
                    ok: &'static str| {
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let r = fut.await;
            busy.set(false);
            match r {
                Ok(new_id) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Success, ok.into()));
                    }
                    if !new_id.is_empty() {
                        let url = format!("/admin/surveys?id={new_id}");
                        navigate.with_value(|n| {
                            n(
                                &url,
                                leptos_router::NavigateOptions {
                                    replace: true,
                                    scroll: false,
                                    ..Default::default()
                                },
                            )
                        });
                    }
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    };
    let do_save = move || {
        let json = serde_json::to_string(&input()).unwrap_or_default();
        run(Box::pin(admin_save_survey(json)), "Survey saved.");
    };
    let save = move |_| {
        if approved
            && status != SurveyStatus::Closed
            && wording_changed(&saved.get_value(), &input())
        {
            confirm_wording.set(true);
        } else {
            do_save();
        }
    };
    let set_status = move |to: SurveyStatus, ok: &'static str| {
        let Some(sid) = id.get_value() else { return };
        run(
            Box::pin(async move {
                admin_set_survey_status(sid, to.as_str().to_string())
                    .await
                    .map(|_| String::new())
            }),
            ok,
        );
    };
    let approve = move |_| {
        let (Some(sid), Some(s)) = (id.get_value(), saved.get_value()) else {
            return;
        };
        let hash = s.wording_hash.clone();
        run(
            Box::pin(async move { admin_approve_survey(sid, hash).await.map(|_| String::new()) }),
            "Wording approved.",
        );
    };
    let delete = Callback::new(move |_| {
        let Some(sid) = id.get_value() else { return };
        spawn_local(async move {
            match admin_delete_survey(sid).await {
                Ok(()) => {
                    navigate.with_value(|n| n("/admin/surveys", Default::default()));
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    });

    let preview = Signal::derive(move || {
        let i = input();
        SurveyPromptView {
            prompt_id: String::new(),
            question: if i.question.trim().is_empty() {
                "Your question appears here.".into()
            } else {
                i.question.replace("{patch}", "16.21")
            },
            kind: i.kind.clone(),
            choices: i
                .choices_text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect(),
            scale_low_label: Some(i.scale_low_label.clone()),
            scale_high_label: Some(i.scale_high_label.clone()),
        }
    });
    let kind_options: Vec<SelectOption> = SurveyKind::ALL
        .iter()
        .map(|k| SelectOption::new(k.as_str(), k.label()))
        .collect();
    let trigger_options: Vec<SelectOption> = SurveyTrigger::ALL
        .iter()
        .map(|t| SelectOption::new(t.as_str(), t.label()))
        .collect();
    let approval_line = survey.as_ref().map(|s| {
        if s.approval_valid() {
            (CalloutTone::Success, format!(
                "Approved by {} on {}.",
                s.approved_by.clone().unwrap_or_else(|| "an admin".into()),
                s.approved_at.as_deref().map(short_date).unwrap_or_default()
            ))
        } else if withdrawn {
            (CalloutTone::Warning, "The wording changed after approval, so the approval was withdrawn. Approve the new wording to go live again.".to_string())
        } else {
            (CalloutTone::Warning, "Not approved. The owner must approve this exact wording before it can go live.".to_string())
        }
    });
    let has_survey = survey.is_some();
    let can_delete = survey.as_ref().is_some_and(|s| {
        s.status == "draft" && s.counts.delivered() + s.counts.pending + s.counts.expired == 0
    });

    let tabs = vec![
        TabItem::new("wording", "Wording"),
        TabItem::new("results", "Results"),
    ];
    view! {
        <div data-testid="survey-editor">
            {if has_survey {
                view! { <Tabs group="survey" label="Survey" tabs=tabs selected=tab /> }.into_any()
            } else {
                ().into_any()
            }}
            <TabPanel group="survey" tab="wording" selected=tab>
                <div class="mt-6 flex flex-col gap-6">
                    {approval_line.map(|(tone, text)| view! { <Callout tone=tone>{text}</Callout> })}
                    <Field id="survey-question" label="Question" hint="One question, up to 200 characters. Plain words, no leading questions.">
                        <TextField value=question maxlength=200 />
                    </Field>
                    <Field id="survey-kind" label="Type">
                        <Select value=kind options=kind_options />
                    </Field>
                    {move || match kind.get().as_str() {
                        "single_choice" => view! {
                            <Field id="survey-choices" label="Options, one per line" hint="Two to five options, up to 60 characters each.">
                                <TextField value=choices rows=5 />
                            </Field>
                        }.into_any(),
                        "scale" => view! {
                            <div class="grid gap-4 sm:grid-cols-2">
                                <Field id="survey-low" label="Label for 1"><TextField value=low maxlength=30 /></Field>
                                <Field id="survey-high" label="Label for 5"><TextField value=high maxlength=30 /></Field>
                            </div>
                        }.into_any(),
                        _ => ().into_any(),
                    }}
                    <Field id="survey-trigger" label="When to ask">
                        <Select value=trigger options=trigger_options />
                    </Field>
                    <p class="-mt-4 text-caption text-muted">{move || SurveyTrigger::parse(&trigger.get()).map(|t| t.explanation()).unwrap_or("")}</p>
                    <Field id="survey-note" label="What we want to learn (admins only)">
                        <TextField value=note rows=2 maxlength=500 />
                    </Field>
                    <section aria-labelledby="survey-preview-h">
                        <SectionHeader title="Preview" id="survey-preview-h" level=crate::components::ui::HeadingLevel::H3 />
                        <div class="border-t border-rule">
                            {move || view! { <SurveyBand prompt=preview.get() preview=true /> }}
                        </div>
                    </section>
                    {move || error.get().map(|e| view! { <p class="text-label text-danger" role="alert">{e}</p> })}
                    <div class="flex flex-wrap items-center gap-3">
                        <Button variant=ButtonVariant::Secondary disabled=Signal::derive(move || busy.get() || status == SurveyStatus::Closed) on_click=Callback::new(save)>"Save"</Button>
                        {(has_survey && status != SurveyStatus::Closed).then(|| view! {
                            <Button
                                variant=ButtonVariant::Secondary
                                disabled=Signal::derive(move || busy.get() || dirty.get() || approved)
                                on_click=Callback::new(approve)
                            >"Approve wording"</Button>
                            {match status {
                                SurveyStatus::Live => view! {
                                    <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| set_status(SurveyStatus::Paused, "Survey paused."))>"Pause"</Button>
                                }.into_any(),
                                SurveyStatus::Paused => view! {
                                    <Button variant=ButtonVariant::Primary disabled=Signal::derive(move || !approved || dirty.get()) on_click=Callback::new(move |_| set_status(SurveyStatus::Live, "Survey is live."))>"Resume"</Button>
                                }.into_any(),
                                _ => view! {
                                    <Button variant=ButtonVariant::Primary disabled=Signal::derive(move || !approved || dirty.get()) on_click=Callback::new(move |_| set_status(SurveyStatus::Live, "Survey is live."))>"Go live"</Button>
                                }.into_any(),
                            }}
                            <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| confirm_close.set(true))>"Close survey"</Button>
                        })}
                        {can_delete.then(|| view! {
                            <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| delete.run(()))>"Delete draft"</Button>
                        })}
                    </div>
                    {(!approved && has_survey && status != SurveyStatus::Closed).then(|| view! {
                        <p class="text-caption text-muted">{move || if dirty.get() { "Save your changes first." } else { "Approve the wording first." }}</p>
                    })}
                </div>
            </TabPanel>
            {has_survey.then(|| view! {
                <TabPanel group="survey" tab="results" selected=tab>
                    <Suspense fallback=|| view! { <Skeleton lines=4 /> }>
                        {move || results.get().map(|r| match r {
                            Some(r) => view! { <Results r=r /> }.into_any(),
                            None => view! { <EmptyState size=EmptySize::Compact message="No answers yet. Answers appear here as people respond; names are never shown." /> }.into_any(),
                        })}
                    </Suspense>
                </TabPanel>
            })}
            <ConfirmDialog
                id="survey-close"
                open=confirm_close
                title="Close this survey?"
                message="No one will be asked this question again. Results stay available."
                confirm_label="Close survey"
                tone=ConfirmTone::Destructive
                on_confirm=Callback::new(move |_| set_status(SurveyStatus::Closed, "Survey closed."))
            />
            <ConfirmDialog
                id="survey-wording"
                open=confirm_wording
                title="Change approved wording?"
                message="Saving withdraws the approval and takes the survey offline until the new wording is approved."
                confirm_label="Save and withdraw approval"
                on_confirm=Callback::new(move |_| do_save())
            />
        </div>
    }
}

fn wording_changed(saved: &Option<Survey>, i: &SurveyInput) -> bool {
    let Some(s) = saved else { return true };
    let lines: Vec<String> = i
        .choices_text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    i.question.trim() != s.wording.question
        || i.kind != s.wording.kind
        || (i.kind == "single_choice" && lines != s.wording.choices)
        || (i.kind == "scale"
            && (Some(i.scale_low_label.trim().to_string()) != s.wording.scale_low_label
                || Some(i.scale_high_label.trim().to_string()) != s.wording.scale_high_label))
}

#[component]
fn Results(r: SurveyResults) -> impl IntoView {
    let c = r.counts.clone();
    let delivered = c.delivered();
    if r.versions.is_empty() {
        return view! {
            <div class="mt-6 flex flex-col gap-6">
                <div class="grid grid-cols-3 gap-6">
                    <StatReadout size=ReadoutSize::Sm label="Shown" value=delivered.to_string() />
                    <StatReadout size=ReadoutSize::Sm label="Answered" value=c.answered.to_string() />
                    <StatReadout size=ReadoutSize::Sm label="Dismissed" value=c.dismissed.to_string() />
                </div>
                <EmptyState size=EmptySize::Compact message="No answers yet. Answers appear here as people respond; names are never shown." />
            </div>
        }
        .into_any();
    }
    let kind = r.kind.clone();
    view! {
        <div class="mt-6 flex flex-col gap-8" data-testid="survey-results">
            <div class="grid grid-cols-3 gap-6">
                <StatReadout size=ReadoutSize::Sm label="Shown" value=delivered.to_string() />
                <StatReadout size=ReadoutSize::Sm label="Answered" value=c.answered.to_string() />
                <StatReadout size=ReadoutSize::Sm label="Dismissed" value=c.dismissed.to_string() />
            </div>
            <p class="text-label text-secondary">{format!("{} answered", percent(c.answered, delivered))}</p>
            {r.versions.into_iter().enumerate().map(|(i, v)| {
                let heading = if i == 0 && !v.question.is_empty() { "Current wording".to_string() } else { "Earlier wording (changed since)".to_string() };
                let total = v.answers;
                let mean = v.mean();
                let scale_n: u32 = v.scale_counts.iter().sum();
                view! {
                    <section class="flex flex-col gap-4">
                        <SectionHeader title=heading level=crate::components::ui::HeadingLevel::H3 />
                        {(!v.question.is_empty()).then(|| view! { <p class="text-body text-secondary">{v.question.clone()}</p> })}
                        {match kind.as_str() {
                            "single_choice" => view! {
                                <Table caption="Answers per option" caption_hidden=true>
                                    <thead><tr><Th>"Answer"</Th><Th numeric=true>"Count"</Th><Th numeric=true>"Share"</Th><Th><span class="sr-only">"Bar"</span></Th></tr></thead>
                                    <tbody>
                                        {v.choice_counts.clone().into_iter().map(|(label, n)| { let shown = label.clone(); view! {
                                            <Tr>
                                                <Td>{shown}</Td>
                                                <Td numeric=true>{n}</Td>
                                                <Td numeric=true>{percent(n, total)}</Td>
                                                <Td><div class="w-40"><ProportionBar max=total as i64 segments=vec![BarSegment::new(n as i64, label, BarInk::Strong)] /></div></Td>
                                            </Tr>
                                        }}).collect_view()}
                                    </tbody>
                                </Table>
                            }.into_any(),
                            "scale" => view! {
                                <Table caption="Answers per score" caption_hidden=true>
                                    <thead><tr><Th>"Score"</Th><Th numeric=true>"Count"</Th><Th numeric=true>"Share"</Th><Th><span class="sr-only">"Bar"</span></Th></tr></thead>
                                    <tbody>
                                        {v.scale_counts.clone().into_iter().enumerate().map(|(s, n)| view! {
                                            <Tr>
                                                <Td>{s + 1}</Td>
                                                <Td numeric=true>{n}</Td>
                                                <Td numeric=true>{percent(n, scale_n)}</Td>
                                                <Td><div class="w-40"><ProportionBar max=scale_n as i64 segments=vec![BarSegment::new(n as i64, format!("{}", s + 1), BarInk::Strong)] /></div></Td>
                                            </Tr>
                                        }).collect_view()}
                                    </tbody>
                                </Table>
                                <p class="text-label text-secondary">{match mean {
                                    Some(m) => format!("Average {m:.1} from {scale_n} answers"),
                                    None => "The average appears from 5 answers.".to_string(),
                                }}</p>
                            }.into_any(),
                            _ => view! {
                                <div>
                                    <h4 class="text-label font-semibold text-primary">"Answers"</h4>
                                    <ul class="mt-2 flex flex-col">
                                        {v.texts.clone().into_iter().map(|(date, t)| view! {
                                            <li class="border-b border-rule py-2">
                                                <p class="whitespace-pre-wrap text-body text-primary">{t}</p>
                                                <p class="text-caption text-muted">{short_date(&date)}</p>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            }.into_any(),
                        }}
                    </section>
                }
            }).collect_view()}
        </div>
    }
    .into_any()
}
