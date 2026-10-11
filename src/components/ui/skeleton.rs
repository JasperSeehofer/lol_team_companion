//! Skeleton: loading placeholders. This file is the only place in the app
//! where `animate-pulse` is allowed (lint rule `pulse`).

use leptos::prelude::*;

/// The outline of the content a [`Skeleton`] stands in for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    /// A line of text (3px radius, 14px high by default).
    #[default]
    Line,
    /// A block: a row, a slot, a chart (square corners: structure).
    Block,
    /// An avatar (round, the only round shape).
    Circle,
}

impl SkeletonShape {
    pub fn class(self) -> &'static str {
        match self {
            SkeletonShape::Line => "rounded-ctl bg-ink-2",
            SkeletonShape::Block => "border border-rule bg-ink-1",
            SkeletonShape::Circle => "shrink-0 rounded-full bg-ink-2",
        }
    }

    fn default_size(self) -> &'static str {
        match self {
            SkeletonShape::Line => "h-3.5",
            SkeletonShape::Block => "h-24",
            SkeletonShape::Circle => "size-8",
        }
    }
}

const PULSE: &str = "animate-pulse motion-reduce:animate-none";

/// A loading placeholder. `width` and `size` take literal Tailwind classes
/// from the caller (`"w-1/3"`, `"h-16"`), so Tailwind sees them at the call
/// site. `lines` repeats a line shape with a shorter last line.
///
/// A11y: decorative (`aria-hidden`). Wrap a group of skeletons in
/// [`SkeletonGroup`] (or put them in a region that already says
/// "Loading") so screen readers hear one status, not shapes. The pulse
/// stops under reduced motion and the shape stays.
#[component]
pub fn Skeleton(
    #[prop(optional)] shape: SkeletonShape,
    /// Width class; defaults to the full width (lines, blocks).
    #[prop(default = "w-full")]
    width: &'static str,
    /// Height (or size, for circles) class; defaults per shape.
    #[prop(optional)]
    size: Option<&'static str>,
    /// Number of text lines (Line shape only); 0 or 1 draws one.
    #[prop(optional)]
    lines: u8,
) -> impl IntoView {
    let size = size.unwrap_or(shape.default_size());
    let width = if shape == SkeletonShape::Circle {
        ""
    } else {
        width
    };
    let one = format!("{PULSE} {} {width} {size}", shape.class());
    if shape == SkeletonShape::Line && lines > 1 {
        let last = format!("{PULSE} {} w-2/3 {size}", shape.class());
        view! {
            <div class="flex w-full flex-col gap-2" aria-hidden="true">
                {(1..lines).map(|_| view! { <div class=one.clone()></div> }).collect_view()}
                <div class=last></div>
            </div>
        }
        .into_any()
    } else {
        view! { <div class=one aria-hidden="true"></div> }.into_any()
    }
}

/// A loading region: a polite status with its label, around decorative
/// skeletons.
///
/// A11y: `role="status"` with a visually hidden label ("Loading drafts"),
/// `aria-busy="true"`; the shapes inside are hidden from screen readers.
#[component]
pub fn SkeletonGroup(
    /// What is loading, read once: "Loading drafts".
    #[prop(into)]
    label: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div role="status" aria-busy="true" class="w-full">
            <span class="sr-only">{label}</span>
            <div class="flex w-full flex-col gap-3" aria-hidden="true">{children()}</div>
        </div>
    }
}

/// A loading line (compatibility name). Width and height are literal
/// Tailwind classes.
///
/// A11y: decorative, as [`Skeleton`].
#[component]
pub fn SkeletonLine(
    #[prop(default = "w-full")] width: &'static str,
    #[prop(default = "h-4")] height: &'static str,
) -> impl IntoView {
    view! { <Skeleton shape=SkeletonShape::Line width=width size=height /> }
}

/// A block-shaped loading placeholder (compatibility name).
///
/// A11y: decorative, as [`Skeleton`].
#[component]
pub fn SkeletonCard(#[prop(default = "h-24")] height: &'static str) -> impl IntoView {
    view! { <Skeleton shape=SkeletonShape::Block size=height /> }
}

/// A grid of skeleton blocks; `cols` is 2, 3 or 4 (compatibility name).
/// Stacks to one column on phones.
///
/// A11y: decorative, like its blocks.
#[component]
pub fn SkeletonGrid(
    #[prop(default = 3u8)] cols: u8,
    #[prop(default = 2u8)] rows: u8,
    #[prop(default = "h-20")] card_height: &'static str,
) -> impl IntoView {
    let col_class = match cols {
        2 => "grid grid-cols-1 gap-3 sm:grid-cols-2",
        3 => "grid grid-cols-1 gap-3 sm:grid-cols-3",
        4 => "grid grid-cols-2 gap-3 sm:grid-cols-4",
        _ => "grid grid-cols-1 gap-3",
    };
    let items: Vec<u8> = (0..cols * rows).collect();
    view! {
        <div class=col_class aria-hidden="true">
            {items.into_iter().map(|_| view! { <SkeletonCard height=card_height /> }).collect_view()}
        </div>
    }
}
