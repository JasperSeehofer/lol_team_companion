//! The survey band (M7): one short question, at most once a week, right
//! after something the user did. Mounted in `app.rs` at the top of `<main>`.
//!
//! The band asks the server only from the browser, after hydration and on
//! every route change, so server rendering, bots and prefetches never mark
//! a question as shown. It pushes the page down by one band (never an
//! overlay), never takes focus, uses no primary button (the page owns the
//! one primary action), and offers "Dismiss" and a link to turn questions
//! off on the profile.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{Button, ButtonSize, ButtonVariant, TextField};
use crate::models::community::{SurveyAnswer, SurveyPromptView};

/// The question to show on `path` now, if any. Signed-out callers get none.
#[server]
pub async fn get_pending_survey(path: String) -> Result<Option<SurveyPromptView>, ServerFnError> {
    use crate::server::{authz, community::store, rate_limit::Bucket};

    let Some(user) = authz::optional_user().await? else {
        return Ok(None);
    };
    // Runs on every route change and takes the global invariant lock, so a
    // client looping it gets nothing rather than slowing everyone down.
    if authz::rate_limit_user(Bucket::SurveyCheck, &user).is_err() {
        return Ok(None);
    }
    let db = authz::db()?;
    // The path is only compared with a constant.
    let path: String = path.chars().take(200).collect();
    store::get_pending_survey(&db, &user.id, &path)
        .await
        .map_err(authz::server_err)
}

/// Answer one's own shown question (`answer_json` is a `SurveyAnswer`).
#[server]
pub async fn answer_survey(prompt_id: String, answer_json: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store, rate_limit::Bucket};

    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    let answer: SurveyAnswer = serde_json::from_str(&answer_json).map_err(authz::server_err)?;
    store::answer_survey(&ctx.db, &ctx.user.id, &prompt_id, &answer)
        .await
        .map_err(authz::server_err)
}

/// Dismiss one's own question; it does not come back.
#[server]
pub async fn dismiss_survey(prompt_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store, rate_limit::Bucket};

    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    store::dismiss_survey(&ctx.db, &ctx.user.id, &prompt_id)
        .await
        .map_err(authz::server_err)
}

/// Whether the user turned survey questions off.
#[server]
pub async fn get_survey_opt_out() -> Result<bool, ServerFnError> {
    use crate::server::{authz, community::store};
    let ctx = authz::authed().await?;
    store::survey_opt_out(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

/// Turn survey questions off (`true`) or on.
#[server]
pub async fn set_survey_opt_out(off: bool) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store, rate_limit::Bucket};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    store::set_survey_opt_out(&ctx.db, &ctx.user.id, off)
        .await
        .map_err(authz::server_err)?;
    tracing::debug!(user = %ctx.user.id, off, "survey opt-out changed");
    Ok(())
}

/// Paths where the band never appears (same as the bug-report widget).
const HIDDEN_PREFIXES: &[&str] = crate::components::bug_report_widget::HIDDEN_PREFIXES;

/// The band in the app shell.
#[component]
pub fn SurveyPrompt() -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    let prompt: RwSignal<Option<SurveyPromptView>> = RwSignal::new(None);
    let thanks = RwSignal::new(false);
    // Effects run only in the browser: nothing is fetched while rendering
    // on the server.
    Effect::new(move |_| {
        let path = location.pathname.get();
        if HIDDEN_PREFIXES.iter().any(|p| path.starts_with(p)) {
            prompt.set(None);
            return;
        }
        spawn_local(async move {
            if let Ok(p) = get_pending_survey(path).await {
                if !thanks.get_untracked() {
                    prompt.set(p);
                }
            }
        });
    });
    view! {
        // A polite live region that exists before the band appears, so a
        // new question and the thank-you line are announced without taking
        // focus (a region inserted together with its text is often not read).
        <div aria-live="polite" data-survey-live>
        {move || {
            if thanks.get() {
                return view! {
                    <div class="border-b border-rule bg-ink-1">
                        <p class="mx-auto max-w-7xl px-4 py-3 text-body text-secondary sm:px-8">"Thanks. That helps."</p>
                    </div>
                }
                .into_any();
            }
            match prompt.get() {
                None => ().into_any(),
                Some(p) => view! {
                    <SurveyBand
                        prompt=p
                        preview=false
                        on_done=Callback::new(move |answered: bool| {
                            prompt.set(None);
                            if answered {
                                thanks.set(true);
                                hide_later(thanks);
                            }
                        })
                    />
                }
                .into_any(),
            }
        }}
        </div>
    }
}

/// Collapse the thank-you line after 4 s (at once under reduced motion).
fn hide_later(thanks: RwSignal<bool>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let reduced = web_sys::window()
            .and_then(|w| {
                w.match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .is_some_and(|m| m.matches());
        let delay = if reduced { 1500 } else { 4000 };
        let cb = Closure::once(move || thanks.set(false));
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                delay,
            );
        }
        cb.forget();
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = thanks;
}

