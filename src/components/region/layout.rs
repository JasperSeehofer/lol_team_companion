//! Shims: `Card` and `SectionHead` render batch 1 primitives.

use leptos::prelude::*;

use crate::components::ui::{Edge, Layer, Pad, SectionHeader, Surface};

/// Legacy panel. Renders a ruled `Surface` on `ink-1` (square corners).
///
/// M3 replaces most panels with plain sections; new code uses `Surface`.
#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <Surface layer=Layer::Ink1 edge=Edge::Rule pad=Pad::Lg>
            {children()}
        </Surface>
    }
}

/// Legacy section header. Renders `SectionHeader` (a real `<h2>`).
#[component]
pub fn SectionHead(
    #[prop(into)] title: String,
    #[prop(optional, into)] eyebrow: Option<String>,
) -> impl IntoView {
    match eyebrow {
        Some(e) => view! { <SectionHeader title=title eyebrow=e /> }.into_any(),
        None => view! { <SectionHeader title=title /> }.into_any(),
    }
}

/// Legacy eyebrow label: the `eyebrow` text role.
#[component]
pub fn Eyebrow(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    view! { <p class=format!("eyebrow {class}")>{children()}</p> }
}
