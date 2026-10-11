//! Menu: a button that opens a short list of actions or links (the WAI-ARIA
//! menu button pattern). Not for navigation bars or for choosing a value
//! (that is `Select` or `SegmentedControl`).

use leptos::prelude::*;

use super::a11y::{click_id, focus_id, roving_index, typeahead};
use super::button::{ButtonSize, ButtonVariant, BASE};
use super::text::join;

/// One entry of a [`Menu`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// Passed to `on_select`.
    pub id: String,
    pub label: String,
    /// Renders a link instead of a button.
    pub href: Option<String>,
    /// Keycap hint shown on the right ("P").
    pub shortcut: Option<&'static str>,
    /// Destructive: danger text, and a rule above it.
    pub danger: bool,
    pub disabled: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            href: None,
            shortcut: None,
            danger: false,
            disabled: false,
        }
    }

    pub fn link(id: impl Into<String>, label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            ..Self::new(id, label)
        }
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn shortcut(mut self, keys: &'static str) -> Self {
        self.shortcut = Some(keys);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Which edge of the button the list lines up with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuAlign {
    #[default]
    Start,
    End,
}

impl MenuAlign {
    fn class(self) -> &'static str {
        match self {
            MenuAlign::Start => "left-0",
            MenuAlign::End => "right-0",
        }
    }
}

/// DOM id of a menu item.
pub fn menu_item_id(menu: &str, index: usize) -> String {
    format!("{menu}-item-{index}")
}

const ITEM: &str = "flex w-full cursor-pointer items-center justify-between gap-6 px-3 py-2 text-left text-body text-secondary no-underline transition-colors hover:bg-ink-3 hover:text-primary focus-visible:bg-ink-3 focus-visible:text-primary focus-visible:-outline-offset-2 aria-disabled:cursor-not-allowed aria-disabled:opacity-50";
const ITEM_DANGER: &str = "flex w-full cursor-pointer items-center justify-between gap-6 px-3 py-2 text-left text-body text-danger no-underline transition-colors hover:bg-ink-3 focus-visible:bg-ink-3 focus-visible:-outline-offset-2";

