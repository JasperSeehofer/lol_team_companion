//! Tabs: switch between views of the same object.

use leptos::prelude::*;

/// One tab of a [`Tabs`] list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabItem {
    /// Stable key, also used in the DOM ids.
    pub id: String,
    pub label: String,
    /// Optional count shown after the label.
    pub count: Option<u32>,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: None,
        }
    }

    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}

/// DOM id of a tab button.
pub fn tab_dom_id(group: &str, tab: &str) -> String {
    format!("{group}-tab-{tab}")
}

/// DOM id of a tab panel.
pub fn panel_dom_id(group: &str, tab: &str) -> String {
    format!("{group}-panel-{tab}")
}

/// Index of the tab a key press moves to, or `None` for other keys.
/// Arrow keys wrap; Home and End jump to the ends.
pub fn next_index(key: &str, current: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "ArrowRight" => Some((current + 1) % len),
        "ArrowLeft" => Some((current + len - 1) % len),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

/// The tab list. Pair each tab with a [`TabPanel`] using the same `group`.
///
/// A11y: WAI-ARIA tabs pattern: `role="tablist"` with a name, `role="tab"`
/// buttons with `aria-selected` and `aria-controls`, roving tabindex (only
/// the selected tab is in the tab order), Left/Right arrows move and select
/// (wrapping), Home/End jump to the first/last tab.
#[component]
pub fn Tabs(
    /// Prefix for DOM ids; unique on the page.
    group: &'static str,
    #[prop(into)] label: String,
    tabs: Vec<TabItem>,
    selected: RwSignal<String>,
) -> impl IntoView {
    let ids: Vec<String> = tabs.iter().map(|t| t.id.clone()).collect();
    let ids = StoredValue::new(ids);
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let list = ids.get_value();
        let cur = selected.get_untracked();
        let idx = list.iter().position(|i| *i == cur).unwrap_or(0);
        if let Some(next) = next_index(&ev.key(), idx, list.len()) {
            ev.prevent_default();
            let next_id = list[next].clone();
            selected.set(next_id.clone());
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::JsCast;
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id(&tab_dom_id(group, &next_id)))
                    .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                {
                    let _ = el.focus();
                }
            }
            #[cfg(not(feature = "hydrate"))]
            let _ = next_id;
        }
    };

    view! {
        <div
            role="tablist"
            aria-label=label
            class="flex gap-6 overflow-x-auto border-b border-rule [scrollbar-width:none]"
            on:keydown=on_keydown
        >
            {tabs
                .into_iter()
                .map(|tab| {
                    let id_sel = tab.id.clone();
                    let id_tab = tab.id.clone();
                    let id_click = tab.id.clone();
                    let is_selected = Memo::new(move |_| selected.get() == id_sel);
                    view! {
                        <button
                            type="button"
                            role="tab"
                            id=tab_dom_id(group, &tab.id)
                            aria-controls=panel_dom_id(group, &tab.id)
                            aria-selected=move || if is_selected.get() { "true" } else { "false" }
                            tabindex=move || if is_selected.get() { "0" } else { "-1" }
                            class=move || {
                                if is_selected.get() {
                                    "-mb-px shrink-0 cursor-pointer whitespace-nowrap border-b-2 border-accent py-3 font-display text-reading font-semibold text-primary"
                                } else {
                                    "-mb-px shrink-0 cursor-pointer whitespace-nowrap border-b-2 border-transparent py-3 font-display text-reading font-semibold text-muted transition-colors hover:text-primary"
                                }
                            }
                            data-tab=id_tab
                            on:click=move |_| selected.set(id_click.clone())
                        >
                            {tab.label}
                            {tab.count.map(|c| view! {
                                <span class="ml-2 figures text-caption font-medium text-muted">{c}</span>
                            })}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// The panel for one tab. Hidden (not removed) while another tab is
/// selected, so server and browser render the same markup.
///
/// A11y: `role="tabpanel"` labelled by its tab, focusable (`tabindex=0`) so
/// keyboard users reach panels without focusable content.
#[component]
pub fn TabPanel(
    group: &'static str,
    #[prop(into)] tab: String,
    selected: RwSignal<String>,
    children: Children,
) -> impl IntoView {
    let tab_for_hidden = tab.clone();
    view! {
        <div
            role="tabpanel"
            id=panel_dom_id(group, &tab)
            aria-labelledby=tab_dom_id(group, &tab)
            tabindex="0"
            class="pt-5"
            hidden=move || selected.get() != tab_for_hidden
        >
            {children()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_wrap_and_home_end_jump() {
        assert_eq!(next_index("ArrowRight", 0, 3), Some(1));
        assert_eq!(next_index("ArrowRight", 2, 3), Some(0));
        assert_eq!(next_index("ArrowLeft", 0, 3), Some(2));
        assert_eq!(next_index("Home", 2, 3), Some(0));
        assert_eq!(next_index("End", 0, 3), Some(2));
        assert_eq!(next_index("Enter", 0, 3), None);
        assert_eq!(next_index("ArrowRight", 0, 0), None);
    }

    #[test]
    fn ids_pair_tabs_with_panels() {
        assert_eq!(tab_dom_id("g", "a"), "g-tab-a");
        assert_eq!(panel_dom_id("g", "a"), "g-panel-a");
    }
}
