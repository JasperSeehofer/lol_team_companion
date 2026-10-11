//! StatReadout: a number that is the point (win rate, KDA, gold at 15),
//! in a fixed slot so a row of readouts lines up and does not jump when
//! values change.

use leptos::prelude::*;

/// Direction of a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    Flat,
}

impl Trend {
    fn arrow(self) -> &'static str {
        match self {
            Trend::Up => "\u{2191}",
            Trend::Down => "\u{2193}",
            Trend::Flat => "\u{2192}",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Trend::Up => "up",
            Trend::Down => "down",
            Trend::Flat => "unchanged",
        }
    }
}

/// Whether a change is good news. Colour follows this status, never the
/// direction and never a side colour (down can be good: deaths).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sentiment {
    Good,
    Bad,
    #[default]
    Neutral,
}

impl Sentiment {
    fn class(self) -> &'static str {
        match self {
            Sentiment::Good => "text-success",
            Sentiment::Bad => "text-danger",
            Sentiment::Neutral => "text-muted",
        }
    }
}

/// A change next to a readout: "+4.2" against something.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    /// As written: "+4.2", "-1".
    pub value: String,
    pub trend: Trend,
    pub sentiment: Sentiment,
    /// What it is compared with: "vs last 10".
    pub against: String,
}

impl Delta {
    pub fn new(value: impl Into<String>, trend: Trend, sentiment: Sentiment) -> Self {
        Self {
            value: value.into(),
            trend,
            sentiment,
            against: String::new(),
        }
    }

    pub fn against(mut self, against: impl Into<String>) -> Self {
        self.against = against.into();
        self
    }
}

/// Size of a [`StatReadout`] figure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReadoutSize {
    /// 14px figure: dense strips.
    Sm,
    /// 24px figure: the default headline number.
    #[default]
    Md,
    /// 32px figure: one per view.
    Lg,
}

impl ReadoutSize {
    fn class(self) -> &'static str {
        match self {
            ReadoutSize::Sm => "text-body font-semibold",
            ReadoutSize::Md => "text-title font-semibold tracking-[-0.02em]",
            ReadoutSize::Lg => "text-headline font-semibold tracking-[-0.03em]",
        }
    }
}

/// A labelled figure with an optional unit and change. Group several in a
/// `<dl>`-less row: each readout is its own description list.
///
/// A11y: a `<dl>` with the label as `<dt>` and the figure as `<dd>`, so
/// the pair is read together. The change is written as text with its
/// direction in words for screen readers ("up 4.2 vs last 10"); the arrow
/// is decorative and the colour follows good/bad, never the side colours.
/// Figures are mono and tabular; `slots` reserves room for that many
/// characters so values of different length line up.
#[component]
pub fn StatReadout(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] unit: Option<String>,
    #[prop(optional)] delta: Option<Delta>,
    #[prop(optional)] size: ReadoutSize,
    /// Characters reserved for the figure (fixed slot).
    #[prop(optional)]
    slots: Option<u8>,
    /// No data yet: shows an em dash and the reason in words.
    #[prop(optional, into)]
    empty_reason: Option<String>,
) -> impl IntoView {
    let figure_class = format!("figures leading-none text-primary {}", size.class());
    let slot_style = slots.map(|n| format!("min-width: {n}ch;"));
    let is_empty = empty_reason.is_some();
    let data_label = label.clone();
    view! {
        <dl class="flex min-w-0 flex-col gap-1.5" data-readout=data_label>
            <dt class="eyebrow">{label}</dt>
            <dd class="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                <span class=figure_class style=slot_style>
                    {if is_empty { "\u{2014}".to_string() } else { value }}
                </span>
                {(!is_empty).then(|| unit.map(|u| view! { <span class="text-label text-muted">{u}</span> }))}
                {(!is_empty).then(|| delta.map(|d| {
                    let class = format!("figures inline-flex items-baseline gap-1 text-label {}", d.sentiment.class());
                    let sr = format!(
                        "{} {}{}",
                        d.trend.word(),
                        d.value.trim_start_matches(['+', '-']),
                        if d.against.is_empty() { String::new() } else { format!(" {}", d.against) }
                    );
                    view! {
                        <span class=class>
                            <span aria-hidden="true">{d.trend.arrow()}</span>
                            <span aria-hidden="true">{d.value}</span>
                            <span class="sr-only">{sr}</span>
                            {(!d.against.is_empty()).then(|| view! {
                                <span class="text-caption text-muted" aria-hidden="true">{d.against.clone()}</span>
                            })}
                        </span>
                    }
                }))}
                {empty_reason.map(|r| view! { <span class="basis-full text-caption text-muted">{r}</span> })}
            </dd>
        </dl>
    }
}
