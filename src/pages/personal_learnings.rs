// The #[server] macro generates a client stub that does not inherit item-level
// attributes, so clippy::too_many_arguments is allowed for this file: save/update
// server fns take one argument per form field. A params struct is planned with
// the API-layer refactor.
#![allow(clippy::too_many_arguments)]

//! Learnings: the personal journal. What happened, what I learned, what I
//! do next time, by champion and matchup, with tags; written from scratch or
//! from a match's timeline.

use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ChampionAvatar, Checkbox, EmptySize, EmptyState, Field, GameResult, GlyphSize, PageHeader,
    PageShell, PageWidth, ResultGlyph, SectionHeader, Segment, SegmentSize, SegmentedControl,
    Select, SelectOption, Skeleton, SkeletonGroup, TextField, Toast, ToastContext,
};
use crate::models::champion::Champion;
use crate::models::personal_learning::{PersonalLearning, LEARNING_TAGS};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions (defined before components per CLAUDE.md rule 34)
// ---------------------------------------------------------------------------

#[server]
pub async fn list_learnings() -> Result<Vec<PersonalLearning>, ServerFnError> {
    use crate::server::{authz, db};

    let Some(user) = authz::optional_user().await? else {
        return Ok(Vec::new());
    };
    let db = authz::db()?;

    db::list_personal_learnings(&db, &user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_learning(id: String) -> Result<Option<PersonalLearning>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    db::get_personal_learning(&db, &user.id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_learning(
    title: String,
    learning_type: String,
    champion: String,
    opponent: String,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    tags_json: String,
    win_loss: String,
    match_riot_id: String,
    game_timestamp_ms: String,
    event_name: String,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(authz::server_err)?;

    let learning = PersonalLearning {
        id: None,
        user_id: user.id,
        title,
        learning_type,
        champion: if champion.is_empty() {
            None
        } else {
            Some(champion)
        },
        opponent: if opponent.is_empty() {
            None
        } else {
            Some(opponent)
        },
        what_happened,
        what_i_learned,
        next_time,
        tags,
        win_loss: if win_loss.is_empty() {
            None
        } else {
            Some(win_loss)
        },
        match_riot_id: if match_riot_id.is_empty() {
            None
        } else {
            Some(match_riot_id)
        },
        game_timestamp_ms: game_timestamp_ms.parse::<i64>().ok(),
        event_name: if event_name.is_empty() {
            None
        } else {
            Some(event_name)
        },
        created_at: None,
    };

    db::create_personal_learning(&surreal, learning)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_learning(
    id: String,
    title: String,
    learning_type: String,
    champion: String,
    opponent: String,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    tags_json: String,
    win_loss: String,
    match_riot_id: String,
    game_timestamp_ms: String,
    event_name: String,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(authz::server_err)?;

    let learning = PersonalLearning {
        id: Some(id),
        user_id: user.id.clone(),
        title,
        learning_type,
        champion: if champion.is_empty() {
            None
        } else {
            Some(champion)
        },
        opponent: if opponent.is_empty() {
            None
        } else {
            Some(opponent)
        },
        what_happened,
        what_i_learned,
        next_time,
        tags,
        win_loss: if win_loss.is_empty() {
            None
        } else {
            Some(win_loss)
        },
        match_riot_id: if match_riot_id.is_empty() {
            None
        } else {
            Some(match_riot_id)
        },
        game_timestamp_ms: game_timestamp_ms.parse::<i64>().ok(),
        event_name: if event_name.is_empty() {
            None
        } else {
            Some(event_name)
        },
        created_at: None,
    };

    db::update_personal_learning(&surreal, &user.id, learning)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_learning(id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    db::delete_personal_learning(&db, &user.id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_champions_for_learnings() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::authz;
    use crate::server::data_dragon;
    data_dragon::champions().await.map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// Browse page
// ---------------------------------------------------------------------------

fn type_word(t: &str) -> &'static str {
    match t {
        "champion" => "Champion",
        "matchup" => "Matchup",
        _ => "General",
    }
}

/// Learnings that pass the type and tag filters ("all" passes everything).
pub fn filter_learnings(list: &[PersonalLearning], kind: &str, tag: &str) -> Vec<PersonalLearning> {
    list.iter()
        .filter(|l| kind == "all" || l.learning_type == kind)
        .filter(|l| tag == "all" || l.tags.iter().any(|t| t == tag))
        .cloned()
        .collect()
}

#[component]
pub fn PersonalLearningsPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            crate::pages::navigate::redirect_to_login();
        }
    });
    let learnings = Resource::new(|| (), |_| list_learnings());
    let kind = RwSignal::new("all".to_string());
    let tag = RwSignal::new("all".to_string());
    let mut tag_options = vec![SelectOption::new("all", "All tags")];
    tag_options.extend(LEARNING_TAGS.iter().map(|t| SelectOption::new(*t, *t)));

    let delete = Callback::new(move |l: PersonalLearning| {
        let id = l.id.clone().unwrap_or_default();
        leptos::task::spawn_local(async move {
            match delete_learning(id).await {
                Ok(()) => {
                    learnings.refetch();
                    let restore = l.clone();
                    toast
                        .push
                        .run(Toast::success("Learning deleted").with_action(
                            "Undo",
                            Callback::new(move |_| {
                                let r = restore.clone();
                                leptos::task::spawn_local(async move {
                                    let tags = serde_json::to_string(&r.tags)
                                        .unwrap_or_else(|_| "[]".into());
                                    let res = save_learning(
                                        r.title,
                                        r.learning_type,
                                        r.champion.unwrap_or_default(),
                                        r.opponent.unwrap_or_default(),
                                        r.what_happened,
                                        r.what_i_learned,
                                        r.next_time,
                                        tags,
                                        r.win_loss.unwrap_or_default(),
                                        r.match_riot_id.unwrap_or_default(),
                                        r.game_timestamp_ms
                                            .map(|t| t.to_string())
                                            .unwrap_or_default(),
                                        r.event_name.unwrap_or_default(),
                                    )
                                    .await;
                                    match res {
                                        Ok(_) => learnings.refetch(),
                                        Err(e) => toast.push.run(Toast::error(e.to_string())),
                                    }
                                });
                            }),
                        ));
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });

    view! {
        <PageShell width=PageWidth::Standard attr:data-feedback-label="Learnings">
            <PageHeader
                title="Learnings"
                eyebrow="You"
                meta=move || view! {
                    <Suspense fallback=|| ()>
                        {move || learnings.get().map(|r| {
                            let n = r.map(|l| l.len()).unwrap_or(0);
                            view! { <span>{if n == 1 { "1 learning".to_string() } else { format!("{n} learnings") }}</span> }
                        })}
                    </Suspense>
                }
                actions=move || view! { <Button variant=ButtonVariant::Primary href="/personal-learnings/new" attr:data-testid="new-learning">"New learning"</Button> }
            />
            <div class="mb-6 flex flex-wrap items-end gap-4">
                <SegmentedControl id="learning-kind" label="Kind of learning" options=vec![Segment::new("all", "All"), Segment::new("general", "General"), Segment::new("champion", "Champion"), Segment::new("matchup", "Matchup")] value=kind size=SegmentSize::Sm />
                <Field id="learning-tag" label="Tag">
                    <Select value=tag options=tag_options size=crate::components::ui::FieldSize::Sm />
                </Field>
            </div>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading learnings"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || learnings.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"Your learnings did not load. Reload the page in a moment."</Callout> }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <EmptyState
                            title="No learnings yet"
                            message="Write down what happened, what you learned and what you will do next time. Learnings from a match's timeline keep the moment they came from."
                            cta_label="Write the first one"
                            cta_href="/personal-learnings/new"
                        />
                    }.into_any(),
                    Ok(list) => {
                        let shown = filter_learnings(&list, &kind.get(), &tag.get());
                        if shown.is_empty() {
                            return view! { <EmptyState size=EmptySize::Compact message="No learning matches these filters. Choose All to see every learning." /> }.into_any();
                        }
                        view! {
                            <ul class="border-b border-rule" data-learning-list>
                                {shown.into_iter().map(|l| view! { <LearningRow l=l on_delete=delete /> }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn LearningRow(l: PersonalLearning, on_delete: Callback<PersonalLearning>) -> impl IntoView {
    let edit = format!(
        "/personal-learnings/new?edit={}",
        l.id.clone().unwrap_or_default()
    );
    let match_href = l
        .match_riot_id
        .clone()
        .filter(|m| !m.is_empty())
        .map(|m| format!("/match/{m}"));
    let l_del = l.clone();
    view! {
        <li class="border-t border-rule py-5" data-learning=l.id.clone().unwrap_or_default()>
            <div class="flex flex-wrap items-start justify-between gap-3">
                <div class="min-w-0">
                    <h2 class="font-display text-heading font-semibold text-primary">{l.title.clone()}</h2>
                    <p class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-label text-muted">
                        <span>{type_word(&l.learning_type)}</span>
                        {l.champion.clone().filter(|c| !c.is_empty()).map(|c| view! {
                            <span class="inline-flex items-center gap-1.5 text-secondary">
                                <ChampionAvatar name=c.clone() size=AvatarSize::Xs decorative=true />
                                {c.clone()}
                            </span>
                        })}
                        {l.opponent.clone().filter(|c| !c.is_empty()).map(|c| view! { <span>{format!("vs {c}")}</span> })}
                        {l.win_loss.as_deref().and_then(|w| match w { "win" => Some(true), "loss" => Some(false), _ => None }).map(|w| view! { <ResultGlyph result=GameResult::from_win(w) size=GlyphSize::Sm /> })}
                        {l.created_at.clone().map(|c| view! { <LocalTime iso=c style=TimeStyle::DayMonth /> })}
                        {l.event_name.clone().filter(|e| !e.is_empty()).map(|e| view! { <span>{format!("From the moment: {}", e.replace('+', " "))}</span> })}
                    </p>
                </div>
                <div class="flex gap-1">
                    {match_href.map(|h| view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href=h>"Match"</Button> })}
                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href=edit>"Edit"</Button>
                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| on_delete.run(l_del.clone()))>"Delete"</Button>
                </div>
            </div>
            <dl class="mt-3 grid gap-x-6 gap-y-2 text-body sm:grid-cols-[9rem_minmax(0,1fr)]">
                <dt class="text-muted">"What happened"</dt>
                <dd class="whitespace-pre-line text-primary">{l.what_happened.clone()}</dd>
                <dt class="text-muted">"What I learned"</dt>
                <dd class="whitespace-pre-line text-primary">{l.what_i_learned.clone()}</dd>
                <dt class="text-muted">"Next time"</dt>
                <dd class="whitespace-pre-line text-primary">{l.next_time.clone()}</dd>
            </dl>
            {(!l.tags.is_empty()).then(|| view! {
                <p class="mt-3 flex flex-wrap gap-1.5">
                    {l.tags.clone().into_iter().map(|t| view! { <Badge tone=BadgeTone::Neutral>{t}</Badge> }).collect_view()}
                </p>
            })}
        </li>
    }
}

// ---------------------------------------------------------------------------
// New / edit page
// ---------------------------------------------------------------------------

/// A title when the writer gives none: "Ahri vs Zed, 12 Oct".
pub fn auto_title(kind: &str, champion: &str, opponent: &str, day: &str) -> String {
    match kind {
        "matchup" if !champion.is_empty() && !opponent.is_empty() => {
            format!("{champion} vs {opponent}, {day}")
        }
        "champion" | "matchup" if !champion.is_empty() => format!("{champion}, {day}"),
        _ => format!("Learning, {day}"),
    }
}

#[component]
pub fn NewLearningPage() -> impl IntoView {
    use leptos_router::hooks::use_query_map;
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            crate::pages::navigate::redirect_to_login();
        }
    });
    let query = use_query_map();
    let q = |k: &str| query.read_untracked().get(k).unwrap_or_default();
    let (q_champion, q_opponent, q_edit, q_tag_hint) =
        (q("champion"), q("opponent"), q("edit"), q("tag_hint"));
    let default_type = if !q_champion.is_empty() && !q_opponent.is_empty() {
        "matchup"
    } else if !q_champion.is_empty() {
        "champion"
    } else {
        "general"
    };
    let kind = RwSignal::new(default_type.to_string());
    let champion = RwSignal::new(q_champion.clone());
    let opponent = RwSignal::new(q_opponent.clone());
    let what_happened = RwSignal::new(String::new());
    let what_i_learned = RwSignal::new(String::new());
    let next_time = RwSignal::new(String::new());
    let title = RwSignal::new(String::new());
    let result = RwSignal::new(q("result"));
    let match_id = RwSignal::new(q("match_id"));
    let event_ts = RwSignal::new(q("event_ts"));
    let event_name = RwSignal::new(q("event_name").replace('+', " "));
    let tag_checks: Vec<(String, RwSignal<bool>)> = LEARNING_TAGS
        .iter()
        .map(|t| {
            (
                t.to_string(),
                RwSignal::new(q_tag_hint.replace('+', " ") == *t),
            )
        })
        .collect();
    let tag_checks = StoredValue::new(tag_checks);
    let errors: [RwSignal<Option<String>>; 3] = std::array::from_fn(|_| RwSignal::new(None));
    let saving = RwSignal::new(false);
    let editing = !q_edit.is_empty();
    let edit_id = StoredValue::new(q_edit.clone());

    let champions = Resource::new(|| (), |_| get_champions_for_learnings());
    let existing = Resource::new(
        move || edit_id.get_value(),
        |id| async move {
            if id.is_empty() {
                Ok(None)
            } else {
                get_learning(id).await
            }
        },
    );
    Effect::new(move || {
        if let Some(Ok(Some(l))) = existing.get() {
            kind.set(l.learning_type.clone());
            champion.set(l.champion.clone().unwrap_or_default());
            opponent.set(l.opponent.clone().unwrap_or_default());
            what_happened.set(l.what_happened.clone());
            what_i_learned.set(l.what_i_learned.clone());
            next_time.set(l.next_time.clone());
            title.set(l.title.clone());
            result.set(l.win_loss.clone().unwrap_or_default());
            match_id.set(l.match_riot_id.clone().unwrap_or_default());
            event_ts.set(
                l.game_timestamp_ms
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
            );
            event_name.set(l.event_name.clone().unwrap_or_default());
            tag_checks.with_value(|tc| {
                for (t, sig) in tc {
                    sig.set(l.tags.contains(t));
                }
            });
        }
    });

    let save = Callback::new(move |_| {
        let fields = [
            what_happened.get_untracked(),
            what_i_learned.get_untracked(),
            next_time.get_untracked(),
        ];
        let mut ok = true;
        for (i, f) in fields.iter().enumerate() {
            if f.trim().is_empty() {
                errors[i].set(Some("Write a sentence here.".into()));
                ok = false;
            } else {
                errors[i].set(None);
            }
        }
        if !ok {
            return;
        }
        let k = kind.get_untracked();
        let champ = if k == "general" {
            String::new()
        } else {
            champion.get_untracked()
        };
        let opp = if k == "matchup" {
            opponent.get_untracked()
        } else {
            String::new()
        };
        let day = chrono::Local::now().format("%-d %b").to_string();
        let t = title.get_untracked();
        let t = if t.trim().is_empty() {
            auto_title(&k, &champ, &opp, &day)
        } else {
            t
        };
        let tags: Vec<String> = tag_checks.with_value(|tc| {
            tc.iter()
                .filter(|(_, s)| s.get_untracked())
                .map(|(t, _)| t.clone())
                .collect()
        });
        let tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into());
        let [wh, wl, nt] = fields;
        let (res, mid, ts, en) = (
            result.get_untracked(),
            match_id.get_untracked(),
            event_ts.get_untracked(),
            event_name.get_untracked(),
        );
        saving.set(true);
        leptos::task::spawn_local(async move {
            let r = if editing {
                update_learning(
                    edit_id.get_value(),
                    t,
                    k,
                    champ,
                    opp,
                    wh,
                    wl,
                    nt,
                    tags,
                    res,
                    mid,
                    ts,
                    en,
                )
                .await
                .map(|_| ())
            } else {
                save_learning(t, k, champ, opp, wh, wl, nt, tags, res, mid, ts, en)
                    .await
                    .map(|_| ())
            };
            match r {
                Ok(()) => {
                    toast.push.run(Toast::success("Learning saved"));
                    #[cfg(feature = "hydrate")]
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_href("/personal-learnings");
                    }
                }
                Err(e) => {
                    toast.push.run(Toast::error(e.to_string()));
                    saving.set(false);
                }
            }
        });
    });

    view! {
        <PageShell width=PageWidth::Reading attr:data-feedback-label="New learning">
            <PageHeader
                title=if editing { "Edit learning" } else { "New learning" }
                eyebrow="Learnings"
                actions=move || view! {
                    <Button variant=ButtonVariant::Quiet href="/personal-learnings">"Cancel"</Button>
                    <Button variant=ButtonVariant::Primary on_click=save loading=saving attr:data-testid="save-learning">"Save learning"</Button>
                }
            />
            {move || {
                let (en, ts) = (event_name.get(), event_ts.get());
                (!en.is_empty() && !ts.is_empty()).then(|| {
                    let s = ts.parse::<i64>().unwrap_or(0) / 1000;
                    view! { <div class="mb-6"><Callout tone=CalloutTone::Info title="From a match moment">{format!("{en} at {}:{:02}", s / 60, s % 60)}</Callout></div> }
                })
            }}
            <div class="flex flex-col gap-6">
                <SegmentedControl id="new-learning-kind" label="Kind of learning" options=vec![Segment::new("general", "General"), Segment::new("champion", "Champion"), Segment::new("matchup", "Matchup")] value=kind />
                <Suspense fallback=|| view! { <Skeleton lines=1 /> }>
                    {move || {
                        let list: Vec<Champion> = champions.get().and_then(|r| r.ok()).unwrap_or_default();
                        let k = kind.get();
                        let list2 = list.clone();
                        view! {
                            {(k != "general").then(|| view! {
                                <Field id="learning-champion" label="Your champion">
                                    <ChampionAutocomplete champions=list.clone() value=champion placeholder="Search a champion" />
                                </Field>
                            })}
                            {(k == "matchup").then(|| view! {
                                <Field id="learning-opponent" label="Against">
                                    <ChampionAutocomplete champions=list2.clone() value=opponent placeholder="Search a champion" />
                                </Field>
                            })}
                        }
                    }}
                </Suspense>
                <SegmentedControl id="learning-result" label="Result of the game" options=vec![Segment::new("", "Not set"), Segment::new("win", "Win"), Segment::new("loss", "Loss")] value=result size=SegmentSize::Sm />
                <Field id="learning-happened" label="What happened" required=true error=Signal::derive(move || errors[0].get())>
                    <TextField value=what_happened rows=3 />
                </Field>
                <Field id="learning-learned" label="What I learned" required=true error=Signal::derive(move || errors[1].get())>
                    <TextField value=what_i_learned rows=3 />
                </Field>
                <Field id="learning-next" label="Next time I will" required=true error=Signal::derive(move || errors[2].get())>
                    <TextField value=next_time rows=2 />
                </Field>
                <Field id="learning-title" label="Title" hint="Left empty, it becomes the champion, the opponent and the date.">
                    <TextField value=title maxlength=120 />
                </Field>
                <fieldset>
                    <legend class="mb-2 text-label font-medium text-primary">"Tags"</legend>
                    <div class="grid gap-2 sm:grid-cols-2">
                        {tag_checks.get_value().into_iter().map(|(t, sig)| view! { <Checkbox label=t checked=sig /> }).collect_view()}
                    </div>
                </fieldset>
            </div>
            <section class="mt-10" aria-labelledby="learning-why">
                <SectionHeader title="Where it shows" id="learning-why" level=crate::components::ui::HeadingLevel::H3 />
                <p class="max-w-prose text-body text-secondary">"Learnings stay yours: only you see them, here on your list. They never appear on the team pages."</p>
            </section>
        </PageShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(kind: &str, tags: &[&str]) -> PersonalLearning {
        PersonalLearning {
            id: None,
            user_id: "user:u".into(),
            title: "t".into(),
            learning_type: kind.into(),
            champion: None,
            opponent: None,
            what_happened: String::new(),
            what_i_learned: String::new(),
            next_time: String::new(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            win_loss: None,
            match_riot_id: None,
            game_timestamp_ms: None,
            event_name: None,
            created_at: None,
        }
    }

    #[test]
    fn filters_and_titles() {
        let list = vec![l("general", &["Vision"]), l("matchup", &["Laning"])];
        assert_eq!(filter_learnings(&list, "all", "all").len(), 2);
        assert_eq!(filter_learnings(&list, "matchup", "all").len(), 1);
        assert_eq!(filter_learnings(&list, "all", "Vision").len(), 1);
        assert_eq!(
            auto_title("matchup", "Ahri", "Zed", "12 Oct"),
            "Ahri vs Zed, 12 Oct"
        );
        assert_eq!(auto_title("champion", "Ahri", "", "12 Oct"), "Ahri, 12 Oct");
        assert_eq!(auto_title("general", "", "", "12 Oct"), "Learning, 12 Oct");
    }
}
