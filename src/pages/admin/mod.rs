pub mod bugs;
pub mod hub;
pub mod invites;
pub mod ledger;
pub mod proposals;
pub mod resets;
pub mod surveys;
pub mod texts;
pub mod traits;

use leptos::prelude::*;
use leptos_router::components::A;

/// One entry in an admin list-and-editor page: a link that selects the
/// record (`?id=`) without scrolling; the selected entry carries
/// `aria-current` and the accent edge (accent marks selection).
#[component]
pub fn ListRow(
    #[prop(into)] href: String,
    selected: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <A
            href=href
            scroll=false
            attr:class=move || if selected.get() {
                "block w-full border-l-2 border-accent bg-ink-2 px-3 py-2.5 text-left"
            } else {
                "block w-full border-l-2 border-transparent px-3 py-2.5 text-left transition-colors hover:bg-ink-2"
            }
            attr:aria-current=move || selected.get().then_some("true")
        >
            {children()}
        </A>
    }
}
