//! Badge: a short label with a tone. Status, side and result tones carry a
//! glyph or text as well as colour.

use leptos::prelude::*;

/// What a [`Badge`] says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
    /// Blue side: round marker.
    SideBlue,
    /// Red side: square marker.
    SideRed,
    /// Win: filled teal.
    Win,
    /// Loss: hollow, no hue.
    Loss,
}

impl BadgeTone {
    pub fn class(self) -> &'static str {
        match self {
            BadgeTone::Neutral => "border-outline/60 text-secondary",
            BadgeTone::Accent => "border-accent/60 bg-accent-soft text-accent",
            BadgeTone::Success => "border-success/50 bg-success/10 text-success",
            BadgeTone::Warning => "border-warning/50 bg-warning/10 text-warning",
            BadgeTone::Danger => "border-danger/50 bg-danger/10 text-danger",
            BadgeTone::Info => "border-info/50 bg-info/10 text-info",
            BadgeTone::SideBlue => "border-side-blue/60 bg-side-blue-deep text-primary",
            BadgeTone::SideRed => "border-side-red/60 bg-side-red-deep text-primary",
            BadgeTone::Win => "border-win bg-win text-win-ink font-semibold",
            BadgeTone::Loss => "border-loss text-secondary",
        }
    }

    /// Shape drawn before the text, so the tone never depends on colour.
    fn marker(self) -> Option<&'static str> {
        match self {
            BadgeTone::SideBlue => Some("size-2 rounded-full bg-side-blue"),
            BadgeTone::SideRed => Some("size-2 rounded-[1px] bg-side-red"),
            BadgeTone::Warning => Some("size-2 rotate-45 bg-warning"),
            BadgeTone::Danger => Some("size-2 bg-danger"),
            BadgeTone::Success => Some("size-2 rounded-full bg-success"),
            _ => None,
        }
    }
}

/// A compact label: status, side, result or a count.
///
/// A11y: plain text in a `span`; the meaning is in the words ("Ready",
/// "Blue side", "W"), the marker shape repeats it and is hidden from screen
/// readers. For W/L glyphs pass `label` ("Win") so the accessible name is
/// the word, not the letter. Every tone is 4.5:1 or better.
#[component]
pub fn Badge(
    #[prop(optional)] tone: BadgeTone,
    /// Accessible name when the visible text is an abbreviation.
    #[prop(optional, into)]
    label: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = format!(
        "inline-flex items-center gap-1.5 h-5 px-1.5 rounded-ctl border text-caption font-medium leading-none whitespace-nowrap {}",
        tone.class()
    );
    let marker = tone
        .marker()
        .map(|m| view! { <span class=format!("shrink-0 {m}") aria-hidden="true"></span> });
    match label {
        Some(label) => view! {
            <span class=class>
                {marker}
                <span aria-hidden="true">{children()}</span>
                <span class="sr-only">{label}</span>
            </span>
        }
        .into_any(),
        None => view! { <span class=class>{marker}{children()}</span> }.into_any(),
    }
}
