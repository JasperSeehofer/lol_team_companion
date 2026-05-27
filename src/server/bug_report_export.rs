//! Phase 19 D-04 — server-start auto-export of open bug reports.
//!
//! Runs once in main.rs after init_db and before axum::serve.
//! Writes the path passed by main.rs (resolved from BUG_REPORT_INBOX_PATH
//! env var, default ./.planning/INBOX/bug-reports.md) with YAML front-matter
//! and one ## heading per open report. Failures are logged and
//! swallowed (D-04.5); never block server start.

use std::path::Path;
use std::sync::Arc;
use surrealdb::{engine::local::Db, Surreal};
use thiserror::Error;

use crate::models::bug_report::BugReport;
use crate::server::db;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("DB error: {0}")]
    Db(#[from] crate::server::db::DbError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Reads open reports from the DB, renders them via [`render_inbox`], and
/// writes the result synchronously to `inbox_path`. Creates parent
/// directories on demand. The caller (main.rs) resolves the path from the
/// `BUG_REPORT_INBOX_PATH` env var; env-var lookup deliberately does NOT
/// live here so unit tests can pass tempdirs without racing on a
/// process-global env var.
pub async fn export_open_reports(
    db: &Arc<Surreal<Db>>,
    inbox_path: &Path,
) -> Result<(), ExportError> {
    if let Some(parent) = inbox_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let reports = db::list_open_bug_reports(db).await?;
    let body = render_inbox(&reports);
    std::fs::write(inbox_path, body)?;
    tracing::info!(
        "Bug-report inbox exported: {} open report(s) -> {}",
        reports.len(),
        inbox_path.display()
    );
    Ok(())
}

/// Pure function — no DB, no filesystem. Builds the full inbox markdown
/// string with a 5-line YAML front-matter header followed by one H2 per
/// open report, grouped bug-first then wishlist, descending by
/// `created_at` within each group.
pub fn render_inbox(reports: &[BugReport]) -> String {
    let total_open = reports.len();
    let bug_count = reports.iter().filter(|r| r.category == "bug").count();
    let wishlist_count = reports
        .iter()
        .filter(|r| r.category == "wishlist")
        .count();

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!(
        "exported_at: {}\n",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));
    out.push_str(&format!("total_open: {}\n", total_open));
    out.push_str("by_category:\n");
    out.push_str(&format!("  bug: {}\n", bug_count));
    out.push_str(&format!("  wishlist: {}\n", wishlist_count));
    out.push_str("---\n\n");

    if total_open == 0 {
        out.push_str("_No open bug reports._\n");
        return out;
    }

    // Group bug-first, wishlist-second; each newest-first.
    let mut sorted: Vec<&BugReport> = reports.iter().collect();
    sorted.sort_by(|a, b| {
        let cat = category_rank(&a.category).cmp(&category_rank(&b.category));
        if cat != std::cmp::Ordering::Equal {
            return cat;
        }
        // Newest first: descending by created_at string (ISO-8601 sorts correctly).
        b.created_at.cmp(&a.created_at)
    });

    for r in sorted {
        out.push_str(&render_report(r));
        out.push('\n');
    }

    out
}

fn category_rank(c: &str) -> u8 {
    match c {
        "bug" => 0,
        "wishlist" => 1,
        _ => 2,
    }
}

fn render_report(r: &BugReport) -> String {
    // Truncate description for the H2 (first 60 chars per D-04.3).
    let snippet: String = r.description.chars().take(60).collect();
    let date_only: String = r
        .created_at
        .as_deref()
        .and_then(|s| s.split('T').next())
        .unwrap_or("unknown-date")
        .to_string();
    let viewport = match (r.viewport_w, r.viewport_h) {
        (Some(w), Some(h)) => format!("{}×{}", w, h),
        _ => "—".to_string(),
    };

    let mut s = String::new();
    s.push_str(&format!(
        "## [{}] {} — {}\n",
        r.category, snippet, date_only
    ));
    s.push_str(&format!("- URL: `{}`\n", r.page_url));
    s.push_str(&format!("- Element: `{}`\n", r.element_label));
    s.push_str(&format!("- User: `{}`\n", r.user_id));
    s.push_str(&format!("- Viewport: {}\n", viewport));
    s.push_str(&format!(
        "- Submitted: {}\n",
        r.created_at.as_deref().unwrap_or("unknown")
    ));
    s.push('\n');
    // T-19-04 partial mitigation: HTML-escape `<` so VS Code markdown
    // preview treats literal `<script>` as text, not active markup. The
    // principal mitigation is the prompt-injection warning in CLAUDE.md.
    // Blockquote prefix on every description line visually flags it as
    // quoted user content, not directives.
    for line in r.description.lines() {
        let escaped = line.replace('<', "&lt;");
        s.push_str("> ");
        s.push_str(&escaped);
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::bug_report::BugReport;

    fn report(category: &str, desc: &str, created: &str) -> BugReport {
        BugReport {
            id: Some(format!("bug_report:{category}-{created}")),
            user_id: "user:u1".into(),
            page_url: "/draft".into(),
            element_label: "Draft → Blue side → Pick 3".into(),
            description: desc.into(),
            category: category.into(),
            viewport_w: Some(1920),
            viewport_h: Some(1080),
            created_at: Some(created.into()),
            status: "open".into(),
        }
    }

    #[test]
    fn empty_list_renders_placeholder() {
        let out = render_inbox(&[]);
        assert!(out.contains("total_open: 0"));
        assert!(out.contains("No open bug reports"));
    }

    #[test]
    fn groups_bug_before_wishlist() {
        let r = vec![
            report("wishlist", "want X", "2026-05-26T10:00:00Z"),
            report("bug", "broke Y", "2026-05-26T09:00:00Z"),
        ];
        let out = render_inbox(&r);
        let bug_idx = out.find("[bug]").expect("bug heading");
        let wish_idx = out.find("[wishlist]").expect("wishlist heading");
        assert!(bug_idx < wish_idx, "bug must come before wishlist");
    }

    #[test]
    fn newest_first_within_group() {
        let r = vec![
            report("bug", "older", "2026-05-26T08:00:00Z"),
            report("bug", "newer", "2026-05-26T10:00:00Z"),
        ];
        let out = render_inbox(&r);
        let new_idx = out.find("newer").unwrap();
        let old_idx = out.find("older").unwrap();
        assert!(new_idx < old_idx);
    }

    #[test]
    fn h2_truncates_description_to_60_chars() {
        let r = vec![report("bug", &"x".repeat(120), "2026-05-26T10:00:00Z")];
        let out = render_inbox(&r);
        // The H2 line should contain only 60 x's.
        let h2_line = out
            .lines()
            .find(|l| l.starts_with("## [bug]"))
            .unwrap();
        let snippet = h2_line
            .trim_start_matches("## [bug] ")
            .split(" — ")
            .next()
            .unwrap();
        assert_eq!(snippet.len(), 60);
    }
}
