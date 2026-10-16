//! A champion search field with a suggestion list (combobox). The value is
//! the champion's Data Dragon id; the field shows the display name.

use crate::components::ui::{AvatarSize, ChampionAvatar, Combobox, InputKind, TextField};
use crate::models::champion::Champion;
use leptos::prelude::*;

/// Search field with up to eight suggestions.
///
/// Typing filters; Up/Down move the highlight, Enter takes it (or the only
/// match), Escape closes the list. Choosing sets `value` to the champion id
/// and runs `on_select` with it. Pass `id` (and a visible label pointing at
/// it) or `aria_label`.
#[component]
pub fn ChampionAutocomplete(
    champions: Vec<Champion>,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] on_select: Option<Callback<String>>,
    /// DOM id of the input (defaults to the surrounding `Field`'s id).
    #[prop(optional, into)]
    id: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    /// Show the champion's icon beside the name once the value names a
    /// champion (M141, report 8tiuijr4); a dashed slot of the same size
    /// otherwise, so nothing shifts. The name stays in the field.
    #[prop(optional)]
    icon: bool,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let text = RwSignal::new(String::new());
    let active = RwSignal::new(0_usize);
    let champions = StoredValue::new(champions);
    // The input's id (the surrounding Field's when not given) also names
    // the list and its options, so they stay unique on the page.
    let input_id = id
        .or_else(|| use_context::<crate::components::ui::FieldContext>().map(|c| c.id))
        .unwrap_or_else(|| "champion-search".to_string());
    let list_id = format!("{input_id}-list");
    let list_id_sv = StoredValue::new(list_id.clone());

    let filtered = Memo::new(move |_| {
        let q = text.get().trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        champions.with_value(|champs| {
            let mut v: Vec<Champion> = champs
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&q))
                .cloned()
                .collect();
            v.sort_by_key(|c| !c.name.to_lowercase().starts_with(&q));
            v.truncate(8);
            v
        })
    });
    Effect::new(move |_| {
        let _ = filtered.get();
        active.set(0);
    });

    // Text written by the code (not typed) must not count as typing.
    let quiet = StoredValue::new(false);
    let set_text_quietly = move |t: String| {
        if text.get_untracked() != t {
            quiet.set_value(true);
            text.set(t);
        }
    };
    let choose = move |c: Champion| {
        value.set(c.id.clone());
        set_text_quietly(c.name.clone());
        open.set(false);
        if let Some(cb) = on_select {
            cb.run(c.id);
        }
    };
    // Typing: the value follows the text until a champion is chosen.
    Effect::new(move |prev: Option<()>| {
        let t = text.get();
        if prev.is_none() {
            return;
        }
        if quiet.get_value() {
            quiet.set_value(false);
            return;
        }
        value.set(t);
        open.set(true);
    });
    // Show the display name when the value is set from outside.
    Effect::new(move |_| {
        let v = value.get();
        let shown = champions.with_value(|champs| {
            champs
                .iter()
                .find(|c| c.id == v || c.name == v)
                .map(|c| c.name.clone())
        });
        set_text_quietly(shown.unwrap_or(v));
    });

    let on_keydown = Callback::new(move |ev: leptos::ev::KeyboardEvent| {
        let len = filtered.with_untracked(|f| f.len());
        match ev.key().as_str() {
            "ArrowDown" if len > 0 => {
                ev.prevent_default();
                open.set(true);
                active.update(|a| *a = (*a + 1) % len);
            }
            "ArrowUp" if len > 0 => {
                ev.prevent_default();
                active.update(|a| *a = (*a + len - 1) % len);
            }
            "Enter" if open.get_untracked() && len > 0 => {
                ev.prevent_default();
                let i = active.get_untracked().min(len - 1);
                if let Some(c) = filtered.with_untracked(|f| f.get(i).cloned()) {
                    choose(c);
                }
            }
            "Escape" => open.set(false),
            _ => {}
        }
    });
    let on_blur = Callback::new(move |_| open.set(false));
    let active_dom = Signal::derive(move || {
        if !open.get() {
            return None;
        }
        let i = active.get();
        filtered.with(|f| {
            f.get(i)
                .map(|c| format!("{}-{}", list_id_sv.get_value(), c.id))
        })
    });

    // The champion the value names, for the icon slot.
    let picked = Memo::new(move |_| {
        let v = value.get();
        let v = v.trim().to_lowercase();
        if v.is_empty() {
            return None;
        }
        champions.with_value(|champs| {
            champs
                .iter()
                .find(|c| c.id.to_lowercase() == v || c.name.to_lowercase() == v)
                .map(|c| c.id.clone())
        })
    });
    let input_class = if icon { "pl-10" } else { "" };
    let icon_slot = icon.then(|| {
        view! {
            <span class="pointer-events-none absolute left-2 top-1/2 z-10 -translate-y-1/2" aria-hidden="true" data-champion-icon-slot>
                {move || match picked.get() {
                    Some(id) => view! { <ChampionAvatar name=id size=AvatarSize::Sm decorative=true /> }.into_any(),
                    None => view! { <span class="block size-6 rounded-ctl border border-dashed border-rule"></span> }.into_any(),
                }}
            </span>
        }
    });
    let placeholder = if placeholder.is_empty() {
        "Champion..."
    } else {
        placeholder
    };
    view! {
        <div class="relative">
            {icon_slot}
            {match aria_label {
                Some(label) => view! {
                    <TextField
                        value=text
                        kind=InputKind::Search
                        placeholder=placeholder
                        autocomplete="off"
                        id=input_id
                        aria_label=label
                        on_keydown=on_keydown
                        on_blur=on_blur
                        combobox=Combobox { controls: list_id.clone(), active: active_dom }
                        class=input_class
                    />
                }.into_any(),
                None => view! {
                    <TextField
                        value=text
                        kind=InputKind::Search
                        placeholder=placeholder
                        autocomplete="off"
                        id=input_id
                        on_keydown=on_keydown
                        on_blur=on_blur
                        combobox=Combobox { controls: list_id.clone(), active: active_dom }
                        class=input_class
                    />
                }.into_any(),
            }}
            <ul
                id=list_id
                role="listbox"
                aria-label="Champions"
                class=move || {
                    if open.get() && !filtered.with(|f| f.is_empty()) {
                        "absolute z-(--z-popover) mt-1 max-h-64 w-full overflow-y-auto border border-outline bg-ink-1 py-1 shadow-overlay"
                    } else {
                        "hidden"
                    }
                }
            >
                {move || {
                    let act = active.get();
                    filtered
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let name = c.name.clone();
                            let opt_id = format!("{}-{}", list_id_sv.get_value(), c.id);
                            let is_active = i == act;
                            let c_click = c.clone();
                            view! {
                                <li
                                    id=opt_id
                                    role="option"
                                    aria-selected=if is_active { "true" } else { "false" }
                                    class=if is_active {
                                        "flex cursor-pointer items-center gap-2 bg-ink-3 px-3 py-1.5 text-label text-primary"
                                    } else {
                                        "flex cursor-pointer items-center gap-2 px-3 py-1.5 text-label text-secondary hover:bg-ink-2 hover:text-primary"
                                    }
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        choose(c_click.clone());
                                    }
                                >
                                    <ChampionAvatar name=name.clone() size=AvatarSize::Sm decorative=true />
                                    <span class="truncate">{name}</span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </div>
    }
}