/// A menu button: a [`super::Button`]-styled trigger and a list of items.
///
/// A11y: WAI-ARIA menu button. The trigger has `aria-haspopup="menu"`,
/// `aria-expanded` and `aria-controls`; the list is `role="menu"`, labelled
/// by the trigger, with `role="menuitem"` entries that are real buttons or
/// links (`tabindex=-1`). Enter, Space or Down open it and focus the first
/// item, Up focuses the last; in the list Up/Down move (wrapping),
/// Home/End jump, a letter jumps to the next item starting with it,
/// Escape closes and returns focus to the trigger, Tab closes. A click
/// outside closes. Destructive items say so in their label.
#[component]
pub fn Menu(
    /// Prefix for DOM ids; unique on the page.
    id: &'static str,
    /// Trigger text (and accessible name when `trigger` replaces it).
    #[prop(into)]
    label: String,
    items: Vec<MenuItem>,
    /// Runs with the item id for button items (links just navigate).
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    #[prop(optional)] align: MenuAlign,
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    /// Custom trigger content (for example an avatar and a name).
    #[prop(optional, into)]
    trigger: Option<ViewFn>,
    /// Gallery only: render open.
    #[prop(optional)]
    force_open: bool,
) -> impl IntoView {
    let open = RwSignal::new(force_open);
    let active = RwSignal::new(None::<usize>);
    let button_id = format!("{id}-button");
    let list_id = format!("{id}-menu");
    let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
    let enabled: Vec<bool> = items.iter().map(|i| !i.disabled).collect();
    let labels = StoredValue::new(labels);
    let enabled = StoredValue::new(enabled);
    let len = items.len();
    let button_id_store = StoredValue::new(button_id.clone());

    let focus_item = move |i: usize| {
        active.set(Some(i));
        focus_id(&menu_item_id(id, i));
    };
    let close = move |refocus: bool| {
        open.set(false);
        active.set(None);
        if refocus {
            focus_id(&button_id_store.get_value());
        }
    };
    // First enabled index from `start` in direction `step` (+1/-1), wrapping.
    let skip_disabled = move |start: usize, forward: bool| -> usize {
        let flags = enabled.get_value();
        let mut i = start;
        for _ in 0..len {
            if flags.get(i).copied().unwrap_or(false) {
                return i;
            }
            i = if forward {
                (i + 1) % len
            } else {
                (i + len - 1) % len
            };
        }
        start
    };

    let open_at = move |first: bool| {
        open.set(true);
        if len == 0 {
            return;
        }
        let start = if first { 0 } else { len - 1 };
        let i = skip_disabled(start, first);
        // Wait for the list to be shown before focusing into it.
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || focus_item(i));
            if let Some(win) = web_sys::window() {
                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                );
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = i;
    };

    let on_button_keydown = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
        "ArrowDown" => {
            ev.prevent_default();
            open_at(true);
        }
        "ArrowUp" => {
            ev.prevent_default();
            open_at(false);
        }
        _ => {}
    };
    let on_button_click = move |ev: leptos::ev::MouseEvent| {
        if open.get_untracked() {
            close(false);
        } else {
            open_at(true);
        }
        let _ = ev;
    };

    let on_list_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        let cur = active.get_untracked();
        match key.as_str() {
            "Escape" => {
                ev.prevent_default();
                close(true);
            }
            "Tab" => close(false),
            " " => {
                // Space activates links too (buttons do it natively).
                if let Some(i) = cur {
                    ev.prevent_default();
                    click_id(&menu_item_id(id, i));
                }
            }
            _ => {
                if let Some(next) = roving_index(&key, cur, len, true) {
                    ev.prevent_default();
                    let forward = !matches!(key.as_str(), "ArrowUp" | "End");
                    focus_item(skip_disabled(next, forward));
                } else if let Some(next) = labels.with_value(|l| typeahead(l, cur, &key)) {
                    ev.prevent_default();
                    focus_item(next);
                }
            }
        }
    };

    let trigger_class = join(&[BASE, size.class(), variant.class()]);
    let panel_class = format!(
        "absolute top-full z-(--z-popover) mt-1 min-w-48 overflow-hidden rounded-ctl border border-outline/60 bg-ink-2 py-1 shadow-overlay {}",
        align.class()
    );
    let label_for_trigger = label.clone();
    // A custom trigger may show something else (an avatar): name it.
    let aria_label = trigger.is_some().then(|| label.clone());
    let trigger_view = match trigger {
        Some(t) => t.run().into_any(),
        None => view! {
            <span>{label_for_trigger}</span>
            <svg class="size-3.5 text-muted" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <path d="M4 6l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        }
        .into_any(),
    };
    let rule_before =
        |i: usize, items: &[MenuItem]| i > 0 && items[i].danger && !items[i - 1].danger;
    let rendered = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item_id = menu_item_id(id, i);
            let class = if item.danger { ITEM_DANGER } else { ITEM };
            let sep = rule_before(i, &items)
                .then(|| view! { <li role="separator" class="my-1 border-t border-rule"></li> });
            let keycap = item.shortcut.map(|k| {
                view! { <kbd class="figures text-caption text-muted" aria-hidden="true">{k}</kbd> }
            });
            let select_id = item.id.clone();
            let is_link = item.href.is_some();
            let disabled = item.disabled;
            let on_click = move |ev: leptos::ev::MouseEvent| {
                if disabled {
                    ev.prevent_default();
                    return;
                }
                if !is_link {
                    if let Some(cb) = on_select {
                        cb.run(select_id.clone());
                    }
                }
                close(!is_link);
            };
            let on_focus = move |_| active.set(Some(i));
            let body = match item.href.clone() {
                Some(href) => view! {
                    <a
                        id=item_id
                        href=href
                        role="menuitem"
                        tabindex="-1"
                        class=class
                        aria-disabled=disabled.then_some("true")
                        aria-keyshortcuts=item.shortcut
                        on:click=on_click
                        on:focus=on_focus
                    >
                        <span>{item.label.clone()}</span>
                        {keycap}
                    </a>
                }
                .into_any(),
                None => view! {
                    <button
                        id=item_id
                        type="button"
                        role="menuitem"
                        tabindex="-1"
                        class=class
                        aria-disabled=disabled.then_some("true")
                        aria-keyshortcuts=item.shortcut
                        on:click=on_click
                        on:focus=on_focus
                    >
                        <span>{item.label.clone()}</span>
                        {keycap}
                    </button>
                }
                .into_any(),
            };
            view! {
                {sep}
                <li role="none">{body}</li>
            }
        })
        .collect_view();

    view! {
        <div class="relative inline-block h-fit" data-menu=id>
            <button
                id=button_id.clone()
                type="button"
                class=trigger_class
                aria-haspopup="menu"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                aria-controls=list_id.clone()
                aria-label=aria_label
                on:click=on_button_click
                on:keydown=on_button_keydown
            >
                {trigger_view}
            </button>
            {move || (open.get() && !force_open).then(|| view! {
                <div class="fixed inset-0 z-(--z-popover)" aria-hidden="true" on:click=move |_| close(false)></div>
            })}
            <ul
                id=list_id
                role="menu"
                aria-labelledby=button_id
                class=panel_class
                hidden=move || !open.get()
                on:keydown=on_list_keydown
            >
                {rendered}
            </ul>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_builders() {
        let i = MenuItem::link("p", "Profile", "/profile").shortcut("P");
        assert_eq!(i.href.as_deref(), Some("/profile"));
        assert_eq!(i.shortcut, Some("P"));
        assert!(MenuItem::new("x", "Delete").danger().danger);
        assert_eq!(menu_item_id("m", 2), "m-item-2");
    }
}
