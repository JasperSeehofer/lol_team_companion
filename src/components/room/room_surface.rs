//! Reusable intra-room tab component.
//!
//! `RoomSurface` renders an ARIA-compliant tab bar and one `<Show>`-gated
//! panel per surface. The active tab is deep-linkable via the `?tab=` URL
//! param. Signal + `<Show>` only — never sub-routes (D-02: sub-routes would
//! remount `AtmosphereRoot` and destroy parallax continuity).

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

// ─── Data ────────────────────────────────────────────────────────────────────

/// A single surface (tab) exposed by a room.
///
/// `id` is the stable ASCII slug used in the `?tab=` URL parameter; `label`
/// is the human-readable display text shown in the tab bar.
#[derive(Clone, PartialEq)]
pub struct SurfaceTab {
    /// URL slug, e.g. `"draft"`. Must be a stable ASCII identifier.
    pub id: &'static str,
    /// Display label shown in the tab button, e.g. `"Draft"`.
    pub label: &'static str,
}

// ─── Pure resolver (unit-testable, framework-free) ───────────────────────────

/// Resolve which tab should be active.
///
/// Returns the index of `requested` inside `tab_ids` when it matches.
/// Falls back to index 0 for an empty string or any value not in `tab_ids`
/// (D-01 discretion: unknown/missing `?tab=` → first tab).
pub(crate) fn resolve_active_tab(tab_ids: &[&str], requested: &str) -> usize {
    if requested.is_empty() {
        return 0;
    }
    tab_ids
        .iter()
        .position(|&id| id == requested)
        .unwrap_or(0)
}

// ─── Component ───────────────────────────────────────────────────────────────

/// Intra-room tab bar + `<Show>`-per-surface wrapper.
///
/// Renders a `role="tablist"` bar above content panels. The active surface
/// is driven by the `?tab=` URL query parameter (readable link) and written
/// back via `use_navigate`; unknown or missing values fall back to the first
/// tab (D-01 discretion).
///
/// The active tab id is provided to descendants as a
/// `Signal<&'static str>` via `provide_context`, so panel children can branch
/// on which surface is showing without re-reading the URL.
///
/// # Contract
/// * `tabs` must be non-empty.
/// * `id` values must be stable ASCII slugs.
/// * Switch surfaces with signal + `<Show>` only — never sub-routes (D-02).
#[component]
pub fn RoomSurface(
    /// Ordered list of surfaces exposed by this room.
    tabs: Vec<SurfaceTab>,
    /// Panel content. Rendered inside the active surface's `tabpanel`. The
    /// active tab id is available via `use_context::<Signal<&'static str>>()`.
    children: ChildrenFn,
) -> impl IntoView {
    let params = use_query_map();
    let navigate = use_navigate();

    // Clone tabs into a StoredValue so closures can access it without moving.
    let tabs_stored = StoredValue::new(tabs);

    // Derive the active tab index from the URL param.
    let active_idx = Signal::derive(move || {
        let q = params.read().get("tab").unwrap_or_default();
        let tab_ids: Vec<&str> = tabs_stored.with_value(|t| t.iter().map(|s| s.id).collect());
        resolve_active_tab(&tab_ids, &q)
    });

    // Expose the active tab id to descendants (panel children can branch on it).
    let active_id: Signal<&'static str> = Signal::derive(move || {
        tabs_stored.with_value(|t| t.get(active_idx.get()).map(|s| s.id).unwrap_or(""))
    });
    provide_context(active_id);

    // Navigate to a new ?tab= value on click. Clone `navigate` for use in
    // multiple closures via StoredValue.
    let navigate_stored = StoredValue::new(navigate);
    let set_active = move |id: &'static str| {
        navigate_stored.with_value(|nav| {
            nav(&format!("?tab={id}"), Default::default());
        });
    };

    view! {
        <nav role="tablist" aria-label="Room surfaces" class="flex gap-2 mb-4">
            {tabs_stored.with_value(|tabs| {
                tabs.iter()
                    .enumerate()
                    .map(|(idx, tab)| {
                        let id = tab.id;
                        let label = tab.label;
                        view! {
                            <button
                                role="tab"
                                id=format!("tab-{id}")
                                aria-selected=move || (active_idx.get() == idx).to_string()
                                aria-controls=format!("panel-{id}")
                                on:click=move |_| set_active(id)
                                class=move || {
                                    if active_idx.get() == idx {
                                        "px-4 py-2 rounded-md text-sm font-medium bg-accent text-accent-contrast"
                                    } else {
                                        "px-4 py-2 rounded-md text-sm font-medium text-secondary hover:text-primary hover:bg-surface"
                                    }
                                }
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()
            })}
        </nav>

        {tabs_stored.with_value(|tabs| {
            tabs.iter()
                .enumerate()
                .map(|(idx, tab)| {
                    let id = tab.id;
                    let tab_label_id = format!("tab-{id}");
                    let panel_id = format!("panel-{id}");
                    // ChildrenFn is an Rc — clone it per panel so each <Show>
                    // body owns a callable handle (the closure runs repeatedly).
                    let children = children.clone();
                    view! {
                        <Show when=move || active_idx.get() == idx>
                            <div
                                id=panel_id.clone()
                                role="tabpanel"
                                aria-labelledby=tab_label_id.clone()
                            >
                                {children()}
                            </div>
                        </Show>
                    }
                })
                .collect_view()
        })}
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_known_tab_returns_its_index() {
        // requested "beta" → index 1 (the "beta" tab).
        let tab_ids = ["alpha", "beta"];
        assert_eq!(resolve_active_tab(&tab_ids, "beta"), 1);
    }

    #[test]
    fn resolve_empty_falls_back_to_first() {
        // missing ?tab= → first tab.
        let tab_ids = ["alpha", "beta"];
        assert_eq!(resolve_active_tab(&tab_ids, ""), 0);
    }

    #[test]
    fn resolve_unknown_falls_back_to_first() {
        // unknown ?tab=zzz → first tab (D-01 discretion).
        let tab_ids = ["alpha", "beta"];
        assert_eq!(resolve_active_tab(&tab_ids, "zzz"), 0);
    }

    #[test]
    fn resolve_first_tab_explicitly_works() {
        let tab_ids = ["alpha", "beta"];
        assert_eq!(resolve_active_tab(&tab_ids, "alpha"), 0);
    }
}
