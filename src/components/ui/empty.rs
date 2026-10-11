//! EmptyState: what an empty place will show, and how to fill it. The
//! honesty rule (MISSION section 6) in a component: never invented data.

use leptos::prelude::*;

use super::button::{Button, ButtonVariant};
use super::layout::HeadingLevel;

/// How much room an [`EmptyState`] takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptySize {
    /// A section of a page: rule above, generous space.
    #[default]
    Section,
    /// Inside a list, a table or a narrow column.
    Compact,
}

impl EmptySize {
    fn class(self) -> &'static str {
        match self {
            EmptySize::Section => "flex flex-col items-start gap-3 border-t border-rule py-10",
            EmptySize::Compact => "flex flex-col items-start gap-2 py-4",
        }
    }
}

/// An empty state: an optional title, a sentence that says what will
/// appear here and how it gets there, and one primary action.
///
/// A11y: plain text, no icon or emoji. `title` is a real heading at
/// `level` (default h3) when it is given, so the empty section stays in
/// the outline. The action is a real link (`cta_href`) or any button or
/// link passed through `action`; there is at most one.
#[component]
pub fn EmptyState(
    /// What will appear here and how: "Drafts you save appear here."
    #[prop(into)]
    message: String,
    /// Short title above the message: "No drafts yet".
    #[prop(optional, into)]
    title: Option<String>,
    #[prop(optional)] level: Option<HeadingLevel>,
    #[prop(optional)] size: EmptySize,
    /// Shorthand for a link action: label plus `cta_href`.
    #[prop(optional)]
    cta_label: Option<&'static str>,
    #[prop(optional)] cta_href: Option<&'static str>,
    /// The primary action as a view (a `Button`), instead of the shorthand.
    #[prop(optional, into)]
    action: Option<ViewFn>,
) -> impl IntoView {
    let heading = title.map(|t| match level.unwrap_or(HeadingLevel::H3) {
        HeadingLevel::H2 => {
            view! { <h2 class="font-display text-heading font-semibold text-primary">{t}</h2> }
                .into_any()
        }
        HeadingLevel::H3 => {
            view! { <h3 class="font-display text-heading font-semibold text-primary">{t}</h3> }
                .into_any()
        }
    });
    let shorthand = match (cta_label, cta_href) {
        (Some(label), Some(href)) => Some(
            view! { <Button href=href variant=ButtonVariant::Secondary>{label}</Button> }
                .into_any(),
        ),
        _ => None,
    };
    let action = action.map(|a| a.run().into_any()).or(shorthand);
    view! {
        <div class=size.class() data-empty="true">
            {heading}
            <p class="max-w-prose text-body text-secondary">{message}</p>
            {action.map(|a| view! { <div class="pt-1">{a}</div> })}
        </div>
    }
}

/// The "no team" state used by every team-scoped page.
///
/// A11y: as [`EmptyState`].
#[component]
pub fn NoTeamState() -> impl IntoView {
    view! {
        <EmptyState
            title="No team yet"
            message="Team pages need a team. Create one or ask to join one on the roster page; your drafts, plans and reviews will appear here."
            cta_label="Go to the roster"
            cta_href="/team/roster"
        />
    }
}
