//! CommandBar: Ctrl+K (Cmd+K on a Mac) opens one input that jumps to any
//! page now and runs commands later (M3/M5 add them). A real entry point,
//! not a search box.

use leptos::prelude::*;

use super::a11y::{click_id, roving_index};

/// One entry of the [`CommandBar`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub label: String,
    /// Group heading ("Go to", "Draft").
    pub group: String,
    /// Extra words that match ("pick ban" for the draft page).
    pub keywords: String,
    /// Where the command goes. Commands without a link run `on_run`.
    pub href: Option<String>,
    /// Short note on the right ("Strategy").
    pub hint: Option<String>,
}

impl Command {
    /// A "go to" command for a route.
    pub fn go(label: impl Into<String>, href: impl Into<String>) -> Self {
        let href = href.into();
        Self {
            id: format!("go:{href}"),
            label: label.into(),
            group: "Go to".to_string(),
            keywords: String::new(),
            href: Some(href),
            hint: None,
        }
    }

    pub fn keywords(mut self, words: impl Into<String>) -> Self {
        self.keywords = words.into();
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Indexes of the commands matching `query`, best first. Every word of the
/// query must appear in the label, the keywords, the hint or the group
/// (ignoring case). Labels that start with the query rank first, then
/// labels that contain it, then the rest, each in list order. An empty
/// query lists everything in order.
pub fn filter_commands(commands: &[Command], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return (0..commands.len()).collect();
    }
    let words: Vec<&str> = q.split_whitespace().collect();
    let mut hits: Vec<(u8, usize)> = commands
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let label = c.label.to_lowercase();
            let hay = format!(
                "{label} {} {} {}",
                c.keywords.to_lowercase(),
                c.hint.as_deref().unwrap_or("").to_lowercase(),
                c.group.to_lowercase()
            );
            if !words.iter().all(|w| hay.contains(w)) {
                return None;
            }
            let rank = if label.starts_with(&q) {
                0
            } else if label.contains(&q) {
                1
            } else {
                2
            };
            Some((rank, i))
        })
        .collect();
    hits.sort();
    hits.into_iter().map(|(_, i)| i).collect()
}

fn option_id(bar: &str, index: usize) -> String {
    format!("{bar}-opt-{index}")
}

