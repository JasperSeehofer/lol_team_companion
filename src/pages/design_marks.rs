//! `/design` gallery section for the nine marks (M151): five roles and four
//! objectives drawn on a 16 px master grid (1.5 px stroke, round joins and
//! caps, stroke only except Baron's solid core, `currentColor`). The same
//! path data ships as `public/marks/<name>.svg` (16 px) and
//! `<name>-24.svg` (24 px, the stroke held at 2 px); a unit test keeps the two in step. A mark is decoration:
//! it is `aria-hidden` and always stands beside its word. All content is
//! fixture text.

use leptos::prelude::*;

use crate::pages::design::GallerySection;

/// Baron's inner diamond: the one filled subpath in the set (filled with
/// `currentColor` and stroked, a solid core). It is the single documented
/// exception to "stroke only": an open inner diamond with 1.5 px gaps does
/// not fit at 16 px.
pub(crate) const BARON_CORE: &str = "M8 6.25L9.75 8L8 9.75L6.25 8Z";

/// Mark table: file name, word, group, path data on the 16-unit grid.
pub(crate) const MARKS: &[(&str, &str, &str, &[&str])] = &[
    (
        "top",
        "Top",
        "Role",
        &[
            "M2 13.5L8 3L14 13.5Z",
            "M7.25 10a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",
        ],
    ),
    (
        "jungle",
        "Jungle",
        "Role",
        &[
            "M2 5a2 2 0 1 0 4 0a2 2 0 1 0-4 0",
            "M4 7v1",
            "M10 4.5a2 2 0 1 0 4 0a2 2 0 1 0-4 0",
            "M12 6.5v1",
            "M6 11a2 2 0 1 0 4 0a2 2 0 1 0-4 0",
            "M8 13v1",
        ],
    ),
    (
        "mid",
        "Mid",
        "Role",
        &[
            "M4 8a4 4 0 1 0 8 0a4 4 0 1 0-8 0",
            "M8 2v2M8 12v2M2 8h2M12 8h2",
        ],
    ),
    (
        "bot",
        "Bot",
        "Role",
        &[
            "M3.25 12a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",
            "M4 3a9 9 0 0 1 9 9",
            "M7.06 4.61L7.83 2.76",
            "M9.66 6.34L11.07 4.93",
            "M11.39 8.94L13.24 8.17",
        ],
    ),
    (
        "support",
        "Support",
        "Role",
        &[
            "M7.25 8.5a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",
            "M3 3V8.5a5 5 0 0 0 10 0V3",
        ],
    ),
    (
        "dragon",
        "Dragon",
        "Objective",
        &["M8 2L14 8L8 14L2 8Z", "M5 8q1.5-2 3 0t3 0"],
    ),
    (
        "baron",
        "Baron",
        "Objective",
        &["M8 2L14 8L8 14L2 8Z", BARON_CORE],
    ),
    (
        "herald",
        "Herald",
        "Objective",
        &["M8 2L14 8L8 14L2 8Z", "M5 8h0M8 8h0M11 8h0"],
    ),
    (
        "tower",
        "Tower",
        "Objective",
        &[
            "M4 13.5V6.5a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v7",
            "M2 13.5h12",
            "M7.25 9.5a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",
        ],
    ),
];

/// The 16-unit grid drawn behind the 64 px specimen.
const GRID: &str = "M0 0V16M0 0H16M1 0V16M0 1H16M2 0V16M0 2H16M3 0V16M0 3H16M4 0V16M0 4H16M5 0V16M0 5H16M6 0V16M0 6H16M7 0V16M0 7H16M8 0V16M0 8H16M9 0V16M0 9H16M10 0V16M0 10H16M11 0V16M0 11H16M12 0V16M0 12H16M13 0V16M0 13H16M14 0V16M0 14H16M15 0V16M0 15H16M16 0V16M0 16H16";

/// One subpath; only `BARON_CORE` is filled.
fn mark_path(d: &'static str) -> impl IntoView {
    let fill = (d == BARON_CORE).then_some("currentColor");
    view! { <path d=d fill=fill /> }
}

