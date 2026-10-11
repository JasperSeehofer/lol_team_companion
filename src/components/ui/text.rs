//! Text roles from the type scale. Pages use these instead of picking
//! sizes, faces and tracking by hand.

use leptos::prelude::*;

/// A role on the type scale (docs/DESIGN.md, "Type").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextRole {
    /// 44px display face. One per page at most (landing, closed beta).
    Display,
    /// 32px display face, the size of a page title.
    Page,
    /// 24px display face, section titles.
    Title,
    /// 20px display face, sub-sections and panel titles.
    Heading,
    /// 16px text for reading (briefs, reviews, long notes), 1.6 leading.
    Reading,
    /// 14px UI text.
    #[default]
    Body,
    /// 13px, medium weight: labels, table text, dense rows.
    Label,
    /// 12px: captions, help text, metadata. The smallest size there is.
    Caption,
    /// 12px mono caps with tracking: eyebrows and section markers.
    Overline,
    /// 14px mono with tabular figures: numbers that line up.
    Figure,
    /// 24px mono, semibold: a headline number (score, clock, fit).
    FigureLarge,
    /// Serif italic for surveyor's notes. Annotations only.
    Note,
}

impl TextRole {
    pub fn class(self) -> &'static str {
        match self {
            TextRole::Display => "font-display text-display font-semibold tracking-[-0.025em]",
            TextRole::Page => "font-display text-headline font-semibold tracking-[-0.02em]",
            TextRole::Title => "font-display text-title font-semibold tracking-[-0.01em]",
            TextRole::Heading => "font-display text-heading font-semibold",
            TextRole::Reading => "text-reading",
            TextRole::Body => "text-body",
            TextRole::Label => "text-label font-medium",
            TextRole::Caption => "text-caption",
            TextRole::Overline => "eyebrow",
            TextRole::Figure => "figures text-body",
            TextRole::FigureLarge => "figures text-title font-semibold tracking-[-0.02em]",
            TextRole::Note => "note text-reading",
        }
    }
}

/// Text colour. `Inherit` leaves the colour to the parent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Inherit,
    Primary,
    Secondary,
    Muted,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
}

impl Tone {
    pub fn class(self) -> &'static str {
        match self {
            Tone::Inherit => "",
            Tone::Primary => "text-primary",
            Tone::Secondary => "text-secondary",
            Tone::Muted => "text-muted",
            Tone::Accent => "text-accent",
            Tone::Success => "text-success",
            Tone::Warning => "text-warning",
            Tone::Danger => "text-danger",
            Tone::Info => "text-info",
        }
    }
}

/// The element a [`Text`] renders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextTag {
    #[default]
    Span,
    P,
    Div,
}

/// Text in one role of the type scale.
///
/// A11y: renders plain `span`/`p`/`div`, never a heading (use `PageHeader`
/// or `SectionHeader` for those, so the outline stays correct). Every tone
/// meets 4.5:1 on every surface layer; nothing renders below 12px.
#[component]
pub fn Text(
    #[prop(optional)] role: TextRole,
    #[prop(optional)] tone: Tone,
    #[prop(optional)] tag: TextTag,
    /// Extra layout classes (margins, alignment); not for colour or size.
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let class = join(&[role.class(), tone.class(), &class]);
    match tag {
        TextTag::Span => view! { <span class=class>{children()}</span> }.into_any(),
        TextTag::P => view! { <p class=class>{children()}</p> }.into_any(),
        TextTag::Div => view! { <div class=class>{children()}</div> }.into_any(),
    }
}

/// Join class fragments, skipping empty ones.
pub(crate) fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_and_tone_has_classes() {
        for role in [
            TextRole::Display,
            TextRole::Page,
            TextRole::Title,
            TextRole::Heading,
            TextRole::Reading,
            TextRole::Body,
            TextRole::Label,
            TextRole::Caption,
            TextRole::Overline,
            TextRole::Figure,
            TextRole::FigureLarge,
            TextRole::Note,
        ] {
            assert!(!role.class().is_empty());
            assert!(
                !role.class().contains("text-["),
                "{role:?} uses an arbitrary size"
            );
        }
        assert_eq!(Tone::Inherit.class(), "");
        assert_eq!(join(&["a", "", "b"]), "a b");
    }
}
