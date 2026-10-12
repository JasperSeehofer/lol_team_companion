//! Community pages (M7): the bug board, What next, the changelog and the
//! transparency page. The first two need a signed-in beta account; the
//! changelog and transparency page are public. None is team-scoped, and no
//! page shows text one user wrote to another user (spec D2).

pub mod bugs;
pub mod changelog;
pub mod transparency;
pub mod what_next;

use leptos::prelude::*;

use crate::components::ui::{Badge, BadgeTone};
use crate::models::bug_report::BugStatus;
use crate::models::community::ProposalStatus;

/// Badge tone for a bug status; the word always carries the meaning.
pub fn bug_tone(status: &str) -> BadgeTone {
    match BugStatus::parse(status) {
        Some(BugStatus::Planned) => BadgeTone::Info,
        Some(BugStatus::Fixed) => BadgeTone::Success,
        _ => BadgeTone::Neutral,
    }
}

#[component]
pub fn BugStatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let label = crate::models::bug_report::status_label(&status);
    view! { <Badge tone=bug_tone(&status)>{label}</Badge> }
}

pub fn proposal_tone(status: &str) -> BadgeTone {
    match ProposalStatus::parse(status) {
        Some(ProposalStatus::Planned) | Some(ProposalStatus::Building) => BadgeTone::Info,
        Some(ProposalStatus::Shipped) => BadgeTone::Success,
        _ => BadgeTone::Neutral,
    }
}

#[component]
pub fn ProposalStatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let label = ProposalStatus::parse(&status)
        .map(|s| s.label())
        .unwrap_or("Unknown");
    view! { <Badge tone=proposal_tone(&status)>{label}</Badge> }
}

/// Redirect signed-out visitors to sign in (client side; the server
/// functions enforce it).
pub fn redirect_signed_out(
    user: Resource<Result<Option<crate::models::user::PublicUser>, ServerFnError>>,
) {
    Effect::new(move || {
        if let Some(Ok(None)) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::models::community::forbidden_money_words;

    /// String literals of a source file (text between double quotes on one
    /// line; good enough for UI copy).
    fn literals(src: &str) -> Vec<String> {
        let mut out = Vec::new();
        for line in src.lines() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            let mut parts = line.split('"');
            parts.next();
            while let Some(lit) = parts.next() {
                out.push(lit.to_string());
                parts.next();
            }
        }
        out
    }

    /// MISSION section 4: the word is "support"; never donation (except
    /// "no donation receipt is issued"), Spende, tip, sponsor, perk, reward.
    #[test]
    fn ui_strings_use_the_approved_money_words() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = [
            "src/pages/community/bugs.rs",
            "src/pages/community/changelog.rs",
            "src/pages/community/transparency.rs",
            "src/pages/community/what_next.rs",
            "src/pages/admin/hub.rs",
            "src/pages/admin/bugs.rs",
            "src/pages/admin/ledger.rs",
            "src/pages/admin/proposals.rs",
            "src/pages/admin/surveys.rs",
            "src/pages/admin/texts.rs",
            "src/components/bug_report_widget.rs",
            "src/components/survey_prompt.rs",
            "src/components/footer.rs",
            "src/components/nav.rs",
            "src/server/community/seeds.rs",
        ];
        let mut hits = Vec::new();
        for f in files {
            let src = std::fs::read_to_string(root.join(f)).expect(f);
            for lit in literals(&src) {
                let found = forbidden_money_words(&lit);
                if !found.is_empty() {
                    hits.push(format!("{f}: {found:?} in {lit:?}"));
                }
            }
        }
        assert!(hits.is_empty(), "{hits:#?}");
    }
}
