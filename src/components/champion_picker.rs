//! Champion picker for the draft boards: a search combobox over a grid of
//! champions. Typing filters, the arrow keys move the highlight, Enter
//! picks the highlighted champion (the first match by default), Escape
//! hands focus back to the board.

use crate::components::ui::{
    AvatarSize, AvatarState, ChampionAvatar, Combobox, InputKind, Segment, SegmentSize,
    SegmentedControl, TextField,
};
use crate::models::champion::Champion;
use leptos::prelude::*;

/// Class tags that usually play a role (a filter, not a rule).
fn role_tags(role: &str) -> &'static [&'static str] {
    match role {
        "top" => &["Fighter", "Tank"],
        "jungle" => &["Fighter", "Assassin", "Tank"],
        "mid" => &["Mage", "Assassin"],
        "bot" => &["Marksman"],
        "support" => &["Support"],
        _ => &[],
    }
}

/// Champions matching the query and role filter, in list order.
pub fn filter_champions<'a>(
    champions: &'a [Champion],
    query: &str,
    role: &str,
) -> Vec<&'a Champion> {
    let q = query.trim().to_lowercase();
    let mut out: Vec<&Champion> = champions
        .iter()
        .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
        .filter(|c| {
            let tags = role_tags(role);
            tags.is_empty() || tags.iter().any(|t| c.tags.iter().any(|ct| ct == t))
        })
        .collect();
    // Names that start with the query come first ("Ash" before "Nasus").
    if !q.is_empty() {
        out.sort_by_key(|c| !c.name.to_lowercase().starts_with(&q));
    }
    out
}

/// DOM id of the picker's search input (pages focus it to "open" the picker).
pub fn picker_input_id(picker: &str) -> String {
    format!("{picker}-search")
}

