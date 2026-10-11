//! Champion trait table (`/admin/traits`): the maintainer's view of the
//! Draft Graph's kit-based traits.
//!
//! Admin-only, like `/admin/invites`: every server function checks
//! `require_admin()`, everyone else (signed out included) gets the 404 page
//! (`SsrMode::Async`, so the status is set). An edit applies to every
//! team's graph at once and marks the row as edited (who, when, and
//! optionally the patch it was reviewed for); "Reset to default" puts the
//! shipped row back.

use leptos::prelude::*;

use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ChampionAvatar, Checkbox, Density, Dialog, DialogSize, EmptySize, EmptyState, Field, InputKind,
    PageHeader, PageShell, PageWidth, Role, RoleLabel, Segment, SegmentSize, SegmentedControl,
    Select, SelectOption, Skeleton, SkeletonGroup, SkeletonShape, Table, Td, TextField, Th, Toast,
    ToastContext, Tr,
};
use crate::error_template::NotFound;
use crate::models::graph::{
    ChampionTraits, Confidence, CrowdControl, DamageProfile, DamageType, Engage, Mobility,
    Playstyle, RangeClass, Scaling, Strength, TraitRecord, TraitSource, WaveClear,
};

/// Every trait row plus the current patch version.
#[server]
pub async fn admin_list_traits() -> Result<(String, Vec<TraitRecord>), ServerFnError> {
    use crate::server::{authz, data_dragon, graph::traits};
    authz::require_admin().await?;
    let db = authz::db()?;
    let table = traits::ensure_loaded(&db)
        .await
        .map_err(authz::server_err)?;
    Ok((data_dragon::version(), table.rows.clone()))
}

/// Save a maintainer's edit. `reviewed` records the current patch as the
/// one the row was checked against.
#[server]
pub async fn admin_save_traits(traits_json: String, reviewed: bool) -> Result<(), ServerFnError> {
    use crate::server::{authz, data_dragon, db, graph::traits};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let t: ChampionTraits = serde_json::from_str(&traits_json)
        .map_err(|_| ServerFnError::new("The row could not be read."))?;
    t.validate().map_err(ServerFnError::new)?;
    let patch = reviewed.then(data_dragon::version);
    db::save_champion_traits(&db, &t, &admin.id, &admin.username, patch)
        .await
        .map_err(authz::server_err)?;
    traits::reload(&db).await.map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, champion = %t.champion, "edited champion traits");
    Ok(())
}

/// Put a champion's row back to the shipped default.
#[server]
pub async fn admin_reset_traits(champion: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db, graph::traits};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let Some(default) = traits::default_for(&champion) else {
        return Err(ServerFnError::new(
            "This champion has no row in the defaults file; edit it instead.",
        ));
    };
    db::reset_champion_traits(&db, default)
        .await
        .map_err(authz::server_err)?;
    traits::reload(&db).await.map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, champion = %champion, "reset champion traits");
    Ok(())
}

/// Segments for a vocabulary.
fn segs<T: Copy>(all: &[T], f: impl Fn(T) -> (&'static str, &'static str)) -> Vec<Segment> {
    all.iter()
        .map(|v| {
            let (k, l) = f(*v);
            Segment::new(k, l)
        })
        .collect()
}

