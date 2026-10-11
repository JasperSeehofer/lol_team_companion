//! Tooltip: a short hint on hover and focus. Never the only place where
//! information lives (phones have no hover): it repeats or explains what
//! is already on screen or in the control's name.

use leptos::prelude::*;

/// Where the bubble sits relative to its trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Bottom,
}

impl TooltipPlacement {
    fn class(self) -> &'static str {
        match self {
            TooltipPlacement::Top => "bottom-full mb-2",
            TooltipPlacement::Bottom => "top-full mt-2",
        }
    }
}

const BUBBLE: &str = "pointer-events-none absolute left-1/2 z-(--z-popover) w-max max-w-64 -translate-x-1/2 rounded-ctl border border-outline/60 bg-ink-3 px-2 py-1 text-caption text-primary shadow-overlay transition-opacity duration-150";
const HIDDEN: &str = "invisible opacity-0 group-hover/tip:visible group-hover/tip:opacity-100 group-hover/tip:delay-300 group-focus-within/tip:visible group-focus-within/tip:opacity-100 group-focus-within/tip:delay-300";
const SHOWN: &str = "visible opacity-100";

/// Wraps one focusable element (a button, a link) and shows `text` above
/// or below it after a short delay on hover or keyboard focus.
///
/// A11y: the bubble is `role="tooltip"` and the wrapped control gets
/// `aria-describedby` pointing at it once the page is interactive, so
/// screen readers read the hint after the name. It appears on focus as
/// well as hover (after 300ms), Escape hides it until the pointer or focus
/// leaves, and it never takes focus or holds interactive content. Do not
/// put information only here.
#[component]
pub fn Tooltip(
    /// DOM id of the bubble; unique on the page.
    id: &'static str,
    #[prop(into)] text: String,
    #[prop(optional)] placement: TooltipPlacement,
    /// Gallery only: show without hover.
    #[prop(optional)]
    force_open: bool,
    children: Children,
) -> impl IntoView {
    let dismissed = RwSignal::new(false);
    let wrapper = NodeRef::<leptos::html::Span>::new();

    // Point the wrapped control at the bubble (after hydration; the
    // server markup is the same with or without it).
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(el) = wrapper.get() {
            if let Ok(Some(target)) =
                el.query_selector("a[href], button, input, select, textarea, [tabindex]")
            {
                let _ = target.set_attribute("aria-describedby", id);
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = wrapper;
    });

    let bubble_class = move || {
        let state = if force_open {
            SHOWN
        } else if dismissed.get() {
            "invisible opacity-0"
        } else {
            HIDDEN
        };
        format!("{BUBBLE} {} {state}", placement.class())
    };
    view! {
        <span
            node_ref=wrapper
            class="group/tip relative inline-flex"
            data-tooltip=id
            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Escape" {
                    dismissed.set(true);
                }
            }
            on:mouseleave=move |_| dismissed.set(false)
            on:focusout=move |_| dismissed.set(false)
        >
            {children()}
            <span id=id role="tooltip" class=bubble_class>{text}</span>
        </span>
    }
}
