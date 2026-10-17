//! Page structure: PageShell, PageHeader, SectionHeader, Surface.
//!
//! One header grammar everywhere: eyebrow, title, meta, actions, then a
//! full-width hairline. Grouping comes from alignment, space and type, so
//! Surface draws nothing unless asked.

use leptos::prelude::*;

use super::text::join;

/// The product name, the end of every browser-tab title.
pub const PRODUCT_NAME: &str = "Team Companion";

/// The browser-tab title for a page: "{page} \u{00b7} Team Companion", or
/// just the product name when the page has no title of its own (M148
/// item 8). Every route sets one through [`PageHeader`] or a
/// `leptos_meta::Title`; `app.rs` keeps the product name as the fallback.
pub fn page_title(page: &str) -> String {
    let page = page.trim();
    if page.is_empty() || page == PRODUCT_NAME {
        PRODUCT_NAME.to_string()
    } else {
        format!("{page} \u{00b7} {PRODUCT_NAME}")
    }
}

/// Maximum content width of a page.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageWidth {
    /// 28rem: sign-in and other single-form pages.
    Narrow,
    /// 48rem: briefs, reviews, legal text (60 to 75 characters per line).
    Reading,
    /// 64rem: settings and medium forms.
    Standard,
    /// 80rem: dashboards, boards and tables.
    #[default]
    Wide,
    /// No maximum: canvases (draft graph, sand table).
    Full,
}

impl PageWidth {
    pub fn class(self) -> &'static str {
        match self {
            PageWidth::Narrow => "max-w-md",
            PageWidth::Reading => "max-w-3xl",
            PageWidth::Standard => "max-w-5xl",
            PageWidth::Wide => "max-w-7xl",
            PageWidth::Full => "max-w-none",
        }
    }
}

/// Row density. Compact for boards and tables, comfortable for reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

impl Density {
    pub fn as_str(self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Comfortable => "comfortable",
        }
    }
}

/// The page frame inside `<main>`: width, gutters and vertical rhythm.
///
/// A11y: renders a `div` (the app shell owns the single `<main id="main">`
/// and the skip link). Side gutters are 16px on phones so nothing scrolls
/// sideways.
#[component]
pub fn PageShell(
    #[prop(optional)] width: PageWidth,
    #[prop(optional)] density: Density,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let class = join(&[
        "mx-auto w-full px-4 sm:px-8 pt-6 pb-16",
        width.class(),
        &class,
    ]);
    view! {
        <div class=class data-density=density.as_str()>
            {children()}
        </div>
    }
}

/// The page title block: eyebrow, `<h1>`, meta line, actions, hairline.
/// It also sets the browser-tab title ([`page_title`] of `title`, or of
/// `document_title` when given).
///
/// A11y: renders the page's only `<h1>`; the tab title names the page
/// first, so tabs and screen-reader page announcements tell pages apart. The eyebrow is plain text before
/// the heading (not part of its accessible name); actions follow the
/// heading in reading and tab order.
#[component]
pub fn PageHeader(
    /// The page title; a closure keeps it live (an editor's document name).
    #[prop(into)]
    title: TextProp,
    #[prop(optional, into)] eyebrow: Option<String>,
    /// One line of facts under the title (date, patch, counts).
    #[prop(optional, into)]
    meta: Option<ViewFn>,
    #[prop(optional, into)] actions: Option<ViewFn>,
    /// The tab title when it must differ from the heading (a public share
    /// page names the kind of page, never the plan's name).
    #[prop(optional, into)]
    document_title: Option<String>,
) -> impl IntoView {
    let tab_title = {
        let title = title.clone();
        move || {
            page_title(
                &document_title
                    .clone()
                    .unwrap_or_else(|| title.get().to_string()),
            )
        }
    };
    view! {
        <leptos_meta::Title text=tab_title />
        <header class="flex flex-wrap items-end justify-between gap-x-6 gap-y-4 pb-4 mb-6 border-b border-rule">
            <div class="min-w-0">
                {eyebrow.map(|e| view! { <p class="eyebrow">{e}</p> })}
                <h1 class="mt-2 font-display text-headline font-semibold tracking-[-0.02em] text-primary text-balance">
                    {move || title.get().to_string()}
                </h1>
                {meta.map(|m| view! {
                    <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-body text-secondary">{m.run()}</div>
                })}
            </div>
            {actions.map(|a| view! {
                <div class="flex flex-wrap items-center gap-2 shrink-0">{a.run()}</div>
            })}
        </header>
    }
}

/// Heading level for a [`SectionHeader`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    #[default]
    H2,
    H3,
}

