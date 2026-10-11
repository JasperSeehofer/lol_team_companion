//! Callout: an inline message that belongs to a place on the page (an error
//! loading a section, a warning about a plan, a confirmation after saving).
//! Toasts are for things that happened; callouts are for things that are.

use leptos::prelude::*;

/// What a [`Callout`] says about its content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalloutTone {
    /// Neutral information. Circle with an "i".
    #[default]
    Info,
    /// Done, saved, ready. Circle with a check.
    Success,
    /// Needs attention. The "!" mark plus words, never colour alone.
    Warning,
    /// Something went wrong or will be destroyed. Square with a "!".
    Danger,
}

impl CalloutTone {
    pub fn class(self) -> &'static str {
        match self {
            CalloutTone::Info => "border-info/45 bg-info/8",
            CalloutTone::Success => "border-success/45 bg-success/8",
            CalloutTone::Warning => "border-warning/50 bg-warning/8",
            CalloutTone::Danger => "border-danger/50 bg-danger/8",
        }
    }

    fn ink(self) -> &'static str {
        match self {
            CalloutTone::Info => "text-info",
            CalloutTone::Success => "text-success",
            CalloutTone::Warning => "text-warning",
            CalloutTone::Danger => "text-danger",
        }
    }

    /// The word a screen reader hears before the message.
    pub fn word(self) -> &'static str {
        match self {
            CalloutTone::Info => "Note",
            CalloutTone::Success => "Done",
            CalloutTone::Warning => "Warning",
            CalloutTone::Danger => "Error",
        }
    }

    /// Danger and warning interrupt (`role="alert"`); the others wait.
    pub fn role(self) -> &'static str {
        match self {
            CalloutTone::Warning | CalloutTone::Danger => "alert",
            CalloutTone::Info | CalloutTone::Success => "status",
        }
    }
}

/// The tone glyph: a shape plus a mark, so the tone never depends on hue.
pub(crate) fn glyph_for(tone: CalloutTone) -> impl IntoView {
    let cls = match tone {
        CalloutTone::Info => "mt-0.5 size-4 shrink-0 text-info",
        CalloutTone::Success => "mt-0.5 size-4 shrink-0 text-success",
        CalloutTone::Warning => "mt-0.5 size-4 shrink-0 text-warning",
        CalloutTone::Danger => "mt-0.5 size-4 shrink-0 text-danger",
    };
    let body = match tone {
        CalloutTone::Info => view! {
            <circle cx="8" cy="8" r="6.25" />
            <path d="M8 7.25v4M8 4.75v.5" stroke-linecap="round" />
        }
        .into_any(),
        CalloutTone::Success => view! {
            <circle cx="8" cy="8" r="6.25" />
            <path d="M5.25 8.25l1.9 1.9 3.6-4" stroke-linecap="round" stroke-linejoin="round" />
        }
        .into_any(),
        CalloutTone::Warning => view! {
            <path d="M8 1.9l6.4 11.6H1.6z" stroke-linejoin="round" />
            <path d="M8 6.25v3.5M8 11.6v.4" stroke-linecap="round" />
        }
        .into_any(),
        CalloutTone::Danger => view! {
            <rect x="1.9" y="1.9" width="12.2" height="12.2" />
            <path d="M8 4.75v4M8 10.9v.4" stroke-linecap="round" />
        }
        .into_any(),
    };
    view! {
        <svg class=cls viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
            {body}
        </svg>
    }
}

/// An inline message with a tone: info, success, warning or danger.
///
/// A11y: danger and warning are `role="alert"` (announced when they
/// appear), info and success `role="status"` (polite). The tone is said in
/// words: a visually hidden prefix ("Warning:") and, for warnings, a visible
/// "!" glyph next to the text; the glyph is decorative. An optional `title`
/// is a bold first line, not a heading. `action` holds at most one link or
/// button (for example "Try again").
#[component]
pub fn Callout(
    #[prop(optional)] tone: CalloutTone,
    #[prop(optional, into)] title: Option<String>,
    /// One follow-up action, placed after the text.
    #[prop(optional, into)]
    action: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    let class = format!(
        "flex items-start gap-3 rounded-ctl border px-4 py-3 {}",
        tone.class()
    );
    let ink = tone.ink();
    let bare_prefix = title
        .is_none()
        .then(|| view! { <span class="sr-only">{format!("{}: ", tone.word())}</span> });
    view! {
        <div role=tone.role() class=class data-tone=tone.word()>
            {glyph_for(tone)}
            <div class="flex min-w-0 flex-1 flex-col gap-1">
                {title.map(|t| view! {
                    <p class=format!("text-body font-semibold {ink}")>
                        <span class="sr-only">{format!("{}: ", tone.word())}</span>
                        {t}
                    </p>
                })}
                <div class="text-body text-primary">{bare_prefix}{children()}</div>
            </div>
            {action.map(|a| view! { <div class="shrink-0 self-center">{a.run()}</div> })}
        </div>
    }
}

/// An error line for page-level and resource errors (compatibility name).
///
/// A11y: a danger [`Callout`] (`role="alert"`). New code uses `Callout`;
/// this shim keeps the 30-odd unmigrated call sites working until M3/M5.
#[component]
pub fn ErrorBanner(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <Callout tone=CalloutTone::Danger>{message}</Callout>
    }
}

/// A success or error line, chosen by whether the text starts with "Error"
/// (compatibility name).
///
/// A11y: a success or danger [`Callout`]; the tone is in the words.
#[component]
pub fn StatusMessage(#[prop(into)] message: String) -> impl IntoView {
    let tone = if message.starts_with("Error") {
        CalloutTone::Danger
    } else {
        CalloutTone::Success
    };
    view! { <Callout tone=tone>{message}</Callout> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danger_and_warning_interrupt_info_and_success_wait() {
        assert_eq!(CalloutTone::Danger.role(), "alert");
        assert_eq!(CalloutTone::Warning.role(), "alert");
        assert_eq!(CalloutTone::Info.role(), "status");
        assert_eq!(CalloutTone::Success.role(), "status");
    }

    #[test]
    fn every_tone_has_a_word() {
        for t in [
            CalloutTone::Info,
            CalloutTone::Success,
            CalloutTone::Warning,
            CalloutTone::Danger,
        ] {
            assert!(!t.word().is_empty());
            assert!(!t.class().is_empty());
        }
    }
}