/// A trait row from the form's keys. Every value must be in the
/// vocabulary; the row must pass [`ChampionTraits::validate`].
#[allow(clippy::too_many_arguments)]
pub fn traits_from_keys(
    champion: &str,
    roles: Vec<Role>,
    keys: [&str; 12],
    styles: Vec<Playstyle>,
) -> Result<ChampionTraits, String> {
    let bad = |what: &str| format!("Choose a value for {what}.");
    let t = ChampionTraits {
        champion: champion.to_string(),
        roles,
        damage_type: DamageType::parse(keys[0]).ok_or_else(|| bad("damage type"))?,
        damage_profile: DamageProfile::parse(keys[1]).ok_or_else(|| bad("damage profile"))?,
        range: RangeClass::parse(keys[2]).ok_or_else(|| bad("range"))?,
        engage: Engage::parse(keys[3]).ok_or_else(|| bad("engage"))?,
        disengage: Strength::parse(keys[4]).ok_or_else(|| bad("disengage"))?,
        peel: Strength::parse(keys[5]).ok_or_else(|| bad("peel"))?,
        frontline: Strength::parse(keys[6]).ok_or_else(|| bad("front line"))?,
        cc: CrowdControl::parse(keys[7]).ok_or_else(|| bad("crowd control"))?,
        wave_clear: WaveClear::parse(keys[8]).ok_or_else(|| bad("wave clear"))?,
        scaling: Scaling::parse(keys[9]).ok_or_else(|| bad("scaling"))?,
        mobility: Mobility::parse(keys[10]).ok_or_else(|| bad("mobility"))?,
        styles,
        confidence: Confidence::parse(keys[11]).ok_or_else(|| bad("confidence"))?,
    };
    t.validate().map_err(|e| {
        if t.roles.is_empty() {
            "Choose at least one role.".to_string()
        } else {
            e
        }
    })?;
    Ok(t)
}

/// Does the row match the search and filter?
pub fn row_matches(r: &TraitRecord, query: &str, filter: &str) -> bool {
    let q = query.trim().to_lowercase();
    let text = q.is_empty()
        || r.name.to_lowercase().contains(&q)
        || r.traits.champion.to_lowercase().contains(&q)
        || r.traits
            .styles
            .iter()
            .any(|s| s.label().to_lowercase().contains(&q));
    let kind = match filter {
        "defaults" => r.source == TraitSource::Default,
        "edited" => r.source == TraitSource::Maintainer,
        "low" => r.traits.confidence == Confidence::Low,
        _ => true,
    };
    text && kind
}

/// The edit form: one signal per field.
#[derive(Clone, Copy)]
struct TraitForm {
    champion: RwSignal<String>,
    name: RwSignal<String>,
    source: RwSignal<Option<TraitSource>>,
    keys: [RwSignal<String>; 12],
    roles: [RwSignal<bool>; 5],
    styles: [RwSignal<bool>; 10],
    reviewed: RwSignal<bool>,
}

impl TraitForm {
    fn new() -> Self {
        Self {
            champion: RwSignal::new(String::new()),
            name: RwSignal::new(String::new()),
            source: RwSignal::new(None),
            keys: std::array::from_fn(|_| RwSignal::new(String::new())),
            roles: std::array::from_fn(|_| RwSignal::new(false)),
            styles: std::array::from_fn(|_| RwSignal::new(false)),
            reviewed: RwSignal::new(false),
        }
    }

    fn load(&self, r: &TraitRecord) {
        let t = &r.traits;
        self.champion.set(t.champion.clone());
        self.name.set(r.name.clone());
        self.source.set(Some(r.source));
        let keys = [
            t.damage_type.key(),
            t.damage_profile.key(),
            t.range.key(),
            t.engage.key(),
            t.disengage.key(),
            t.peel.key(),
            t.frontline.key(),
            t.cc.key(),
            t.wave_clear.key(),
            t.scaling.key(),
            t.mobility.key(),
            t.confidence.key(),
        ];
        for (sig, k) in self.keys.iter().zip(keys) {
            sig.set(k.to_string());
        }
        for (sig, role) in self.roles.iter().zip(Role::ALL) {
            sig.set(t.roles.contains(&role));
        }
        for (sig, style) in self.styles.iter().zip(Playstyle::ALL) {
            sig.set(t.styles.contains(style));
        }
        self.reviewed.set(false);
    }

