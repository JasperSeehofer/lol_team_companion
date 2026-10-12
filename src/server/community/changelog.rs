//! `CHANGELOG.md`, compiled into the binary and parsed once (M7, D4).
//!
//! An entry is a `## [x.y.z] - YYYY-MM-DD` release or a `### ...` heading
//! inside `## [Unreleased]` (the milestone sections), dated by a trailing
//! ` (YYYY-MM-DD)`. Sections are the `####` headings inside unreleased
//! entries (`###` inside releases); items are `- ` bullets with their
//! continuation lines. Inline markup kept: `code` and **bold**; links become
//! their text, plus the target when it is a relative path. No HTML passes
//! through: everything is text that Leptos escapes.

use std::sync::OnceLock;

use crate::models::community::{ChangelogChoice, ChangelogEntry, ChangelogSection, Span, SpanKind};

const SOURCE: &str = include_str!("../../../CHANGELOG.md");

/// The parsed changelog of the running build.
pub fn entries() -> &'static [ChangelogEntry] {
    static PARSED: OnceLock<Vec<ChangelogEntry>> = OnceLock::new();
    PARSED.get_or_init(|| parse(SOURCE))
}

/// Whether `slug` names an entry of the running build.
pub fn entry_exists(slug: &str) -> bool {
    entries().iter().any(|e| e.slug == slug)
}

/// The title of the entry `slug`, if it exists in the running build.
pub fn entry_title(slug: &str) -> Option<String> {
    entries()
        .iter()
        .find(|e| e.slug == slug)
        .map(|e| e.title.clone())
}

/// (slug, title) of every entry, in file order (newest first).
pub fn choices() -> Vec<ChangelogChoice> {
    entries()
        .iter()
        .map(|e| (e.slug.clone(), e.title.clone()))
        .collect()
}

/// "Milestone M7: Community and transparency loop" ->
/// `milestone-m7-community-and-transparency-loop`.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    s.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && s.chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Split "Title (2026-10-12)" into the title and the date.
fn split_trailing_date(heading: &str) -> (String, Option<String>) {
    let h = heading.trim();
    if let Some(rest) = h.strip_suffix(')') {
        if let Some(idx) = rest.rfind(" (") {
            let date = &rest[idx + 2..];
            if is_date(date) {
                return (rest[..idx].trim().to_string(), Some(date.to_string()));
            }
        }
    }
    (h.to_string(), None)
}

/// Inline markup to spans.
pub fn parse_inline(text: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut buf = String::new();
    let push = |spans: &mut Vec<Span>, kind: SpanKind, text: String| {
        if text.is_empty() {
            return;
        }
        if kind == SpanKind::Text {
            if let Some(last) = spans.last_mut() {
                if last.kind == SpanKind::Text {
                    last.text.push_str(&text);
                    return;
                }
            }
        }
        spans.push(Span { kind, text });
    };
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('`') {
            if let Some(end) = after.find('`') {
                push(&mut spans, SpanKind::Text, std::mem::take(&mut buf));
                push(&mut spans, SpanKind::Code, after[..end].to_string());
                rest = &after[end + 1..];
                continue;
            }
        }
        if let Some(after) = rest.strip_prefix("**") {
            if let Some(end) = after.find("**") {
                push(&mut spans, SpanKind::Text, std::mem::take(&mut buf));
                push(&mut spans, SpanKind::Strong, after[..end].to_string());
                rest = &after[end + 2..];
                continue;
            }
        }
        if let Some(after) = rest.strip_prefix('[') {
            if let Some(close) = after.find("](") {
                if let Some(end) = after[close + 2..].find(')') {
                    let label = &after[..close];
                    let url = &after[close + 2..close + 2 + end];
                    buf.push_str(label);
                    if url.starts_with('/') && !url.starts_with("//") {
                        buf.push_str(&format!(" ({url})"));
                    }
                    rest = &after[close + 2 + end + 1..];
                    continue;
                }
            }
        }
        let mut chars = rest.chars();
        if let Some(c) = chars.next() {
            buf.push(c);
        }
        rest = chars.as_str();
    }
    push(&mut spans, SpanKind::Text, buf);
    spans
}

