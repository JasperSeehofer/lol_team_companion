//! ProportionBar: segments of one quantity on a shared scale (M7). Used by
//! the transparency ledger (support split into costs and remainder) and the
//! survey results (share of each answer).

use leptos::prelude::*;

/// Ink of a [`BarSegment`]: three tones that differ in lightness and edge,
/// never in hue (no accent, side, result or status colour).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarInk {
    /// Solid secondary ink.
    #[default]
    Strong,
    /// A lighter layer with an outline edge.
    Soft,
    /// Outline only.
    Hollow,
}

impl BarInk {
    pub fn class(self) -> &'static str {
        match self {
            BarInk::Strong => "h-full bg-secondary",
            BarInk::Soft => "h-full bg-ink-3 border border-outline",
            BarInk::Hollow => "h-full border border-dashed border-outline",
        }
    }
}

/// Height of a [`ProportionBar`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarSize {
    /// 8px: inside table rows.
    #[default]
    Sm,
    /// 12px: on its own.
    Md,
}

impl BarSize {
    pub fn class(self) -> &'static str {
        match self {
            BarSize::Sm => "flex h-2 w-full min-w-16 bg-ink-2",
            BarSize::Md => "flex h-3 w-full min-w-16 bg-ink-2",
        }
    }
}

/// One segment: its value (same unit as `max`), a label (for the title
/// attribute only) and its ink.
#[derive(Clone, Debug, PartialEq)]
pub struct BarSegment {
    pub value: i64,
    pub label: String,
    pub ink: BarInk,
}

impl BarSegment {
    pub fn new(value: i64, label: impl Into<String>, ink: BarInk) -> Self {
        Self {
            value,
            label: label.into(),
            ink,
        }
    }
}

/// Whole-percent widths of `values` on a track that `max` fills. The sum
/// never exceeds 100, and a non-zero value gets at least 1% so it stays
/// visible. Negative values count as zero.
pub fn bar_widths(values: &[i64], max: i64) -> Vec<u32> {
    let total: i64 = values.iter().map(|v| (*v).max(0)).sum();
    let max = max.max(total).max(1);
    let mut widths: Vec<u32> = values
        .iter()
        .map(|v| {
            let v = (*v).max(0);
            if v == 0 {
                0
            } else {
                (((v as f64 / max as f64) * 100.0).floor() as u32).max(1)
            }
        })
        .collect();
    // The 1% minimum can push the sum over 100: take it back from the
    // widest segments.
    while widths.iter().sum::<u32>() > 100 {
        if let Some(w) = widths.iter_mut().max() {
            *w -= 1;
        } else {
            break;
        }
    }
    widths
}

/// A horizontal bar of segments on a shared scale.
///
/// A11y: Decorative (`aria-hidden`): the numbers it draws are always written
/// out in text next to it (a table cell or a list), so the bar adds no
/// information a screen reader misses. Never the only place a value
/// appears. Radius 0 (structure).
#[component]
pub fn ProportionBar(
    segments: Vec<BarSegment>,
    /// The value that fills the whole track (lets rows share a scale).
    max: i64,
    #[prop(optional)] size: BarSize,
) -> impl IntoView {
    let values: Vec<i64> = segments.iter().map(|s| s.value).collect();
    let widths = bar_widths(&values, max);
    view! {
        <div class=size.class() aria-hidden="true" data-proportion-bar>
            {segments
                .into_iter()
                .zip(widths)
                .filter(|(_, w)| *w > 0)
                .map(|(s, w)| view! {
                    <span class=s.ink.class() style=format!("width: {w}%") title=s.label></span>
                })
                .collect_view()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_never_exceed_the_track() {
        assert_eq!(bar_widths(&[50, 50], 100), vec![50, 50]);
        assert_eq!(bar_widths(&[1, 2], 3), vec![33, 66]);
        assert_eq!(bar_widths(&[25], 100), vec![25]);
        assert_eq!(bar_widths(&[0, 0], 0), vec![0, 0]);
        assert_eq!(bar_widths(&[1, 1000], 1001), vec![1, 99]);
        // Values above max widen the scale instead of overflowing.
        assert_eq!(bar_widths(&[150, 50], 100), vec![75, 25]);
        let w = bar_widths(&[1, 1, 1, 997], 1000);
        assert!(w.iter().sum::<u32>() <= 100);
        assert!(w.iter().all(|x| *x >= 1));
        assert_eq!(bar_widths(&[-5, 10], 10), vec![0, 100]);
    }
}