/// The command bar: a modal combobox with a list of results.
///
/// A11y: a native modal `<dialog>` (focus trapped, Escape closes, focus
/// returns to where it was) holding an input with `role="combobox"`,
/// `aria-expanded`, `aria-controls` and `aria-activedescendant`, and a
/// `role="listbox"` of `role="option"` links grouped under named
/// `role="group"`s. Up/Down move the active option (wrapping), Home/End
/// jump, Enter follows it; typing filters, and the result count is
/// announced politely. Ctrl+K and Cmd+K toggle it from anywhere
/// (`aria-keyshortcuts` on the trigger is the caller's job; see the nav).
#[component]
pub fn CommandBar(
    /// Prefix for DOM ids; unique on the page.
    id: &'static str,
    open: RwSignal<bool>,
    commands: Vec<Command>,
    /// Runs with the command id for commands without a link.
    #[prop(optional)]
    on_run: Option<Callback<String>>,
    /// Listen for Ctrl+K / Cmd+K on the window (default true).
    #[prop(default = true)]
    global_shortcut: bool,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let query = RwSignal::new(String::new());
    let active = RwSignal::new(0usize);
    let commands = StoredValue::new(commands);
    let results = Memo::new(move |_| {
        let q = query.get();
        commands.with_value(|c| filter_commands(c, &q))
    });
    let listbox_id = format!("{id}-list");

    #[cfg(feature = "hydrate")]
    let prev_focus = StoredValue::new_local(None::<web_sys::HtmlElement>);

    Effect::new(move |_| {
        let want_open = open.get();
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let Some(dialog) = dialog_ref.get() else {
                return;
            };
            if want_open && !dialog.open() {
                let active_el = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
                prev_focus.set_value(active_el);
                query.set(String::new());
                active.set(0);
                let _ = dialog.show_modal();
                if let Some(input) = input_ref.get() {
                    let _ = input.focus();
                }
            } else if !want_open && dialog.open() {
                dialog.close();
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (want_open, dialog_ref, input_ref);
    });

    #[cfg(feature = "hydrate")]
    if global_shortcut {
        let handle = window_event_listener(leptos::ev::keydown, move |ev| {
            if (ev.ctrl_key() || ev.meta_key()) && ev.key().eq_ignore_ascii_case("k") {
                ev.prevent_default();
                open.update(|o| *o = !*o);
            }
        });
        on_cleanup(move || handle.remove());
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = global_shortcut;

    let on_close = move |_| {
        open.set(false);
        #[cfg(feature = "hydrate")]
        if let Some(el) = prev_focus.get_value() {
            let _ = el.focus();
        }
    };
    let on_click = move |ev: leptos::ev::MouseEvent| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let target = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
            if let (Some(t), Some(d)) = (target, dialog_ref.get()) {
                let d: &web_sys::Element = d.as_ref();
                if t == *d {
                    open.set(false);
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    let run = move |index: usize| {
        let Some(cmd) = commands.with_value(|c| c.get(index).cloned()) else {
            return;
        };
        if cmd.href.is_some() {
            // A click on the option link lets the router navigate.
            click_id(&option_id(id, index));
        } else {
            if let Some(cb) = on_run {
                cb.run(cmd.id);
            }
            open.set(false);
        }
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        let list = results.get_untracked();
        let pos = list.iter().position(|&i| i == active.get_untracked());
        if key == "Enter" {
            ev.prevent_default();
            if let Some(p) = pos {
                run(list[p]);
            }
            return;
        }
        if key == "Home" || key == "End" {
            // Home/End stay with the text field while typing.
            return;
        }
        if let Some(next) = roving_index(&key, pos, list.len(), true) {
            ev.prevent_default();
            active.set(list[next]);
            #[cfg(feature = "hydrate")]
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(&option_id(id, list[next])))
            {
                el.scroll_into_view_with_bool(false);
            }
        }
    };

    let listbox_for_input = listbox_id.clone();
    view! {
        <dialog
            node_ref=dialog_ref
            class="mx-auto mb-auto mt-[12vh] w-[min(36rem,calc(100vw-2rem))] overflow-visible border border-outline/70 bg-ink-1 p-0 text-primary shadow-overlay backdrop:bg-scrim"
            aria-label="Command bar"
            aria-modal="true"
            data-dialog=id
            on:close=on_close
            on:click=on_click
        >
            <div class="flex flex-col">
                <div class="flex items-center gap-3 border-b border-rule px-4 focus-within:border-accent">
                    <svg class="size-4 shrink-0 text-muted" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                        <circle cx="7" cy="7" r="4.5" />
                        <path d="M10.5 10.5L14 14" stroke-linecap="round" />
                    </svg>
                    <input
                        node_ref=input_ref
                        type="text"
                        role="combobox"
                        aria-label="Go to a page or run a command"
                        aria-expanded="true"
                        aria-controls=listbox_for_input
                        aria-autocomplete="list"
                        aria-activedescendant=move || {
                            let a = active.get();
                            results.get().contains(&a).then(|| option_id(id, a))
                        }
                        autocomplete="off"
                        spellcheck="false"
                        placeholder="Go to a page"
                        class="h-12 min-w-0 flex-1 bg-transparent text-reading text-primary placeholder:text-muted focus-visible:outline-none"
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            query.set(event_target_value(&ev));
                            let first = results.get_untracked().first().copied().unwrap_or(0);
                            active.set(first);
                        }
                        on:keydown=on_keydown
                    />
                    <kbd class="hidden shrink-0 rounded-ctl border border-outline border-b-2 px-1.5 text-caption text-muted sm:inline-block" aria-hidden="true">"Esc"</kbd>
                </div>
                <ul
                    id=listbox_id
                    role="listbox"
                    aria-label="Results"
                    class="max-h-[min(24rem,60dvh)] overflow-y-auto py-2"
                >
                    {move || {
                        let list = results.get();
                        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
                        commands.with_value(|c| {
                            for &i in &list {
                                let g = c[i].group.clone();
                                match groups.iter_mut().find(|(name, _)| *name == g) {
                                    Some((_, v)) => v.push(i),
                                    None => groups.push((g, vec![i])),
                                }
                            }
                        });
                        groups
                            .into_iter()
                            .enumerate()
                            .map(|(gi, (name, idx))| {
                                let head_id = format!("{id}-group-{gi}");
                                let labelled_by = head_id.clone();
                                view! {
                                    <li role="presentation">
                                        <ul role="group" aria-labelledby=labelled_by>
                                            <li role="presentation" id=head_id class="eyebrow px-4 pb-1 pt-2">{name}</li>
                                            {idx
                                                .into_iter()
                                                .map(|i| {
                                                    let cmd = commands.with_value(|c| c[i].clone());
                                                    let has_href = cmd.href.is_some();
                                                    let href = cmd.href.unwrap_or_else(|| "#".to_string());
                                                    let selected = move || active.get() == i;
                                                    view! {
                                                        <li role="presentation">
                                                            <a
                                                                id=option_id(id, i)
                                                                role="option"
                                                                href=href
                                                                tabindex="-1"
                                                                aria-selected=move || if selected() { "true" } else { "false" }
                                                                class="flex items-center justify-between gap-4 px-4 py-2 text-body text-secondary no-underline hover:bg-ink-2 aria-selected:bg-ink-3 aria-selected:text-primary"
                                                                on:mousemove=move |_| active.set(i)
                                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                                    if has_href {
                                                                        open.set(false);
                                                                    } else {
                                                                        ev.prevent_default();
                                                                        run(i);
                                                                    }
                                                                }
                                                            >
                                                                <span>{cmd.label}</span>
                                                                {cmd.hint.map(|h| view! { <span class="text-caption text-muted">{h}</span> })}
                                                            </a>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
                <p class="border-t border-rule px-4 py-2 text-caption text-muted" aria-live="polite">
                    {move || match results.get().len() {
                        0 => "No page matches. Commands arrive with the draft and Match Day pages.".to_string(),
                        1 => "1 result".to_string(),
                        n => format!("{n} results"),
                    }}
                </p>
            </div>
        </dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmds() -> Vec<Command> {
        vec![
            Command::go("Draft", "/draft").keywords("pick ban"),
            Command::go("Tree drafter", "/tree-drafter"),
            Command::go("Post-game review", "/post-game").keywords("debrief"),
            Command::go("Game plan", "/game-plan"),
        ]
    }

    #[test]
    fn empty_query_lists_everything_in_order() {
        assert_eq!(filter_commands(&cmds(), "  "), vec![0, 1, 2, 3]);
    }

    #[test]
    fn prefix_matches_rank_before_inner_matches() {
        // "dra": "Draft" starts with it, "Tree drafter" contains it.
        assert_eq!(filter_commands(&cmds(), "dra"), vec![0, 1]);
    }

    #[test]
    fn keywords_and_every_word_must_match() {
        assert_eq!(filter_commands(&cmds(), "ban"), vec![0]);
        assert_eq!(filter_commands(&cmds(), "game plan"), vec![3]);
        assert_eq!(filter_commands(&cmds(), "GAME"), vec![3, 2]);
        assert!(filter_commands(&cmds(), "zzz").is_empty());
    }
}