/// Parse a Keep a Changelog file into entries, in file order.
pub fn parse(source: &str) -> Vec<ChangelogEntry> {
    let mut entries: Vec<ChangelogEntry> = Vec::new();
    let mut in_unreleased = false;
    let mut item: Option<String> = None;

    fn flush(entries: &mut [ChangelogEntry], item: &mut Option<String>) {
        if let Some(text) = item.take() {
            if let Some(section) = entries.last_mut().and_then(|e| e.sections.last_mut()) {
                section.items.push(parse_inline(text.trim()));
            }
        }
    }
    fn open_section(entries: &mut [ChangelogEntry], heading: &str) {
        if let Some(e) = entries.last_mut() {
            e.sections.push(ChangelogSection {
                heading: heading.trim().to_string(),
                items: Vec::new(),
            });
        }
    }

    for line in source.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            flush(&mut entries, &mut item);
            let h = h.trim();
            if h.eq_ignore_ascii_case("[unreleased]") {
                in_unreleased = true;
                continue;
            }
            in_unreleased = false;
            let (title, date) = match h.split_once(" - ") {
                Some((t, d)) if is_date(d.trim()) => (t.to_string(), Some(d.trim().to_string())),
                _ => (h.to_string(), None),
            };
            let title = title.trim_matches(['[', ']']).trim().to_string();
            entries.push(ChangelogEntry {
                slug: slugify(&title),
                title,
                date,
                sections: Vec::new(),
            });
        } else if let Some(h) = line.strip_prefix("#### ") {
            flush(&mut entries, &mut item);
            open_section(&mut entries, h);
        } else if let Some(h) = line.strip_prefix("### ") {
            flush(&mut entries, &mut item);
            if in_unreleased {
                let (title, date) = split_trailing_date(h);
                entries.push(ChangelogEntry {
                    slug: slugify(&title),
                    title,
                    date,
                    sections: Vec::new(),
                });
            } else {
                open_section(&mut entries, h);
            }
        } else if let Some(text) = line.strip_prefix("- ") {
            flush(&mut entries, &mut item);
            item = Some(text.to_string());
        } else if line.trim().is_empty() {
            flush(&mut entries, &mut item);
        } else if line.starts_with(char::is_whitespace) {
            if let Some(text) = item.as_mut() {
                text.push(' ');
                text.push_str(line.trim());
            }
        } else {
            flush(&mut entries, &mut item);
        }
    }
    flush(&mut entries, &mut item);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "# Changelog\n\nIntro text.\n\n## [Unreleased]\n\n### Milestone M7: Community loop (2026-10-12)\n\n#### Added\n- A `code` item with **bold** and a [link](/changelog) and an [external](https://example.org).\n- Two lines\n  joined here.\n\n#### Fixed\n- Fixed thing.\n\n### Milestone M6: Data\n\n#### Changed\n- Changed thing.\n\n## [0.1.0] - 2026-01-02\n\n### Added\n- First release.\n";

    #[test]
    fn parses_entries_sections_and_items() {
        let e = parse(FIXTURE);
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].title, "Milestone M7: Community loop");
        assert_eq!(e[0].slug, "milestone-m7-community-loop");
        assert_eq!(e[0].date.as_deref(), Some("2026-10-12"));
        assert_eq!(e[0].sections.len(), 2);
        assert_eq!(e[0].sections[0].heading, "Added");
        assert_eq!(e[0].sections[0].items.len(), 2);
        let first = &e[0].sections[0].items[0];
        assert_eq!(first[0].text, "A ");
        assert_eq!(
            first[1],
            Span {
                kind: SpanKind::Code,
                text: "code".into()
            }
        );
        assert_eq!(
            first[3],
            Span {
                kind: SpanKind::Strong,
                text: "bold".into()
            }
        );
        assert_eq!(first[4].text, " and a link (/changelog) and an external.");
        assert_eq!(e[0].sections[0].items[1][0].text, "Two lines joined here.");
        assert_eq!(e[1].date, None);
        assert_eq!(e[2].title, "0.1.0");
        assert_eq!(e[2].date.as_deref(), Some("2026-01-02"));
        assert_eq!(e[2].sections[0].heading, "Added");
        assert_eq!(e[2].sections[0].items.len(), 1);
    }

    #[test]
    fn slugs_and_dates() {
        assert_eq!(
            slugify("Milestone M7: Community and transparency loop"),
            "milestone-m7-community-and-transparency-loop"
        );
        assert_eq!(slugify("  M2 review fixes!! "), "m2-review-fixes");
        assert_eq!(
            split_trailing_date("Title (2026-10-12)"),
            ("Title".to_string(), Some("2026-10-12".to_string()))
        );
        assert_eq!(split_trailing_date("Title (stage A)").1, None);
        assert_eq!(parse_inline("unclosed `code")[0].text, "unclosed `code");
    }

    #[test]
    fn the_real_changelog_parses() {
        let e = entries();
        assert!(!e.is_empty());
        let mut slugs = std::collections::BTreeSet::new();
        for entry in e {
            assert!(!entry.title.is_empty());
            assert!(
                slugs.insert(entry.slug.clone()),
                "duplicate slug {}",
                entry.slug
            );
            assert!(
                entry.sections.iter().any(|s| !s.items.is_empty()),
                "{} has no items",
                entry.title
            );
            // Every entry is dated (M7 review: earlier ones backfilled from git).
            let d = entry.date.as_deref();
            assert!(d.is_some_and(is_date), "{} has no date", entry.title);
        }
        assert!(entry_exists(
            "milestone-m5-remaining-pages-navigation-mobile"
        ));
        assert!(!entry_exists("nope"));
    }
}
