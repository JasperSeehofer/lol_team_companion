//! Suggestion chip and strength dots (M150): the game plan's suggestions
//! about a matchup. A chip is a paper tag in ink (a claim, a pin-dot and an
//! evidence glyph), never a pill and never chalk at rest; the dots draw a
//! suggestion's strength without a number.

use leptos::prelude::*;

use super::button::Force;
use crate::models::graph::ReasonSource;

/// The mono glyph for a source (decorative; the source is spoken as words).
pub fn evidence_glyph(source: ReasonSource) -> &'static str {
    match source {
        ReasonSource::Rule => "\u{a7}",
        ReasonSource::TeamNote => "\u{b6}",
        ReasonSource::MatchData => "\u{3a3}",
        ReasonSource::Pool => "\u{25c7}",
        ReasonSource::Scouting => "\u{203b}",
    }
}

/// The tag's tilt, fixed by its place in the row (identical on server and
/// browser, at most 0.6 degrees).
fn tilt(index: usize) -> &'static str {
    match index % 3 {
        0 => "rotate-[-0.6deg]",
        1 => "rotate-[0.4deg]",
        _ => "rotate-[-0.3deg]",
    }
}

const CHIP: &str = "inline-flex min-h-11 max-w-[16rem] cursor-pointer items-center gap-2 rounded-ctl border border-outline bg-ink-3 py-1.5 pl-2.5 pr-2 transition-[background-color,border-color,opacity] duration-[160ms] ease-out hover:bg-ink-2 hover:border-secondary active:translate-y-px aria-expanded:border-primary aria-expanded:bg-ink-2 data-[force=hover]:bg-ink-2 data-[force=hover]:border-secondary data-[force=active]:translate-y-px data-[retiring=true]:opacity-0 motion-reduce:transition-none md:min-h-8 md:max-w-[24rem]";

/// One suggestion as a tag: tapping it opens its card. The page wraps each
/// chip in an `<li>` of a `role="list"` row.
///
/// A11y: a native `<button>` named by its claim and source; `aria-expanded`/`aria-controls` tie it to its card; tilt, pin-dot and glyph are decorative (`aria-hidden`); 44 px tall below `md`. A retiring chip fades out (instantly under reduced motion) and is then removed.
#[component]
pub fn SuggestionChip(
    /// The claim (at most 60 characters), rendered as text.
    #[prop(into)]
    claim: String,
    source: ReasonSource,
    /// Place in the row; picks the tilt.
    #[prop(optional)]
    index: usize,
    #[prop(optional, into)] expanded: Signal<bool>,
    /// Fading out after "Add to plan" or "Not for us".
    #[prop(optional, into)]
    retiring: Signal<bool>,
    /// The id of the card this chip opens.
    #[prop(optional, into)]
    controls: Option<String>,
    #[prop(optional)] on_open: Option<Callback<()>>,
    #[prop(optional)] force: Option<Force>,
) -> impl IntoView {
    let class = format!("{CHIP} {}", tilt(index));
    let feedback = format!("Suggestion: {claim}");
    let from = format!(", from {}", source.label());
    view! {
        <button
            type="button"
            class=class
            aria-expanded=move || if expanded.get() { "true" } else { "false" }
            aria-controls=controls
            data-force=force.map(Force::as_str)
            data-retiring=move || retiring.get().then_some("true")
            data-feedback-label=feedback
            on:click=move |_| {
                if let Some(cb) = on_open {
                    cb.run(());
                }
            }
        >
            <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-muted"></span>
            <span class="line-clamp-2 min-w-0 text-left text-label font-medium text-primary md:line-clamp-1">{claim}</span>
            <span aria-hidden="true" class="figures shrink-0 text-caption text-muted">{evidence_glyph(source)}</span>
            <span class="sr-only">{from}</span>
        </button>
    }
}

/// Strength as three dots (1 slight, 2 clear, 3 strong), no number.
///
/// A11y: one `role="img"` named "Strength N of 3"; the dots are filled
/// versus outlined (shape, not colour alone).
#[component]
pub fn StrengthDots(
    /// 1 to 3; other values are clamped.
    value: u8,
) -> impl IntoView {
    let v = value.clamp(1, 3);
    let label = format!("Strength {v} of 3");
    view! {
        <span role="img" aria-label=label class="inline-flex items-center gap-1">
            {(1..=3u8).map(|i| {
                let class = if i <= v {
                    "size-1.5 rounded-full bg-secondary"
                } else {
                    "size-1.5 rounded-full border border-muted"
                };
                view! { <span class=class></span> }
            }).collect_view()}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilt_is_small_and_varies() {
        assert_ne!(tilt(0), tilt(1));
        assert_ne!(tilt(1), tilt(2));
        assert_eq!(tilt(0), tilt(3));
    }

    #[test]
    fn every_source_has_a_glyph() {
        let glyphs: Vec<_> = [
            ReasonSource::Rule,
            ReasonSource::TeamNote,
            ReasonSource::MatchData,
            ReasonSource::Pool,
            ReasonSource::Scouting,
        ]
        .into_iter()
        .map(evidence_glyph)
        .collect();
        for (i, g) in glyphs.iter().enumerate() {
            assert!(!g.is_empty());
            assert!(!glyphs[i + 1..].contains(g));
        }
    }
}