/// A section title with optional eyebrow, meta text and actions.
///
/// A11y: renders a real `<h2>` or `<h3>`; pick the level that follows the
/// outline (an `h3` only inside an `h2` section). Pass `id` to label a
/// region with `aria-labelledby`.
#[component]
pub fn SectionHeader(
    #[prop(into)] title: String,
    #[prop(optional)] level: HeadingLevel,
    #[prop(optional, into)] eyebrow: Option<String>,
    #[prop(optional, into)] meta: Option<String>,
    #[prop(optional, into)] actions: Option<ViewFn>,
    #[prop(optional, into)] id: Option<String>,
) -> impl IntoView {
    let heading = match level {
        HeadingLevel::H2 => view! {
            <h2 id=id class="font-display text-title font-semibold tracking-[-0.01em] text-primary">{title}</h2>
        }
        .into_any(),
        HeadingLevel::H3 => view! {
            <h3 id=id class="font-display text-heading font-semibold text-primary">{title}</h3>
        }
        .into_any(),
    };
    view! {
        <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-2 mb-4">
            <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1 min-w-0">
                {eyebrow.map(|e| view! { <p class="eyebrow w-full">{e}</p> })}
                {heading}
                {meta.map(|m| view! { <p class="text-label text-muted">{m}</p> })}
            </div>
            {actions.map(|a| view! { <div class="flex items-center gap-2 shrink-0">{a.run()}</div> })}
        </div>
    }
}

/// Background layer of a [`Surface`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layer {
    /// No background: sits on whatever is below. The default.
    #[default]
    Inherit,
    Page,
    Ink1,
    Ink2,
    Ink3,
    /// Map and graph ground. Textures are allowed only here.
    Canvas,
}

impl Layer {
    pub fn class(self) -> &'static str {
        match self {
            Layer::Inherit => "",
            Layer::Page => "bg-page",
            Layer::Ink1 => "bg-ink-1",
            Layer::Ink2 => "bg-ink-2",
            Layer::Ink3 => "bg-ink-3",
            Layer::Canvas => "bg-canvas",
        }
    }
}

/// Edge treatment of a [`Surface`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Edge {
    /// Nothing drawn. The default: group with space and type first.
    #[default]
    None,
    /// Hairline above only: separates a section from the one before.
    RuleTop,
    /// Hairlines above and below: a band (community strip, toolbars).
    RuleY,
    /// Hairline box, square corners: a structural panel.
    Rule,
    /// Outline box with control radius: an object you can act on.
    Object,
}

impl Edge {
    pub fn class(self) -> &'static str {
        match self {
            Edge::None => "",
            Edge::RuleTop => "border-t border-rule",
            Edge::RuleY => "border-y border-rule",
            Edge::Rule => "border border-rule",
            Edge::Object => "border border-outline rounded-ctl",
        }
    }
}

/// Inner padding of a [`Surface`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pad {
    #[default]
    None,
    Sm,
    Md,
    Lg,
}

impl Pad {
    pub fn class(self) -> &'static str {
        match self {
            Pad::None => "",
            Pad::Sm => "p-3",
            Pad::Md => "p-4",
            Pad::Lg => "p-6",
        }
    }
}

/// The element a [`Surface`] renders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceTag {
    #[default]
    Div,
    Section,
    Article,
    Aside,
}

/// A region with an optional tonal layer and edge. Undecorated by default.
///
/// A11y: `section`/`aside` become landmarks only with a name, so pass
/// `label` (or `labelled_by` pointing at a heading id) when using them.
/// Attributes added with `attr:` (for example `data-feedback-label`) land on
/// the root element.
#[component]
pub fn Surface(
    #[prop(optional)] layer: Layer,
    #[prop(optional)] edge: Edge,
    #[prop(optional)] pad: Pad,
    #[prop(optional)] tag: SurfaceTag,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] labelled_by: Option<String>,
    /// Extra layout classes (grid, gap, margins).
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let class = join(&[layer.class(), edge.class(), pad.class(), &class]);
    match tag {
        SurfaceTag::Div => view! {
            <div class=class aria-label=label aria-labelledby=labelled_by>{children()}</div>
        }
        .into_any(),
        SurfaceTag::Section => view! {
            <section class=class aria-label=label aria-labelledby=labelled_by>{children()}</section>
        }
        .into_any(),
        SurfaceTag::Article => view! {
            <article class=class aria-label=label aria-labelledby=labelled_by>{children()}</article>
        }
        .into_any(),
        SurfaceTag::Aside => view! {
            <aside class=class aria-label=label aria-labelledby=labelled_by>{children()}</aside>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::page_title;

    #[test]
    fn tab_titles_name_the_page_then_the_product() {
        assert_eq!(page_title("Draft"), "Draft \u{00b7} Team Companion");
        assert_eq!(page_title("  Sign in "), "Sign in \u{00b7} Team Companion");
        assert_eq!(page_title(""), "Team Companion");
        assert_eq!(page_title("Team Companion"), "Team Companion");
    }
}
