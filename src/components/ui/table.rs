//! Table: dense data with tabular figures, hairline rows, optional sticky
//! header and row links.

use leptos::prelude::*;

use super::layout::Density;
use super::text::join;

#[derive(Clone, Copy)]
struct TableCtx {
    density: Density,
    sticky: bool,
}

fn ctx() -> TableCtx {
    use_context::<TableCtx>().unwrap_or(TableCtx {
        density: Density::Compact,
        sticky: false,
    })
}

/// A data table. Children are `<thead>`/`<tbody>` built from [`Th`], [`Tr`]
/// and [`Td`].
///
/// A11y: a real `<table>` with a `<caption>` (visually hidden with
/// `caption_hidden`); the scroll wrapper is a named, focusable region so
/// keyboard users can scroll wide tables. Header cells are `th scope="col"`
/// and carry `aria-sort` when sorted. Numbers are right-aligned with
/// tabular figures. A `fit` table never scrolls, so it has no scroll region
/// and no extra tab stop.
#[component]
pub fn Table(
    #[prop(into)] caption: String,
    #[prop(optional)] caption_hidden: bool,
    #[prop(optional)] density: Density,
    /// Keep the header visible while the body scrolls (caps the height).
    #[prop(optional)]
    sticky_header: bool,
    /// Fixed layout that fits its container and never scrolls (M141, the
    /// match scoreboards): columns take the widths of a `<colgroup>`, cells
    /// truncate instead of widening the table. Hide columns on phones rather
    /// than letting the table overflow.
    #[prop(optional)]
    fit: bool,
    children: Children,
) -> impl IntoView {
    provide_context(TableCtx {
        density,
        sticky: sticky_header,
    });
    // `relative`: the wrapper is the containing block of anything absolutely
    // positioned inside (an `sr-only` header), so it scrolls with the table
    // instead of widening the page on phones.
    let wrapper = if sticky_header {
        "relative overflow-auto max-h-[28rem]"
    } else {
        "relative overflow-x-auto"
    };
    let caption_class = if caption_hidden {
        "sr-only"
    } else {
        "pb-2 text-left text-label text-muted"
    };
    if fit {
        return view! {
            <div class="relative">
                <table class="w-full table-fixed border-collapse text-body tabular-nums" data-table-fit="true">
                    <caption class=caption_class>{caption}</caption>
                    {children()}
                </table>
            </div>
        }
        .into_any();
    }
    let region_label = caption.clone();
    view! {
        <div class=wrapper role="region" aria-label=region_label tabindex="0">
            <table class="w-full border-collapse text-body tabular-nums">
                <caption class=caption_class>{caption}</caption>
                {children()}
            </table>
        </div>
    }
    .into_any()
}

/// Alignment of a table cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

impl Align {
    fn class(self) -> &'static str {
        match self {
            Align::Start => "text-left",
            Align::Center => "text-center",
            Align::End => "text-right",
        }
    }
}

/// Sort state of a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDir {
    Ascending,
    Descending,
}

/// A column header cell.
///
/// A11y: `th scope="col"`; `sort` sets `aria-sort`.
#[component]
pub fn Th(
    #[prop(optional)] align: Align,
    /// Right-aligned numeric column.
    #[prop(optional)]
    numeric: bool,
    #[prop(optional)] sort: Option<SortDir>,
    /// Extra classes, literal at the call site (for example
    /// `hidden md:table-cell` to drop a column on phones).
    #[prop(optional)]
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let c = ctx();
    let align = if numeric { Align::End } else { align };
    let extra = class;
    let class = join(&[
        "eyebrow font-medium whitespace-nowrap border-b border-outline pr-4 last:pr-0",
        extra,
        match c.density {
            Density::Compact => "py-2",
            Density::Comfortable => "py-3",
        },
        align.class(),
        if c.sticky {
            "sticky top-0 z-10 bg-page"
        } else {
            ""
        },
    ]);
    let aria_sort = sort.map(|s| match s {
        SortDir::Ascending => "ascending",
        SortDir::Descending => "descending",
    });
    view! { <th scope="col" class=class aria-sort=aria_sort>{children()}</th> }
}

/// A body row. Rows are hairline-separated; hover tints them.
///
/// A11y: a plain `<tr>`; a row link is the [`RowLink`] in its first cell,
/// so the row has one tab stop and a real link name. `highlight` tints the
/// row from outside (the same role in the other team's scoreboard); it is a
/// visual echo only, the control that sets it carries the state. `current`
/// sets `aria-current` (the signed-in player's row).
#[component]
pub fn Tr(
    #[prop(optional)] selected: bool,
    #[prop(optional, into)] highlight: Option<Signal<bool>>,
    #[prop(optional)] current: bool,
    children: Children,
) -> impl IntoView {
    let class = move || {
        if selected {
            "relative bg-ink-2"
        } else if highlight.and_then(|h| h.try_get()).unwrap_or(false) {
            "relative bg-ink-3 transition-colors"
        } else {
            "relative hover:bg-ink-2/70 transition-colors"
        }
    };
    view! {
        <tr class=class aria-selected=selected.then_some("true") aria-current=current.then_some("true")>
            {children()}
        </tr>
    }
}

/// A body cell.
///
/// A11y: a plain `<td>`; `header` renders `th scope="row"` for the cell
/// that names the row.
#[component]
pub fn Td(
    #[prop(optional)] numeric: bool,
    #[prop(optional)] muted: bool,
    /// This cell names the row (`th scope="row"`).
    #[prop(optional)]
    header: bool,
    #[prop(optional)] align: Align,
    /// Extra classes, literal at the call site.
    #[prop(optional)]
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let c = ctx();
    let align = if numeric { Align::End } else { align };
    let extra = class;
    let class = join(&[
        "border-b border-rule pr-4 last:pr-0 align-baseline",
        extra,
        match c.density {
            Density::Compact => "py-2",
            Density::Comfortable => "py-3",
        },
        align.class(),
        // Numbers never wrap ("12 / 10 / 24" stays on one line, M141).
        if numeric {
            "figures whitespace-nowrap"
        } else {
            ""
        },
        if muted { "text-muted" } else { "text-primary" },
        if header { "font-medium text-left" } else { "" },
    ]);
    if header {
        view! { <th scope="row" class=class>{children()}</th> }.into_any()
    } else {
        view! { <td class=class>{children()}</td> }.into_any()
    }
}

/// A link that makes its whole row clickable.
///
/// A11y: a normal `<a>` (one tab stop per row, Enter follows it); a
/// pseudo-element stretches its hit area over the row.
#[component]
pub fn RowLink(#[prop(into)] href: String, children: Children) -> impl IntoView {
    view! {
        <a href=href class="font-medium text-primary hover:underline underline-offset-4 after:absolute after:inset-0 after:content-['']">
            {children()}
        </a>
    }
}