    fn read(&self) -> Result<ChampionTraits, String> {
        let keys: Vec<String> = self.keys.iter().map(|k| k.get_untracked()).collect();
        let refs: [&str; 12] = std::array::from_fn(|i| keys[i].as_str());
        let roles = Role::ALL
            .iter()
            .zip(self.roles.iter())
            .filter(|(_, s)| s.get_untracked())
            .map(|(r, _)| *r)
            .collect();
        let styles = Playstyle::ALL
            .iter()
            .zip(self.styles.iter())
            .filter(|(_, s)| s.get_untracked())
            .map(|(p, _)| *p)
            .collect();
        traits_from_keys(&self.champion.get_untracked(), roles, refs, styles)
    }
}

#[component]
pub fn AdminTraitsPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let rows = Resource::new(|| (), |_| admin_list_traits());
    let query = RwSignal::new(String::new());
    let filter = RwSignal::new("all".to_string());
    let form = TraitForm::new();
    let open = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let t = match form.read() {
            Ok(t) => t,
            Err(e) => {
                error.set(Some(e));
                return;
            }
        };
        let reviewed = form.reviewed.get_untracked();
        let name = form.name.get_untracked();
        busy.set(true);
        leptos::task::spawn_local(async move {
            let json = serde_json::to_string(&t).unwrap_or_default();
            match admin_save_traits(json, reviewed).await {
                Ok(()) => {
                    open.set(false);
                    toast
                        .push
                        .run(Toast::success(format!("Saved {name}'s traits")));
                    rows.refetch();
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            busy.set(false);
        });
    });
    let reset = Callback::new(move |_: leptos::ev::MouseEvent| {
        let champion = form.champion.get_untracked();
        let name = form.name.get_untracked();
        busy.set(true);
        leptos::task::spawn_local(async move {
            match admin_reset_traits(champion).await {
                Ok(()) => {
                    open.set(false);
                    toast.push.run(Toast::success(format!(
                        "{name} is back to the shipped default"
                    )));
                    rows.refetch();
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            busy.set(false);
        });
    });

    view! {
        <Suspense fallback=|| view! {
            <PageShell>
                <SkeletonGroup label="Loading the trait table">
                    <Skeleton shape=SkeletonShape::Block size="h-96 w-full" />
                </SkeletonGroup>
            </PageShell>
        }>
            {move || Suspend::new(async move {
                match rows.await {
                    Err(_) => view! { <NotFound /> }.into_any(),
                    Ok((version, list)) => {
                        let total = list.len();
                        let defaults = list.iter().filter(|r| r.source == TraitSource::Default).count();
                        let low = list.iter().filter(|r| r.traits.confidence == Confidence::Low).count();
                        let list = StoredValue::new(list);
                        let review_label = format!("Reviewed for patch {version}");
                        view! {
                            <PageShell width=PageWidth::Full density=Density::Compact attr:data-feedback-label="Champion traits">
                                <PageHeader
                                    title="Champion traits"
                                    eyebrow="Admin"
                                    meta=move || view! {
                                        <span>{format!("{total} champions")}</span>
                                        <span>{format!("{defaults} shipped defaults")}</span>
                                        <span>{format!("{} edited", total - defaults)}</span>
                                        <span>{format!("{low} low confidence")}</span>
                                    }
                                />
                                <div class="flex max-w-3xl flex-col gap-2">
                                    <p class="text-body text-secondary">
                                        "What each champion's kit does, the Draft Graph's ground truth. Rows ship as kit-based defaults; an edit here changes every team's graph at once and is never overwritten by a new defaults file."
                                    </p>
                                </div>
                                <div class="flex flex-wrap items-end gap-x-6 gap-y-3">
                                    <div class="w-full max-w-xs">
                                        <Field id="traits-search" label="Search">
                                            <TextField value=query kind=InputKind::Search placeholder="Champion or playstyle" />
                                        </Field>
                                    </div>
                                    <div class="flex flex-col gap-1.5">
                                        <span class="text-label font-medium text-secondary">"Show"</span>
                                        <div class="overflow-x-auto">
                                            <SegmentedControl
                                                id="traits-filter"
                                                label="Show"
                                                options=vec![
                                                    Segment::new("all", "All"),
                                                    Segment::new("defaults", "Defaults only"),
                                                    Segment::new("edited", "Edited"),
                                                    Segment::new("low", "Low confidence"),
                                                ]
                                                value=filter
                                                size=SegmentSize::Sm
                                            />
                                        </div>
                                    </div>
                                </div>
                                {move || {
                                    let q = query.get();
                                    let f = filter.get();
                                    let shown: Vec<TraitRecord> = list.with_value(|l| {
                                        l.iter().filter(|r| row_matches(r, &q, &f)).cloned().collect()
                                    });
                                    if shown.is_empty() {
                                        return view! {
                                            <EmptyState size=EmptySize::Compact message="No champion matches the search and filter." />
                                        }.into_any();
                                    }
                                    let count = shown.len();
                                    view! {
                                        <p class="text-caption text-muted" aria-live="polite">{format!("{count} shown")}</p>
                                        <Table caption="Champion traits" caption_hidden=true sticky_header=true density=Density::Compact>
                                            <thead>
                                                <tr>
                                                    <Th>"Champion"</Th>
                                                    <Th>"Roles"</Th>
                                                    <Th>"Damage"</Th>
                                                    <Th>"Range"</Th>
                                                    <Th>"Engage"</Th>
                                                    <Th>"Disengage"</Th>
                                                    <Th>"Peel"</Th>
                                                    <Th>"Front line"</Th>
                                                    <Th>"CC"</Th>
                                                    <Th>"Wave clear"</Th>
                                                    <Th>"Scaling"</Th>
                                                    <Th>"Mobility"</Th>
                                                    <Th>"Playstyles"</Th>
                                                    <Th>"Source"</Th>
                                                    <Th><span class="sr-only">"Edit"</span></Th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {shown.into_iter().map(|r| {
                                                    let t = r.traits.clone();
                                                    let name = r.name.clone();
                                                    let low = t.confidence == Confidence::Low;
                                                    let source = match r.source {
                                                        TraitSource::Default => view! { <Badge>"Default"</Badge> }.into_any(),
                                                        TraitSource::Maintainer => {
                                                            let who = r.edited_by.clone().unwrap_or_default();
                                                            let when = r.edited_at.clone().unwrap_or_default().chars().take(10).collect::<String>();
                                                            let patch = r.reviewed_for_patch.clone();
                                                            view! {
                                                                <span class="flex flex-col">
                                                                    <span><Badge tone=BadgeTone::Info>"Edited"</Badge></span>
                                                                    <span class="text-caption text-muted">{format!("{who}, {when}")}</span>
                                                                    {patch.map(|p| view! { <span class="text-caption text-muted">{format!("reviewed for {p}")}</span> })}
                                                                </span>
                                                            }.into_any()
                                                        }
                                                    };
                                                    let rec = r.clone();
                                                    let edit_label = format!("Edit {name}");
                                                    view! {
                                                        <Tr>
                                                            <Td header=true>
                                                                <span class="inline-flex items-center gap-2 whitespace-nowrap">
                                                                    <ChampionAvatar name=name.clone() size=AvatarSize::Xs decorative=true />
                                                                    {name.clone()}
                                                                    {low.then(|| view! { <Badge tone=BadgeTone::Warning>"Low confidence"</Badge> })}
                                                                </span>
                                                            </Td>
                                                            <Td>
                                                                <span class="inline-flex gap-1.5">
                                                                    {t.roles.iter().map(|r| view! { <RoleLabel role=*r /> }).collect_view()}
                                                                </span>
                                                            </Td>
                                                            <Td>
                                                                <span class="flex flex-col whitespace-nowrap">
                                                                    <span>{t.damage_type.label()}</span>
                                                                    <span class="text-caption text-muted">{t.damage_profile.label()}</span>
                                                                </span>
                                                            </Td>
                                                            <Td>{t.range.label()}</Td>
                                                            <Td>{t.engage.label()}</Td>
                                                            <Td>{t.disengage.label()}</Td>
                                                            <Td>{t.peel.label()}</Td>
                                                            <Td>{t.frontline.label()}</Td>
                                                            <Td>{t.cc.label()}</Td>
                                                            <Td>{t.wave_clear.label()}</Td>
                                                            <Td>{t.scaling.label()}</Td>
                                                            <Td>{t.mobility.label()}</Td>
                                                            <Td>
                                                                <span class="block min-w-[10rem] text-caption text-secondary">
                                                                    {if t.styles.is_empty() { "\u{2014}".to_string() } else { t.styles.iter().map(|s| s.label()).collect::<Vec<_>>().join(", ") }}
                                                                </span>
                                                            </Td>
                                                            <Td>{source}</Td>
                                                            <Td>
                                                                <Button
                                                                    variant=ButtonVariant::Quiet
                                                                    size=ButtonSize::Sm
                                                                    aria_label=edit_label
                                                                    on_click=Callback::new(move |_| {
                                                                        form.load(&rec);
                                                                        error.set(None);
                                                                        open.set(true);
                                                                    })
                                                                >"Edit"</Button>
                                                            </Td>
                                                        </Tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </Table>
                                    }.into_any()
                                }}
                                <TraitDialog form=form open=open error=error busy=busy save=save reset=reset review_label=review_label />
                            </PageShell>
                        }.into_any()
                    }
                }
            })}
        </Suspense>
    }
}

/// The edit dialog.
#[component]
fn TraitDialog(
    form: TraitForm,
    open: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    busy: RwSignal<bool>,
    save: Callback<leptos::ev::MouseEvent>,
    reset: Callback<leptos::ev::MouseEvent>,
    review_label: String,
) -> impl IntoView {
    let k = form.keys;
    let damage_types = StoredValue::new(
        DamageType::ALL
            .iter()
            .map(|v| SelectOption::new(v.key(), v.label()))
            .collect::<Vec<_>>(),
    );
    let damage_profiles = StoredValue::new(
        DamageProfile::ALL
            .iter()
            .map(|v| SelectOption::new(v.key(), v.label()))
            .collect::<Vec<_>>(),
    );
    let seg = move |id: &'static str, label: &'static str, i: usize, options: Vec<Segment>| {
        view! {
            <div class="flex flex-col gap-1.5">
                <span class="text-label font-medium text-secondary">{label}</span>
                <div class="overflow-x-auto">
                    <SegmentedControl id=id label=label options=options value=k[i] size=SegmentSize::Sm />
                </div>
            </div>
        }
    };
    view! {
        <Dialog
            id="trait-edit"
            open=open
            title="Edit champion traits"
            description="Kit-based values only: what the abilities structurally do, nothing about this patch's numbers. When unsure, the neutral value and low confidence."
            size=DialogSize::Lg
            footer=move || {
                let review_label = review_label.clone();
                view! {
                    <div class="mr-auto">
                        <Checkbox label=review_label checked=form.reviewed id="trait-reviewed" />
                    </div>
                    {move || (form.source.get() == Some(TraitSource::Maintainer)).then(|| view! {
                        <Button variant=ButtonVariant::Quiet on_click=reset loading=Signal::derive(move || busy.get())>"Reset to default"</Button>
                    })}
                    <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click=save
                        loading=Signal::derive(move || busy.get())
                        attr:data-testid="trait-save"
                    >"Save traits"</Button>
                }
            }
        >
            <div class="flex flex-col gap-5" data-trait-form="">
                <p class="font-display text-heading font-semibold text-primary">{move || form.name.get()}</p>
                {move || error.get().map(|e| view! { <Callout tone=CalloutTone::Danger>{e}</Callout> })}
                <fieldset class="flex flex-col gap-2">
                    <legend class="text-label font-medium text-secondary">"Roles (most played first is kept in table order)"</legend>
                    <div class="flex flex-wrap gap-x-5 gap-y-2">
                        {Role::ALL.iter().zip(form.roles).map(|(r, sig)| view! {
                            <Checkbox label=r.label() checked=sig />
                        }).collect_view()}
                    </div>
                </fieldset>
                <div class="grid grid-cols-1 gap-x-6 gap-y-4 sm:grid-cols-2">
                    <Field id="trait-damage-type" label="Damage type">
                        <Select value=k[0] options=damage_types.get_value() />
                    </Field>
                    <Field id="trait-damage-profile" label="Damage profile">
                        <Select value=k[1] options=damage_profiles.get_value() />
                    </Field>
                    {seg("trait-range", "Range", 2, segs(RangeClass::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-engage", "Engage", 3, segs(Engage::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-disengage", "Disengage", 4, segs(Strength::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-peel", "Peel", 5, segs(Strength::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-frontline", "Front line", 6, segs(Strength::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-cc", "Crowd control", 7, segs(CrowdControl::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-wave", "Wave clear", 8, segs(WaveClear::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-scaling", "Scaling", 9, segs(Scaling::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-mobility", "Mobility", 10, segs(Mobility::ALL, |v| (v.key(), v.label())))}
                    {seg("trait-confidence", "Confidence", 11, segs(Confidence::ALL, |v| (v.key(), v.label())))}
                </div>
                <fieldset class="flex flex-col gap-2">
                    <legend class="text-label font-medium text-secondary">"Playstyles"</legend>
                    <div class="grid grid-cols-1 gap-x-5 gap-y-2 sm:grid-cols-2">
                        {Playstyle::ALL.iter().zip(form.styles).map(|(p, sig)| view! {
                            <Checkbox label=p.label() checked=sig />
                        }).collect_view()}
                    </div>
                </fieldset>
            </div>
        </Dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, source: TraitSource, low: bool) -> TraitRecord {
        let mut t = ChampionTraits::neutral(id, vec![Role::Mid]);
        t.confidence = if low {
            Confidence::Low
        } else {
            Confidence::Normal
        };
        t.styles = vec![Playstyle::Poke];
        TraitRecord {
            traits: t,
            name: id.to_string(),
            source,
            edited_by: None,
            edited_at: None,
            reviewed_for_patch: None,
        }
    }

    #[test]
    fn search_and_filter() {
        let a = rec("Ahri", TraitSource::Default, false);
        let z = rec("Zed", TraitSource::Maintainer, true);
        assert!(row_matches(&a, "ah", "all"));
        assert!(!row_matches(&a, "zed", "all"));
        assert!(row_matches(&a, "poke", "all"), "playstyles are searchable");
        assert!(row_matches(&a, "", "defaults"));
        assert!(!row_matches(&z, "", "defaults"));
        assert!(row_matches(&z, "", "edited"));
        assert!(row_matches(&z, "", "low"));
        assert!(!row_matches(&a, "", "low"));
    }

    #[test]
    fn form_keys_must_be_in_the_vocabulary() {
        let keys = [
            "magic",
            "burst",
            "long",
            "conditional",
            "none",
            "some",
            "none",
            "light",
            "ok",
            "mid",
            "high",
            "normal",
        ];
        let t = traits_from_keys("Ahri", vec![Role::Mid], keys, vec![Playstyle::Pick]).unwrap();
        assert_eq!(t.peel, Strength::Partial);
        let mut bad = keys;
        bad[3] = "always";
        assert!(traits_from_keys("Ahri", vec![Role::Mid], bad, vec![]).is_err());
        assert_eq!(
            traits_from_keys("Ahri", vec![], keys, vec![]).unwrap_err(),
            "Choose at least one role."
        );
    }
}