/// Search, role filter and champion grid.
///
/// A11y: the search field is a combobox controlling a `role="listbox"` of
/// `role="option"` champions; Up/Down/Left/Right move the active option
/// (`aria-activedescendant`), Enter picks it, Escape runs `on_escape`.
/// Champions already used are `aria-disabled` and skipped by the keys. The
/// role filter is a SegmentedControl. Pointer users click an option or
/// drag it onto a board slot.
#[component]
pub fn ChampionPicker(
    /// Prefix for DOM ids; unique on the page.
    id: &'static str,
    champions: Vec<Champion>,
    /// Names or ids that cannot be picked (already drafted, fearless).
    #[prop(into)]
    unavailable: Signal<Vec<String>>,
    on_select: Callback<Champion>,
    /// Escape in the search field.
    #[prop(optional)]
    on_escape: Option<Callback<()>>,
    /// Accessible name of the search field ("Champion for Blue ban 1").
    #[prop(into)]
    label: Signal<String>,
    /// False when there is no slot to fill (the draft is complete): no
    /// highlight, and Enter does nothing.
    #[prop(optional, into)]
    enabled: Option<Signal<bool>>,
) -> impl IntoView {
    let enabled = enabled.unwrap_or(Signal::stored(true));
    let champions = StoredValue::new(champions);
    let query = RwSignal::new(String::new());
    let role = RwSignal::new("all".to_string());
    let active = RwSignal::new(0_usize);
    let listbox_id = format!("{id}-list");
    let option_id = move |cid: &str| format!("{id}-opt-{cid}");

    let is_unavailable =
        move |c: &Champion| unavailable.with(|u| u.iter().any(|x| *x == c.name || *x == c.id));
    // Indices (into the full list) of the visible champions.
    let visible = Memo::new(move |_| {
        let q = query.get();
        let r = role.get();
        champions.with_value(|all| {
            filter_champions(all, &q, &r)
                .into_iter()
                .filter_map(|c| all.iter().position(|x| x.id == c.id))
                .collect::<Vec<usize>>()
        })
    });
    // Visible and pickable, in display order.
    let pickable = Memo::new(move |_| {
        let v = visible.get();
        champions.with_value(|all| {
            v.into_iter()
                .filter(|i| !is_unavailable(&all[*i]))
                .collect::<Vec<usize>>()
        })
    });
    // Reset the highlight to the first match when the list changes.
    Effect::new(move |_| {
        let _ = pickable.get();
        active.set(0);
    });
    let active_champion = move || {
        if !enabled.get() {
            return None;
        }
        let p = pickable.get();
        p.get(active.get().min(p.len().saturating_sub(1)))
            .copied()
            .filter(|_| !p.is_empty())
    };
    let active_dom = Signal::derive(move || {
        active_champion().map(|i| champions.with_value(|all| option_id(&all[i].id)))
    });

    let pick = move |i: usize| {
        let Some(c) = champions.with_value(|all| all.get(i).cloned()) else {
            return;
        };
        if is_unavailable(&c) {
            return;
        }
        on_select.run(c);
        query.set(String::new());
    };

    let on_keydown = Callback::new(move |ev: leptos::ev::KeyboardEvent| {
        let len = pickable.with_untracked(|p| p.len());
        match ev.key().as_str() {
            "ArrowDown" | "ArrowRight" if len > 0 => {
                ev.prevent_default();
                active.update(|a| *a = (*a + 1) % len);
            }
            "ArrowUp" | "ArrowLeft" if len > 0 => {
                ev.prevent_default();
                active.update(|a| *a = (*a + len - 1) % len);
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(i) = active_champion() {
                    pick(i);
                }
            }
            "Escape" => {
                if let Some(cb) = on_escape {
                    ev.prevent_default();
                    cb.run(());
                }
            }
            _ => {}
        }
    });

    let input_label = move || label.get();
    view! {
        <div class="flex min-w-0 flex-col gap-3" data-champion-picker=id>
            <div class="flex flex-col gap-2">
                <label for=picker_input_id(id) class="text-label font-medium text-secondary">
                    {input_label}
                </label>
                <TextField
                    id=picker_input_id(id)
                    kind=InputKind::Search
                    value=query
                    placeholder="Type a name, Enter picks the first match"
                    autocomplete="off"
                    on_keydown=on_keydown
                    combobox=Combobox { controls: listbox_id.clone(), active: active_dom }
                />
            </div>
            <div class="min-w-0 overflow-x-auto">
                <SegmentedControl
                    id="picker-role"
                    label="Filter by role"
                    options=vec![
                        Segment::new("all", "All"),
                        Segment::new("top", "Top"),
                        Segment::new("jungle", "Jungle"),
                        Segment::new("mid", "Mid"),
                        Segment::new("bot", "Bot"),
                        Segment::new("support", "Support"),
                    ]
                    value=role
                    size=SegmentSize::Sm
                />
            </div>
            <ul
                id=listbox_id
                role="listbox"
                aria-label="Champions"
                class="grid max-h-80 grid-cols-[repeat(auto-fill,minmax(4.25rem,1fr))] gap-1 overflow-y-auto pr-1"
            >
                {move || {
                    let vis = visible.get();
                    if vis.is_empty() {
                        return view! {
                            <li role="presentation" class="col-span-full py-4 text-body text-secondary">
                                "No champion matches. Clear the search or the role filter."
                            </li>
                        }
                        .into_any();
                    }
                    let act = active_champion();
                    champions.with_value(|all| {
                        vis.into_iter()
                            .map(|i| {
                                let c = all[i].clone();
                                let off = is_unavailable(&c);
                                let is_active = act == Some(i);
                                let class = if off {
                                    "flex min-w-0 cursor-not-allowed flex-col items-center gap-1 rounded-ctl border border-transparent p-1"
                                } else if is_active {
                                    "flex min-w-0 cursor-pointer flex-col items-center gap-1 rounded-ctl border border-accent bg-accent-soft p-1"
                                } else {
                                    "flex min-w-0 cursor-pointer flex-col items-center gap-1 rounded-ctl border border-transparent p-1 transition-colors hover:border-outline hover:bg-ink-2"
                                };
                                let name = c.name.clone();
                                let drag_name = c.name.clone();
                                view! {
                                    <li
                                        id=option_id(&c.id)
                                        role="option"
                                        class=class
                                        aria-selected=if is_active { "true" } else { "false" }
                                        aria-disabled=off.then_some("true")
                                        data-champion=c.name.clone()
                                        draggable=if off { "false" } else { "true" }
                                        on:mousedown=|ev| ev.prevent_default()
                                        on:click=move |_| pick(i)
                                        on:dragstart=move |ev: web_sys::DragEvent| {
                                            if let Some(dt) = ev.data_transfer() {
                                                let _ = dt.set_data("text/plain", &drag_name);
                                            }
                                        }
                                    >
                                        <ChampionAvatar
                                            name=c.name.clone()
                                            size=AvatarSize::Lg
                                            state=if off { AvatarState::Dimmed } else { AvatarState::Normal }
                                            decorative=true
                                        />
                                        <span class=if off {
                                            "w-full truncate text-center text-caption text-muted line-through"
                                        } else {
                                            "w-full truncate text-center text-caption text-secondary"
                                        }>{name}</span>
                                    </li>
                                }
                            })
                            .collect_view()
                            .into_any()
                    })
                }}
            </ul>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn champ(name: &str, tags: &[&str]) -> Champion {
        Champion {
            id: name.replace(' ', ""),
            name: name.to_string(),
            title: String::new(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            image_full: String::new(),
        }
    }

    #[test]
    fn prefix_matches_come_first_and_roles_filter_by_class() {
        let list = vec![
            champ("Nasus", &["Fighter"]),
            champ("Ashe", &["Marksman"]),
            champ("Lulu", &["Support"]),
        ];
        let names: Vec<&str> = filter_champions(&list, "as", "all")
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["Ashe", "Nasus"]);
        let sup: Vec<&str> = filter_champions(&list, "", "support")
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(sup, vec!["Lulu"]);
    }
}
