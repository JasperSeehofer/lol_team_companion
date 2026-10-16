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
    let aria_sort = sort.map(SortDir::aria);
    view! { <th scope="col" class=class aria-sort=aria_sort>{children()}</th> }
}

impl SortDir {
    /// The `aria-sort` value.
    pub fn aria(self) -> &'static str {
        match self {
            SortDir::Ascending => "ascending",
            SortDir::Descending => "descending",
        }
    }

    pub fn flip(self) -> SortDir {
        match self {
            SortDir::Ascending => SortDir::Descending,
            SortDir::Descending => SortDir::Ascending,
        }
    }
}

/// Which column a sortable table is sorted by (`key: None`: the order the
/// rows came in).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortState {
    pub key: Option<&'static str>,
    pub dir: SortDir,
}

impl SortState {
    /// Sorted by `key` in `dir`.
    pub fn by(key: &'static str, dir: SortDir) -> Self {
        Self {
            key: Some(key),
            dir,
        }
    }

    /// Unsorted: the rows' own order.
    pub fn none() -> Self {
        Self {
            key: None,
            dir: SortDir::Ascending,
        }
    }

    /// The state after a click on the header of `key`: a new column starts
    /// in its default direction (numbers descending, words ascending), the
    /// same column again flips.
    pub fn clicked(self, key: &'static str, numeric: bool) -> Self {
        if self.key == Some(key) {
            Self {
                key: Some(key),
                dir: self.dir.flip(),
            }
        } else {
            Self::by(
                key,
                if numeric {
                    SortDir::Descending
                } else {
                    SortDir::Ascending
                },
            )
        }
    }
}

/// Sort `rows` by `cmp` in `dir`, stably: rows that compare equal keep
/// their incoming order in both directions.
pub fn sort_rows<T>(rows: &mut [T], dir: SortDir, cmp: impl Fn(&T, &T) -> std::cmp::Ordering) {
    match dir {
        SortDir::Ascending => rows.sort_by(&cmp),
        SortDir::Descending => rows.sort_by(|a, b| cmp(b, a)),
    }
}

/// A sortable column header (M141): a click, Enter or Space sorts the table
/// by this column; again flips the direction. Render the `<thead>` outside
/// any closure that reads `sort`, so re-sorting re-renders only the body and
/// focus stays on the header button.
///
/// A11y: `th scope="col"` with `aria-sort` on the sorted column; the
/// control is a native `<button>` named "Sort by {label}"; the direction is
/// drawn as a chevron (up ascending, down descending, a muted pair
/// unsorted) and the text turns primary, never by colour alone. Pair it
/// with one [`SortStatus`] per table to announce the new order.
#[component]
pub fn SortTh(
    label: &'static str,
    /// The column's sort key (`SortState::key`).
    key: &'static str,
    sort: RwSignal<SortState>,
    /// Right-aligned numeric column; first sorted descending.
    #[prop(optional)]
    numeric: bool,
    /// Extra classes, literal at the call site.
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    let c = ctx();
    let extra = class;
    let class = join(&[
        "eyebrow font-medium whitespace-nowrap border-b border-outline pr-4 last:pr-0",
        extra,
        match c.density {
            Density::Compact => "py-1",
            Density::Comfortable => "py-2",
        },
        if numeric { "text-right" } else { "text-left" },
        if c.sticky {
            "sticky top-0 z-10 bg-page"
        } else {
            ""
        },
    ]);
    let current = move || {
        let s = sort.get();
        (s.key == Some(key)).then_some(s.dir)
    };
    let button_class = move || {
        let base = if numeric {
            "eyebrow inline-flex min-h-11 flex-row-reverse items-center gap-1 whitespace-nowrap hover:text-primary md:min-h-8"
        } else {
            "eyebrow inline-flex min-h-11 items-center gap-1 whitespace-nowrap hover:text-primary md:min-h-8"
        };
        if current().is_some() {
            format!("{base} text-primary")
        } else {
            format!("{base} text-secondary")
        }
    };
    view! {
        <th scope="col" class=class aria-sort=move || current().map(SortDir::aria)>
            <button
                type="button"
                class=button_class
                aria-label=format!("Sort by {label}")
                data-sort-key=key
                on:click=move |_| sort.update(|s| *s = s.clicked(key, numeric))
            >
                <span>{label}</span>
                <svg viewBox="0 0 10 10" class="size-2.5 shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    {move || match current() {
                        Some(SortDir::Ascending) => view! { <path d="M2 6.5 5 3.5 8 6.5" /> }.into_any(),
                        Some(SortDir::Descending) => view! { <path d="M2 3.5 5 6.5 8 3.5" /> }.into_any(),
                        None => view! { <path class="text-muted" d="M2.5 4 5 1.5 7.5 4M2.5 6 5 8.5 7.5 6" /> }.into_any(),
                    }}
                </svg>
            </button>
        </th>
    }
}

/// Announces a sortable table's order after it changes ("Sorted by Games,
/// descending").
///
/// A11y: a `role="status"` region that is always mounted (its text changes,
/// so screen readers announce it) and visually hidden. `labels` maps sort
/// keys to the column names.
#[component]
pub fn SortStatus(
    sort: RwSignal<SortState>,
    labels: &'static [(&'static str, &'static str)],
) -> impl IntoView {
    let text = move || {
        let s = sort.get();
        s.key
            .and_then(|k| labels.iter().find(|(key, _)| *key == k))
            .map(|(_, l)| format!("Sorted by {l}, {}", s.dir.aria()))
            .unwrap_or_default()
    };
    view! { <p class="sr-only" role="status">{text}</p> }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_sorts_then_flips() {
        let s = SortState::none().clicked("games", true);
        assert_eq!(s, SortState::by("games", SortDir::Descending));
        assert_eq!(s.clicked("games", true).dir, SortDir::Ascending);
        assert_eq!(
            s.clicked("champion", false),
            SortState::by("champion", SortDir::Ascending)
        );
    }

    #[test]
    fn sorting_is_stable_both_ways() {
        let mut rows = vec![("a", 2), ("b", 1), ("c", 2), ("d", 1)];
        sort_rows(&mut rows, SortDir::Descending, |x, y| x.1.cmp(&y.1));
        assert_eq!(rows, vec![("a", 2), ("c", 2), ("b", 1), ("d", 1)]);
        sort_rows(&mut rows, SortDir::Ascending, |x, y| x.1.cmp(&y.1));
        assert_eq!(rows, vec![("b", 1), ("d", 1), ("a", 2), ("c", 2)]);
    }
}