/// One mark as inline SVG. `stroke` is in grid units: 1.5 at 16 px and at
/// 64 px (the master scaled), 1.33333 at 24 px (2 px on screen).
fn mark_svg(
    paths: &'static [&'static str],
    class: &'static str,
    stroke: &'static str,
) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width=stroke stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            {paths.iter().copied().map(mark_path).collect_view()}
        </svg>
    }
}

/// The 64 px specimen: grid, live area (1 to 15) and keylines under the mark.
fn mark_specimen(paths: &'static [&'static str]) -> impl IntoView {
    view! {
        <svg class="size-16 shrink-0" viewBox="0 0 16 16" fill="none" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d=GRID class="stroke-rule" stroke-width="0.0625" />
            <g class="stroke-outline" stroke-width="0.0625" stroke-dasharray="0.5 0.25">
                <rect x="1" y="1" width="14" height="14" />
                <circle cx="8" cy="8" r="6" />
                <rect x="2.5" y="2.5" width="11" height="11" />
                <path d="M8 2L14 8L8 14L2 8Z" />
                <path d="M2 13.25L8 2.75L14 13.25Z" />
            </g>
            <g stroke="currentColor" stroke-width="1.5">
                {paths.iter().copied().map(mark_path).collect_view()}
            </g>
        </svg>
    }
}

#[component]
pub fn MarksSection() -> impl IntoView {
    view! {
        <GallerySection id="marks" title="Marks" lede="Nine original marks for the roles and the objectives, drawn from surveying instruments and map symbols on a 16 px grid: 1.5 px stroke, round joins and caps, stroke only, in the text colour beside them (secondary). 24 px holds the stroke at 2 px; the 64 px specimen shows the grid, the live area and the keylines. A mark never replaces its word and is hidden from screen readers.">
            <div class="flex flex-col gap-8">
                <div class="flex flex-wrap items-center gap-x-5 gap-y-3" data-testid="marks-inline">
                    {MARKS
                        .iter()
                        .map(|(key, label, _, paths)| view! {
                            <span class="inline-flex items-center gap-1.5 text-secondary" data-mark=*key>
                                {mark_svg(paths, "size-4 shrink-0", "1.5")}
                                <span class="text-label text-primary">{*label}</span>
                            </span>
                        })
                        .collect_view()}
                </div>
                <div class="grid grid-cols-2 gap-6 sm:grid-cols-3 lg:grid-cols-5 xl:grid-cols-9">
                    {MARKS
                        .iter()
                        .map(|(key, label, group, paths)| view! {
                            <div class="flex flex-col items-start gap-3 text-secondary" data-mark-card=*key>
                                <span class="inline-flex items-center gap-2">
                                    {mark_svg(paths, "size-6 shrink-0", "1.33333")}
                                    <span class="text-reading text-primary">{*label}</span>
                                </span>
                                {mark_specimen(paths)}
                                <span class="text-caption text-muted">{*group}</span>
                            </div>
                        })
                        .collect_view()}
                </div>
            </div>
        </GallerySection>
    }
}

#[cfg(test)]
mod tests {
    use super::{BARON_CORE, MARKS};

    /// `public/marks/*.svg` carry exactly the path data of the table.
    #[test]
    fn mark_files_match_the_table() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/public/marks");
        assert_eq!(MARKS.len(), 9);
        for (key, label, _, paths) in MARKS {
            for (file, stroke) in [
                (format!("{dir}/{key}.svg"), "stroke-width=\"1.5\""),
                (format!("{dir}/{key}-24.svg"), "stroke-width=\"1.33333\""),
            ] {
                let svg = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"));
                assert!(svg.contains(stroke), "{file}: stroke");
                assert!(
                    svg.contains("stroke=\"currentColor\""),
                    "{file}: currentColor"
                );
                assert!(svg.contains("fill=\"none\""), "{file}: fill");
                assert!(
                    svg.contains(&format!("<title>{label}</title>")),
                    "{file}: title"
                );
                assert_eq!(
                    svg.contains("fill=\"currentColor\""),
                    paths.contains(&BARON_CORE),
                    "{file}: only Baron's core is filled"
                );
                let count = svg.matches("<path ").count();
                assert_eq!(count, paths.len(), "{file}: path count");
                for d in *paths {
                    assert!(svg.contains(&format!("d=\"{d}\"")), "{file}: missing {d}");
                }
            }
        }
    }
}
