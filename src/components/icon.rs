//! Shared SVG `<Icon>` primitive matching the design's icon set in
//! `components.jsx::Icon`. Path data is transcribed verbatim from
//! `/tmp/lol-design-handoff/lol-team-companion-app/project/components.jsx:108-141`.
//!
//! Stroke uses `currentColor` so callers can tint via Tailwind utilities
//! (`text-accent`, `text-muted`, etc.). No raw hex colors here.

use leptos::prelude::*;

/// Render a 24x24 viewBox icon by `name`. Falls back to a small dot when
/// the name is unknown — mirrors the JSX `paths[name] || <circle />` pattern.
#[component]
pub fn Icon(
    name: &'static str,
    #[prop(optional, default = 18)] size: u32,
    #[prop(optional, default = "")] class: &'static str,
) -> impl IntoView {
    // Each icon's path data is the inner SVG fragment for a 24x24 viewBox.
    // Multi-path icons concatenate fragments with raw <path> elements.
    let inner = match name {
        "home" => view! {
            <path d="M3 11l9-8 9 8" />
            <path d="M5 10v10h14V10" />
        }.into_any(),
        "user" => view! {
            <circle cx="12" cy="8" r="4" />
            <path d="M4 21c0-4 4-7 8-7s8 3 8 7" />
        }.into_any(),
        "swords" => view! {
            <path d="M14 5l5-2-2 5-7 7" />
            <path d="M3 21l4-1 7-7" />
            <path d="M5 3l5 2 2 5" />
            <path d="M19 21l-4-1-2-3" />
        }.into_any(),
        "tree" => view! {
            <path d="M12 2L7 9h3v4H6l-3 5h18l-3-5h-4V9h3l-5-7z" />
            <path d="M12 18v4" />
        }.into_any(),
        "chart" => view! {
            <path d="M3 3v18h18" />
            <path d="M7 16l4-6 4 4 5-7" />
        }.into_any(),
        "target" => view! {
            <circle cx="12" cy="12" r="9" />
            <circle cx="12" cy="12" r="5" />
            <circle cx="12" cy="12" r="1" fill="currentColor" />
        }.into_any(),
        "book" => view! {
            <path d="M4 4h12a3 3 0 013 3v13H7a3 3 0 01-3-3V4z" />
            <path d="M4 17h15" />
        }.into_any(),
        "shield" => view! {
            <path d="M12 3l8 3v6c0 5-4 8-8 9-4-1-8-4-8-9V6l8-3z" />
        }.into_any(),
        "flame" => view! {
            <path d="M12 3c1 4 5 5 5 10a5 5 0 01-10 0c0-3 2-4 2-7 2 1 3 2 3 4z" />
        }.into_any(),
        "bell" => view! {
            <path d="M6 16V11a6 6 0 0112 0v5l2 2H4l2-2z" />
            <path d="M10 20a2 2 0 004 0" />
        }.into_any(),
        "settings" => view! {
            <circle cx="12" cy="12" r="3" />
            <path d="M19 12a7 7 0 00-.1-1.2l2-1.5-2-3.4-2.3.9a7 7 0 00-2.1-1.2L14 3h-4l-.5 2.6a7 7 0 00-2.1 1.2l-2.3-.9-2 3.4 2 1.5A7 7 0 005 12c0 .4 0 .8.1 1.2l-2 1.5 2 3.4 2.3-.9a7 7 0 002.1 1.2L10 21h4l.5-2.6a7 7 0 002.1-1.2l2.3.9 2-3.4-2-1.5c.1-.4.1-.8.1-1.2z" />
        }.into_any(),
        "trophy" => view! {
            <path d="M7 4h10v5a5 5 0 01-10 0V4z" />
            <path d="M5 6H3v2a3 3 0 003 3M19 6h2v2a3 3 0 01-3 3" />
            <path d="M9 21h6M12 14v7" />
        }.into_any(),
        "sparkle" => view! {
            <path d="M12 3l2 7 7 2-7 2-2 7-2-7-7-2 7-2 2-7z" />
        }.into_any(),
        "check" => view! {
            <path d="M5 12l5 5L20 7" />
        }.into_any(),
        "eye" => view! {
            <path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z" />
            <circle cx="12" cy="12" r="3" />
        }.into_any(),
        "clock" => view! {
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 2" />
        }.into_any(),
        "play" => view! {
            <path d="M7 4l13 8-13 8V4z" fill="currentColor" />
        }.into_any(),
        "pause" => view! {
            <rect x="6" y="5" width="4" height="14" fill="currentColor" stroke="none" />
            <rect x="14" y="5" width="4" height="14" fill="currentColor" stroke="none" />
        }.into_any(),
        "plus" => view! {
            <path d="M12 5v14M5 12h14" />
        }.into_any(),
        "arrow" => view! {
            <path d="M5 12h14M13 6l6 6-6 6" />
        }.into_any(),
        "grip" => view! {
            <circle cx="9" cy="6" r="1" fill="currentColor" />
            <circle cx="9" cy="12" r="1" fill="currentColor" />
            <circle cx="9" cy="18" r="1" fill="currentColor" />
            <circle cx="15" cy="6" r="1" fill="currentColor" />
            <circle cx="15" cy="12" r="1" fill="currentColor" />
            <circle cx="15" cy="18" r="1" fill="currentColor" />
        }.into_any(),
        "feather" => view! {
            <path d="M20 4c-2 8-8 14-16 16l3-3a14 14 0 0010-10l3-3z" />
            <path d="M16 8L4 20" />
        }.into_any(),
        "hammer" => view! {
            <path d="M14 4l6 6-3 3-6-6 3-3z" />
            <path d="M11 7L3 15v4h4l8-8" />
        }.into_any(),
        "seal" => view! {
            <circle cx="12" cy="10" r="6" />
            <path d="M9 14l-3 7 6-3 6 3-3-7" />
        }.into_any(),
        "quill" => view! {
            <path d="M5 19l14-14a3 3 0 010 4l-9 9-5 1z" />
            <path d="M5 19h14" />
        }.into_any(),
        "scroll" => view! {
            <path d="M6 4h12a2 2 0 012 2v12a2 2 0 01-2 2H8a2 2 0 01-2-2V4z" />
            <path d="M6 4a2 2 0 00-2 2v3h2" />
            <path d="M9 9h8M9 13h8M9 17h5" />
        }.into_any(),
        "search" => view! {
            <circle cx="11" cy="11" r="7" />
            <path d="M21 21l-5-5" />
        }.into_any(),
        "radio" => view! {
            <circle cx="12" cy="12" r="2" fill="currentColor" />
            <path d="M9 9a4 4 0 016 0M6 6a8 8 0 0112 0M15 15a4 4 0 01-6 0M18 18a8 8 0 01-12 0" />
        }.into_any(),
        "layers" => view! {
            <path d="M12 3l9 5-9 5-9-5 9-5z" />
            <path d="M3 12l9 5 9-5M3 17l9 5 9-5" />
        }.into_any(),
        "drake" => view! {
            <path d="M3 12l4-3 5 3 5-3 4 3-9 8-9-8z" />
            <path d="M7 9l5-3 5 3" />
        }.into_any(),
        // Fallback dot — matches JSX behaviour for unknown names.
        _ => view! {
            <circle cx="12" cy="12" r="3" />
        }.into_any(),
    };

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            {inner}
        </svg>
    }
}