/// One question with its answer controls. `preview` renders the same band
/// with inert buttons (the admin survey editor).
#[component]
pub fn SurveyBand(
    prompt: SurveyPromptView,
    preview: bool,
    #[prop(optional)] on_done: Option<Callback<bool>>,
) -> impl IntoView {
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let busy = RwSignal::new(false);
    let text = RwSignal::new(String::new());
    let id = StoredValue::new(prompt.prompt_id.clone());
    let question_id = format!("survey-q-{}", if preview { "preview" } else { "live" });

    let submit = move |answer: SurveyAnswer| {
        if preview || busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let json = serde_json::to_string(&answer).unwrap_or_default();
        spawn_local(async move {
            let result = answer_survey(id.get_value(), json).await;
            busy.set(false);
            match result {
                Ok(()) => {
                    if let Some(cb) = on_done {
                        cb.run(true);
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    error.set(Some(if msg.contains("closed") || msg.contains("already") {
                        "This question has closed. Thanks anyway.".to_string()
                    } else {
                        "Your answer could not be saved. Please try again.".to_string()
                    }));
                }
            }
        });
    };
    let submit = StoredValue::new(submit);

    let dismiss = move |_| {
        if preview {
            return;
        }
        spawn_local(async move {
            let _ = dismiss_survey(id.get_value()).await;
        });
        if let Some(cb) = on_done {
            cb.run(false);
        }
    };

    let controls = match prompt.kind.as_str() {
        "single_choice" => view! {
            <div class="flex flex-wrap gap-2" role="group" aria-labelledby=question_id.clone()>
                {prompt.choices.clone().into_iter().enumerate().map(|(i, c)| {
                    let i = i as u32;
                    view! {
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Secondary disabled=busy
                            on_click=Callback::new(move |_| submit.with_value(|f| f(SurveyAnswer::Choice(i))))>
                            {c}
                        </Button>
                    }
                }).collect_view()}
            </div>
        }
        .into_any(),
        "scale" => {
            let low = prompt.scale_low_label.clone().unwrap_or_default();
            let high = prompt.scale_high_label.clone().unwrap_or_default();
            view! {
                <div class="flex flex-wrap items-center gap-2">
                    <span class="text-caption text-secondary" aria-hidden="true">{low.clone()}</span>
                    <div class="flex gap-1" role="group" aria-labelledby=question_id.clone()>
                        {(1u32..=5).map(|v| {
                            let name = match v {
                                1 => format!("1, {low}"),
                                5 => format!("5, {high}"),
                                n => n.to_string(),
                            };
                            view! {
                                <Button size=ButtonSize::Sm variant=ButtonVariant::Secondary disabled=busy aria_label=name
                                    on_click=Callback::new(move |_| submit.with_value(|f| f(SurveyAnswer::Scale(v))))>
                                    {v.to_string()}
                                </Button>
                            }
                        }).collect_view()}
                    </div>
                    <span class="text-caption text-secondary" aria-hidden="true">{high.clone()}</span>
                </div>
            }
            .into_any()
        }
        _ => view! {
            <div class="flex w-full max-w-xl flex-col gap-2">
                <TextField value=text rows=2 maxlength=500 aria_label=prompt.question.clone() disabled=Signal::derive(move || preview) on_keydown=crate::components::ui::ctrl_enter_submits(Callback::new(move |_| if !busy.get_untracked() { submit.with_value(|f| f(SurveyAnswer::Text(text.get_untracked()))) })) />
                <div class="flex items-center justify-between gap-3">
                    <span class="figures text-caption text-muted">{move || format!("{} of 500", text.get().chars().count())}</span>
                    <Button size=ButtonSize::Sm variant=ButtonVariant::Secondary disabled=busy
                        on_click=Callback::new(move |_| submit.with_value(|f| f(SurveyAnswer::Text(text.get_untracked()))))>
                        "Send"
                    </Button>
                </div>
            </div>
        }
        .into_any(),
    };

    view! {
        <section class="border-b border-rule bg-ink-1" role="region" aria-label="Quick question" data-survey-band>
            <div class="mx-auto flex max-w-7xl flex-col gap-3 px-4 py-4 sm:px-8">
                <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                    <p class="eyebrow">"Quick question"</p>
                    {preview.then(|| view! { <p class="text-caption text-muted">"Preview. Buttons do nothing here."</p> })}
                </div>
                <p id=question_id class="text-body text-primary">{prompt.question.clone()}</p>
                <div class="flex flex-wrap items-center justify-between gap-3">
                    {controls}
                    <div class="flex items-center gap-4">
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(dismiss)>"Dismiss"</Button>
                        <a href="/profile#surveys" class="text-caption text-muted underline decoration-rule underline-offset-4 hover:text-primary">"Turn these questions off"</a>
                    </div>
                </div>
                {move || error.get().map(|e| view! { <p class="text-caption text-danger" role="alert">{e}</p> })}
            </div>
        </section>
    }
}
